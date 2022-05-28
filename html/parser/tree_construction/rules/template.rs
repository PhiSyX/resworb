/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use html_elements::{interface::IsOneOfTagsInterface, tag_names};

use crate::{
    state::InsertionMode,
    tokenization::{HTMLTagToken, HTMLToken},
    tree_construction::{
        HTMLTreeConstruction, HTMLTreeConstructionControlFlow,
    },
    HTMLParserFlag, HTMLParserState,
};

impl HTMLTreeConstruction {
    pub(crate) fn handle_in_template_insertion_mode(
        &mut self,
        token: HTMLToken,
    ) -> HTMLTreeConstructionControlFlow {
        match token {
            // A character token
            // A comment token
            // A DOCTYPE token
            //
            // Traiter le jeton selon les règles du mode d'insertion
            // "in body".
            | HTMLToken::Character(_)
            | HTMLToken::Comment(_)
            | HTMLToken::DOCTYPE(_) => {
                return self.process_using_the_rules_for(
                    InsertionMode::InBody,
                    token,
                );
            }

            // A start tag whose tag name is one of: "base", "basefont",
            // "bgsound", "link", "meta", "noframes", "script", "style",
            // "template", "title"
            // An end tag whose tag name is "template"
            //
            // Traiter le jeton selon les règles du mode d'insertion
            // "in head".
            #[allow(deprecated)]
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
                return self.process_using_the_rules_for(
                    InsertionMode::InHead,
                    token,
                );
            }

            // A start tag whose tag name is one of: "caption", "colgroup",
            // "tbody", "tfoot", "thead"
            //
            // Retirer le mode d'insertion template actuel de la pile des
            // modes d'insertion des templates.
            // Ajouter "in table" sur la pile des modes d'insertion de
            // template de sorte qu'il soit le nouveau mode d'insertion
            // de template actuel.
            // Passer le mode d'insertion à "in table", puis retraiter le
            // jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if name.is_one_of([
                tag_names::caption,
                tag_names::colgroup,
                tag_names::tbody,
                tag_names::tfoot,
                tag_names::thead,
            ]) =>
            {
                self.stack_of_template_insertion_modes.pop();
                self.stack_of_template_insertion_modes
                    .push(InsertionMode::InTable);
                self.insertion_mode.switch_to(InsertionMode::InTable);
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }

            // A start tag whose tag name is "col"
            //
            // Retirer le mode d'insertion template actuel de la pile des
            // modes d'insertion des templates.
            // Ajouter "in column group" sur la pile des modes d'insertion
            // de template de sorte qu'il soit le nouveau mode
            // d'insertion de template actuel.
            // Passer le mode d'insertion à "in column group", puis
            // retraiter le jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if tag_names::col == name => {
                self.stack_of_template_insertion_modes.pop();
                self.stack_of_template_insertion_modes
                    .push(InsertionMode::InColumnGroup);
                self.insertion_mode
                    .switch_to(InsertionMode::InColumnGroup);
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }

            // A start tag whose tag name is "tr"
            //
            // Retirer le mode d'insertion template actuel de la pile des
            // modes d'insertion des templates.
            // Ajouter "in table body" sur la pile des modes d'insertion
            // de template de sorte qu'il soit le nouveau mode
            // d'insertion de template actuel.
            // Passer le mode d'insertion à "in table body", puis
            // retraiter le jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if tag_names::tr == name => {
                self.stack_of_template_insertion_modes.pop();
                self.stack_of_template_insertion_modes
                    .push(InsertionMode::InTableBody);
                self.insertion_mode.switch_to(InsertionMode::InTableBody);
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }

            // A start tag whose tag name is one of: "td", "th"
            //
            // Retirer le mode d'insertion template actuel de la pile des
            // modes d'insertion des templates.
            // Ajouter "in row" sur la pile des modes d'insertion de
            // template de sorte qu'il soit le nouveau mode d'insertion
            // de template actuel.
            // Passer le mode d'insertion à "in row", puis retraiter le
            // jeton.
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: false,
                ..
            }) if name.is_one_of([tag_names::td, tag_names::th]) => {
                self.stack_of_template_insertion_modes.pop();
                self.stack_of_template_insertion_modes
                    .push(InsertionMode::InRow);
                self.insertion_mode.switch_to(InsertionMode::InRow);
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }

            // Any other start tag
            //
            // Retirer le mode d'insertion template actuel de la pile des
            // modes d'insertion des templates.
            // Ajouter "in body" sur la pile des modes d'insertion de
            // template de sorte qu'il soit le nouveau mode d'insertion
            // de template actuel.
            // Passer le mode d'insertion à "in body", puis retraiter le
            // jeton.
            | HTMLToken::Tag(HTMLTagToken { is_end: false, .. }) => {
                self.stack_of_template_insertion_modes.pop();
                self.stack_of_template_insertion_modes
                    .push(InsertionMode::InBody);
                self.insertion_mode.switch_to(InsertionMode::InBody);
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }

            // Any other end tag
            //
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::Tag(HTMLTagToken { is_end: true, .. }) => {
                self.parse_error(&token);
                /* Ignore */
            }

            // An end-of-file token
            //
            // S'il n'y a pas d'élément template sur la pile des éléments
            // ouverts, alors nous devons arrêter l'analyse. (cas du
            // fragment)
            // Sinon, il s'agit d'une erreur d'analyse.
            // Retirer des éléments de la pile d'éléments ouverts jusqu'à
            // ce qu'un élément template ait été extrait de la pile.
            // Effacer la liste des éléments de mise en forme actifs
            // jusqu'au dernier marqueur.
            // Supprimer le mode d'insertion de template actuel de la pile
            // des modes d'insertion de template.
            // Réinitialiser le mode d'insertion de manière appropriée.
            // Retraiter le jeton.
            | HTMLToken::EOF => {
                if !self
                    .stack_of_open_elements
                    .has_element_with_tag_name(tag_names::template)
                {
                    return HTMLTreeConstructionControlFlow::Break(
                        HTMLParserFlag::Stop,
                    );
                }

                self.parse_error(&token);
                self.stack_of_open_elements
                    .pop_until_tag(tag_names::template);
                self.list_of_active_formatting_elements
                    .clear_up_to_the_last_marker();
                self.reset_insertion_mode_appropriately();
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }
        }
        HTMLTreeConstructionControlFlow::Continue(
            HTMLParserState::Continue,
        )
    }
}
