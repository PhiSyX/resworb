/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![feature(explicit_generic_args_with_impl_trait, type_name_of_val)]

mod codepoint;
mod error;
mod state;
mod token;
mod tokenizer;

use std::{borrow::BorrowMut, ops::Deref, rc::Rc};

use dom::node::{
    CommentNode, Document, DocumentNode, DocumentType, Node, QuirksMode,
    TextNode,
};
use html_elements::{
    interface::IsOneOfTagsInterface, tag_attributes, tag_names,
};
use infra::{
    self, namespace::Namespace, primitive::codepoint::CodePoint,
    structure::tree::TreeNode,
};
use macros::dd;
use state::ListOfActiveFormattingElements;
use tokenizer::State;

pub use self::{
    state::{Entry, InsertionMode, StackOfOpenElements},
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
    frameset_ok_flag: FramesetOkFlag,
    parsing_fragment: bool,
    scripting_enabled: bool,
    stop_parsing: bool,
    context_element: Option<TreeNode<Node>>,
    character_insertion_node: Option<TreeNode<Node>>,
    character_insertion_builder: String,
    head_element: Option<TreeNode<Node>>,
    form_element: Option<TreeNode<Node>>,
}

struct AdjustedInsertionLocation {
    parent: Option<TreeNode<Node>>,
    insert_before_sibling: Option<TreeNode<Node>>,
}

#[derive(PartialEq)]
enum FramesetOkFlag {
    Ok,
    NotOk,
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
            frameset_ok_flag: FramesetOkFlag::Ok,
            foster_parenting: false,
            parsing_fragment: false,
            scripting_enabled: true,
            stop_parsing: false,
            context_element: None,
            character_insertion_node: None,
            character_insertion_builder: String::new(),
            head_element: None,
            form_element: None,
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
            | InsertionMode::AfterHead => {
                self.handle_after_head_insertion_mode(token)
            }
            | InsertionMode::InBody => {
                self.handle_in_body_insertion_mode(token)
            }
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
                self.frameset_ok_flag = FramesetOkFlag::NotOk;
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
    /// pile des éléments ouverts, alors : nous devons laisser
    /// l'emplacement d'insertion ajusté à l'intérieur du contenu du
    /// template du dernier template, après son dernier enfant (s'il y
    /// en a), et abandonner ces étapes.
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

    /// <https://html.spec.whatwg.org/multipage/parsing.html#generate-implied-end-tags>
    fn generate_implied_end_tags(&mut self) {
        self.generate_implied_end_tags_with_predicate(|name| {
            !name.is_empty()
        });
    }

    fn generate_implied_end_tags_with_predicate(
        &mut self,
        predicate: impl Fn(&str) -> bool,
    ) {
        let node = self.current_node();
        let element = node.element_ref();
        let name = element.local_name();
        while predicate(&name)
            && name.is_one_of([
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
            ])
        {
            self.stack_of_open_elements.pop();
        }
    }

    fn generate_implied_end_tags_except_for(
        &mut self,
        exception: tag_names,
    ) {
        self.generate_implied_end_tags_with_predicate(|name| {
            exception != name
        });
    }

    /// Lorsque les étapes ci-dessous exigent que l'UA génère de manière
    /// exhaustive toutes les balises de fin implicites, alors, si le noeud
    /// actuel est un élément caption, un élément colgroup, un élément dd,
    /// un élément dt, un élément "li", un élément optgroup, un élément
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

    /// Lorsque les étapes ci-dessous exigent que l'UA reconstruise les
    /// éléments de mise en forme actifs, l'UA doit effectuer les étapes
    /// suivantes :
    ///   1. S'il n'y a aucune entrée dans la liste des éléments de
    /// formatage actifs, alors il n'y a rien à reconstruire ; stopper
    /// l'algorithme.
    ///   2. Si la dernière entrée (la plus récemment ajoutée) dans la
    /// liste des éléments de mise en forme actifs est un marqueur, ou si
    /// c'est un élément qui se trouve dans la pile des éléments ouverts,
    /// alors il n'y a rien à reconstruire ; stopper l'algorithme.
    ///   3. Laisser entry être le dernier élément (le plus récemment
    /// ajouté) dans la liste des éléments de formatage actifs.
    ///   4. `rewind` : s'il n'y a aucune entrée avant l'entrée dans la
    /// liste des éléments de mise en forme actifs, nous devons passer à
    /// l'étape intitulée `create`.
    ///   5. Laisser entry être l'entrée antérieure à entry dans la liste
    /// des éléments de mise en forme actifs.
    ///   6. Si l'entrée n'est ni un marqueur ni un élément qui se trouve
    /// également dans la pile des éléments ouverts, nous devons passer à
    /// l'étape intitulée `rewind`.
    ///   7. `advance` : l'entrée est l'élément qui suit l'entrée dans la
    /// liste des éléments de mise en forme actifs.
    ///   8. `create` : insérer un élément HTML pour le jeton pour lequel
    /// l'entrée de l'élément a été créée, pour obtenir un nouvel élément.
    ///   9. Remplacer l'entrée pour l'élément dans la liste par une entrée
    /// pour le nouvel élément.
    ///   10. Si l'entrée pour le nouvel élément dans la liste des éléments
    /// de formatage actifs n'est pas la dernière entrée de la liste,
    /// revener à l'étape intitulée Avancer.
    ///
    /// Cela a pour effet de rouvrir tous les éléments de mise en forme qui
    /// ont été ouverts dans le body, cell ou caption courant (selon le
    /// plus jeune) et qui n'ont pas été explicitement fermés.
    ///
    /// Note: La liste des éléments de formatage actifs est toujours
    /// constituée d'éléments dans l'ordre chronologique, l'élément le
    /// moins récemment ajouté étant le premier et l'élément le plus
    /// récemment ajouté le dernier (sauf pendant l'exécution des étapes 7
    /// à 10 de l'algorithme ci-dessus, bien sûr).
    fn reconstruct_active_formatting_elements(&mut self) {
        if self.list_of_active_formatting_elements.is_empty() {
            return;
        }

        let size = self.list_of_active_formatting_elements.len();

        let (mut entry, mut idx) = if let Some(last) =
            self.list_of_active_formatting_elements.last_mut()
        {
            if last.is_marker() {
                return;
            }

            if let Some(node) = last.element() {
                if self.stack_of_open_elements.contains(node) {
                    return;
                }
            }

            (last, size - 1)
        } else {
            log::info!("Ne devrait jamais tomber dans cette condition.");
            return;
        };

        'main: loop {
            // Rewind
            'rewind: loop {
                if idx == 0 {
                    break 'rewind /* continue in 'create */;
                }

                idx -= 1;
                entry = unsafe {
                    self.list_of_active_formatting_elements
                        .get_unchecked_mut(idx)
                }
                .borrow_mut();

                if !entry.is_marker()
                    && !self
                        .stack_of_open_elements
                        .contains(entry.element_unchecked())
                {
                    continue 'rewind;
                }
            }

            'create: loop {
                let element = self
                    .list_of_active_formatting_elements
                    .get(idx)
                    .and_then(|entry| entry.element())
                    .unwrap_or_else(|| {
                        panic!("L'élément à index {}", idx)
                    });

                let element = {
                    let tag_token = HTMLTagToken::start()
                        .with_name(element.element_ref().local_name());
                    self.insert_html_element(&tag_token)
                }
                .unwrap();

                self.list_of_active_formatting_elements
                    .get(idx)
                    .replace(&Entry::Element(element));

                if idx == size - 1 {
                    break 'create; /* continue in 'advance */
                }
            }

            'advance: loop {
                idx += 1;
                entry = unsafe {
                    self.list_of_active_formatting_elements
                        .get_unchecked_mut(idx)
                };
            }
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
                        && !doctype_data.is_about_legacy_compat();

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
            // nous devons mettre le document en mode quirks. Et dans tous
            // les cas, passer le mode d'insertion à "before
            // html", puis retraiter le jeton.
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
            // alors nous devons changer l'encodage pour l'encodage
            // résultant.
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
            // Pousser "in template" sur la pile des modes d'insertion
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
                self.frameset_ok_flag = FramesetOkFlag::NotOk;
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

    fn handle_after_head_insertion_mode(&mut self, token: HTMLToken) {
        match token {
            // U+0009 CHARACTER TABULATION
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+000D CARRIAGE RETURN (CR)
            // U+0020 SPACE
            //
            // Insérer le caractère.
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
                /* Ignore */
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

            // A start tag whose tag name is "body"
            //
            // Insérer un élément HTML pour le jeton.
            // Définir le drapeau frameset-ok à "not ok".
            // Passer le mode d'insertion à "in body".
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if tag_names::body == name => {
                self.insert_html_element(tag_token);
                self.frameset_ok_flag = FramesetOkFlag::NotOk;
                self.insertion_mode.switch_to(InsertionMode::InBody);
            }

            // A start tag whose tag name is "frameset"
            //
            // Insérer un élément HTML pour le jeton.
            // Passer le mode d'insertion à "in frameset".
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if tag_names::frameset == name => {
                self.insert_html_element(tag_token);
                self.insertion_mode.switch_to(InsertionMode::InFrameset);
            }

            // A start tag whose tag name is one of:
            // "base", "basefont", "bgsound", "link", "meta", "noframes",
            // "script", "style", "template", "title"
            //
            // Erreur d'analyse.
            // Pousser le nœud pointé par le pointeur de l'élément "head"
            // sur la pile des éléments ouverts.
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in head".
            // Retirer le noeud pointé par le pointeur de l'élément "head"
            // de la pile des éléments ouverts. (Il se peut que ce ne soit
            // pas le nœud actuel à ce stade).
            //
            // Note: le pointeur de l'élément de tête ne peut pas être nul
            // à ce stade.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if name.is_one_of([
                tag_names::base,
                tag_names::basefont,
                tag_names::bgsound,
                tag_names::link,
                tag_names::meta,
                tag_names::noframes,
                tag_names::script,
                tag_names::style,
                tag_names::template,
                tag_names::title,
            ]) =>
            {
                self.parse_error(token.clone());
                if let Some(head) = self.head_element.as_ref() {
                    self.stack_of_open_elements.put(head.clone());
                }
                self.process_using_the_rules_for(
                    InsertionMode::InHead,
                    token,
                );

                self.stack_of_open_elements.remove_first_tag_matching(
                    |node| {
                        if let Some(head) = self.head_element.as_ref() {
                            return node == head;
                        }
                        false
                    },
                );

                assert!(matches!(self.head_element, Some(_)));
            }

            // An end tag whose tag name is "template"
            //
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in head".
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if tag_names::template == name => {
                self.process_using_the_rules_for(
                    InsertionMode::InHead,
                    token,
                );
            }

            // An end tag whose tag name is one of: "body", "html", "br"
            // Agir comme décrit dans l'entrée "Anything else" ci-dessous.
            //
            // A start tag whose tag name is "head"
            // Any other end tag
            //
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name, is_end, ..
            }) if (!is_end && tag_names::head == name)
                || (is_end
                    && !name.is_one_of([
                        tag_names::body,
                        tag_names::html,
                        tag_names::br,
                    ])) =>
            {
                self.parse_error(token);
                /* ignore */
            }

            // Anything else
            //
            // Insérer un élément HTML pour un jeton de balise de début
            // "body" sans attributs.
            // Passer le mode d'insertion sur "in body".
            // Retraiter le jeton actuel.
            | _ => {
                let body_element =
                    HTMLTagToken::start().with_name(tag_names::body);
                self.insert_html_element(&body_element);
                self.insertion_mode.switch_to(InsertionMode::InBody);
                self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }
        }
    }

    fn handle_in_body_insertion_mode(&mut self, token: HTMLToken) {
        /// Lorsque les étapes ci-dessous indiquent que l'agent utilisateur
        /// doit fermer un élément p, cela signifie que l'agent
        /// utilisateur doit exécuter les étapes suivantes :
        ///   1. Générer des balises de fin implicites, sauf pour les
        /// éléments p.
        ///   2. Si le nœud actuel n'est pas un élément p, il s'agit d'une
        /// erreur d'analyse.
        ///   3. Extraire des éléments de la pile des éléments ouverts
        /// jusqu'à ce qu'un élément p ait été extrait de la pile.
        fn close_p_element<C>(parser: &mut HTMLParser<C>, token: HTMLToken)
        where
            C: Iterator<Item = CodePoint>,
        {
            let tag_name = tag_names::p;

            parser.generate_implied_end_tags_except_for(tag_name);

            if tag_name != parser.current_node().element_ref().local_name()
            {
                parser.parse_error(token);
            }

            parser.stack_of_open_elements.pop_until_tag(tag_name);
        }

        /// <https://html.spec.whatwg.org/multipage/parsing.html#special>
        fn is_special_tag(tag_name: tag_names, namespace: &str) -> bool {
            if namespace
                .parse::<Namespace>()
                .ok()
                .filter(|ns| Namespace::HTML.eq(ns))
                .is_some()
            {
                return tag_name.is_one_of([
                    tag_names::address,
                    tag_names::applet,
                    tag_names::area,
                    tag_names::article,
                    tag_names::aside,
                    tag_names::base,
                    tag_names::basefont,
                    tag_names::bgsound,
                    tag_names::blockquote,
                    tag_names::body,
                    tag_names::br,
                    tag_names::button,
                    tag_names::caption,
                    tag_names::center,
                    tag_names::col,
                    tag_names::colgroup,
                    tag_names::dd,
                    tag_names::details,
                    tag_names::dir,
                    tag_names::div,
                    tag_names::dl,
                    tag_names::dt,
                    tag_names::embed,
                    tag_names::fieldset,
                    tag_names::figcaption,
                    tag_names::figure,
                    tag_names::footer,
                    tag_names::form,
                    tag_names::frame,
                    tag_names::frameset,
                    tag_names::h1,
                    tag_names::h2,
                    tag_names::h3,
                    tag_names::h4,
                    tag_names::h5,
                    tag_names::h6,
                    tag_names::head,
                    tag_names::header,
                    tag_names::hgroup,
                    tag_names::hr,
                    tag_names::html,
                    tag_names::iframe,
                    tag_names::img,
                    tag_names::input,
                    tag_names::keygen,
                    tag_names::li,
                    tag_names::link,
                    tag_names::listing,
                    tag_names::main,
                    tag_names::marquee,
                    tag_names::menu,
                    tag_names::meta,
                    tag_names::nav,
                    tag_names::noembed,
                    tag_names::noframes,
                    tag_names::noscript,
                    tag_names::object,
                    tag_names::ol,
                    tag_names::p,
                    tag_names::param,
                    tag_names::plaintext,
                    tag_names::pre,
                    tag_names::script,
                    tag_names::section,
                    tag_names::select,
                    tag_names::source,
                    tag_names::style,
                    tag_names::summary,
                    tag_names::table,
                    tag_names::tbody,
                    tag_names::td,
                    tag_names::template,
                    tag_names::textarea,
                    tag_names::tfoot,
                    tag_names::th,
                    tag_names::thead,
                    tag_names::title,
                    tag_names::tr,
                    tag_names::track,
                    tag_names::ul,
                    tag_names::wbr,
                    tag_names::xmp,
                ]);
            }

            // todo: mathml, svg

            false
        }

        match token {
            // A character token that is U+0000 NULL
            //
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::Character('\0') => {
                self.parse_error(token);
                /* Ignore */
            }

            // U+0009 CHARACTER TABULATION
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+000D CARRIAGE RETURN (CR)
            // U+0020 SPACE
            //
            // Reconstruire les éléments de mise en forme actifs, s'il y en
            // a.
            // Insérer le caractère du jeton.
            //
            // Any other character token
            //
            // Définir l'indicateur frameset-ok à "not ok".
            | HTMLToken::Character(ch) => {
                self.reconstruct_active_formatting_elements();
                self.insert_character(ch);

                if !ch.is_ascii_whitespace() {
                    self.frameset_ok_flag = FramesetOkFlag::NotOk;
                }
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
                /* Ignore */
            }

            // A start tag whose tag name is "html"
            //
            // Erreur d'analyse.
            // S'il y a un élément template sur la pile des éléments
            // ouverts, alors ignorer le jeton. Sinon, pour chaque attribut
            // du jeton, on vérifie si l'attribut est déjà présent sur
            // l'élément supérieur de la pile d'éléments ouverts. Si ce
            // n'est pas le cas, ajoute l'attribut et sa valeur
            // correspondante à cet élément.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                ref attributes,
                is_end: false,
                ..
            }) if tag_names::html == name => {
                self.parse_error(token.clone());
                if self
                    .stack_of_open_elements
                    .has_element_with_tag_name(tag_names::template)
                {
                    return;
                }

                attributes.iter().for_each(|attribute| {
                    let element = self.current_node().element_ref();
                    if !element.has_attribute(&attribute.0) {
                        element.set_attribute(&attribute.0, &attribute.1);
                    }
                });
            }

            // A start tag whose tag name is one of:
            // "base", "basefont", "bgsound", "link", "meta", "noframes",
            // "script", "style", "template", "title"
            // An end tag whose tag name is "template"
            //
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in head".
            | HTMLToken::Tag(HTMLTagToken {
                ref name, is_end, ..
            }) if !is_end
                && name.is_one_of([
                    tag_names::base,
                    tag_names::basefont,
                    tag_names::bgsound,
                    tag_names::link,
                    tag_names::meta,
                    tag_names::noframes,
                    tag_names::script,
                    tag_names::style,
                    tag_names::template,
                    tag_names::title,
                ])
                || is_end && tag_names::template == name =>
            {
                self.process_using_the_rules_for(
                    InsertionMode::InHead,
                    token,
                );
            }

            // A start tag whose tag name is "body"
            //
            // Erreur d'analyse.
            // Si le deuxième élément de la pile d'éléments ouverts n'est
            // pas un élément body, si la pile d'éléments ouverts ne
            // comporte qu'un seul nœud ou s'il existe un élément de modèle
            // sur la pile d'éléments ouverts, nous devons ignorer le
            // jeton (cas du fragment). Sinon, nous devons définir le
            // drapeau frameset-ok sur "not ok" ; ensuite, pour chaque
            // attribut du jeton, nous devons vérifier si l'attribut est
            // déjà présent sur l'élément body (le deuxième élément) de la
            // pile d'éléments ouverts, et si ce n'est pas le
            // cas, nous devons ajouter l'attribut et sa valeur
            // correspondante à cet élément.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                ref attributes,
                is_end: false,
                ..
            }) if tag_names::body == name => {
                if self.stack_of_open_elements.len() == 1 {
                    return;
                }

                let element = unsafe {
                    self.stack_of_open_elements.get_unchecked(1)
                }
                .element_ref();
                if tag_names::body != element.local_name() {
                    return;
                }

                if self
                    .stack_of_open_elements
                    .has_element_with_tag_name(tag_names::template)
                {
                    return;
                }

                self.frameset_ok_flag = FramesetOkFlag::NotOk;

                let body_element = unsafe {
                    self.stack_of_open_elements.get_unchecked(1)
                }
                .element_ref();

                attributes.iter().for_each(|attribute| {
                    if !body_element.has_attribute(&attribute.0) {
                        body_element
                            .set_attribute(&attribute.0, &attribute.1);
                    }
                });
            }

            // A start tag whose tag name is "frameset"
            //
            // Erreur d'analyse.
            // Si la pile d'éléments ouverts ne comporte qu'un seul nœud,
            // ou si le deuxième élément de la pile d'éléments ouverts
            // n'est pas un élément body, nous devons ignorer le jeton
            // (cas du fragment).
            // Si le drapeau frameset-ok est défini sur "not ok", nous
            // devons ignorer le jeton.
            // Sinon, nous devons exécuter les étapes suivantes :
            //   1. Retirer le deuxième élément de la pile des éléments
            // ouverts de son nœud parent, s'il en a un.
            //   2. Retirer tous les noeuds à partir du bas de la pile
            // d'éléments ouverts, du noeud actuel jusqu'à l'élément html
            // racine, mais sans l'inclure.
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if tag_names::frameset == name => {
                self.parse_error(token.clone());

                if self.stack_of_open_elements.len() == 1 {
                    return;
                }

                let element = unsafe {
                    self.stack_of_open_elements.get_unchecked(1)
                }
                .element_ref();
                if tag_names::body != element.local_name() {
                    return;
                }

                if self.frameset_ok_flag == FramesetOkFlag::NotOk {
                    return;
                }

                let second_element = self.stack_of_open_elements.remove(1);
                second_element.detach_node();

                while tag_names::html
                    != self.current_node().element_ref().local_name()
                {
                    self.stack_of_open_elements.pop();
                }

                self.insert_html_element(tag_token);
                self.insertion_mode.switch_to(InsertionMode::InFrameset);
            }

            // An end-of-file token
            //
            // Si la pile des modes d'insertion template n'est pas vide,
            // le jeton est traité selon les règles du mode d'insertion
            // "in template".
            | HTMLToken::EOF
                if self.stack_of_template_insertion_modes.is_empty() =>
            {
                self.process_using_the_rules_for(
                    InsertionMode::InTemplate,
                    token,
                );
            }

            // An end-of-file token
            //
            // Autrement, suivre les étapes suivantes :
            //
            //   1. Si un noeud de la pile d'éléments ouverts n'est pas un
            // élément dd, un élément dt, un élément "li", un élément
            // optgroup, un élément option, un élément p, un élément rb, un
            // élément rp, un élément rt, un élément rtc, un élément tbody,
            // un élément td, un élément tfoot, un élément th, un élément
            // thead, un élément tr, l'élément body ou l'élément html, il
            // s'agit d'une erreur d'analyse.
            //  2. Arrêter l'analyse.
            | HTMLToken::EOF => {
                if !self.stack_of_open_elements.iter().any(|node| {
                    let local_name = node.element_ref().local_name();
                    local_name.is_one_of([
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
                        tag_names::body,
                        tag_names::html,
                    ])
                }) {
                    self.parse_error(token);
                    return;
                }

                self.stop_parsing = true;
            }

            // An end tag whose tag name is "body"
            //
            // Si la pile d'éléments ouverts n'a pas d'élément body dans sa
            // portée, il s'agit d'une erreur d'analyse ; ignorer le jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if tag_names::body == name
                && self.stack_of_open_elements.has_element_in_scope(
                    tag_names::body,
                    StackOfOpenElements::SCOPE_ELEMENTS,
                ) =>
            {
                self.parse_error(token);
            }

            // An end tag whose tag name is "body"
            //
            // S'il existe un noeud dans la pile d'éléments ouverts qui
            // n'est pas un élément dd, un élément dt, un élément "li", un
            // élément optgroup, un élément option, un élément p, un
            // élément rb, un élément rp, un élément rt, un élément rtc, un
            // élément tbody, un élément td, un élément tfoot, un élément
            // th, un élément thead, un élément tr, l'élément body ou
            // l'élément html; il s'agit d'une erreur d'analyse.
            // Passer le mode d'insertion sur "after body".
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if tag_names::body == name => {
                if self.stack_of_open_elements.iter().any(|node| {
                    let element = node.element_ref();
                    let name = element.local_name();
                    !name.is_one_of([
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
                        tag_names::body,
                        tag_names::html,
                    ])
                }) {
                    self.parse_error(token);
                }

                self.insertion_mode.switch_to(InsertionMode::AfterBody);
            }

            // An end tag whose tag name is "html"
            //
            // Si la pile d'éléments ouverts n'a pas d'élément body dans sa
            // portée, il s'agit d'une erreur d'analyse ; ignorer le jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if tag_names::html == name
                && self.stack_of_open_elements.has_element_in_scope(
                    tag_names::body,
                    StackOfOpenElements::SCOPE_ELEMENTS,
                ) =>
            {
                self.parse_error(token);
            }

            // An end tag whose tag name is "html"
            //
            // S'il existe un noeud dans la pile d'éléments ouverts
            // qui n'est pas un élément dd, un élément dt, un élément "li",
            // un élément optgroup, un élément option, un élément p, un
            // élément rb, un élément rp, un élément rt, un élément rtc, un
            // élément tbody, un élément td, un élément tfoot, un élément
            // th, un élément thead, un élément tr, l'élément body ou
            // l'élément html, il s'agit d'une erreur d'analyse.
            // Passer le mode d'insertion à "after body".
            // Retraiter le jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if tag_names::html == name => {
                if self.stack_of_open_elements.iter().any(|node| {
                    let element = node.element_ref();
                    let name = element.local_name();
                    !name.is_one_of([
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
                        tag_names::body,
                        tag_names::html,
                    ])
                }) {
                    self.parse_error(token.clone());
                }
                self.insertion_mode.switch_to(InsertionMode::AfterBody);
                self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }

            // A start tag whose tag name is one of:
            // "address", "article", "aside", "blockquote", "center",
            // "details", "dialog", "dir", "div", "dl", "fieldset",
            // "figcaption", "figure", "footer", "header", "hgroup",
            // "main", "menu", "nav", "ol", "p", "section", "summary", "ul"
            //
            // Si la pile d'éléments ouverts comporte un élément p dans la
            // portée du bouton, alors nous devons fermer l'élément p.
            // Insérer un élément HTML pour le jeton.
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if name.is_one_of([
                tag_names::address,
                tag_names::article,
                tag_names::aside,
                tag_names::blockquote,
                tag_names::center,
                tag_names::details,
                tag_names::dialog,
                tag_names::dir,
                tag_names::div,
                tag_names::dl,
                tag_names::fieldset,
                tag_names::figcaption,
                tag_names::figure,
                tag_names::footer,
                tag_names::header,
                tag_names::hgroup,
                tag_names::main,
                tag_names::menu,
                tag_names::nav,
                tag_names::ol,
                tag_names::p,
                tag_names::section,
                tag_names::summary,
                tag_names::ul,
            ]) =>
            {
                if self.stack_of_open_elements.has_element_in_scope(
                    tag_names::p,
                    StackOfOpenElements::scoped_elements_with::<10>([
                        tag_names::button,
                    ]),
                ) {
                    close_p_element(self, token.clone());
                }

                self.insert_html_element(tag_token);
            }

            // A start tag whose tag name is one of:
            // "h1", "h2", "h3", "h4", "h5", "h6"
            //
            // Si la pile d'éléments ouverts comporte un élément p dans la
            // portée du bouton, alors nous devons fermer un élément p.
            // Si le noeud actuel est un élément HTML dont le nom de balise
            // est l'un des éléments "h1", "h2", "h3", "h4", "h5" ou "h6",
            // il s'agit d'une erreur d'analyse ; retirer le noeud actuel
            // de la pile des éléments ouverts.
            // Insérer un élément HTML pour le jeton.
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if name.is_one_of([
                tag_names::h1,
                tag_names::h2,
                tag_names::h3,
                tag_names::h4,
                tag_names::h5,
                tag_names::h6,
            ]) =>
            {
                if self.stack_of_open_elements.has_element_in_scope(
                    tag_names::p,
                    StackOfOpenElements::scoped_elements_with::<10>([
                        tag_names::button,
                    ]),
                ) {
                    close_p_element(self, token.clone());
                }

                if self
                    .current_node()
                    .element_ref()
                    .local_name()
                    .is_one_of([
                        tag_names::h1,
                        tag_names::h2,
                        tag_names::h3,
                        tag_names::h4,
                        tag_names::h5,
                        tag_names::h6,
                    ])
                {
                    self.parse_error(token.clone());
                    self.stack_of_open_elements.pop();
                }

                self.insert_html_element(tag_token);
            }

            // A start tag whose tag name is one of:
            // "pre", "listing"
            //
            // Si la pile d'éléments ouverts comporte un élément p dans la
            // portée du bouton, alors nous devons fermer un élément p.
            // Insérer un élément HTML pour le jeton.
            // Si le jeton suivant est un jeton de caractère U+000A LINE
            // FEED (LF), nous devons ignorer ce jeton et passer au
            // suivant. (Les sauts de ligne au début des
            // pré-blocs sont ignorés par convenance pour les auteurs).
            // Définir le drapeau frameset-ok sur "not ok".
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if name.is_one_of([tag_names::pre, tag_names::listing]) => {
                if self.stack_of_open_elements.has_element_in_scope(
                    tag_names::p,
                    StackOfOpenElements::scoped_elements_with::<10>([
                        tag_names::button,
                    ]),
                ) {
                    close_p_element(self, token.clone());
                }

                self.insert_html_element(tag_token);

                match self.tokenizer.next_token() {
                    | Some(HTMLToken::Character('\n')) => {}
                    | Some(next) => self.process_using_the_rules_for(
                        self.insertion_mode,
                        next,
                    ),
                    | _ => {}
                };

                self.frameset_ok_flag = FramesetOkFlag::NotOk;
            }

            // A start tag whose tag name is "form"
            //
            // Si le pointeur de l'élément form n'est pas null et qu'il n'y
            // a pas d'élément template sur la pile des éléments ouverts,
            // il s'agit d'une erreur d'analyse ; ignorer le jeton.
            // Sinon :
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if tag_names::form == name
                && self.form_element.is_some()
                && self
                    .stack_of_open_elements
                    .has_element_with_tag_name(tag_names::template) =>
            {
                self.parse_error(token);
            }

            // A start tag whose tag name is "form"
            //
            // Si la pile d'éléments ouverts possède un élément p dans la
            // portée du bouton, alors nous devons fermer un élément p.
            // Insérer un élément HTML pour le jeton et, s'il n'y a pas
            // d'élément template sur la pile d'éléments ouverts,
            // définir le pointeur d'élément form pour qu'il pointe sur
            // l'élément créé.
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if tag_names::form == name => {
                if self.stack_of_open_elements.has_element_in_scope(
                    tag_names::p,
                    StackOfOpenElements::scoped_elements_with::<10>([
                        tag_names::button,
                    ]),
                ) {
                    close_p_element(self, token.clone());
                }

                let element = self.insert_html_element(tag_token);
                if !self
                    .stack_of_open_elements
                    .has_element_with_tag_name(tag_names::template)
                {
                    self.form_element = element;
                }
            }

            // A start tag whose tag name is "li"
            //
            // Suivre ces étapes :
            //   1. Définir le drapeau frameset-ok à "not ok".
            //   2. Initialiser un nœud comme étant le nœud actuel (le nœud
            // le plus bas de la pile).
            //   3. Dans une boucle : si le nœud est un élément "li", alors
            // nous devons exécuter ces sous-étapes :
            //      3.1. Générer des balises de fin implicites, sauf pour
            // les éléments li.
            //      3.2. Si le nœud actuel n'est pas un élément "li", il
            // s'agit d'une erreur d'analyse.
            //      3.3. Extraire des éléments de la pile d'éléments
            // ouverts jusqu'à ce qu'un élément "li" ait été extrait de la
            // pile.
            //   4. Si le noeud est dans la catégorie spéciale, mais n'est
            // pas un élément "address", "div" ou "p", alors nous devons
            // passer à l'étape intitulée "done" ci-dessous.
            //   5. Sinon, nous devons placer le nœud à l'entrée précédente
            // dans la pile des éléments ouverts et retourner à la boucle
            // étiquetée étape.
            //   6. "Done" : Si la pile d'éléments ouverts a un élément p
            // dans la portée du bouton, alors nous devons fermer un
            // élément p.
            //   7. Et enfin, insérer un élément HTML pour le jeton.
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if tag_names::li == name => {
                const LI: tag_names = tag_names::li;

                self.frameset_ok_flag = FramesetOkFlag::NotOk;

                for node in self.stack_of_open_elements.iter() {
                    let element = node.element_ref();
                    let name = element.local_name();
                    let tag_name = name.parse::<tag_names>().unwrap();

                    if LI == tag_name {
                        self.generate_implied_end_tags_except_for(LI);
                        if LI
                            == self
                                .current_node()
                                .element_ref()
                                .local_name()
                        {
                            self.parse_error(token.clone());
                        }
                        self.stack_of_open_elements.pop_until_tag(LI);
                        break;
                    }

                    if is_special_tag(tag_name, &element.namespace())
                        && name.is_one_of([
                            tag_names::address,
                            tag_names::div,
                            tag_names::p,
                        ])
                    {
                        break;
                    }
                }

                if self.stack_of_open_elements.has_element_in_scope(
                    tag_names::p,
                    StackOfOpenElements::scoped_elements_with::<10>([
                        tag_names::button,
                    ]),
                ) {
                    close_p_element(self, token.clone());
                }

                self.insert_html_element(tag_token);
            }

            // A start tag whose tag name is one of: "dd", "dt"
            //
            // Suivre ces étapes :
            //   1. Définir le drapeau frameset-ok à "not ok".
            //   2. Initialise le nœud comme étant le nœud actuel (le nœud
            // le plus bas de la pile).
            //   3. "Dans une boucle" : Si le noeud est un élément dd,
            // alors nous devons exécuter ces sous-étapes :
            //      3.1. Générer des balises de fin implicites, sauf pour
            // les éléments dd.
            //      3.2. Si le nœud actuel n'est pas un élément dd, il
            // s'agit d'une erreur d'analyse.
            //      3.3. Extraire des éléments de la pile d'éléments
            // ouverts jusqu'à ce qu'un élément dd ait été extrait de la
            // pile.
            //   4. Si le noeud est un élément dt, alors nous devons
            // exécuter ces sous-étapes :
            //      4.1. Générer des balises de fin implicites, sauf pour
            // les éléments dt.
            //      4.2. Si le nœud actuel n'est pas un élément dt, il
            // s'agit d'une erreur d'analyse.
            //      4.3. Extraire des éléments de la pile d'éléments
            // ouverts jusqu'à ce qu'un élément dt ait été extrait de la
            // pile.
            //   5. Si le noeud est dans la catégorie spéciale, mais n'est
            // pas un élément address, div ou  p, alors nous devons passer
            // à l'étape intitulée "done" ci-dessous.
            //   6. Sinon, nous devons placer le nœud à l'entrée précédente
            // dans la pile des éléments ouverts et retourner à l'étape
            // "Dans une Boucle".
            //   7. "Done" : si la pile d'éléments ouverts a un élément p
            // dans la portée du bouton, alors nous devons fermer un
            // élément p.
            //   8. Et enfin, insérer un élément HTML pour le jeton.
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if name.is_one_of([tag_names::dd, tag_names::dt]) => {
                const DD: tag_names = tag_names::dd;
                const DT: tag_names = tag_names::dt;

                self.frameset_ok_flag = FramesetOkFlag::NotOk;

                for node in self.stack_of_open_elements.iter() {
                    let element = node.element_ref();
                    let name = element.local_name();
                    let tag_name = name.parse::<tag_names>().unwrap();

                    if DD == tag_name || DT == tag_name {
                        self.generate_implied_end_tags_except_for(
                            tag_name,
                        );
                        if tag_name
                            == self
                                .current_node()
                                .element_ref()
                                .local_name()
                        {
                            self.parse_error(token.clone());
                        }
                        self.stack_of_open_elements
                            .pop_until_tag(tag_name);
                        break;
                    }

                    if is_special_tag(tag_name, &element.namespace())
                        && name.is_one_of([
                            tag_names::address,
                            tag_names::div,
                            tag_names::p,
                        ])
                    {
                        break;
                    }
                }

                if self.stack_of_open_elements.has_element_in_scope(
                    tag_names::p,
                    StackOfOpenElements::scoped_elements_with::<10>([
                        tag_names::button,
                    ]),
                ) {
                    close_p_element(self, token.clone());
                }

                self.insert_html_element(tag_token);
            }

            // A start tag whose tag name is "plaintext"
            //
            // Si la pile d'éléments ouverts comporte un élément p dans la
            // portée du bouton, alors nous devons fermer un élément p.
            // Insérer un élément HTML pour le jeton.
            // Passer le tokenizer à l'état PLAINTEXT.
            //
            // Note: Une fois qu'une balise de début avec le nom de balise
            // "plaintext" a été vue, ce sera le dernier jeton vu autre que
            // les jetons de caractères (et le jeton de fin de fichier),
            // car il n'y a aucun moyen de sortir de l'état PLAINTEXT.
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if tag_names::plaintext == name => {
                if self.stack_of_open_elements.has_element_in_scope(
                    tag_names::p,
                    StackOfOpenElements::scoped_elements_with::<10>([
                        tag_names::button,
                    ]),
                ) {
                    close_p_element(self, token.clone());
                }
                self.insert_html_element(tag_token);
                self.tokenizer.switch_state_to("plaintext");
            }

            // A start tag whose tag name is "button"
            //
            // 1. Si la pile d'éléments ouverts contient un élément bouton,
            // exécutez ces sous-étapes :
            //    1.1. Erreur d'analyse.
            //    1.2. Générer des balises de fin implicites.
            //    1.3. Extraire des éléments de la pile d'éléments ouverts
            // jusqu'à ce qu'un élément de bouton ait été extrait de la
            // pile.
            // 2. Reconstruire les éléments de mise en forme actifs, s'il y
            // en a.
            // 3. Insérer un élément HTML pour le jeton.
            // 4. Définir l'indicateur frameset-ok à "not ok".
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if tag_names::button == name => {
                const BUTTON: tag_names = tag_names::button;
                if self
                    .stack_of_open_elements
                    .has_element_with_tag_name(BUTTON)
                {
                    self.parse_error(token.clone());
                    self.generate_implied_end_tags();
                    self.stack_of_open_elements.pop_until_tag(BUTTON);
                }

                self.reconstruct_active_formatting_elements();
                self.insert_html_element(tag_token);
                self.frameset_ok_flag = FramesetOkFlag::NotOk;
            }

            // An end tag whose tag name is one of:
            // "address", "article", "aside", "blockquote", "button",
            // "center", "details", "dialog", "dir", "div", "dl",
            // "fieldset", "figcaption", "figure", "footer", "header",
            // "hgroup", "listing", "main", "menu", "nav", "ol", "pre",
            // "section", "summary", "ul"
            //
            // Si la pile d'éléments ouverts ne contient pas d'élément HTML
            // ayant le même nom de balise que celui du jeton, il s'agit
            // d'une erreur d'analyse ; ignorer le jeton.
            // Sinon, suivre ces étapes:
            //   1. Générer des balises de fin implicites.
            //   2. Si le nœud actuel n'est pas un élément HTML ayant le
            // même nom de balise que celui du jeton, il s'agit d'une
            // erreur d'analyse.
            //   3. Extraire les éléments de la pile des éléments ouverts
            // jusqu'à ce qu'un élément HTML ayant le même nom de balise
            // que le jeton ait été retiré de la pile.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if name.is_one_of([
                tag_names::address,
                tag_names::article,
                tag_names::aside,
                tag_names::blockquote,
                tag_names::button,
                tag_names::center,
                tag_names::details,
                tag_names::dialog,
                tag_names::dir,
                tag_names::div,
                tag_names::dl,
                tag_names::fieldset,
                tag_names::figcaption,
                tag_names::figure,
                tag_names::footer,
                tag_names::header,
                tag_names::hgroup,
                tag_names::listing,
                tag_names::main,
                tag_names::menu,
                tag_names::nav,
                tag_names::ol,
                tag_names::pre,
                tag_names::section,
                tag_names::summary,
                tag_names::ul,
            ]) =>
            {
                let tag_name = name
                    .parse::<tag_names>()
                    .expect("devrait être un nom de balise valide.");
                if !self
                    .stack_of_open_elements
                    .has_element_with_tag_name(tag_name)
                {
                    self.parse_error(token.clone());
                    return;
                }

                self.generate_implied_end_tags();
                if !self
                    .stack_of_open_elements
                    .has_element_with_tag_name(tag_name)
                {
                    self.parse_error(token.clone());
                }

                self.stack_of_open_elements.pop_until_tag(tag_name);
            }

            // An end tag whose tag name is "form"
            //
            // S'il n'y a pas d'élément template sur la pile des éléments
            // ouverts, nous devons exécuter ces sous-étapes :
            //   1. Laisser node être l'élément sur lequel le pointeur
            // d'élément form est placé, soit null s'il n'est pas placé sur
            // un élément.
            //   2. Définit le pointeur de l'élément form à null.
            //   3. Si node est null ou si la pile d'éléments ouverts n'a
            // pas node dans son champ d'application, alors il s'agit d'une
            // erreur d'analyse ; retourner et ignorer le jeton.
            //   4. Générer des balises de fin implicites.
            //   5. Si le nœud actuel n'est pas un noeud, il s'agit d'une
            // erreur d'analyse.
            //   6. Extraire le noeud de la pile des éléments ouverts.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if tag_names::form == name
                && !self
                    .stack_of_open_elements
                    .has_element_with_tag_name(tag_names::template) =>
            {
                let maybe_node = self.form_element.take();
                match &maybe_node {
                    | Some(node) => {
                        let element_name = node
                            .element_ref()
                            .local_name()
                            .parse()
                            .expect(
                                "devrait être un nom de balise valide.",
                            );
                        if !self
                            .stack_of_open_elements
                            .has_element_in_scope(
                                element_name,
                                StackOfOpenElements::SCOPE_ELEMENTS,
                            )
                        {
                            self.parse_error(token.clone());
                            return;
                        }
                    }
                    | None => {
                        self.parse_error(token.clone());
                        return;
                    }
                };

                self.generate_implied_end_tags();
                if self.stack_of_open_elements.current_node()
                    == maybe_node.as_ref()
                {
                    self.parse_error(token.clone());
                }

                if let Some(node) = maybe_node {
                    self.stack_of_open_elements.remove_first_tag_matching(
                        |first_node| Rc::ptr_eq(first_node, &node),
                    );
                }
            }

            // An end tag whose tag name is "form"
            //
            // S'il existe un élément template sur la pile des éléments
            // ouverts, nous devons exécuter ces sous-étapes à la place :
            //   1. Si la pile d'éléments ouverts ne contient pas d'élément
            // form, il s'agit d'une erreur d'analyse ; retourner
            // et ignorer le jeton.
            //   2. Générer des balises de fin implicites.
            //   3. Si le noeud actuel n'est pas un élément form, alors
            // il s'agit d'une erreur d'analyse.
            //   4. Extraire des éléments de la pile des éléments ouverts
            // jusqu'à ce qu'un élément form ait été extrait de la pile.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if tag_names::form == name
                && self
                    .stack_of_open_elements
                    .has_element_with_tag_name(tag_names::template) =>
            {
                self.generate_implied_end_tags();
                if tag_names::form
                    != self.current_node().element_ref().local_name()
                {
                    self.parse_error(token);
                }

                self.stack_of_open_elements.pop_until_tag(tag_names::form);
            }

            // An end tag whose tag name is "p"
            //
            // Si la pile d'éléments ouverts ne comporte pas d'élément p
            // dans la portée du bouton, il s'agit d'une erreur d'analyse ;
            // insérer un élément HTML pour un jeton de balise de début "p"
            // sans attributs.
            // Fermer un élément p.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if tag_names::p == name => {
                if !self.stack_of_open_elements.has_element_in_scope(
                    tag_names::p,
                    StackOfOpenElements::scoped_elements_with::<10>([
                        tag_names::button,
                    ]),
                ) {
                    let p = HTMLTagToken::start().with_name(tag_names::p);
                    self.parse_error(token.clone());
                    self.insert_html_element(&p);
                }

                close_p_element(self, token);
            }

            // An end tag whose tag name is "li"
            //
            // Si la pile d'éléments ouverts ne comporte pas d'élément "li"
            // dans la portée de l'élément de liste, il s'agit d'une erreur
            // d'analyse ; ignorer le jeton.
            //
            // Sinon, nous devons exécuter ces étapes :
            //   1. Générer des balises de fin implicites, sauf pour les
            // éléments li.
            //   2. Si le noeud actuel n'est pas un élément li, il s'agit
            // d'une erreur d'analyse.
            //   3. Retirer les éléments de la pile des éléments ouverts
            // jusqu'à ce qu'un élément li ait été retiré de la
            // pile.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if tag_names::li == name => {
                if !self.stack_of_open_elements.has_element_in_scope(
                    tag_names::li,
                    StackOfOpenElements::SCOPE_ELEMENTS,
                ) {
                    self.parse_error(token.clone());
                    return;
                }

                self.generate_implied_end_tags_except_for(tag_names::li);

                if tag_names::li
                    != self.current_node().element_ref().local_name()
                {
                    self.parse_error(token.clone());
                }

                self.stack_of_open_elements.pop_until_tag(tag_names::li);
            }

            | _ => todo!(),
        }
    }
}

// ---- //
// Test //
// ---- //

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! load_fixture {
        ($filename:literal) => {{
            let document_node = DocumentNode::new();
            let html_file = include_str!($filename);
            HTMLParser::new(document_node, html_file.chars())
        }};
    }

    macro_rules! test_the_str {
        ($str:literal) => {{
            let document_node = DocumentNode::new();
            let html_file = $str;
            HTMLParser::new(document_node, html_file.chars())
        }};
    }

    #[test]
    #[should_panic]
    fn test_parse_document() {
        let mut parser = load_fixture!("crashtests/test.html");
        parser.run();
    }

    #[test]
    fn test_initial_insertion_mode() {
        // Comment token

        let mut parser = test_the_str!("<!-- Comment -->");
        let token = parser.tokenizer.next_token().unwrap();
        parser.handle_initial_insertion_mode(token);
        let node = parser.document.get_first_child().clone().unwrap();
        assert!(node.is_comment());
        assert!(!node.is_document());

        // Doctype

        let mut parser = test_the_str!("<!DOCTYPE html>");
        let token = parser.tokenizer.next_token().unwrap();
        parser.handle_initial_insertion_mode(token);
        let doc = parser.document.document_ref();
        let doctype = doc.get_doctype().clone().unwrap();
        assert_eq!(doctype.name, "html");
        assert_eq!(doctype.public_id, "");
        assert_eq!(doctype.system_id, "");

        // Anything else

        let mut parser = test_the_str!("a");
        let token = parser.tokenizer.next_token().unwrap();
        parser.handle_initial_insertion_mode(token);
        let doc = parser.document.document_ref();
        assert_eq!(doc.quirks_mode.borrow().clone(), QuirksMode::Yes);
        assert_eq!(parser.insertion_mode, InsertionMode::BeforeHTML);
    }

    #[test]
    fn test_before_html_insertion_mode() {
        // Comment

        let mut parser = test_the_str!("<!-- comment -->");
        let token = parser.tokenizer.next_token().unwrap();
        parser.handle_before_html_insertion_mode(token);
        let doc = parser.document.get_first_child().clone().unwrap();
        assert!(doc.is_comment());

        // Tag

        let mut parser = test_the_str!("<html><head>");
        // <html>
        let token = parser.tokenizer.next_token().unwrap();
        parser.handle_before_html_insertion_mode(token);
        let doc = parser.document.get_first_child().clone().unwrap();
        assert_eq!(tag_names::html, doc.element_ref().local_name());
        assert_eq!(parser.insertion_mode, InsertionMode::BeforeHead);

        // Anything else (<heap>)

        let token = parser.tokenizer.next_token().unwrap();
        parser.handle_before_html_insertion_mode(token);
        let doc = parser.document.get_last_child().clone().unwrap();
        assert_eq!(tag_names::html, doc.element_ref().local_name());
        assert_ne!(tag_names::head, doc.element_ref().local_name());
    }
}
