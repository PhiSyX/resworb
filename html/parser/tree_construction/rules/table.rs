/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use html_elements::{interface::IsOneOfTagsInterface, tag_names};

use crate::{
    state::{Entry, InsertionMode, StackOfOpenElements},
    tokenization::{HTMLTagToken, HTMLToken},
    tree_construction::{
        HTMLTreeConstruction, HTMLTreeConstructionControlFlow,
    },
    HTMLParserState,
};

impl HTMLTreeConstruction {
    pub(crate) fn handle_in_table_insertion_mode(
        &mut self,
        mut token: HTMLToken,
    ) -> HTMLTreeConstructionControlFlow {
        /// Lorsque les étapes ci-dessous demandent à l'UA de vider la pile
        /// pour revenir à un contexte de tableau, cela signifie que l'UA
        /// doit, tant que le nœud actuel n'est pas un élément de tableau,
        /// de modèle ou html, extraire des éléments de la pile d'éléments
        /// ouverts.
        fn clear_stack_back_to_table_context(
            tree: &mut HTMLTreeConstruction,
        ) {
            while let Some(cnode) = tree.current_node() {
                if !cnode.element_ref().tag_name().is_one_of([
                    tag_names::table,
                    tag_names::template,
                    tag_names::html,
                ]) {
                    tree.stack_of_open_elements.pop();
                } else {
                    break;
                }
            }

            if let Some(cnode) = tree.current_node() {
                if cnode.element_ref().tag_name() == tag_names::html {
                    assert!(tree.parsing_fragment);
                }
            }
        }

        match token {
            // A character token, if the current node is table, tbody,
            // tfoot, thead, or tr element
            //
            // La table en attente de jetons de caractères doit être une
            // liste de jetons vide.
            // Le mode d'insertion d'origine est le mode d'insertion
            // actuel.
            // Passer le mode d'insertion à "in table text" puis retraiter
            // le jeton.
            | HTMLToken::Character(_)
                if self.current_node().is_some()
                    && !self
                        .current_node()
                        .unwrap()
                        .element_ref()
                        .tag_name()
                        .is_one_of([
                            tag_names::table,
                            tag_names::tbody,
                            tag_names::tfoot,
                            tag_names::thead,
                            tag_names::tr,
                        ]) =>
            {
                self.pending_table_character_tokens.clear();
                self.original_insertion_mode
                    .switch_to(self.insertion_mode);
                self.insertion_mode.switch_to(InsertionMode::InTableText);
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
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
                self.parse_error(&token);
                /* Ignore */
            }

            // A start tag whose tag name is "caption"
            //
            // Effacer la pile pour revenir à un contexte de table. (Voir
            // ci-dessus.)
            // Insérer un marqueur à la fin de la liste des éléments de
            // mise en forme actifs.
            // Insérer un élément HTML pour le jeton, puis passer le mode
            // d'insertion à "in caption".
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if tag_names::caption == name => {
                clear_stack_back_to_table_context(self);
                self.list_of_active_formatting_elements
                    .push(Entry::Marker);
                self.insert_html_element(tag_token);
                self.insertion_mode.switch_to(InsertionMode::InCaption);
            }

            // A start tag whose tag name is "colgroup"
            //
            // Effacer la pile pour revenir à un contexte de table. (Voir
            // ci-dessus.)
            // Insérer un élément HTML pour le jeton, puis passer le mode
            // d'insertion à "in column group".
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if tag_names::colgroup == name => {
                clear_stack_back_to_table_context(self);
                self.insert_html_element(tag_token);
                self.insertion_mode
                    .switch_to(InsertionMode::InColumnGroup);
            }

            // A start tag whose tag name is "col"
            //
            // Effacer la pile pour revenir à un contexte de table. (Voir
            // ci-dessus.)
            // Insérer un élément HTML pour un jeton de balise de début
            // "colgroup" sans attributs, puis passer le mode d'insertion à
            // "in column group".
            // Retraiter le jeton actuel.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if tag_names::col == name => {
                clear_stack_back_to_table_context(self);
                self.insert_html_element(
                    &HTMLTagToken::start().with_name(tag_names::colgroup),
                );
                self.insertion_mode
                    .switch_to(InsertionMode::InColumnGroup);
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }

            // A start tag whose tag name is one of: "tbody", "tfoot",
            // "thead"
            //
            // Effacer la pile pour revenir à un contexte de table. (Voir
            // ci-dessus.)
            // Insérer un élément HTML pour le jeton, puis passer le mode
            // d'insertion à "in table body".
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if name.is_one_of([
                tag_names::tbody,
                tag_names::tfoot,
                tag_names::thead,
            ]) =>
            {
                clear_stack_back_to_table_context(self);
                self.insert_html_element(tag_token);
                self.insertion_mode.switch_to(InsertionMode::InTableBody);
            }

            // A start tag whose tag name is one of: "td", "th", "tr"
            //
            // Effacer la pile pour revenir à un contexte de table. (Voir
            // ci-dessus.)
            // Insérer un élément HTML pour un jeton de balise de début
            // "tbody" sans attributs, puis passer le mode d'insertion à
            // "in table body".
            // Retraiter le jeton actuel.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if name.is_one_of([
                tag_names::td,
                tag_names::th,
                tag_names::tr,
            ]) =>
            {
                clear_stack_back_to_table_context(self);
                self.insert_html_element(
                    &HTMLTagToken::start().with_name(tag_names::tbody),
                );
                self.insertion_mode.switch_to(InsertionMode::InTableBody);
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }

            // A start tag whose tag name is "table"
            //
            // Erreur d'analyse.
            // Si la pile d'éléments ouverts ne comporte pas d'élément
            // table dans la portée de la table, nous devons ignorer le
            // jeton.
            // Sinon:
            // Retirer les éléments de cette pile jusqu'à ce qu'un élément
            // de table ait été sorti de la pile.
            // Réinitialiser le mode d'insertion de manière appropriée.
            // Retraiter le jeton actuel.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if tag_names::table == name => {
                self.parse_error(&token);

                if !self.stack_of_open_elements.has_element_in_scope(
                    tag_names::table,
                    StackOfOpenElements::table_scope_elements(),
                ) {
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                self.stack_of_open_elements
                    .pop_until_tag(tag_names::table);
                self.reset_insertion_mode_appropriately();
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }

            // An end tag whose tag name is "table"
            //
            // Si la pile d'éléments ouverts ne comporte pas d'élément
            // table dans la portée de la table, il s'agit d'une erreur
            // d'analyse ; ignorer le jeton.
            // Sinon:
            // Retirer les éléments de cette pile jusqu'à ce qu'un élément
            // de table ait été sorti de la pile.
            // Réinitialiser le mode d'insertion de manière appropriée.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if tag_names::table == name => {
                if !self.stack_of_open_elements.has_element_in_scope(
                    tag_names::table,
                    StackOfOpenElements::table_scope_elements(),
                ) {
                    self.parse_error(&token);
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                self.stack_of_open_elements
                    .pop_until_tag(tag_names::table);
                self.reset_insertion_mode_appropriately();
            }

            // An end tag whose tag name is one of: "body", "caption",
            // "col", "colgroup", "html", "tbody", "td", "tfoot", "th",
            // "thead", "tr"
            //
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if name.is_one_of([
                tag_names::body,
                tag_names::caption,
                tag_names::col,
                tag_names::colgroup,
                tag_names::html,
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

            // A start tag whose tag name is one of: "style", "script",
            // "template"
            // An end tag whose tag name is "template"
            //
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in head".
            | HTMLToken::Tag(HTMLTagToken {
                ref name, is_end, ..
            }) if !is_end
                && name.is_one_of([
                    tag_names::style,
                    tag_names::script,
                    tag_names::template,
                ])
                || is_end && tag_names::template == name =>
            {
                return self.process_using_the_rules_for(
                    InsertionMode::InHead,
                    token,
                );
            }

            // A start tag whose tag name is "input"
            //
            // Si le jeton ne possède pas d'attribut portant le nom "type",
            // ou s'il en possède un, mais que la valeur de cet attribut
            // n'est pas une correspondance ASCII insensible à la casse
            // pour la chaîne "hidden", alors : nous devons agir comme
            // décrit dans l'entrée "anything else" ci-dessous.
            // Sinon:
            // Erreur d'analyse.
            // Insérer un élément HTML pour le jeton.
            // Retirer cet élément d'entrée de la pile des éléments
            // ouverts.
            // Accusé réception du le drapeau self-closing du jeton, s'il
            // est activé.
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    ref attributes,
                    is_end: false,
                    ..
                },
            ) if tag_names::input == name
                && attributes.iter().any(|attr| {
                    attr.name == "type"
                        && attr.value.eq_ignore_ascii_case("hidden")
                }) =>
            {
                self.parse_error(&token);
                let element = self.insert_html_element(tag_token);

                self.stack_of_open_elements.remove_first_tag_matching(
                    |node| element.contains(node),
                );

                token.as_tag_mut().set_acknowledge_self_closing_flag();
            }

            // A start tag whose tag name is "form"
            //
            // Erreur d'analyse.
            // S'il existe un élément template sur la pile des éléments
            // ouverts, ou si le pointeur de l'élément de formulaire n'est
            // pas null, nous devons ignorer le jeton.
            // Sinon:
            // Insérer un élément HTML pour le jeton, et définir le
            // pointeur de l'élément form pour qu'il pointe sur l'élément
            // créé.
            // Retirer cet élément de formulaire de la pile des éléments
            // ouverts.
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if tag_names::form == name => {
                self.parse_error(&token);

                if self
                    .stack_of_open_elements
                    .has_element_with_tag_name(tag_names::template)
                    || self.form_element_pointer.is_some()
                {
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                let element = self.insert_html_element(tag_token);
                self.form_element_pointer = element.clone();

                self.stack_of_open_elements.remove_first_tag_matching(
                    |node| element.contains(node),
                );
            }

            // An end-of-file token
            //
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in body".
            | HTMLToken::EOF => {
                return self.process_using_the_rules_for(
                    InsertionMode::InBody,
                    token,
                );
            }

            // Anything else
            //
            // Erreur d'analyse. Activer le foster_parenting, traiter le
            // jeton en utilisant les règles du mode d'insertion "in body",
            // puis désactiver foster_parenting.
            | _ => {
                self.parse_error(&token);
                self.foster_parenting = true;
                self.process_using_the_rules_for(
                    InsertionMode::InBody,
                    token,
                );
                self.foster_parenting = false;
            }
        };

        HTMLTreeConstructionControlFlow::Continue(
            HTMLParserState::Continue,
        )
    }

    pub(crate) fn handle_in_table_text_insertion_mode(
        &mut self,
        token: HTMLToken,
    ) -> HTMLTreeConstructionControlFlow {
        match token {
            // A character token that is U+0000 NULL
            //
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::Character('\0') => {
                self.parse_error(&token);
                /* Ignore */
            }

            // Any other character token
            //
            // Ajouter le jeton de caractère à la liste des jetons de
            // caractère de la table en attente.
            | HTMLToken::Character(_) => {
                self.pending_table_character_tokens.push(token);
            }

            // Anything else
            //
            // Si l'un des jetons de la liste des jetons de caractères de
            // la table en attente est un jeton de caractère qui n'est pas
            // un espace blanc ASCII, il s'agit d'une erreur d'analyse :
            // retraiter les jetons de caractères de la liste des jetons de
            // caractères de la table en attente en appliquant les règles
            // données dans l'entrée "anything else" du mode d'insertion
            // "in table".
            // Sinon, insérer les caractères donnés par la liste des jetons
            // de caractères de la table en attente.
            // Passer le mode d'insertion au mode d'insertion original et
            // retraiter le jeton.
            | _ => {
                let pending_token_does_have_whitespace = self
                    .pending_table_character_tokens
                    .iter()
                    .any(|token| !token.is_ascii_whitespace());

                if pending_token_does_have_whitespace {
                    self.parse_error(&token);

                    for pending_token in
                        self.pending_table_character_tokens.clone()
                    {
                        // Jeton "Anything else" du mode d'insertion "in
                        // table"
                        self.foster_parenting = true;
                        self.process_using_the_rules_for(
                            InsertionMode::InBody,
                            pending_token,
                        );
                        self.foster_parenting = false;
                    }
                } else {
                    for pending_token in
                        self.pending_table_character_tokens.clone()
                    {
                        if let HTMLToken::Character(ch) = pending_token {
                            self.insert_character(ch);
                        }
                    }
                }

                self.insertion_mode
                    .switch_to(self.original_insertion_mode);
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
