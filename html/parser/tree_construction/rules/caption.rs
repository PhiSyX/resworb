/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use html_elements::{interface::IsOneOfTagsInterface, tag_names};

use crate::{
    state::{InsertionMode, StackOfOpenElements},
    tokenization::HTMLToken,
    tree_construction::{
        HTMLTreeConstruction, HTMLTreeConstructionControlFlow,
    },
    HTMLParserState,
};

impl HTMLTreeConstruction {
    pub(crate) fn handle_in_caption_insertion_mode(
        &mut self,
        token: HTMLToken,
    ) -> HTMLTreeConstructionControlFlow {
        match token {
            // An end tag whose tag name is "caption"
            //
            // Si la pile d'éléments ouverts ne comporte pas d'élément
            // caption dans la portée de la table, il s'agit d'une erreur
            // d'analyse ; ignorer le jeton. (cas du fragment)
            // Sinon:
            // Générer des balises de fin implicites.
            // Si le nœud actuel n'est pas un élément caption, il s'agit
            // d'une erreur d'analyse.
            // Retirer des éléments de cette pile jusqu'à ce qu'un élément
            // caption ait été extrait de la pile.
            // Effacer la liste des éléments de mise en forme actifs
            // jusqu'au dernier marqueur.
            // Passer le mode d'insertion à "in table".
            | HTMLToken::Tag {
                ref name,
                is_end: true,
                ..
            } if tag_names::caption == name => {
                if !self.stack_of_open_elements.has_element_in_scope(
                    tag_names::caption,
                    StackOfOpenElements::table_scope_elements(),
                ) {
                    self.parse_error(&token);
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                self.generate_implied_end_tags();

                if let Some(cnode) = self.current_node() {
                    if cnode.element_ref().tag_name() != tag_names::caption
                    {
                        self.parse_error(&token);
                    }
                }

                self.stack_of_open_elements
                    .pop_until_tag(tag_names::caption);
                self.list_of_active_formatting_elements
                    .clear_up_to_the_last_marker();
                self.insertion_mode.switch_to(InsertionMode::InTable);
            }

            // A start tag whose tag name is one of: "caption", "col",
            // "colgroup", "tbody", "td", "tfoot", "th", "thead", "tr"
            // An end tag whose tag name is "table"
            //
            // Si la pile d'éléments ouverts ne comporte pas d'élément
            // caption dans la portée de la table, il s'agit d'une erreur
            // d'analyse ; ignorer le jeton. (cas du fragment)
            // Sinon:
            // Générer des balises de fin implicites.
            // Si le nœud actuel n'est pas un élément caption, il s'agit
            // d'une erreur d'analyse.
            // Retirer des éléments de cette pile jusqu'à ce qu'un élément
            // caption ait été extrait de la pile.
            // Effacer la liste des éléments de mise en forme actifs
            // jusqu'au dernier marqueur.
            // Passer le mode d'insertion à "in table".
            // Retraiter le jeton.
            | HTMLToken::Tag {
                ref name, is_end, ..
            } if !is_end
                && name.is_one_of([
                    tag_names::caption,
                    tag_names::col,
                    tag_names::colgroup,
                    tag_names::tbody,
                    tag_names::td,
                    tag_names::tfoot,
                    tag_names::th,
                    tag_names::thead,
                    tag_names::tr,
                ])
                || is_end && tag_names::table == name =>
            {
                if !self.stack_of_open_elements.has_element_in_scope(
                    tag_names::caption,
                    StackOfOpenElements::table_scope_elements(),
                ) {
                    self.parse_error(&token);
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                self.generate_implied_end_tags();
                if let Some(cnode) = self.current_node() {
                    if cnode.element_ref().tag_name() != tag_names::caption
                    {
                        self.parse_error(&token);
                    }
                }
                self.stack_of_open_elements
                    .pop_until_tag(tag_names::caption);
                self.list_of_active_formatting_elements
                    .clear_up_to_the_last_marker();
                self.insertion_mode.switch_to(InsertionMode::InTable);
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }

            // An end tag whose tag name is one of: "body", "col",
            // "colgroup", "html", "tbody", "td", "tfoot", "th", "thead",
            // "tr"
            //
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::Tag {
                ref name,
                is_end: true,
                ..
            } if name.is_one_of([
                tag_names::body,
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

            // Anything else
            //
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in body".
            | _ => {
                return self.process_using_the_rules_for(
                    InsertionMode::InBody,
                    token,
                );
            }
        }

        HTMLTreeConstructionControlFlow::Continue(
            HTMLParserState::Continue,
        )
    }
}
