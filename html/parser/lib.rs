/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![feature(type_name_of_val)]

mod codepoint;
mod error;
mod state;
mod token;
mod tokenizer;

use std::{
    borrow::{Borrow, BorrowMut},
    ops::Deref,
};

use dom::node::{
    CommentNode, Document, DocumentNode, DocumentType, Node, QuirksMode,
    TextNode,
};
use html_elements::{
    interface::IsOneOfTagsInterface, tag_attributes, tag_names,
    HTMLScriptElement,
};
use infra::{
    self, namespace::Namespace, primitive::codepoint::CodePoint,
    structure::tree::TreeNode,
};
use macros::dd;
use state::ListOfActiveFormattingElements;
use tokenizer::State;

pub use self::{
    state::{InsertionMode, StackOfOpenElements},
    token::{HTMLTagToken, HTMLToken},
    tokenizer::HTMLTokenizer,
};

// --------- //
// Structure //
// --------- //

pub struct HTMLParser<C>
where
    C: Iterator<Item = CodePoint>,
{
    tokenizer: HTMLTokenizer<C>,
    document: DocumentNode,
    insertion_mode: InsertionMode,
    original_insertion_mode: InsertionMode,
    stack_of_template_insertion_modes: Vec<InsertionMode>,
    stack_of_open_elements: StackOfOpenElements,
    list_of_active_formatting_elements: ListOfActiveFormattingElements,
    foster_parenting: bool,
    frameset_ok: bool,
    parsing_fragment: bool,
    scripting_enabled: bool,
    stop_parsing: bool,
    context_element: Option<TreeNode<Node>>,
    character_insertion_node: Option<TreeNode<Node>>,
    character_insertion_builder: String,
    head_element: Option<TreeNode<Node>>,
}

struct AdjustedInsertionLocation {
    parent: Option<TreeNode<Node>>,
    insert_before_sibling: Option<TreeNode<Node>>,
}

// -------------- //
// Implémentation //
// -------------- //

impl<C> HTMLParser<C>
where
    C: Iterator<Item = CodePoint>,
{
    pub fn new(document: DocumentNode, input: C) -> Self {
        let tokenizer = HTMLTokenizer::new(input);

        Self {
            tokenizer,
            document,
            insertion_mode: InsertionMode::default(),
            original_insertion_mode: InsertionMode::default(),
            stack_of_template_insertion_modes: Vec::default(),
            stack_of_open_elements: StackOfOpenElements::default(),
            list_of_active_formatting_elements:
                ListOfActiveFormattingElements::default(),
            frameset_ok: true,
            foster_parenting: false,
            parsing_fragment: false,
            scripting_enabled: true,
            stop_parsing: false,
            context_element: None,
            character_insertion_node: None,
            character_insertion_builder: String::new(),
            head_element: None,
        }
    }
}

impl<C> HTMLParser<C>
where
    C: Iterator<Item = CodePoint>,
{
    pub fn run(&mut self) {
        loop {
            match dd!(&self.tokenizer.next_token()) {
                | None => break,

                // Lorsque chaque jeton est émis par le tokenizer, l'agent
                // utilisateur doit suivre les étapes appropriées de la
                // liste suivante, connue sous le nom de dispatcher de
                // construction d'arbre :
                //    - Si la pile d'éléments ouverts est vide
                //    - Si le nœud courant ajusté est un élément dans
                //      l'espace de nom HTML
                // TODO //   - Si le nœud courant ajusté est un point
                //      d'intégration de texte MathML et que le jeton est
                //      une balise de début dont le nom de balise n'est ni
                //      "mglyph" ni "malignmark"
                // TODO //   - Si le nœud courant ajusté est un point
                //      d'intégration de texte MathML et que le jeton est
                //      un jeton de caractère
                // TODO //   - Si le nœud courant ajusté est un élément
                //      MathML annotation-xml et que le jeton est une
                //      balise de départ dont le nom de balise est "svg"
                //    - Si le nœud courant ajusté est un point
                //      d'intégration HTML et que le jeton est une balise
                //      de départ
                //    - Si le nœud courant ajusté est un point
                //      d'intégration HTML et que le jeton est un jeton de
                //      caractère
                //    - Si le jeton est un jeton de fin de fichier
                //
                // Traiter le jeton selon les règles données dans la
                // section correspondant au mode d'insertion actuel dans le
                // contenu HTML.
                | Some(token)
                    if self.stack_of_open_elements.is_empty()
                        || self
                            .adjusted_current_node()
                            .is_in_html_namespace()
                        || (self
                            .adjusted_current_node()
                            .is_html_text_integration_point()
                            && (token.is_start_tag()
                                || token.is_character()))
                        || token.is_eof() =>
                {
                    self.process_using_the_rules_for(
                        self.insertion_mode,
                        token.to_owned(),
                    );
                }

                // Otherwise
                //
                // Traiter le jeton selon les règles indiquées dans la
                // section relative à l'analyse syntaxique des jetons dans
                // le contenu étranger.
                | Some(token) => self
                    .process_using_the_rules_for_foreign_content(
                        token.to_owned(),
                    ),
            }

            if self.stop_parsing {
                break;
            }
        }
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#using-the-rules-for>
    fn process_using_the_rules_for(
        &mut self,
        m: InsertionMode,
        token: HTMLToken,
    ) {
        match dd!(&m) {
            | InsertionMode::Initial => {
                self.handle_initial_insertion_mode(token)
            }
            | InsertionMode::BeforeHTML => {
                self.handle_before_html_insertion_mode(token)
            }
            | InsertionMode::BeforeHead => {
                self.handle_before_head_insertion_mode(token)
            }
            | InsertionMode::InHead => {
                self.handle_in_head_insertion_mode(token)
            }
            | InsertionMode::InHeadNoscript => todo!(),
            | InsertionMode::AfterHead => todo!(),
            | InsertionMode::InBody => todo!(),
            | InsertionMode::Text => todo!(),
            | InsertionMode::InTable => todo!(),
            | InsertionMode::InTableText => todo!(),
            | InsertionMode::InCaption => todo!(),
            | InsertionMode::InColumnGroup => todo!(),
            | InsertionMode::InTableBody => todo!(),
            | InsertionMode::InRow => todo!(),
            | InsertionMode::InCell => todo!(),
            | InsertionMode::InSelect => todo!(),
            | InsertionMode::InSelectInTable => todo!(),
            | InsertionMode::InTemplate => todo!(),
            | InsertionMode::AfterBody => todo!(),
            | InsertionMode::InFrameset => todo!(),
            | InsertionMode::AfterFrameset => todo!(),
            | InsertionMode::AfterAfterBody => todo!(),
            | InsertionMode::AfterAfterFrameset => todo!(),
        }
    }

    fn process_using_the_rules_for_foreign_content(
        &mut self,
        token: HTMLToken,
    ) {
        match token {
            // A character token that is U+0000 NULL
            //
            // Erreur d'analyse. Insérer un caractère U+FFFD REPLACEMENT
            // CHARACTER.
            | HTMLToken::Character('\0') => {
                self.parse_error(token);
                self.insert_character(char::REPLACEMENT_CHARACTER);
            }

            // U+0009 CHARACTER TABULATION
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+000D CARRIAGE RETURN (CR)
            // U+0020 SPACE
            //
            // Insérer le caractère du jeton.
            | HTMLToken::Character(ch) if ch.is_ascii_whitespace() => {
                self.insert_character(ch);
            }

            // Any other character token
            //
            // Insérer le caractère du jeton.
            | HTMLToken::Character(ch) => {
                self.insert_character(ch);
                self.frameset_ok = false;
            }

            // A comment token
            //
            // Insérer le commentaire.
            | HTMLToken::Comment(comment) => self.insert_comment(comment),

            // A DOCTYPE token
            //
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::DOCTYPE(_) => {
                self.parse_error(token);
                /* ignore */
            }

            // A start tag whose tag name is one of:
            //   - "b", "big", "blockquote", "body", "br", "center",
            //     "code", "dd", "div", "dl", "dt", "em", "embed", "h1",
            //     "h2", "h3", "h4", "h5", "h6", "head", "hr", "i", "img",
            //     "li", "listing", "menu", "meta", "nobr", "ol", "p",
            //     "pre", "ruby", "s", "small", "span", "strong", "strike",
            //     "sub", "sup", "table", "tt", "u", "ul", "var"
            // A start tag whose tag name is "font", if the token has any
            // attributes named "color", "face", or "size"
            // An end tag whose tag name is "br", "p"
            //
            // Erreur d'analyse.
            // Si le nœud actuel n'est pas un point d'intégration de texte
            // MathML, un point d'intégration HTML ou un élément de
            // l'espace de noms HTML, il faut extraire les éléments de la
            // pile des éléments ouverts.
            // Retraiter le jeton selon les règles données dans la section
            // correspondant au mode d'insertion actuel dans le contenu
            // HTML.
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name, is_end, ..
                },
            ) if (!is_end
                && (name.is_one_of([
                    tag_names::b,
                    tag_names::big,
                    tag_names::blockquote,
                    tag_names::body,
                    tag_names::br,
                    tag_names::center,
                    tag_names::code,
                    tag_names::dd,
                    tag_names::div,
                    tag_names::dl,
                    tag_names::dt,
                    tag_names::em,
                    tag_names::embed,
                    tag_names::h1,
                    tag_names::h2,
                    tag_names::h3,
                    tag_names::h4,
                    tag_names::h5,
                    tag_names::h6,
                    tag_names::head,
                    tag_names::hr,
                    tag_names::i,
                    tag_names::img,
                    tag_names::li,
                    tag_names::listing,
                    tag_names::menu,
                    tag_names::meta,
                    tag_names::nobr,
                    tag_names::ol,
                    tag_names::p,
                    tag_names::pre,
                    tag_names::ruby,
                    tag_names::s,
                    tag_names::small,
                    tag_names::span,
                    tag_names::strong,
                    tag_names::strike,
                    tag_names::sub,
                    tag_names::sup,
                    tag_names::table,
                    tag_names::tt,
                    tag_names::u,
                    tag_names::ul,
                    tag_names::var,
                ]) || tag_names::font == name
                    && tag_token.has_attributes([
                        tag_attributes::color,
                        tag_attributes::face,
                        tag_attributes::size,
                    ])))
                || is_end
                    && name.is_one_of([tag_names::br, tag_names::p]) =>
            {
                self.parse_error(token);

                while !self
                    .current_node()
                    .is_mathml_text_integration_point()
                    && !self
                        .current_node()
                        .is_html_text_integration_point()
                    && !self.current_node().is_in_html_namespace()
                {
                    self.stack_of_open_elements.pop();
                }

                todo!()
            }

            | _ => todo!(),
        }
    }

    /// Le noeud courant ajusté est l'élément de contexte si l'analyseur a
    /// été créé dans le cadre de l'algorithme d'analyse des fragments HTML
    /// et que la pile d'éléments ouverts ne contient qu'un seul élément
    /// (cas du fragment) ; sinon, le noeud courant ajusté est le noeud
    /// courant.
    fn adjusted_current_node(&self) -> &TreeNode<Node> {
        if self.parsing_fragment && self.stack_of_open_elements.len() == 1
        {
            self.context_element.as_ref().expect("Context Element")
        } else {
            self.current_node()
        }
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#create-an-element-for-the-token>
    /// todo: FIXME
    fn create_element_for(
        &mut self,
        token: &HTMLTagToken,
        namespace: Namespace,
        intended_parent: Option<&TreeNode<Node>>,
    ) -> Option<TreeNode<Node>> {
        let HTMLTagToken {
            name: local_name,
            attributes,
            ..
        } = token;

        let document = intended_parent.unwrap_or(&self.document);

        let maybe_element = Document::create_element(local_name, None);

        if let Ok(element) = maybe_element.as_ref() {
            element.set_document(document);

            attributes.iter().for_each(|attribute| {
                element
                    .element_ref()
                    .borrow_mut()
                    .set_attribute(&attribute.0, &attribute.1);
            });
        }

        maybe_element.ok()
    }

    /// Le nœud actuel est le nœud le plus bas de cette pile d'éléments
    /// ouverts.
    fn current_node(&self) -> &TreeNode<Node> {
        self.stack_of_open_elements
            .current_node()
            .expect("Le noeud actuel")
    }

    /// L'endroit approprié pour insérer un nœud, en utilisant
    /// éventuellement une cible prioritaire particulière, est la position
    /// dans un élément renvoyé par l'exécution des étapes suivantes :
    ///
    /// 1. Si une cible prioritaire a été spécifiée, alors la cible est la
    /// cible prioritaire.
    ///
    /// 2. Déterminer l'emplacement d'insertion ajusté en utilisant les
    /// premières étapes de correspondance de la liste suivante :
    ///
    ///    2.1. Si le `foster parenting` est activée et que la cible est
    /// un élément table, tbody, tfoot, thead ou tr.
    ///
    ///    Note: Le `foster parenting` se produit lorsque le contenu est
    /// mal intégré dans les table's.
    ///
    ///      2.1.1. Le dernier template est le dernier élément template
    /// dans la pile d'éléments ouverts, s'il y en a.
    ///
    ///      2.1.2. Le dernier table est le dernier élément table dans la
    /// pile des éléments ouverts, s'il y en a.
    ///
    ///      2.1.3. S'il y en a un dernier template et qu'il n'y a pas de
    /// dernière table, ou s'il y en a une, mais que le dernier template
    /// est plus bas (plus récemment ajouté) que la dernière table dans la
    /// pile des éléments ouverts, alors : laissez l'emplacement
    /// d'insertion ajusté à l'intérieur du contenu du template du dernier
    /// template, après son dernier enfant (s'il y en a), et abandonnez ces
    /// étapes.
    ///
    ///      2.1.4. S'il n'y a pas de dernier table, alors l'emplacement
    /// d'insertion ajusté se trouve à l'intérieur du premier élément de la
    /// pile d'éléments ouverts (l'élément html), après son dernier enfant
    /// (s'il y en a un), et on abandonne ces étapes. (cas d'un fragment)
    ///
    ///      2.1.5. Si la dernière table a un noeud parent, alors
    /// l'emplacement d'insertion ajusté sera à l'intérieur du noeud parent
    /// de la dernière table, immédiatement avant la dernière
    ///        table, et annulera ces étapes.
    ///
    ///      2.1.6. Laisser "l'élément précédent" être l'élément
    /// directement au-dessus de la dernière table dans la pile des
    /// éléments ouverts.
    ///
    ///      2.1.7. Que l'emplacement d'insertion ajusté soit à
    /// l'intérieur de l'élément précédent, après son dernier enfant (le
    /// cas échéant).
    ///
    ///    Note: Ces étapes sont nécessaires en partie parce qu'il est
    /// possible que des éléments, en particulier l'élément table dans ce
    /// cas, aient été déplacés par un script dans le DOM, ou même
    /// entièrement retirés du DOM, après que l'élément ait été inséré par
    /// l'analyseur.
    ///
    ///    2.2. Sinon : l'emplacement d'insertion ajusté doit être à
    /// l'intérieur de la cible, après son dernier enfant (s'il y en a).
    ///
    /// 3. Si l'emplacement d'insertion ajusté se trouve à l'intérieur d'un
    /// élément template, il doit plutôt se trouver à l'intérieur du
    /// contenu template de l'élément template, après son dernier enfant
    /// (s'il y en a).
    ///
    /// 4. Retourner l'emplacement d'insertion ajusté.
    fn find_appropriate_place_for_inserting_node(
        &self,
        override_target: Option<&TreeNode<Node>>,
    ) -> AdjustedInsertionLocation {
        let maybe_target =
            override_target.or_else(|| Some(self.current_node()));

        let mut adjusted_insertion_location = AdjustedInsertionLocation {
            insert_before_sibling: None,
            parent: None,
        };

        if self.foster_parenting
            && [
                tag_names::table,
                tag_names::tbody,
                tag_names::tfoot,
                tag_names::thead,
                tag_names::tr,
            ]
            .into_iter()
            .any(|local_name| {
                if let Some(target) = maybe_target {
                    local_name == target.element_ref().local_name()
                } else {
                    false
                }
            })
        {
            let last_template = self
                .stack_of_open_elements
                .get_last_element_with_tag_name(tag_names::template);
            let last_table = self
                .stack_of_open_elements
                .get_last_element_with_tag_name(tag_names::table);

            if let Some((template_index, template)) = last_template {
                fn return_adjusted_insertion_location(
                    template: &TreeNode<Node>,
                ) -> AdjustedInsertionLocation {
                    let tc = template
                        .element_ref()
                        .content()
                        .map(|t| t.to_owned());
                    AdjustedInsertionLocation {
                        parent: tc,
                        insert_before_sibling: None,
                    }
                }

                if last_table.is_none() {
                    return return_adjusted_insertion_location(template);
                }

                if let Some((table_index, _)) = last_table {
                    if template_index > table_index {
                        return return_adjusted_insertion_location(
                            template,
                        );
                    }
                }
            }

            if last_table.is_none() {
                assert!(self.parsing_fragment);

                return AdjustedInsertionLocation {
                    parent: self.stack_of_open_elements.first().cloned(),
                    insert_before_sibling: None,
                };
            }

            if let Some((table_index, table)) = last_table {
                let parent = table.parent_node();
                if let Some(node) = parent {
                    adjusted_insertion_location.parent = node.into();
                    adjusted_insertion_location
                        .insert_before_sibling
                        .replace(table.to_owned());
                } else {
                    let previous_element = self
                        .stack_of_open_elements
                        .element_immediately_above(table_index);
                    adjusted_insertion_location.parent =
                        previous_element.cloned();
                }
            }
        } else {
            adjusted_insertion_location = AdjustedInsertionLocation {
                parent: maybe_target.cloned(),
                insert_before_sibling: None,
            };
        }

        adjusted_insertion_location
    }

    fn find_character_insertion_node(&self) -> Option<TreeNode<Node>> {
        let adjusted_insertion_location =
            self.find_appropriate_place_for_inserting_node(None);

        if adjusted_insertion_location.insert_before_sibling.is_some() {
            todo!()
        }

        let parent = adjusted_insertion_location.parent?;

        if parent.is_document() {
            return None;
        }

        let is_text = parent
            .get_last_child()
            .as_ref()
            .filter(|last_child| last_child.is_text())
            .cloned();

        if is_text.is_some() {
            return is_text;
        }

        let new_text_node = TextNode::new(&self.document, String::new());
        parent.append_child(new_text_node.to_owned());
        Some(new_text_node.to_owned())
    }

    fn flush_character_insertions(&mut self) {
        if self.character_insertion_builder.is_empty() {
            return;
        }

        if let Some(character_insertion_node) =
            self.character_insertion_node.as_ref()
        {
            character_insertion_node
                .set_data(&self.character_insertion_builder);
            self.character_insertion_builder.clear();
        }
    }

    /// Lorsque les étapes ci-dessous exigent que l'UA génère de manière
    /// exhaustive toutes les balises de fin implicites, alors, si le noeud
    /// actuel est un élément caption, un élément colgroup, un élément dd,
    /// un élément dt, un élément li, un élément optgroup, un élément
    /// option, un élément p, un élément rb, un élément rp, un élément rt,
    /// un élément rtc, un élément tbody, un élément td, un élément tfoot,
    /// un élément th, un élément thead ou un élément tr, l'UA doit retirer
    /// le noeud actuel de la pile des éléments ouverts.
    fn generate_all_implied_end_tags_thoroughly(&mut self) {
        while self.current_node().element_ref().local_name().is_one_of([
            tag_names::caption,
            tag_names::colgroup,
            tag_names::dd,
            tag_names::dt,
            tag_names::li,
            tag_names::optgroup,
            tag_names::option,
            tag_names::p,
            tag_names::rb,
            tag_names::rp,
            tag_names::rt,
            tag_names::rtc,
            tag_names::tbody,
            tag_names::td,
            tag_names::tfoot,
            tag_names::th,
            tag_names::thead,
            tag_names::tr,
        ]) {
            self.stack_of_open_elements.pop();
        }
    }

    /// L'algorithme générique d'analyse syntaxique des éléments de
    /// texte brut et l'algorithme générique d'analyse syntaxique
    /// des éléments RCDATA comportent les étapes suivantes. Ces
    /// algorithmes sont toujours invoqués en réponse à un jeton de
    /// balise de début.
    ///
    ///   1. Insertion d'un élément HTML pour le jeton.
    ///   2. Si l'algorithme invoqué est l'algorithme générique
    /// d'analyse syntaxique des éléments de texte brut, faire
    /// passer le tokenizer à l'état RAWTEXT ; sinon, si
    /// l'algorithme invoqué est l'algorithme générique d'analyse
    /// syntaxique des éléments RCDATA, faire passer le tokenizer à
    /// l'état RCDATA.
    ///   3. Le mode d'insertion d'origine est le mode d'insertion
    /// actuel.
    ///   4. Ensuite, faire passer le mode d'insertion à "text".
    fn parse_generic_element(
        &mut self,
        tag_token: &HTMLTagToken,
        state: State,
    ) {
        self.insert_html_element(tag_token);
        self.tokenizer.switch_state_to(state.to_string());
        self.original_insertion_mode.switch_to(self.insertion_mode);
        self.insertion_mode.switch_to(InsertionMode::Text);
    }

    // <https://html.spec.whatwg.org/multipage/parsing.html#insert-a-character>
    fn insert_character(&mut self, ch: CodePoint) {
        let maybe_node = self.find_character_insertion_node();

        if maybe_node == self.character_insertion_node {
            self.character_insertion_builder.push(ch);
            return;
        }

        if self.character_insertion_node.is_none() {
            self.character_insertion_node = maybe_node;
            self.character_insertion_builder.push(ch);
            return;
        }

        self.flush_character_insertions();
        self.character_insertion_node = maybe_node;
        self.character_insertion_builder.push(ch);
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#insert-a-comment>
    fn insert_comment(&self, comment: String) {
        let mut adjusted_insertion_location =
            self.find_appropriate_place_for_inserting_node(None);

        let comment = CommentNode::new(&self.document, comment);

        if let Some(ref mut parent) = adjusted_insertion_location.parent {
            parent.insert_before(
                comment.to_owned(),
                adjusted_insertion_location.insert_before_sibling.as_ref(),
            );
        }
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#insert-an-html-element>
    fn insert_html_element(
        &mut self,
        token: &HTMLTagToken,
    ) -> Option<TreeNode<Node>> {
        self.insert_foreign_element(token, Namespace::HTML)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#insert-a-foreign-element>
    fn insert_foreign_element(
        &mut self,
        token: &HTMLTagToken,
        namespace: Namespace,
    ) -> Option<TreeNode<Node>> {
        let adjusted_insertion_location =
            self.find_appropriate_place_for_inserting_node(None);

        let maybe_element =
            self.create_element_for(token, namespace, None);

        if let Some(element) = maybe_element.as_ref() {
            self.stack_of_open_elements.put(element.to_owned());
            if let Some(parent) = adjusted_insertion_location.parent {
                if let Some(sibling) = adjusted_insertion_location
                    .insert_before_sibling
                    .as_ref()
                {
                    parent
                        .insert_before(element.to_owned(), Some(sibling));
                    return maybe_element;
                }

                parent.append_child(element.to_owned());
            }
        }

        maybe_element
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parse-errors>
    fn parse_error(&self, token: HTMLToken) {
        match token {
            | HTMLToken::Tag(HTMLTagToken { name, is_end, .. }) => {
                if is_end {
                    log::error!("Balise de fin inattendue: {name}");
                } else {
                    log::error!("Balise de début inattendue: {name}");
                }
            }
            | HTMLToken::DOCTYPE(_) => log::error!("DOCTYPE inattendu"),
            | HTMLToken::Comment(_) => {
                log::error!("Commentaire inattendu")
            }
            | HTMLToken::Character(_) => {
                log::error!("Caractère inattendu")
            }
            | HTMLToken::EOF => log::error!("End Of File: inattendu"),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#reset-the-insertion-mode-appropriately>
    fn reset_insertion_mode_appropriately(&mut self) {
        for (index, node) in self
            .stack_of_open_elements
            .elements
            .iter()
            .enumerate()
            .rev()
        {
            let last = index == 0;

            let node = if last && self.parsing_fragment {
                self.context_element.clone().unwrap()
            } else {
                node.clone()
            };

            let element = node.element_ref();

            if tag_names::select == element.local_name() {
                for ancestor in self.stack_of_open_elements.elements
                    [0..index]
                    .iter()
                    .rev()
                {
                    let ancestor_tag_name =
                        ancestor.element_ref().local_name();
                    if ancestor_tag_name == "template" {
                        self.insertion_mode
                            .switch_to(InsertionMode::InSelect);
                        return;
                    } else if ancestor_tag_name == "table" {
                        self.insertion_mode
                            .switch_to(InsertionMode::InSelectInTable);
                        return;
                    }
                }
                self.insertion_mode.switch_to(InsertionMode::InSelect);
                return;
            }

            if element
                .local_name()
                .is_one_of([tag_names::td, tag_names::th])
                && !last
            {
                self.insertion_mode.switch_to(InsertionMode::InCell);
                return;
            }

            if tag_names::tr == element.local_name() {
                self.insertion_mode.switch_to(InsertionMode::InRow);
                return;
            }

            if element.local_name().is_one_of([
                tag_names::tbody,
                tag_names::thead,
                tag_names::tfoot,
            ]) && !last
            {
                self.insertion_mode.switch_to(InsertionMode::InTableBody);
                return;
            }

            if tag_names::caption == element.local_name() {
                self.insertion_mode.switch_to(InsertionMode::InCaption);
                return;
            }

            if tag_names::colgroup == element.local_name() {
                self.insertion_mode
                    .switch_to(InsertionMode::InColumnGroup);
                return;
            }

            if tag_names::table == element.local_name() {
                self.insertion_mode.switch_to(InsertionMode::InTable);
                return;
            }

            if tag_names::template == element.local_name() {
                self.insertion_mode.switch_to(
                    *self
                        .stack_of_template_insertion_modes
                        .last()
                        .unwrap(),
                );
                return;
            }

            if tag_names::head == element.local_name() {
                self.insertion_mode.switch_to(InsertionMode::InHead);
                return;
            }

            if tag_names::body == element.local_name() {
                self.insertion_mode.switch_to(InsertionMode::InBody);
                return;
            }

            if tag_names::frameset == element.local_name() {
                self.insertion_mode.switch_to(InsertionMode::InFrameset);
                return;
            }

            if tag_names::html == element.local_name() {
                if self.head_element.is_none() {
                    self.insertion_mode
                        .switch_to(InsertionMode::BeforeHead);
                    return;
                }

                self.insertion_mode.switch_to(InsertionMode::AfterHead);
                return;
            }
        }

        assert!(self.parsing_fragment);
        self.insertion_mode.switch_to(InsertionMode::InBody);
    }
}

impl<C> HTMLParser<C>
where
    C: Iterator<Item = CodePoint>,
{
    // TODO: si le document n'est pas un document iframe srcdoc
    fn handle_initial_insertion_mode(&mut self, token: HTMLToken) {
        match token {
            // U+0009 CHARACTER TABULATION
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+000D CARRIAGE RETURN (CR)
            // U+0020 SPACE
            //
            // Ignorer le jeton.
            | HTMLToken::Character(ch) if ch.is_ascii_whitespace() => {
                /* ignore */
            }

            // A comment token
            //
            // Insérer un commentaire comme dernier enfant de l'objet
            // [Document].
            | HTMLToken::Comment(comment) => {
                self.document.insert_comment(comment);
            }

            // A DOCTYPE token
            //
            // Si le nom du [HTMLToken::DOCTYPE] n'est pas "html", ou si
            // l'identificateur public du token n'est pas manquant, ou si
            // l'identificateur système du token n'est ni manquant ni
            // "about:legacy-compat", alors il y a une erreur d'analyse.
            //
            // Ajouter un noeud [DocumentType] au noeud [Document], avec
            // son nom défini comme le nom donné dans le
            // [HTMLToken::DOCTYPE], ou la chaîne de caractères vide si le
            // nom est manquant ; son ID public défini comme l'identifiant
            // public donné dans le [HTMLToken::DOCTYPE], ou la chaîne de
            // caractères vide si l'identifiant public est manquant ; et
            // son ID système défini comme l'identifiant système donné dans
            // le [HTMLToken::DOCTYPE], ou la chaîne de caractères vide si
            // l'identifiant système est manquant.
            //
            // Note: cela garantit également que le noeud [DocumentType]
            // est renvoyé comme valeur de l'attribut doctype de l'objet
            // [Document].
            //
            // Ensuite, si le document n'est pas un document iframe srcdoc,
            // et que l'analyseur syntaxique ne peut pas changer le drapeau
            // de mode est faux, et que le token DOCTYPE correspond à l'une
            // des conditions de la liste suivante, alors définir le
            // document en mode quirks :
            //   - Le drapeau force-quirks est activé.
            //   - Le nom du doctype n'est pas "html".
            //   - L'identifiant public est défini à l'une des entrées du
            //     tableau [HTMLDoctypeToken::PUBLIC_ID_DEFINED_RULE_1]
            //   - L'identifiant système est défini à l'une des entrées du
            //     tableau [HTMLDoctypeToken::SYSTEM_ID_DEFINED_RULE_1]
            //   - L'identifiant public commence par l'une des entrées du
            //     tableau [HTMLDoctypeToken::PUBLIC_ID_STARTS_WITH_RULE_1]
            //   - L'identifiant système commence par l'une des entrées du
            //     tableau [HTMLDoctypeToken::SYSTEM_ID_STARTS_WITH_RULE_1]
            //
            // Sinon, si le document n'est pas un document iframe srcdoc,
            // que l'analyseur syntaxique ne peut pas modifier le drapeau
            // de mode est faux, et que le jeton DOCTYPE correspond à l'une
            // des conditions de la liste suivante, le document est alors
            // défini en mode limited-quirks :
            //   - L'identifiant public commence par l'une des entrées du
            //     tableau [HTMLDoctypeToken::PUBLIC_ID_DEFINED_RULE_2]
            //   - L'identifiant système n'est pas manquant et l'identifier
            //     public commence par l'une des entrées du tableau
            //     [HTMLDoctypeToken::PUBLIC_ID_DEFINED_RULE_2_1]
            //
            // Les chaînes de l'identifiant système et de l'identifiant
            // public doivent être comparées aux valeurs indiquées dans les
            // listes ci-dessus de manière insensible à la casse ASCII. Un
            // identifiant de système dont la valeur est la chaîne de
            // caractères vide n'est pas considéré comme manquant aux fins
            // des conditions ci-dessus.
            //
            // Ensuite, passer le mode d'insertion à "before html".
            | HTMLToken::DOCTYPE(ref doctype_data) => {
                let is_parse_error = !doctype_data.is_html_name()
                    || !doctype_data.is_public_identifier_missing()
                    || !doctype_data.is_system_identifier_missing()
                    || !doctype_data.is_about_legacy_compat();

                if is_parse_error {
                    self.parse_error(token);
                    return;
                }

                let mut doctype =
                    DocumentType::new(doctype_data.name.as_ref());

                doctype.set_public_id(
                    doctype_data.public_identifier.as_ref(),
                );
                doctype.set_system_id(
                    doctype_data.system_identifier.as_ref(),
                );

                self.document
                    .get_mut()
                    .set_doctype(doctype)
                    .set_quirks_mode(doctype_data.quirks_mode());
                self.insertion_mode.switch_to(InsertionMode::BeforeHTML);
            }

            // Anything else
            //
            // Si le document n'est pas un document iframe srcdoc, il
            // s'agit d'une erreur d'analyse syntaxique ; si l'analyseur
            // syntaxique ne peut pas changer le drapeau de mode est faux,
            // mettez le document en mode quirks. Dans tous les
            // cas, passez le mode d'insertion à "before html", puis
            // retraitez le jeton.
            | _ => {
                self.parse_error(token);
                self.document.get_mut().set_quirks_mode(QuirksMode::Yes);
                self.insertion_mode.switch_to(InsertionMode::BeforeHTML);
            }
        }
    }

    fn handle_before_html_insertion_mode(&mut self, token: HTMLToken) {
        match token {
            // A DOCTYPE token
            //
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::DOCTYPE(_) => {
                self.parse_error(token);
                /* ignore */
            }

            // A comment token
            //
            // Insérer un commentaire comme dernier enfant de l'objet
            // [Document].
            | HTMLToken::Comment(comment) => {
                self.document.insert_comment(comment);
            }

            // U+0009 CHARACTER TABULATION
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+000D CARRIAGE RETURN (CR)
            // U+0020 SPACE
            //
            // Ignorer le jeton.
            | HTMLToken::Character(ch) if ch.is_ascii_whitespace() => {
                /* ignore */
            }

            // Une balise de départ dont le nom de balise est: "html"
            //
            // Créer un élément pour le jeton dans l'espace de noms HTML,
            // avec le [Document] comme parent prévu. L'ajouter à l'objet
            // [Document]. Placer l'élément dans la pile des éléments
            // ouverts.
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if tag_names::html == name => {
                let element = self
                    .create_element_for(tag_token, Namespace::HTML, None)
                    .expect("Un élément DOM HTMLHtmlElement");
                self.document.append_child(element.clone());
                self.stack_of_open_elements.put(element);
                self.insertion_mode.switch_to(InsertionMode::BeforeHead);
            }

            // Une balise de fin dont le nom de balise est l'un des
            // éléments suivants: "head", "body", "html", "br".
            // Agir comme décrit dans l'entrée "Anything else" ci-dessous.
            //
            // Toute autre nom de balise de fin:
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if !name.is_one_of([
                tag_names::head,
                tag_names::body,
                tag_names::html,
                tag_names::br,
            ]) =>
            {
                self.parse_error(token);
            }

            // Anything else
            //
            // Créer un élément html dont le document node est l'objet
            // [Document]. L'ajouter à l'objet [Document]. Placer cet
            // élément dans la pile des éléments ouverts.
            //
            // Passer le mode d'insertion à "before head", puis retraiter
            // le jeton.
            | _ => {
                let element = Document::create_element(
                    tag_names::html.to_string(),
                    None,
                )
                .expect("Un élément DOM HTMLHtmlElement");
                element.set_document(self.document.deref());
                self.document.append_child(element.clone());
                self.stack_of_open_elements.put(element);
                self.insertion_mode.switch_to(InsertionMode::BeforeHead);
                self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }
        }
    }

    fn handle_before_head_insertion_mode(&mut self, token: HTMLToken) {
        match token {
            // U+0009 CHARACTER TABULATION
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+000D CARRIAGE RETURN (CR)
            // U+0020 SPACE
            //
            // Ignorer le jeton.
            | HTMLToken::Character(ch) if ch.is_ascii_whitespace() => {
                /* ignore */
            }

            // A comment token
            | HTMLToken::Comment(comment) => {
                self.insert_comment(comment);
            }

            // A DOCTYPE token
            //
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::DOCTYPE(_) => self.parse_error(token),

            // A start tag whose tag name is "html"
            //
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in body".
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if tag_names::html == name => {
                self.process_using_the_rules_for(
                    InsertionMode::InBody,
                    token,
                );
            }

            // A start tag whose tag name is "head"
            //
            // Insérer un élément HTML pour le jeton.
            // Placer le pointeur de l'élément head sur le nouvel élément
            // head fraîchement créé.
            // Passer le mode d'insertion à "in head".
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if tag_names::head == name => {
                let head_element = self.insert_html_element(tag_token);
                self.head_element = head_element;
                self.insertion_mode.switch_to(InsertionMode::InHead);
            }

            // Une balise de fin dont le nom de balise est l'un des
            // éléments suivants: "head", "body", "html", "br".
            // Agir comme décrit dans l'entrée "Anything else" ci-dessous.
            //
            // Toute autre nom de balise de fin:
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if !name.is_one_of([
                tag_names::head,
                tag_names::body,
                tag_names::html,
                tag_names::br,
            ]) =>
            {
                self.parse_error(token);
            }

            // Anything else
            //
            // Insérer un élément HTML pour un jeton de balise de début
            // "head" sans attributs.
            // Placer le pointeur de l'élément "head" sur l'élément "head"
            // fraîchement créé.
            // Passer le mode d'insertion à "in head".
            // Retraiter le jeton en cours.
            | _ => {
                let head_element =
                    HTMLTagToken::start().with_name(tag_names::head);
                self.head_element =
                    self.insert_html_element(&head_element);
                self.insertion_mode.switch_to(InsertionMode::InHead);
                self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }
        }
    }

    fn handle_in_head_insertion_mode(&mut self, token: HTMLToken) {
        match token {
            // U+0009 CHARACTER TABULATION
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+000D CARRIAGE RETURN (CR)
            // U+0020 SPACE
            //
            // Insérer un caractère.
            | HTMLToken::Character(ch) if ch.is_ascii_whitespace() => {
                self.insert_character(ch);
            }

            // A comment token
            //
            // Insérer un commentaire.
            | HTMLToken::Comment(comment) => {
                self.insert_comment(comment);
            }

            // A DOCTYPE token
            //
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::DOCTYPE(_) => {
                self.parse_error(token);
                /* ignore */
            }

            // A start tag whose tag name is "html"
            //
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in body".
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if tag_names::html == name => {
                self.process_using_the_rules_for(
                    InsertionMode::InBody,
                    token,
                );
            }

            // A start tag whose tag name is one of:
            //   "base", "basefont", "bgsound", "link"
            //
            // Insérer un élément HTML pour le jeton. Retirer immédiatement
            // le noeud actuel de la pile des éléments ouverts.
            // Accuser la réception du drapeau d'auto-fermeture du jeton,
            // s'il est activé.
            | HTMLToken::Tag(mut tag_token)
                if token.is_start_tag()
                    && tag_token.name.is_one_of([
                        tag_names::base,
                        tag_names::basefont,
                        tag_names::bgsound,
                        tag_names::link,
                    ]) =>
            {
                self.insert_html_element(&tag_token);
                self.stack_of_open_elements.pop();
                tag_token.set_acknowledge_self_closing_flag();
            }

            // A start tag whose tag name is "meta"
            //
            // Insérer un élément HTML pour le jeton. Retirer immédiatement
            // le noeud actuel de la pile des éléments ouverts.
            // Accuser réception du drapeau d'auto-fermeture du jeton, s'il
            // est activé.
            //
            // TODO:
            // Si l'analyseur HTML spéculatif actif est nul, alors :
            //
            //   1. Si l'élément possède un attribut charset, et que
            // l'obtention d'un encodage à partir de sa valeur donne un
            // encodage, et que la confiance est actuellement provisoire,
            // alors changez l'encodage pour l'encodage résultant.
            //
            //   2. Sinon, si l'élément possède un attribut http-equiv dont
            // la valeur est une correspondance ASCII insensible à la casse
            // pour la chaîne "Content-Type", et que l'élément possède un
            // attribut content, et que l'application de l'algorithme
            // d'extraction d'un encodage de caractères d'un méta-élément à
            // la valeur de cet attribut renvoie un encodage, et que la
            // confiance est actuellement provisoire, alors nous devons
            // changer l'encodage pour l'encodage extrait.
            //
            // Note: L'analyseur HTML spéculatif n'applique pas de manière
            // spéculative les déclarations de codage des caractères afin
            // de réduire la complexité de l'implémentation.
            | HTMLToken::Tag(mut tag_token)
                if token.is_start_tag()
                    && tag_names::meta == tag_token.name =>
            {
                self.insert_html_element(&tag_token);
                self.stack_of_open_elements.pop();
                tag_token.set_acknowledge_self_closing_flag();
            }

            // A start tag whose tag name is "title"
            //
            // Suivre l'algorithme générique d'analyse syntaxique de
            // l'élément RCDATA.
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if tag_names::title == name => {
                self.parse_generic_element(tag_token, State::RCDATA);
            }

            // A start tag whose tag name is "noscript", if the scripting
            // flag is enabled.
            // A start tag whose tag name is one of: "noframes", "style".
            //
            // Suivre l'algorithme générique d'analyse syntaxique des
            // éléments de texte brut (RAWTEXT).
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if (tag_names::noscript == name
                && self.scripting_enabled)
                || name.is_one_of([
                    tag_names::noframes,
                    tag_names::style,
                ]) =>
            {
                self.parse_generic_element(tag_token, State::RAWTEXT);
            }

            // A start tag whose tag name is "noscript", if the scripting
            // flag is disabled
            //
            // Insérer un élément HTML pour le jeton.
            // Passer le mode d'insertion à "in head noscript".
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if tag_names::noscript == name
                && !self.scripting_enabled =>
            {
                self.insert_html_element(tag_token);
                self.insertion_mode
                    .switch_to(InsertionMode::InHeadNoscript);
            }

            // A start tag whose tag name is "script"
            //
            //   1. Laisser l'emplacement d'insertion ajusté l'endroit
            // approprié pour insérer un nœud.
            //   2. Créer un élément pour le jeton dans l'espace de noms
            // HTML, le parent prévu étant l'élément dans lequel se trouve
            // l'emplacement d'insertion ajusté.
            //  3. Définir le document de l'analyseur de l'élément comme
            // étant le Document, et désactive l'indicateur "non-bloquant"
            // de l'élément.
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if tag_names::script == name => {
                let mut adjusted_insertion_location =
                    self.find_appropriate_place_for_inserting_node(None);

                let element = self
                    .create_element_for(
                        tag_token,
                        Namespace::HTML,
                        adjusted_insertion_location.parent.as_ref(),
                    )
                    .expect("Un élément HTMLScriptElement");

                let script_element = element
                    .script_ref()
                    .borrow_mut()
                    .set_parser_document(&self.document)
                    .set_non_blocking(false);

                if self.parsing_fragment {
                    script_element.set_already_started(true);
                }

                // todo(document.write/ln): Si l'analyseur syntaxique a été
                // invoqué par l'intermédiaire des méthodes
                // document.write() ou document.writeln(), il est possible
                // de marquer l'élément script comme "déjà lancé". (Par
                // exemple, l'agent utilisateur peut utiliser cette clause
                // pour empêcher l'exécution de scripts d'origine croisée
                // insérés via document.write() dans des conditions de
                // réseau lent, ou lorsque le chargement de la page a déjà
                // pris beaucoup de temps).

                if let Some(ref mut parent) =
                    adjusted_insertion_location.parent
                {
                    parent.insert_before(
                        element.to_owned(),
                        adjusted_insertion_location
                            .insert_before_sibling
                            .as_ref(),
                    );
                }

                self.stack_of_open_elements.put(element);
                let token_state = State::ScriptData;
                self.tokenizer.switch_state_to(token_state.to_string());
                self.original_insertion_mode
                    .switch_to(self.insertion_mode);
                self.insertion_mode.switch_to(InsertionMode::Text);
            }

            // An end tag whose tag name is "head"
            //
            // Retirer le nœud actuel (qui sera l'élément de tête) de la
            // pile des éléments ouverts.
            // Passer le mode d'insertion sur "après la tête".
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if tag_names::head == name => {
                self.stack_of_open_elements.pop();
                self.insertion_mode.switch_to(InsertionMode::AfterHead);
            }

            // A start tag whose tag name is "template"
            //
            // Insérer un élément HTML pour le jeton.
            // Insérer un marqueur à la fin de la liste des éléments de
            // mise en forme actifs.
            // Définir le drapeau frameset-ok à "not ok".
            // Passer le mode d'insertion sur "in template".
            // Poussez "in template" sur la pile des modes d'insertion
            // de template afin qu'il soit le nouveau mode d'insertion de
            // template actuel.
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if tag_names::template == name => {
                self.insert_html_element(tag_token);
                self.list_of_active_formatting_elements
                    .insert_marker_at_end();
                self.frameset_ok = false;
                self.stack_of_template_insertion_modes
                    .push(InsertionMode::InTemplate);
            }

            // An end tag whose tag name is "template"
            //
            // S'il n'y a pas d'élément de template sur la pile des
            // éléments ouverts, il s'agit d'une erreur d'analyse ; ignorer
            // le jeton.
            //
            // Sinon:
            //   1. générer minutieusement toutes les balises de fin
            // implicites.
            //   2. Si le nœud actuel n'est pas un élément template, il
            // s'agit d'une erreur d'analyse.
            //   3. Extraire des éléments de la pile d'éléments ouverts
            // jusqu'à ce qu'un élément template ait été extrait de la
            // pile.
            //   4. Effacer la liste des éléments de mise en forme actifs
            // jusqu'au dernier marqueur.
            //   5. Supprimer le mode d'insertion template actuel de la
            // pile des modes d'insertion template.
            //   6. Réinitialiser le mode d'insertion de manière
            // appropriée.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if tag_names::template == name => {
                if self.stack_of_template_insertion_modes.is_empty() {
                    self.parse_error(token);
                    return;
                }

                self.generate_all_implied_end_tags_thoroughly();

                let element_name =
                    self.current_node().element_ref().local_name();

                if tag_names::template != element_name {
                    self.parse_error(token);
                }

                self.stack_of_open_elements
                    .pop_until_tag(tag_names::template);
                self.list_of_active_formatting_elements
                    .clear_up_to_the_last_marker();
                self.stack_of_template_insertion_modes.pop();

                self.reset_insertion_mode_appropriately();
            }

            // A start tag whose tag name is "head"
            // Any other end tag
            //
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name, is_end, ..
            }) if (!is_end && tag_names::head == name) || is_end => {
                self.parse_error(token)
            }

            // Anything else
            //
            // Retirer le nœud actuel (qui sera l'élément de tête) de la
            // pile des éléments ouverts.
            | _ => {
                self.stack_of_open_elements.pop();
                self.insertion_mode.switch_to(InsertionMode::AfterHead);
                self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }
        }
    }
}

// ---- //
// Test //
// ---- //

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_document() {
        let document_node = DocumentNode::new();
        let html_file = include_str!("crashtests/site.html.local");
        let mut parser = HTMLParser::new(document_node, html_file.chars());
        parser.run();
    }
}
