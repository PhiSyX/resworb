/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom::node::CommentNode;
use html_elements::{
    interface::IsOneOfTagsInterface, tag_attributes, tag_names,
};
use infra::namespace::Namespace;

use crate::{
    state::{
        Entry, FramesetOkFlag, InsertionMode, ScriptingFlag,
        StackOfOpenElements,
    },
    tokenization::{HTMLToken, HTMLTokenizerState},
    tree_construction::{
        HTMLTreeConstruction, HTMLTreeConstructionControlFlow,
    },
    HTMLParserFlag, HTMLParserState,
};

impl HTMLTreeConstruction {
    pub(crate) fn handle_in_body_insertion_mode(
        &mut self,
        mut token: HTMLToken,
    ) -> HTMLTreeConstructionControlFlow {
        // Sera utilisé dans plusieurs endroit dans le code.
        fn handle_any_other_end_tag(
            tree: &mut HTMLTreeConstruction,
            token: &HTMLToken,
        ) {
            let tag_token = token.as_tag();

            let mut index: Option<usize> = None;
            for (idx, node) in
                tree.stack_of_open_elements.iter().enumerate().rev()
            {
                let current_tag_name = node.element_ref().tag_name();
                if current_tag_name == tag_token.local_name() {
                    if node
                        == tree.current_node().expect("Le noeud actuel")
                    {
                        tree.parse_error(token);
                    }
                    index = Some(idx);
                    break;
                }

                if is_special_tag(
                    current_tag_name,
                    node.element_ref()
                        .namespace()
                        .expect("Devrait être un espace de nom valide"),
                ) {
                    tree.parse_error(token);
                    return;
                }
            }

            let match_idx = match index {
                | Some(idx) => idx,
                | None => {
                    tree.parse_error(token);
                    return;
                }
            };

            tree.generate_implied_end_tags_except_for(
                tag_token.tag_name(),
            );

            while tree.stack_of_open_elements.len() > match_idx {
                tree.stack_of_open_elements.pop();
            }
        }

        /// Lorsque les étapes ci-dessous indiquent que l'agent utilisateur
        /// doit fermer un élément p, cela signifie que l'agent
        /// utilisateur doit exécuter les étapes suivantes :
        ///   1. Générer des balises de fin implicites, sauf pour les
        /// éléments p.
        ///   2. Si le nœud actuel n'est pas un élément p, il s'agit d'une
        /// erreur d'analyse.
        ///   3. Extraire des éléments de la pile des éléments ouverts
        /// jusqu'à ce qu'un élément p ait été extrait de la pile.
        fn close_p_element(
            tree: &mut HTMLTreeConstruction,
            token: &HTMLToken,
        ) {
            let tag_name = tag_names::p;

            tree.generate_implied_end_tags_except_for(tag_name);

            if let Some(cnode) = tree.current_node() {
                if tag_name != cnode.element_ref().local_name() {
                    tree.parse_error(token);
                }
            }

            tree.stack_of_open_elements.pop_until_tag(tag_name);
        }

        /// <https://html.spec.whatwg.org/multipage/parsing.html#special>
        #[allow(deprecated)]
        fn is_special_tag(
            tag_name: tag_names,
            namespace: Namespace,
        ) -> bool {
            match namespace {
                | Namespace::HTML => tag_name.is_one_of([
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
                ]),
                | Namespace::MathML => tag_name.is_one_of([
                    tag_names::mi,
                    tag_names::mo,
                    tag_names::mn,
                    tag_names::ms,
                    tag_names::mtext,
                    tag_names::annotationXml,
                ]),
                | Namespace::SVG => tag_name.is_one_of([
                    tag_names::foreignObject,
                    tag_names::desc,
                    tag_names::title,
                ]),

                | _ => false,
            }
        }

        match token {
            // A character token that is U+0000 NULL
            //
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::Character('\0') => {
                self.parse_error(&token);
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
            | HTMLToken::DOCTYPE { .. } => {
                self.parse_error(&token);
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
            | HTMLToken::Tag {
                ref name,
                ref attributes,
                is_end: false,
                ..
            } if tag_names::html == name => {
                self.parse_error(&token);

                if self
                    .stack_of_open_elements
                    .has_element_with_tag_name(tag_names::template)
                {
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                attributes.iter().for_each(|attribute| {
                    let element = self
                        .current_node()
                        .expect("Le noeud actuel")
                        .element_ref();
                    if !element.has_attribute(&attribute.name) {
                        element.set_attribute(
                            &attribute.name,
                            &attribute.value,
                        );
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
            #[allow(deprecated)]
            | HTMLToken::Tag {
                ref name, is_end, ..
            } if !is_end
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
                return self.process_using_the_rules_for(
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
            | HTMLToken::Tag {
                ref name,
                ref attributes,
                is_end: false,
                ..
            } if tag_names::body == name => {
                if self.stack_of_open_elements.len() == 1 {
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                let element = unsafe {
                    self.stack_of_open_elements.get_unchecked(1)
                }
                .element_ref();
                if tag_names::body != element.local_name() {
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                if self
                    .stack_of_open_elements
                    .has_element_with_tag_name(tag_names::template)
                {
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                self.frameset_ok_flag = FramesetOkFlag::NotOk;

                let body_element = unsafe {
                    self.stack_of_open_elements.get_unchecked(1)
                }
                .element_ref();

                attributes.iter().for_each(|attribute| {
                    if !body_element.has_attribute(&attribute.name) {
                        body_element.set_attribute(
                            &attribute.name,
                            &attribute.value,
                        );
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
            #[allow(deprecated)]
            | HTMLToken::Tag {
                ref name,
                is_end: false,
                ..
            } if tag_names::frameset == name => {
                self.parse_error(&token);

                if self.stack_of_open_elements.len() == 1 {
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                let element = unsafe {
                    self.stack_of_open_elements.get_unchecked(1)
                }
                .element_ref();
                if tag_names::body != element.local_name() {
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                if self.frameset_ok_flag == FramesetOkFlag::NotOk {
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                let second_element = self.stack_of_open_elements.remove(1);
                second_element.detach_node();

                while tag_names::html
                    != self
                        .current_node()
                        .expect("Le noeud actuel")
                        .element_ref()
                        .local_name()
                {
                    self.stack_of_open_elements.pop();
                }

                self.insert_html_element(token.as_tag());
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
                return self.process_using_the_rules_for(
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
            #[allow(deprecated)]
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
                    self.parse_error(&token);
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                return HTMLTreeConstructionControlFlow::Break(
                    HTMLParserFlag::Stop,
                );
            }

            // An end tag whose tag name is "body"
            //
            // Si la pile d'éléments ouverts n'a pas d'élément body dans sa
            // portée, il s'agit d'une erreur d'analyse ; ignorer le jeton.
            | HTMLToken::Tag {
                ref name,
                is_end: true,
                ..
            } if tag_names::body == name
                && self.stack_of_open_elements.has_element_in_scope(
                    tag_names::body,
                    StackOfOpenElements::SCOPE_ELEMENTS,
                ) =>
            {
                self.parse_error(&token);
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
            #[allow(deprecated)]
            | HTMLToken::Tag {
                ref name,
                is_end: true,
                ..
            } if tag_names::body == name => {
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
                    self.parse_error(&token);
                }

                self.insertion_mode.switch_to(InsertionMode::AfterBody);
            }

            // An end tag whose tag name is "html"
            //
            // Si la pile d'éléments ouverts n'a pas d'élément body dans sa
            // portée, il s'agit d'une erreur d'analyse ; ignorer le jeton.
            | HTMLToken::Tag {
                ref name,
                is_end: true,
                ..
            } if tag_names::html == name
                && self.stack_of_open_elements.has_element_in_scope(
                    tag_names::body,
                    StackOfOpenElements::SCOPE_ELEMENTS,
                ) =>
            {
                self.parse_error(&token);
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
            #[allow(deprecated)]
            | HTMLToken::Tag {
                ref name,
                is_end: true,
                ..
            } if tag_names::html == name => {
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
                    self.parse_error(&token);
                }
                self.insertion_mode.switch_to(InsertionMode::AfterBody);
                return self.process_using_the_rules_for(
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
            #[allow(deprecated)]
            | HTMLToken::Tag {
                ref name,
                is_end: false,
                ..
            } if name.is_one_of([
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
                    StackOfOpenElements::button_scope_elements(),
                ) {
                    close_p_element(self, token.as_tag());
                }

                self.insert_html_element(token.as_tag());
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
            | HTMLToken::Tag {
                ref name,
                is_end: false,
                ..
            } if name.is_one_of([
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
                    StackOfOpenElements::button_scope_elements(),
                ) {
                    close_p_element(self, token.as_tag());
                }

                if let Some(cnode) = self.current_node() {
                    if cnode.element_ref().local_name().is_one_of([
                        tag_names::h1,
                        tag_names::h2,
                        tag_names::h3,
                        tag_names::h4,
                        tag_names::h5,
                        tag_names::h6,
                    ]) {
                        self.parse_error(&token);
                        self.stack_of_open_elements.pop();
                    }
                }

                self.insert_html_element(token.as_tag());
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
            #[allow(deprecated)]
            | HTMLToken::Tag {
                ref name,
                is_end: false,
                ..
            } if name.is_one_of([tag_names::pre, tag_names::listing]) => {
                if self.stack_of_open_elements.has_element_in_scope(
                    tag_names::p,
                    StackOfOpenElements::button_scope_elements(),
                ) {
                    close_p_element(self, token.as_tag());
                }

                self.insert_html_element(token.as_tag());
                self.frameset_ok_flag = FramesetOkFlag::NotOk;

                return HTMLTreeConstructionControlFlow::Continue(
                    HTMLParserState::ProcessNextTokenExceptLF,
                );
            }

            // A start tag whose tag name is "form"
            //
            // Si le pointeur de l'élément form n'est pas null et qu'il n'y
            // a pas d'élément template sur la pile des éléments ouverts,
            // il s'agit d'une erreur d'analyse ; ignorer le jeton.
            // Sinon :
            | HTMLToken::Tag {
                ref name,
                is_end: false,
                ..
            } if tag_names::form == name
                && self.form_element_pointer.is_some()
                && self
                    .stack_of_open_elements
                    .has_element_with_tag_name(tag_names::template) =>
            {
                self.parse_error(&token);
            }

            // A start tag whose tag name is "form"
            //
            // Si la pile d'éléments ouverts possède un élément p dans la
            // portée du bouton, alors nous devons fermer un élément p.
            // Insérer un élément HTML pour le jeton et, s'il n'y a pas
            // d'élément template sur la pile d'éléments ouverts,
            // définir le pointeur d'élément form pour qu'il pointe sur
            // l'élément créé.
            | HTMLToken::Tag {
                ref name,
                is_end: false,
                ..
            } if tag_names::form == name => {
                if self.stack_of_open_elements.has_element_in_scope(
                    tag_names::p,
                    StackOfOpenElements::button_scope_elements(),
                ) {
                    close_p_element(self, token.as_tag());
                }

                let element = self.insert_html_element(token.as_tag());
                if !self
                    .stack_of_open_elements
                    .has_element_with_tag_name(tag_names::template)
                {
                    self.form_element_pointer = element;
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
            | HTMLToken::Tag {
                ref name,
                is_end: false,
                ..
            } if tag_names::li == name => {
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
                                .expect("Le noeud actuel")
                                .element_ref()
                                .local_name()
                        {
                            self.parse_error(&token);
                        }
                        self.stack_of_open_elements.pop_until_tag(LI);
                        break;
                    }

                    if is_special_tag(
                        tag_name,
                        element.namespace().expect(
                            "Devrait être un espace de nom valide",
                        ),
                    ) && name.is_one_of([
                        tag_names::address,
                        tag_names::div,
                        tag_names::p,
                    ]) {
                        break;
                    }
                }

                if self.stack_of_open_elements.has_element_in_scope(
                    tag_names::p,
                    StackOfOpenElements::button_scope_elements(),
                ) {
                    close_p_element(self, token.as_tag());
                }

                self.insert_html_element(token.as_tag());
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
            | HTMLToken::Tag {
                ref name,
                is_end: false,
                ..
            } if name.is_one_of([tag_names::dd, tag_names::dt]) => {
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
                                .expect("Le noeud actuel")
                                .element_ref()
                                .local_name()
                        {
                            self.parse_error(&token);
                        }
                        self.stack_of_open_elements
                            .pop_until_tag(tag_name);
                        break;
                    }

                    if is_special_tag(
                        tag_name,
                        element.namespace().expect(
                            "Devrait être un espace de nom valide",
                        ),
                    ) && name.is_one_of([
                        tag_names::address,
                        tag_names::div,
                        tag_names::p,
                    ]) {
                        break;
                    }
                }

                if self.stack_of_open_elements.has_element_in_scope(
                    tag_names::p,
                    StackOfOpenElements::button_scope_elements(),
                ) {
                    close_p_element(self, token.as_tag());
                }

                self.insert_html_element(token.as_tag());
            }

            // A start tag whose tag name is "plaintext"
            //
            // Si la pile d'éléments ouverts comporte un élément p dans la
            // portée du bouton, alors nous devons fermer un élément p.
            // Insérer un élément HTML pour le jeton.
            // Passer le tokenizer à l'état PLAINTEXT.
            //
            // NOTE(html): Une fois qu'une balise de début avec le nom de
            // balise "plaintext" a été vue, ce sera le dernier jeton vu
            // autre que les jetons de caractères (et le jeton de fin de
            // fichier), car il n'y a aucun moyen de sortir de l'état
            // PLAINTEXT.
            #[allow(deprecated)]
            | HTMLToken::Tag {
                ref name,
                is_end: false,
                ..
            } if tag_names::plaintext == name => {
                if self.stack_of_open_elements.has_element_in_scope(
                    tag_names::p,
                    StackOfOpenElements::button_scope_elements(),
                ) {
                    close_p_element(self, token.as_tag());
                }

                self.insert_html_element(token.as_tag());

                return HTMLTreeConstructionControlFlow::Continue(
                    HTMLParserState::SwitchTo("plaintext".to_owned()),
                );
            }

            // A start tag whose tag name is "button"
            //
            // 1. Si la pile d'éléments ouverts contient un élément bouton,
            // nous devons exécuter ces sous-étapes :
            //    1.1. Erreur d'analyse.
            //    1.2. Générer des balises de fin implicites.
            //    1.3. Extraire des éléments de la pile d'éléments ouverts
            // jusqu'à ce qu'un élément de bouton ait été extrait de la
            // pile.
            // 2. Reconstruire les éléments de mise en forme actifs, s'il y
            // en a.
            // 3. Insérer un élément HTML pour le jeton.
            // 4. Définir l'indicateur frameset-ok à "not ok".
            | HTMLToken::Tag {
                ref name,
                is_end: false,
                ..
            } if tag_names::button == name => {
                const BUTTON: tag_names = tag_names::button;
                if self
                    .stack_of_open_elements
                    .has_element_with_tag_name(BUTTON)
                {
                    self.parse_error(&token);
                    self.generate_implied_end_tags();
                    self.stack_of_open_elements.pop_until_tag(BUTTON);
                }

                self.reconstruct_active_formatting_elements();
                self.insert_html_element(token.as_tag());
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
            #[allow(deprecated)]
            | HTMLToken::Tag {
                ref name,
                is_end: true,
                ..
            } if name.is_one_of([
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
                    self.parse_error(&token);
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                self.generate_implied_end_tags();
                if !self
                    .stack_of_open_elements
                    .has_element_with_tag_name(tag_name)
                {
                    self.parse_error(&token);
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
            | HTMLToken::Tag {
                ref name,
                is_end: true,
                ..
            } if tag_names::form == name
                && !self
                    .stack_of_open_elements
                    .has_element_with_tag_name(tag_names::template) =>
            {
                let maybe_node = self.form_element_pointer.take();
                match &maybe_node {
                    | Some(node) => {
                        let element_name = node.element_ref().tag_name();
                        if !self
                            .stack_of_open_elements
                            .has_element_in_scope(
                                element_name,
                                StackOfOpenElements::SCOPE_ELEMENTS,
                            )
                        {
                            self.parse_error(&token);
                            return HTMLTreeConstructionControlFlow::Continue(
                                HTMLParserState::Ignore,
                            );
                        }
                    }
                    | None => {
                        self.parse_error(&token);
                        return HTMLTreeConstructionControlFlow::Continue(
                            HTMLParserState::Ignore,
                        );
                    }
                };

                self.generate_implied_end_tags();
                if self.stack_of_open_elements.current_node()
                    == maybe_node.as_ref()
                {
                    self.parse_error(&token);
                }

                if let Some(node) = maybe_node {
                    self.stack_of_open_elements.remove_first_tag_matching(
                        |first_node| first_node == &node,
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
            | HTMLToken::Tag {
                ref name,
                is_end: true,
                ..
            } if tag_names::form == name
                && self
                    .stack_of_open_elements
                    .has_element_with_tag_name(tag_names::template) =>
            {
                self.generate_implied_end_tags();
                if tag_names::form
                    != self
                        .current_node()
                        .expect("Le noeud actuel")
                        .element_ref()
                        .local_name()
                {
                    self.parse_error(&token);
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
            | HTMLToken::Tag {
                ref name,
                is_end: true,
                ..
            } if tag_names::p == name => {
                if !self.stack_of_open_elements.has_element_in_scope(
                    tag_names::p,
                    StackOfOpenElements::button_scope_elements(),
                ) {
                    let p =
                        HTMLToken::new_start_tag().with_name(tag_names::p);
                    self.parse_error(&token);
                    self.insert_html_element(&p);
                }

                close_p_element(self, token.as_tag());
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
            | HTMLToken::Tag {
                ref name,
                is_end: true,
                ..
            } if tag_names::li == name => {
                if !self.stack_of_open_elements.has_element_in_scope(
                    tag_names::li,
                    StackOfOpenElements::SCOPE_ELEMENTS,
                ) {
                    self.parse_error(&token);
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                self.generate_implied_end_tags_except_for(tag_names::li);

                if tag_names::li
                    != self
                        .current_node()
                        .expect("Le noeud actuel")
                        .element_ref()
                        .local_name()
                {
                    self.parse_error(&token);
                }

                self.stack_of_open_elements.pop_until_tag(tag_names::li);
            }

            // An end tag whose tag name is one of: "dd", "dt"
            //
            // Si la pile d'éléments ouverts ne contient pas d'élément HTML
            // ayant le même nom de balise que celui du jeton, il s'agit
            // d'une erreur d'analyse ; ignorer le jeton.
            //
            // Sinon, nous devons exécuter ces étapes :
            //   1. Générer les balises de fin implicites, sauf pour les
            // éléments HTML ayant le même nom de balise que le jeton.
            //   2. Si le noeud actuel n'est pas un élément HTML ayant le
            // même nom de balise que celui du jeton, il s'agit d'une
            // erreur d'analyse.
            //   3. Retirer les éléments de la pile des éléments ouverts
            // jusqu'à ce qu'un élément HTML ayant le même nom de balise
            // que le jeton ait été retiré de la pile.
            | HTMLToken::Tag {
                ref name,
                is_end: true,
                ..
            } if tag_names::dd == name || tag_names::dt == name => {
                let tag_name = name
                    .parse()
                    .expect("Devrait être un nom de balise valide");

                if !self.stack_of_open_elements.has_element_in_scope(
                    tag_name,
                    StackOfOpenElements::SCOPE_ELEMENTS,
                ) {
                    self.parse_error(&token);
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                self.generate_implied_end_tags_except_for(tag_name);

                if tag_name
                    != self
                        .current_node()
                        .expect("Le noeud actuel")
                        .element_ref()
                        .local_name()
                {
                    self.parse_error(&token);
                }

                self.stack_of_open_elements.pop_until_tag(tag_name);
            }

            // An end tag whose tag name is one of:
            // "h1", "h2", "h3", "h4", "h5", "h6"
            //
            // Si la pile d'éléments ouverts ne contient pas d'élément HTML
            // dont le nom de balise est l'un des suivants : "h1", "h2",
            // "h3", "h4", "h5" ou "h6", il s'agit d'une erreur d'analyse ;
            // ignorer le jeton.
            // Sinon, nous devons exécuter ces étapes :
            //   1. Génère des balises de fin implicites.
            //   2. Si le nœud actuel n'est pas un élément HTML ayant le
            // même nom de balise que celui du jeton, il s'agit
            // d'une erreur d'analyse.
            //   3. Extraire des éléments de la pile des éléments ouverts
            // jusqu'à ce qu'un élément HTML dont le nom de balise est l'un
            // de "h1", "h2", "h3", "h4", "h5" ou "h6" ait été extrait de
            // la pile.
            | HTMLToken::Tag {
                ref name,
                is_end: true,
                ..
            } if name.is_one_of([
                tag_names::h1,
                tag_names::h2,
                tag_names::h3,
                tag_names::h4,
                tag_names::h5,
                tag_names::h6,
            ]) =>
            {
                let tag_name: tag_names = name
                    .parse()
                    .expect("Devrait être un nom de balise valide");

                if [
                    tag_names::h1,
                    tag_names::h2,
                    tag_names::h3,
                    tag_names::h4,
                    tag_names::h5,
                    tag_names::h6,
                ]
                .into_iter()
                .all(|heading| {
                    !self.stack_of_open_elements.has_element_in_scope(
                        heading,
                        StackOfOpenElements::SCOPE_ELEMENTS,
                    )
                }) {
                    self.parse_error(&token);
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                self.generate_implied_end_tags();

                if tag_name
                    != self
                        .current_node()
                        .expect("Le noeud actuel")
                        .element_ref()
                        .local_name()
                {
                    self.parse_error(&token);
                }

                loop {
                    let popped_node = self.stack_of_open_elements.pop();
                    if let Some(popped_node) = popped_node {
                        if tag_name
                            == popped_node.element_ref().local_name()
                        {
                            break;
                        }
                    }
                }
            }

            // A start tag whose tag name is one of: "b", "big", "code",
            // "em", "font", "i", "s", "small", "strike", "strong", "tt",
            // "u"
            //
            // Reconstruire les éléments de mise en forme actifs, s'il y en
            // a.
            // Insérer un élément HTML pour le jeton. Pousser cet élément
            // dans la liste des éléments de formatage actifs.
            #[allow(deprecated)]
            | HTMLToken::Tag {
                ref name,
                is_end: false,
                ..
            } if name.is_one_of([
                tag_names::b,
                tag_names::big,
                tag_names::code,
                tag_names::em,
                tag_names::font,
                tag_names::i,
                tag_names::s,
                tag_names::small,
                tag_names::strike,
                tag_names::strong,
                tag_names::tt,
                tag_names::u,
            ]) =>
            {
                self.reconstruct_active_formatting_elements();
                let element = self.insert_html_element(token.as_tag());
                if let Some(element) = element {
                    self.list_of_active_formatting_elements
                        .push(Entry::Element(element));
                }
            }

            // A start tag whose tag name is "nobr"
            //
            // Reconstruire les éléments de mise en forme actifs, s'il y en
            // a.
            // Si la pile d'éléments ouverts comporte un élément nobr dans
            // sa portée, il s'agit d'une erreur d'analyse ; exécuter
            // l'algorithme de l'agence d'adoption pour le jeton, puis
            // reconstruire à nouveau les éléments de formatage actifs,
            // s'il y en a. Insérer un élément HTML pour le jeton. Pousser
            // cet élément dans la liste des éléments de formatage actifs.
            #[allow(deprecated)]
            | HTMLToken::Tag {
                ref name,
                is_end: false,
                ..
            } if tag_names::nobr == name => {
                self.reconstruct_active_formatting_elements();

                if self.stack_of_open_elements.has_element_in_scope(
                    tag_names::nobr,
                    StackOfOpenElements::SCOPE_ELEMENTS,
                ) {
                    self.run_adoption_agency_algorithm(
                        &token,
                        &is_special_tag,
                    );
                    self.reconstruct_active_formatting_elements();
                }

                let element = self.insert_html_element(token.as_tag());
                if let Some(element) = element {
                    self.list_of_active_formatting_elements
                        .push(Entry::Element(element));
                }
            }

            // An end tag whose tag name is one of: "a", "b", "big",
            // "code", "em", "font", "i", "nobr", "s", "small", "strike",
            // "strong", "tt", "u"
            //
            // Exécuter l'algorithme de l'agence d'adoption pour le jeton.
            #[allow(deprecated)]
            | HTMLToken::Tag {
                ref name,
                is_end: true,
                ..
            } if name.is_one_of([
                tag_names::a,
                tag_names::b,
                tag_names::big,
                tag_names::code,
                tag_names::em,
                tag_names::font,
                tag_names::i,
                tag_names::nobr,
                tag_names::s,
                tag_names::small,
                tag_names::strike,
                tag_names::strong,
                tag_names::tt,
                tag_names::u,
            ]) =>
            {
                self.run_adoption_agency_algorithm(
                    &token,
                    &is_special_tag,
                );
            }

            // A start tag whose tag name is one of: "applet", "marquee",
            // "object"
            //
            // Reconstruire les éléments de mise en forme actifs, s'il y en
            // a.
            // Insérer un élément HTML pour le jeton.
            // Insérer un marqueur à la fin de la liste des éléments de
            // mise en forme actifs.
            // Définir l'indicateur frameset-ok à "not ok".
            #[allow(deprecated)]
            | HTMLToken::Tag {
                ref name,
                is_end: false,
                ..
            } if name.is_one_of([
                tag_names::applet,
                tag_names::marquee,
                tag_names::object,
            ]) =>
            {
                self.reconstruct_active_formatting_elements();
                self.insert_html_element(token.as_tag());
                self.list_of_active_formatting_elements
                    .push(Entry::Marker);
                self.frameset_ok_flag = FramesetOkFlag::NotOk;
            }

            // An end tag token whose tag name is one of: "applet",
            // "marquee", "object"
            //
            // Si la pile d'éléments ouverts ne contient pas d'élément HTML
            // ayant le même nom de balise que celui du jeton, il s'agit
            // d'une erreur d'analyse ; ignorer le jeton.
            //
            // Sinon, nous devons exécuter ces étapes :
            //   1. Générer un élément de fin de balise.
            //   2. Si le nœud actuel n'est pas un élément HTML ayant le
            // même nom de balise que celui du jeton, il s'agit d'une
            // erreur d'analyse.
            //   3. Retirer les éléments de la pile des éléments ouverts
            // jusqu'à ce qu'un élément HTML ayant le même nom de balise
            // que le jeton ait été retiré de la pile.
            //   4. Effacer la liste des éléments de mise en forme actifs
            // jusqu'au dernier marqueur.
            #[allow(deprecated)]
            | HTMLToken::Tag {
                ref name,
                is_end: true,
                ..
            } if name.is_one_of([
                tag_names::applet,
                tag_names::marquee,
                tag_names::object,
            ]) =>
            {
                let tag_name = name
                    .parse()
                    .expect("Devrait être un nom de balise valide");

                if !self.stack_of_open_elements.has_element_in_scope(
                    tag_name,
                    StackOfOpenElements::SCOPE_ELEMENTS,
                ) {
                    self.parse_error(&token);
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                self.generate_implied_end_tags();

                let cnode = self.current_node().expect("Le noeud actuel");
                let element = cnode.element_ref();
                if element.tag_name() != tag_name {
                    self.parse_error(&token);
                }

                self.stack_of_open_elements.pop_until_tag(tag_name);
                self.list_of_active_formatting_elements
                    .clear_up_to_the_last_marker();
            }

            // A start tag whose tag name is "table"
            //
            // Si le document n'est pas en mode quirks mode et que la pile
            // d'éléments ouverts comporte un élément p dans la portée du
            // bouton, alors nous devons fermer un élément p.
            // Insérer un élément HTML pour le jeton.
            // Définir l'indicateur frameset-ok à "not ok".
            // Passer au mode d'insertion "in table".
            | HTMLToken::Tag {
                ref name,
                is_end: false,
                ..
            } if tag_names::table == name => {
                let document = self.document.document_ref();
                if !document.isin_quirks_mode()
                    && self.stack_of_open_elements.has_element_in_scope(
                        tag_names::p,
                        StackOfOpenElements::button_scope_elements(),
                    )
                {
                    close_p_element(self, token.as_tag());
                }

                self.insert_html_element(token.as_tag());
                self.frameset_ok_flag = FramesetOkFlag::NotOk;
                self.insertion_mode.switch_to(InsertionMode::InTable);
            }

            // An end tag whose tag name is "br"
            //
            // Erreur d'analyse. Supprimer les attributs du jeton, et
            // agir comme décrit dans l'entrée suivante ; c'est-à-dire
            // agir comme s'il s'agissait d'un jeton de balise de début
            // "br" sans attributs, plutôt que du jeton de balise de fin
            // qu'il est en réalité.
            //
            // A start tag whose tag name is one of: "area", "br", "embed",
            // "img", "keygen", "wbr"
            //
            // Reconstruire les éléments de mise en forme actifs, s'il y en
            // a.
            // Insérer un élément HTML pour le jeton. Retirer immédiatement
            // le nœud actuel de la pile des éléments ouverts.
            // Accusé réception du le drapeau self-closing du jeton, s'il
            // est activé.
            // Définir l'indicateur frameset-ok à "not ok".
            #[allow(deprecated)]
            | HTMLToken::Tag {
                ref name, is_end, ..
            } if (is_end && tag_names::br == name)
                || !is_end
                    && name.is_one_of([
                        tag_names::area,
                        tag_names::br,
                        tag_names::embed,
                        tag_names::img,
                        tag_names::keygen,
                        tag_names::wbr,
                    ]) =>
            {
                if is_end && tag_names::br == name {
                    token.as_tag_mut().clear_attributes();
                }

                self.reconstruct_active_formatting_elements();
                self.insert_html_element(token.as_tag());
                self.stack_of_open_elements.pop();
                token.as_tag_mut().set_acknowledge_self_closing_flag();
                self.frameset_ok_flag = FramesetOkFlag::NotOk;
            }

            // A start tag whose tag name is "input"
            //
            // Reconstruire les éléments de mise en forme actifs, s'il y en
            // en a.
            // Insérer un élément HTML pour le jeton. Retirer immédiatement
            // le nœud actuel de la pile des éléments ouverts.
            // Accusé réception du le drapeau self-closing du jeton, s'il
            // est activé.
            // Si le jeton n'a pas d'attribut avec le nom "type", ou s'il
            // en a un, mais que la valeur de cet attribut n'est pas une
            // correspondance ASCII insensible à la casse pour la chaîne
            // "hidden", alors nous devons mettre le drapeau frameset-ok à
            // "not ok".
            | HTMLToken::Tag {
                ref name,
                is_end: false,
                ..
            } if tag_names::input == name => {
                self.reconstruct_active_formatting_elements();
                self.insert_html_element(token.as_tag());
                self.stack_of_open_elements.pop();
                token.as_tag_mut().set_acknowledge_self_closing_flag();

                if !token.as_tag().has_attributes([tag_attributes::hidden])
                {
                    self.frameset_ok_flag = FramesetOkFlag::NotOk;
                }
            }

            // A start tag whose tag name is one of: "param", "source",
            // "track"
            //
            // Insérer un élément HTML pour le jeton. Retirer immédiatement
            // le nœud actuel de la pile des éléments ouverts.
            // Accusé réception du le drapeau self-closing du jeton, s'il
            // est activé.
            #[allow(deprecated)]
            | HTMLToken::Tag {
                ref name,
                is_end: false,
                ..
            } if name.is_one_of([
                tag_names::param,
                tag_names::source,
                tag_names::track,
            ]) =>
            {
                self.insert_html_element(token.as_tag());
                self.stack_of_open_elements.pop();
                token.as_tag_mut().set_acknowledge_self_closing_flag();
            }

            // A start tag whose tag name is "hr"
            //
            // Si la pile des éléments ouverts a un élément p dans la
            // portée du bouton, alors nous devons fermer un élément p.
            // Insérer un élément HTML pour le jeton. Retirer immédiatement
            // le nœud actuel de la pile des éléments ouverts.
            // Accusé réception du le drapeau self-closing du jeton, s'il
            // est activé.
            // Définir l'indicateur frameset-ok à "not ok".
            | HTMLToken::Tag {
                ref name,
                is_end: false,
                ..
            } if tag_names::hr == name => {
                if self.stack_of_open_elements.has_element_in_scope(
                    tag_names::p,
                    StackOfOpenElements::button_scope_elements(),
                ) {
                    close_p_element(self, token.as_tag());
                }

                self.insert_html_element(token.as_tag());
                self.stack_of_open_elements.pop();
                token.as_tag_mut().set_acknowledge_self_closing_flag();
                self.frameset_ok_flag = FramesetOkFlag::NotOk;
            }

            // A start tag whose tag name is "image"
            //
            // Erreur d'analyse. Changer le nom de balise du jeton en "img"
            // et puis retraiter (ne demandez pas).
            #[allow(deprecated)]
            | HTMLToken::Tag {
                ref name,
                is_end: false,
                ..
            } if tag_names::image == name => {
                self.parse_error(&token);

                token.as_tag_mut().update_name(tag_names::img);

                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }

            // A start tag whose tag name is "textarea"
            //
            // 1. Insérer un élément HTML pour le jeton.
            // 2. Si le jeton suivant est un jeton de caractère U+000A LINE
            // FEED (LF), nous devons ignorer ce jeton et passer au
            // suivant. (Les nouvelles lignes au début des
            // éléments de la zone de texte sont ignorées par
            // commodité pour les auteurs).
            // 3. Faire passer le tokenizer à l'état "RCDATA".
            // 4. Laisser le mode d'insertion d'origine être le mode
            // d'insertion actuel.
            // 5. Définir l'indicateur frameset-ok à "not ok".
            // 6. Passer le mode d'insertion à "text".
            | HTMLToken::Tag {
                ref name,
                is_end: false,
                ..
            } if tag_names::textarea == name => {
                self.insert_html_element(token.as_tag());
                // TODO(phisyx): améliorer cette partie ci.
                return HTMLTreeConstructionControlFlow::Continue(
                    HTMLParserState::CustomRcdata,
                );
            }

            // A start tag whose tag name is "xmp"
            //
            // Si la pile des éléments ouverts a un élément p dans la
            // portée du bouton, alors nous devons fermer un élément p.
            // Reconstruire les éléments de mise en forme actifs, s'il y en
            // a.
            // Définir l'indicateur frameset-ok à "not ok".
            // Suivre l'algorithme générique d'analyse syntaxique des
            // éléments de texte brut.
            #[allow(deprecated)]
            | HTMLToken::Tag {
                ref name,
                is_end: false,
                ..
            } if tag_names::xmp == name => {
                if self.stack_of_open_elements.has_element_in_scope(
                    tag_names::p,
                    StackOfOpenElements::button_scope_elements(),
                ) {
                    close_p_element(self, token.as_tag());
                }

                self.reconstruct_active_formatting_elements();
                self.frameset_ok_flag = FramesetOkFlag::NotOk;
                self.parse_generic_element(
                    token.as_tag(),
                    HTMLTokenizerState::RAWTEXT,
                );
            }

            // A start tag whose tag name is "iframe"
            //
            // Définir l'indicateur frameset-ok à "not ok".
            // Suivre l'algorithme générique d'analyse syntaxique des
            // éléments de texte brut.
            | HTMLToken::Tag {
                ref name,
                is_end: false,
                ..
            } if tag_names::iframe == name => {
                self.frameset_ok_flag = FramesetOkFlag::NotOk;
                self.parse_generic_element(
                    token.as_tag(),
                    HTMLTokenizerState::RAWTEXT,
                );
            }

            // A start tag whose tag name is "noembed"
            // A start tag whose tag name is "noscript", if the scripting
            // flag is enabled
            //
            // Suivre l'algorithme générique d'analyse syntaxique des
            // éléments de texte brut.
            #[allow(deprecated)]
            | HTMLToken::Tag {
                ref name,
                is_end: false,
                ..
            } if tag_names::noembed == name
                || (tag_names::noscript == name
                    && self.scripting_flag == ScriptingFlag::Enabled) =>
            {
                self.parse_generic_element(
                    token.as_tag(),
                    HTMLTokenizerState::RAWTEXT,
                );
            }

            // A start tag whose tag name is "select"
            //
            // Reconstruire les éléments de mise en forme actifs, s'il y en
            // a.
            // Insérer un élément HTML pour le jeton.
            // Définir l'indicateur frameset-ok à "not ok".
            // Si le mode d'insertion est l'un des suivants : "in table",
            // "in caption", "in table body", "in row", ou "in cell",
            // nous devons passer à "in select in table".
            // Sinon, passer le mode d'insertion à "in select".
            | HTMLToken::Tag {
                ref name,
                is_end: false,
                ..
            } if tag_names::select == name => {
                self.reconstruct_active_formatting_elements();
                self.insert_html_element(token.as_tag());
                self.frameset_ok_flag = FramesetOkFlag::NotOk;
                self.insertion_mode.switch_to(match self.insertion_mode {
                    | InsertionMode::InTable
                    | InsertionMode::InCaption
                    | InsertionMode::InTableBody
                    | InsertionMode::InRow
                    | InsertionMode::InCell => {
                        InsertionMode::InSelectInTable
                    }
                    | _ => InsertionMode::InSelect,
                });
            }

            // A start tag whose tag name is one of: "optgroup", "option"
            //
            // Si le nœud actuel est un élément d'option, il est alors
            // retiré de la pile des éléments ouverts.
            // Reconstruire les éléments de mise en forme actifs, s'il y en
            // a.
            // Insérer un élément HTML pour le jeton.
            | HTMLToken::Tag {
                ref name,
                is_end: false,
                ..
            } if name
                .is_one_of([tag_names::optgroup, tag_names::option]) =>
            {
                let cnode = self.current_node().expect("Le noeud actuel");
                if cnode.element_ref().tag_name() == tag_names::option {
                    self.stack_of_open_elements.pop();
                }
                self.reconstruct_active_formatting_elements();
                self.insert_html_element(token.as_tag());
            }

            // A start tag whose tag name is one of: "rb", "rtc"
            //
            // Si la pile d'éléments ouverts a un élément ruby dans la
            // portée, alors génère des balises de fin implicites. Si le
            // nœud actuel n'est pas un élément ruby, il s'agit d'une
            // erreur d'analyse.
            // Insérer un élément HTML pour le jeton.
            #[allow(deprecated)]
            | HTMLToken::Tag {
                ref name,
                is_end: false,
                ..
            } if name.is_one_of([tag_names::rb, tag_names::rtc]) => {
                if self.stack_of_open_elements.has_element_in_scope(
                    tag_names::ruby,
                    StackOfOpenElements::SCOPE_ELEMENTS,
                ) {
                    self.generate_implied_end_tags();
                    let cnode =
                        self.current_node().expect("Le noeud actuel");
                    if cnode.element_ref().tag_name() != tag_names::ruby {
                        self.parse_error(&token);
                    }
                }

                self.insert_html_element(token.as_tag());
            }

            // A start tag whose tag name is one of: "rp", "rt"
            //
            // Si la pile d'éléments ouverts a un élément ruby dans sa
            // portée, alors génère des balises de fin implicites, sauf
            // pour les éléments rtc. Si le noeud actuel n'est pas un
            // élément rtc ou un élément ruby, ceci est une erreur
            // d'analyse.
            // Insérer un élément HTML pour le jeton.
            #[allow(deprecated)]
            | HTMLToken::Tag {
                ref name,
                is_end: false,
                ..
            } if name.is_one_of([tag_names::rp, tag_names::rt]) => {
                if self.stack_of_open_elements.has_element_in_scope(
                    tag_names::ruby,
                    StackOfOpenElements::SCOPE_ELEMENTS,
                ) {
                    self.generate_implied_end_tags_except_for(
                        tag_names::rtc,
                    );

                    let cnode =
                        self.current_node().expect("Le noeud actuel");
                    let cnode_name = cnode.element_ref().tag_name();
                    if cnode_name != tag_names::rtc
                        || cnode_name != tag_names::ruby
                    {
                        self.parse_error(&token);
                    }
                }

                self.insert_html_element(token.as_tag());
            }

            // TODO(html): A start tag whose tag name is one of: "math"
            // TODO(html): A start tag whose tag name is one of: "svg"

            // A start tag whose tag name is one of: "caption", "col",
            // "colgroup", "frame", "head", "tbody", "td", "tfoot", "th",
            // "thead", "tr"
            //
            // Erreur d'analyse. Ignorer le jeton.
            #[allow(deprecated)]
            | HTMLToken::Tag {
                ref name,
                is_end: false,
                ..
            } if name.is_one_of([
                tag_names::caption,
                tag_names::col,
                tag_names::colgroup,
                tag_names::frame,
                tag_names::head,
                tag_names::tbody,
                tag_names::td,
                tag_names::tfoot,
                tag_names::th,
                tag_names::thead,
                tag_names::tr,
            ]) =>
            {
                self.parse_error(&token);
                /* Ignore */
            }

            // Any other start tag
            //
            // Reconstruire les éléments de mise en forme actifs, s'il y en
            // a.
            // Insérer un élément HTML pour le jeton.
            //
            // NOTE(html): Cet élément sera un élément ordinaire.
            | HTMLToken::Tag { is_end: false, .. } => {
                self.reconstruct_active_formatting_elements();
                self.insert_html_element(token.as_tag());
            }

            // Any other end tag
            | HTMLToken::Tag { is_end: true, .. } => {
                handle_any_other_end_tag(self, &token);
            }
        }

        HTMLTreeConstructionControlFlow::Continue(
            HTMLParserState::Continue,
        )
    }

    pub(crate) fn handle_after_body_insertion_mode(
        &mut self,
        token: HTMLToken,
    ) -> HTMLTreeConstructionControlFlow {
        match token {
            // U+0009 CHARACTER TABULATION
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+000D CARRIAGE RETURN (CR)
            // U+0020 SPACE
            //
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in body".
            | HTMLToken::Character(ch) if ch.is_ascii_whitespace() => {
                return self.process_using_the_rules_for(
                    InsertionMode::InBody,
                    token,
                );
            }

            // A comment token
            //
            // Insérer un commentaire comme dernier enfant du premier
            // élément de la pile d'éléments ouverts (l'élément html).
            | HTMLToken::Comment(comment) => {
                let maybe_insertion_location =
                    self.stack_of_open_elements.first();
                if let Some(insertion_location) = maybe_insertion_location
                {
                    let comment =
                        CommentNode::new(&self.document, comment);
                    insertion_location.append_child(comment.to_owned());
                }
            }

            // A DOCTYPE token
            //
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::DOCTYPE { .. } => {
                self.parse_error(&token);
                /* Ignore */
            }

            // A start tag whose tag name is "html"
            //
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in body".
            | HTMLToken::Tag {
                ref name,
                is_end: false,
                ..
            } if tag_names::html == name => {
                return self.process_using_the_rules_for(
                    InsertionMode::InBody,
                    token,
                );
            }

            // An end tag whose tag name is "html"
            //
            // Si l'analyseur a été créé dans le cadre de l'algorithme
            // d'analyse des fragments HTML, il s'agit d'une erreur
            // d'analyse ; ignorer le jeton (cas du fragment).
            // Sinon, nous devons passer le mode d'insertion sur
            // "after after body".
            | HTMLToken::Tag {
                ref name,
                is_end: true,
                ..
            } if tag_names::html == name => {
                if self.parsing_fragment {
                    self.parse_error(&token);
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                self.insertion_mode
                    .switch_to(InsertionMode::AfterAfterBody);
            }

            // An end-of-file token
            //
            // Arrêter l'analyse.
            | HTMLToken::EOF => {
                return HTMLTreeConstructionControlFlow::Break(
                    HTMLParserFlag::Stop,
                );
            }

            // Anything else
            //
            // Erreur d'analyse. Passer le mode d'insertion à "in body" et
            // retraiter le jeton.
            | _ => {
                self.parse_error(&token);
                self.insertion_mode.switch_to(InsertionMode::InBody);
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }
        };

        HTMLTreeConstructionControlFlow::Continue(
            HTMLParserState::Continue,
        )
    }

    pub(crate) fn handle_after_after_body_insertion_mode(
        &mut self,
        token: HTMLToken,
    ) -> HTMLTreeConstructionControlFlow {
        match token {
            // A comment token
            //
            // Insérer un commentaire comme dernier enfant de l'objet
            // Document.
            | HTMLToken::Comment(comment) => {
                let comment = CommentNode::new(&self.document, comment);
                self.document.append_child(comment.to_owned());
            }

            // A DOCTYPE token,
            // U+0009 CHARACTER TABULATION
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+000D CARRIAGE RETURN (CR)
            // U+0020 SPACE,
            // A start tag whose tag name is "html"
            //
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in body".
            | HTMLToken::DOCTYPE { .. } => {
                return self.process_using_the_rules_for(
                    InsertionMode::InBody,
                    token,
                );
            }
            | HTMLToken::Character(ch) if ch.is_ascii_whitespace() => {
                return self.process_using_the_rules_for(
                    InsertionMode::InBody,
                    token,
                );
            }
            | HTMLToken::Tag {
                ref name,
                is_end: false,
                ..
            } if tag_names::html == name => {
                return self.process_using_the_rules_for(
                    InsertionMode::InBody,
                    token,
                );
            }

            // An end-of-file token
            //
            // Arrêter l'analyse.
            | HTMLToken::EOF => {
                return HTMLTreeConstructionControlFlow::Break(
                    HTMLParserFlag::Stop,
                );
            }

            // Anything else
            //
            // Erreur d'analyse. Passer le mode d'insertion à "in body" et
            // retraiter le jeton.
            | _ => {
                self.parse_error(&token);
                self.insertion_mode.switch_to(InsertionMode::InBody);
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }
        };

        HTMLTreeConstructionControlFlow::Continue(
            HTMLParserState::Continue,
        )
    }
}
