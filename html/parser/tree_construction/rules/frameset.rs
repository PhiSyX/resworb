/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom::node::CommentNode;
use html_elements::tag_names;

use crate::{
    state::InsertionMode,
    tokenization::HTMLToken,
    tree_construction::{
        HTMLTreeConstruction, HTMLTreeConstructionControlFlow,
    },
    HTMLParserFlag, HTMLParserState,
};

impl HTMLTreeConstruction {
    pub(crate) fn handle_in_frameset_insertion_mode(
        &mut self,
        mut token: HTMLToken,
    ) -> HTMLTreeConstructionControlFlow {
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

            // A start tag whose tag name is "frameset"
            //
            // Insérer un élément HTML pour le jeton.
            #[allow(deprecated)]
            | HTMLToken::Tag {
                ref name,
                is_end: false,
                ..
            } if tag_names::frameset == name => {
                self.insert_html_element(token.as_tag());
            }

            // An end tag whose tag name is "frameset"
            //
            // Si le nœud actuel est l'élément html racine, il s'agit d'une
            // erreur d'analyse ; ignorer le jeton (cas d'un fragment).
            // Sinon, extraire le nœud de la pile d'éléments ouverts.
            // Si l'analyseur syntaxique n'a pas été créé dans le cadre de
            // l'algorithme d'analyse syntaxique des fragments HTML (cas
            // des fragments) et que le nœud actuel n'est plus un élément
            // frameset, le mode d'insertion doit alors passer à "after
            // frameset".
            #[allow(deprecated)] // frameset
            | HTMLToken::Tag {
                ref name,
                is_end: true,
                ..
            } if tag_names::frameset == name => {
                if let Some(cnode) = self.current_node() {
                    if cnode.element_ref().tag_name() == tag_names::html {
                        self.parse_error(&token);
                        return HTMLTreeConstructionControlFlow::Continue(
                            HTMLParserState::Ignore,
                        );
                    }
                }

                self.stack_of_open_elements.pop();

                if !self.parsing_fragment {
                    self.insertion_mode
                        .switch_to(InsertionMode::AfterFrameset);
                }
            }

            // A start tag whose tag name is "frame"
            //
            // Insérer un élément HTML pour le jeton. Extraire
            // immédiatement le nœud de la pile d'éléments ouverts.
            // Accuser réception du drapeau de fermeture automatique du
            // jeton, si défini.
            #[allow(deprecated)] // frame
            | HTMLToken::Tag {
                ref name,
                is_end: false,
                ..
            } if tag_names::frame == name => {
                self.insert_html_element(token.as_tag());
                self.stack_of_open_elements.pop();
                token.as_tag_mut().set_acknowledge_self_closing_flag();
            }

            // A start tag whose tag name is "noframes"
            //
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in head".
            #[allow(deprecated)] // noframes
            | HTMLToken::Tag {
                ref name,
                is_end: false,
                ..
            } if tag_names::noframes == name => {
                return self.process_using_the_rules_for(
                    InsertionMode::InHead,
                    token,
                );
            }

            // An end-of-file token
            //
            // Si le nœud actuel n'est pas l'élément html racine, il s'agit
            // d'une erreur d'analyse.
            // Note: Le nœud actuel ne peut être que l'élément html racine
            // dans le cas d'un fragment.
            // Arrêter l'analyse.
            | HTMLToken::EOF => {
                if let Some(cnode) = self.current_node() {
                    if cnode.element_ref().tag_name() != tag_names::html {
                        self.parse_error(&token);
                        return HTMLTreeConstructionControlFlow::Continue(
                            HTMLParserState::Ignore,
                        );
                    }
                }

                return HTMLTreeConstructionControlFlow::Break(
                    HTMLParserFlag::Stop,
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

    pub(crate) fn handle_after_frameset_insertion_mode(
        &mut self,
        token: HTMLToken,
    ) -> HTMLTreeConstructionControlFlow {
        match token {
            // U+0009 CHARACTER TABULATION
            // U+000A LINE FEED (LF)
            // U+000C
            // FORM FEED (FF)
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
            // Passer le mode d'insertion à "after after frameset".
            | HTMLToken::Tag {
                ref name,
                is_end: true,
                ..
            } if tag_names::html == name => {
                self.insertion_mode
                    .switch_to(InsertionMode::AfterAfterFrameset);
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

    pub(crate) fn handle_after_after_frameset_insertion_mode(
        &mut self,
        token: HTMLToken,
    ) -> HTMLTreeConstructionControlFlow {
        match token {
            // A comment token
            //
            // Insérer un commentaire comme dernier enfant de l'objet
            // [Document].
            | HTMLToken::Comment(comment) => {
                let comment = CommentNode::new(&self.document, comment);
                self.document.append_child(comment.to_owned());
            }

            // A DOCTYPE token
            // U+0009 CHARACTER TABULATION
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+000D CARRIAGE RETURN (CR)
            // U+0020 SPACE
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

            // A start tag whose tag name is "noframes"
            //
            // Traiter le jeton en utilisant les règles du mode d'insertion
            // "in head".
            #[allow(deprecated)]
            | HTMLToken::Tag {
                ref name,
                is_end: false,
                ..
            } if tag_names::noframes == name => {
                return self.process_using_the_rules_for(
                    InsertionMode::InHead,
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
}
