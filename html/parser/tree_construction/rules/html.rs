/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom::node::Document;
use html_elements::{interface::IsOneOfTagsInterface, tag_names};
use infra::namespace::Namespace;

use crate::{
    state::InsertionMode,
    tokenization::{HTMLTagToken, HTMLToken},
    tree_construction::{
        HTMLTreeConstruction, HTMLTreeConstructionControlFlow,
    },
    HTMLParserState,
};

impl HTMLTreeConstruction {
    pub(crate) fn handle_before_html_insertion_mode(
        &mut self,
        token: HTMLToken,
    ) -> HTMLTreeConstructionControlFlow {
        match token {
            // A DOCTYPE token
            //
            // Erreur d'analyse. Ignorer le jeton.
            | HTMLToken::DOCTYPE {  .. } => {
                self.parse_error(&token);
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
                self.document.append_child(element.to_owned());
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
                self.parse_error(&token);
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
                element.set_document(&self.document);
                self.document.append_child(element.to_owned());
                self.stack_of_open_elements.put(element);
                self.insertion_mode.switch_to(InsertionMode::BeforeHead);
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
