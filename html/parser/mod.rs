/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod codepoint;
mod error;
mod state;
mod token;
mod tokenizer;

use std::{borrow::BorrowMut, ops::Deref};

use dom::{
    document::{Document, DocumentNode, QuirksMode},
    node::{Comment, DocumentType, Node},
};
use infra::{
    self, namespace::Namespace, primitive::codepoint::CodePoint,
    structure::tree::TreeNode,
};

use self::{
    state::{
        InsertionMode, ListOfActiveFormattingElements, StackOfOpenElements,
    },
    token::{HTMLTagToken, HTMLToken},
    tokenizer::HTMLTokenizer,
};
use crate::tag_names;

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
    stack_of_open_elements: StackOfOpenElements,
    parsing_fragment: bool,
    stop_parsing: bool,
    context_element: Option<TreeNode<Node>>,

    // Head Element
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
            stack_of_open_elements: StackOfOpenElements::default(),
            parsing_fragment: false,
            stop_parsing: false,
            context_element: None,
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
            match self.tokenizer.next_token() {
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
                            .is_an_html_integration_point()
                            && (token.is_start_tag()
                                || token.is_character()))
                        || token.is_eof() =>
                {
                    self.process_using_the_rules_for(
                        self.insertion_mode,
                        token,
                    );
                }

                // Otherwise
                //
                // Traiter le jeton selon les règles indiquées dans la
                // section relative à l'analyse syntaxique des jetons dans
                // le contenu étranger.
                | Some(token) => {
                    self.process_using_the_rules_for_foreign_content(token)
                }
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
        match dbg!(&m) {
            | InsertionMode::Initial => {
                self.handle_initial_insertion_mode(token)
            }
            | InsertionMode::BeforeHTML => {
                self.handle_before_html_insertion_mode(token)
            }
            | InsertionMode::BeforeHead => {
                self.handle_before_head_insertion_mode(token)
            }
            | InsertionMode::InHead => todo!(),
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
        todo!()
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
    ) -> Option<TreeNode<Node>> {
        let HTMLTagToken {
            name: local_name,
            attributes,
            ..
        } = token;

        let maybe_element = Document::create_element(local_name, None);

        if let Ok(element) = maybe_element.as_ref() {
            element.set_document(self.document.deref());

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
            .any(|tag_name| {
                if let Some(target) = maybe_target {
                    target.element_ref().local_name() == tag_name
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
                    let tc =
                        template.element_ref().content().map(|t| t.into());
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

    /// <https://html.spec.whatwg.org/multipage/parsing.html#insert-a-comment>
    fn insert_comment(&self, comment: String) {
        let mut adjusted_insertion_location =
            self.find_appropriate_place_for_inserting_node(None);

        let comment: TreeNode<Node> =
            Comment::new(comment.into()).into_tree(&self.document);

        if let Some(ref mut parent) = adjusted_insertion_location.parent {
            parent.insert_before(
                comment,
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

        let maybe_element = self.create_element_for(token, namespace);

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
            | HTMLToken::Tag(HTMLTagToken {
                name, is_end_token, ..
            }) => {
                if is_end_token {
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
                    is_end_token: false,
                    ..
                },
            ) if name == tag_names::html => {
                let element = self
                    .create_element_for(tag_token, Namespace::HTML)
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
                is_end_token: true,
                ..
            }) if ![
                tag_names::head,
                tag_names::body,
                tag_names::html,
                tag_names::br,
            ]
            .into_iter()
            .any(|x| name.to_lowercase() == x) =>
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
                let element =
                    Document::create_element(tag_names::html, None)
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
                is_end_token: false,
                ..
            }) if name == tag_names::html => {
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
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end_token: false,
                    ..
                },
            ) if name == tag_names::head => {
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
                is_end_token: true,
                ..
            }) if ![
                tag_names::head,
                tag_names::body,
                tag_names::html,
                tag_names::br,
            ]
            .into_iter()
            .any(|x| name.to_lowercase() == x) =>
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
}

