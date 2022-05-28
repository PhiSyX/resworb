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
    HTMLParserState,
};

impl HTMLTreeConstruction {
    pub(crate) fn handle_before_head_insertion_mode(
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
            | HTMLToken::DOCTYPE(_) => self.parse_error(&token),

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
                self.head_element_pointer = head_element;
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
                self.parse_error(&token);
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
                self.head_element_pointer =
                    self.insert_html_element(&head_element);
                self.insertion_mode.switch_to(InsertionMode::InHead);
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
