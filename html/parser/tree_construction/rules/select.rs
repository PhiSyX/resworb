/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use html_elements::{interface::IsOneOfTagsInterface, tag_names};

use crate::{
    state::{InsertionMode, StackOfOpenElements},
    tokenization::{HTMLTagToken, HTMLToken},
    tree_construction::{
        HTMLTreeConstruction, HTMLTreeConstructionControlFlow,
    },
    HTMLParserState,
};

impl HTMLTreeConstruction {
    pub(crate) fn handle_in_select_insertion_mode(
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
            // Insérer le caractère du jeton.
            | HTMLToken::Character(ch) => {
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
            | HTMLToken::DOCTYPE {  .. } => {
                self.parse_error(&token);
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
                return self.process_using_the_rules_for(
                    InsertionMode::InBody,
                    token,
                );
            }

            // A start tag whose tag name is "option"
            //
            // Si le noeud actuel est un élément d'option, il faut retirer
            // ce noeud de la pile des éléments ouverts.
            // Insérer un élément HTML pour le jeton.
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if tag_names::option == name => {
                if let Some(cnode) = self.current_node() {
                    if cnode.element_ref().tag_name() == tag_names::option
                    {
                        self.stack_of_open_elements.pop();
                    }
                }

                self.insert_html_element(tag_token);
            }

            // A start tag whose tag name is "optgroup"
            //
            // Si le noeud actuel est un élément option, il faut retirer
            // ce noeud de la pile des éléments ouverts.
            // Si le noeud actuel est un élément optgroup, il faut
            // retirer ce noeud de la pile des éléments ouverts.
            // Insérer un élément HTML pour le jeton.
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: false,
                    ..
                },
            ) if tag_names::optgroup == name => {
                if let Some(cnode) = self.current_node() {
                    if cnode.element_ref().tag_name() == tag_names::option
                    {
                        self.stack_of_open_elements.pop();
                    }
                }

                if let Some(cnode) = self.current_node() {
                    if cnode.element_ref().tag_name()
                        == tag_names::optgroup
                    {
                        self.stack_of_open_elements.pop();
                    }
                }

                self.insert_html_element(tag_token);
            }

            // An end tag whose tag name is "optgroup"
            //
            // Tout d'abord, si le nœud actuel est un élément d'option et
            // que le nœud qui le précède immédiatement dans la pile des
            // éléments ouverts est un élément de groupe d'options, il faut
            // sortir le nœud actuel de la pile des éléments ouverts.
            // Si le noeud actuel est un élément d'optgroup, alors il faut
            // sortir ce noeud de la pile des éléments ouverts. Sinon, il
            // s'agit d'une erreur d'analyse ; ignorer le jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if tag_names::optgroup == name => {
                if let (Some(pnode), Some(cnode)) =
                    (self.before_current_node(), self.current_node())
                {
                    let pelement = pnode.element_ref();
                    let celement = cnode.element_ref();

                    if celement.tag_name() == tag_names::option
                        && pelement.tag_name() == tag_names::optgroup
                    {
                        self.stack_of_open_elements.pop();
                    }
                }

                if let Some(cnode) = self.current_node() {
                    if cnode.element_ref().tag_name()
                        == tag_names::optgroup
                    {
                        self.stack_of_open_elements.pop();
                    } else {
                        self.parse_error(&token);
                        /* Ignore */
                    }
                }
            }

            // An end tag whose tag name is "option"
            //
            // Si le noeud actuel est un élément option, alors il faut
            // sortir ce noeud de la pile des éléments ouverts. Sinon, il
            // s'agit d'une erreur d'analyse ; ignorer le jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if tag_names::option == name => {
                if let Some(cnode) = self.current_node() {
                    if cnode.element_ref().tag_name() == tag_names::option
                    {
                        self.stack_of_open_elements.pop();
                    } else {
                        self.parse_error(&token);
                        /* Ignore */
                    }
                }
            }

            // An end tag whose tag name is "select"
            //
            // Si la pile d'éléments ouverts ne comporte pas d'élément
            // select dans la portée select, il s'agit d'une erreur
            // d'analyse ; ignorer le jeton. (cas d'un fragment)
            // Sinon:
            // Retirer des éléments de la pile d'éléments ouverts jusqu'à
            // ce qu'un élément select ait été retiré de la pile.
            // Réinitialiser le mode d'insertion de manière appropriée.
            //
            // Note: Il est juste traité comme une balise de fin.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if tag_names::select == name => {
                if !self
                    .stack_of_open_elements
                    .has_element_in_scope_except(
                        tag_names::select,
                        StackOfOpenElements::select_scope_elements(),
                    )
                {
                    self.parse_error(&token);
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                self.stack_of_open_elements
                    .pop_until_tag(tag_names::select);
                self.reset_insertion_mode_appropriately();
            }

            // A start tag whose tag name is one of: "input", "keygen",
            // "textarea"
            //
            // Erreur d'analyse.
            // Si la pile d'éléments ouverts ne comporte pas d'élément
            // select dans la portée select, nous devons ignorer le jeton.
            // (cas du fragment)
            // Sinon:
            // Retirer les éléments de la pile des éléments ouverts jusqu'à
            // ce qu'un élément sélect ait été sorti de la pile.
            // Réinitialiser le mode d'insertion de manière appropriée.
            // Retraiter le jeton.
            #[allow(deprecated)]
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if name.is_one_of([
                tag_names::input,
                tag_names::keygen,
                tag_names::textarea,
            ]) =>
            {
                if !self
                    .stack_of_open_elements
                    .has_element_in_scope_except(
                        tag_names::select,
                        StackOfOpenElements::select_scope_elements(),
                    )
                {
                    self.parse_error(&token);
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                self.stack_of_open_elements
                    .pop_until_tag(tag_names::select);
                self.reset_insertion_mode_appropriately();
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }

            // A start tag whose tag name is one of: "script", "template"
            // An end tag whose tag name is "template"
            //
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in head".
            | HTMLToken::Tag(HTMLTagToken {
                ref name, is_end, ..
            }) if !is_end
                && name.is_one_of([
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

            // An end-of-file token
            //
            // Traite rle jeton en utilisant les règles du mode d'insertion
            // "in body".
            | HTMLToken::EOF => {
                return self.process_using_the_rules_for(
                    InsertionMode::InBody,
                    token,
                );
            }

            // Anything else
            //
            // Erreur d'analyse. Ignorer le jeton.
            | _ => {
                self.parse_error(&token);
                /* Ignore */
            }
        };

        HTMLTreeConstructionControlFlow::Continue(
            HTMLParserState::Continue,
        )
    }

    pub(crate) fn handle_in_select_in_table_insertion_mode(
        &mut self,
        token: HTMLToken,
    ) -> HTMLTreeConstructionControlFlow {
        match token {
            // A start tag whose tag name is one of: "caption", "table",
            // "tbody", "tfoot", "thead", "tr", "td", "th"
            //
            // Erreur d'analyse.
            // Retirer les éléments de la pile des éléments ouverts jusqu'à
            // ce qu'un élément select ait été retiré de la pile.
            // Réinitialiser le mode d'insertion de manière appropriée.
            // Retraiter le jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if name.is_one_of([
                tag_names::caption,
                tag_names::table,
                tag_names::tbody,
                tag_names::tfoot,
                tag_names::thead,
                tag_names::tr,
                tag_names::td,
                tag_names::th,
            ]) =>
            {
                self.parse_error(&token);
                self.stack_of_open_elements
                    .pop_until_tag(tag_names::select);
                self.reset_insertion_mode_appropriately();
                self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                )
            }

            // An end tag whose tag name is one of: "caption", "table",
            // "tbody", "tfoot", "thead", "tr", "td", "th"
            //
            // Erreur d'analyse.
            // Si la pile d'éléments ouverts ne contient pas d'élément dans
            // la portée de la table qui soit un élément HTML avec le même
            // nom de balise que celui du jeton, alors nous devons ignorer
            // le jeton.
            // Sinon:
            // Retirer les éléments de la pile des éléments ouverts jusqu'à
            // ce qu'un élément sélect ait été retiré de la pile.
            // Réinitialiser le mode d'insertion de manière appropriée.
            // Retraiter le jeton.
            | HTMLToken::Tag(
                ref tag_token @ HTMLTagToken {
                    ref name,
                    is_end: true,
                    ..
                },
            ) if name.is_one_of([
                tag_names::caption,
                tag_names::table,
                tag_names::tbody,
                tag_names::tfoot,
                tag_names::thead,
                tag_names::tr,
                tag_names::td,
                tag_names::th,
            ]) =>
            {
                self.parse_error(&token);

                if !self.stack_of_open_elements.has_element_in_scope(
                    tag_token.tag_name(),
                    StackOfOpenElements::table_scope_elements(),
                ) {
                    return HTMLTreeConstructionControlFlow::Continue(
                        HTMLParserState::Ignore,
                    );
                }

                self.stack_of_open_elements
                    .pop_until_tag(tag_names::select);
                self.reset_insertion_mode_appropriately();
                self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                )
            }

            // Anything else
            //
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in select".
            | _ => self.process_using_the_rules_for(
                InsertionMode::InSelect,
                token,
            ),
        }
    }
}
