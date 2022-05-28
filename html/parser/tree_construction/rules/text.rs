/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use html_elements::tag_names;

use crate::{
    tokenization::{HTMLTagToken, HTMLToken},
    tree_construction::{
        HTMLTreeConstruction, HTMLTreeConstructionControlFlow,
    },
    HTMLParserState,
};

impl HTMLTreeConstruction {
    pub(crate) fn handle_text_insertion_mode(
        &mut self,
        token: HTMLToken,
    ) -> HTMLTreeConstructionControlFlow {
        match token {
            // A character token
            //
            // Insérer le caractère du jeton.
            //
            // Note: il ne peut jamais s'agir d'un caractère U+0000 NULL ;
            // le tokenizer les convertit en caractères
            // U+FFFD REPLACEMENT CHARACTER.
            | HTMLToken::Character(ch) => {
                self.insert_character(ch);
            }

            // An end-of-file token
            //
            // Erreur d'analyse.
            // Si le noeud actuel est un élément de type "script", alors
            // définir sa propriété `already_started` à true.
            // Retirer le noeud actuel de la pile d'éléments ouverts.
            // Passer le mode d'insertion au mode d'insertion original puis
            // retraiter le jeton.
            | HTMLToken::EOF => {
                self.parse_error(&token);

                if let Some(cnode) = self.current_node() {
                    let cnode_element = cnode.element_ref();
                    if tag_names::script == cnode_element.tag_name() {
                        cnode.script_ref().set_already_started(true);
                    }
                }

                self.stack_of_open_elements.pop();
                self.insertion_mode
                    .switch_to(self.original_insertion_mode);
                return self.process_using_the_rules_for(
                    self.insertion_mode,
                    token,
                );
            }

            // TODO: active spéculative html tree
            // An end tag whose tag name is "script"
            | HTMLToken::Tag(HTMLTagToken {
                ref name,
                is_end: true,
                ..
            }) if tag_names::script == name => {
                todo!()
            }

            // Any other end tag
            //
            // Retirer le nœud actuel de la pile des éléments ouverts.
            // Passer le mode d'insertion sur le mode d'insertion
            // d'origine.
            | HTMLToken::Tag(HTMLTagToken { is_end: true, .. }) => {
                self.stack_of_open_elements.pop();
                self.insertion_mode = self.original_insertion_mode;
            }

            // Rien n'est mentionné dans ce cas-ci dans la spécification.
            // Que faire ici?
            | _ => {}
        };

        HTMLTreeConstructionControlFlow::Continue(
            HTMLParserState::Continue,
        )
    }
}
