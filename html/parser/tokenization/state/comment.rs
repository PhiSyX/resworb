/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use infra::{
    primitive::codepoint::CodePointIterator,
    structure::lists::peekable::PeekableInterface,
};
use parser::StreamIteratorInterface;

use crate::tokenization::{
    tokenizer::{
        HTMLTokenizerProcessInterface, HTMLTokenizerProcessResult,
    },
    HTMLToken, HTMLTokenizer,
};

impl<C> HTMLTokenizer<C>
where
    C: CodePointIterator,
{
    pub(crate) fn handle_bogus_comment_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.stream.consume_next_input_character() {
            // U+003E GREATER-THAN SIGN (>)
            //
            // Passer à l'état `data`. Émettre le jeton `comment` actuel.
            | Some('>') => self.switch_state_to("data").and_emit(),

            // EOF
            //
            // Émettre `comment`. Émettre un jeton `end-of-file`.
            | None => self
                .and_emit_current_token()
                .set_token(HTMLToken::EOF)
                .and_emit(),

            // U+0000 NULL
            //
            // Il s'agit d'une erreur d'analyse de type
            // `unexpected-null-character`. Ajouter un caractère U+FFFD
            // `REPLACEMENT_CHARACTER` aux données du jeton `comment`.
            | Some('\0') => self
                .change_current_token(|token| {
                    token.append_character(char::REPLACEMENT_CHARACTER);
                })
                .and_continue_with_error("unexpected-null-character"),

            // Anything else
            //
            // Ajouter le caractère actuel aux données du jeton `comment`.
            | Some(ch) => self
                .change_current_token(|token| {
                    token.append_character(ch);
                })
                .and_continue(),
        }
    }

    pub(crate) fn handle_markup_declaration_open_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        if let Some(word) = self.stream.peek_until::<String>(7) {
            // Correspondance ASCII insensible à la casse pour le mot
            // "DOCTYPE".
            //
            // Consommer ces caractères et passer à l'état `doctype`.
            if word.to_ascii_lowercase() == "doctype" {
                self.stream.advance(7);
                return self.switch_state_to("doctype").and_continue();
            }
            // La chaîne "[CDATA[" (les cinq lettres majuscules "CDATA"
            // avec un caractère U+005B LEFT SQUARE BRACKET avant et après)
            //
            // Consommer ces caractères. S'il existe un noeud courant
            // ajusté et qu'il ne s'agit pas d'un élément de l'espace de
            // noms HTML, alors passer à l'état de section CDATA. Sinon, il
            // s'agit d'une erreur d'analyse `cdata-in-html-content`. Créer
            // un jeton `comment` dont les données sont une chaîne de
            // caractères "[CDATA[". Passer à l'état `bogus-comment`.
            else if word == "[CDATA[" {
                self.stream.advance(7);

                if !self
                    .tree_construction
                    .adjusted_current_node()
                    .element_ref()
                    .isin_html_namespace()
                {
                    return self.switch_state_to("cdata").and_continue();
                }

                return self
                    .set_token(HTMLToken::new_comment(word))
                    .switch_state_to("bogus-comment")
                    .and_continue_with_error("cdata-in-html-content");
            }
        }

        // Two U+002D HYPHEN-MINUS characters (-)
        //
        // Consommer ces deux caractères, créer un jeton `comment`
        // dont les données sont une chaîne de caractères vide, passer à
        // l'état `comment-start`.
        if let Some(word) = self.stream.peek_until::<String>(2) {
            if word == "--" {
                self.stream.advance(2);
                return self
                    .set_token(HTMLToken::new_comment(String::new()))
                    .switch_state_to("comment-start")
                    .and_continue();
            }
        }

        // Anything else
        //
        // Il s'agit d'une erreur d'analyse de type
        // `incorrectly-opened-comment`. Créer un jeton `comment` dont les
        // données sont une chaîne de caractères vide. Passer à l'état
        // `bogus-comment` (ne pas consommer dans l'état actuel).
        self.set_token(HTMLToken::new_comment(String::new()))
            .switch_state_to("bogus-comment")
            .and_continue_with_error("incorrectly-opened-comment")
    }

    pub(crate) fn handle_comment_start_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.stream.consume_next_input_character() {
            // U+002D HYPHEN-MINUS (-)
            //
            // Passer à l'état `comment-start-dash`.
            | Some('-') => {
                self.switch_state_to("comment-start-dash").and_continue()
            }

            // U+003E GREATER-THAN SIGN (>)
            //
            // Il s'agit d'une erreur d'analyse de type
            // `abrupt-closing-of-empty-comment`. Passer à l'état `data`.
            // Émettre le jeton `comment` actuel.
            | Some('>') => self
                .switch_state_to("data")
                .and_emit_with_error("abrupt-closing-of-empty-comment"),

            // Anything else
            //
            // Reprendre dans l'état de commentaire.
            | _ => self.reconsume("comment").and_continue(),
        }
    }

    pub(crate) fn handle_comment_less_than_sign_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.stream.consume_next_input_character() {
            // U+0021 EXCLAMATION MARK (!)
            //
            // Ajouter le caractère actuel aux données du jeton `comment`.
            // Passer à l'état `comment-less-than-sign-bang`.
            | Some(ch @ '!') => self
                .change_current_token(|comment_tok| {
                    comment_tok.append_character(ch)
                })
                .switch_state_to("comment-less-than-sign-bang")
                .and_continue(),

            // U+003C LESS-THAN SIGN (<)
            //
            // Ajoute le caractère actuel aux données du jeton `comment`.
            | Some(ch @ '<') => self
                .change_current_token(|comment_tok| {
                    comment_tok.append_character(ch)
                })
                .and_continue(),

            // Anything else
            //
            // Reprendre dans l'état `comment`.
            | _ => self.reconsume("comment").and_continue(),
        }
    }

    pub(crate) fn handle_comment_less_than_sign_bang_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.stream.consume_next_input_character() {
            // U+002D HYPHEN-MINUS (-)
            //
            // Passer à l'état `comment-less-than-sign-bang-dash`.
            | Some('-') => self
                .switch_state_to("comment-less-than-sign-bang-dash")
                .and_continue(),

            // Anything else
            //
            // Reprendre dans l'état `comment`.
            | _ => self.reconsume("comment").and_continue(),
        }
    }

    pub(crate) fn handle_comment_less_than_sign_bang_dash_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.stream.consume_next_input_character() {
            // U+002D HYPHEN-MINUS (-)
            //
            // Passer à l'état `comment-less-than-sign-bang-dash-dash`.
            | Some('-') => self
                .switch_state_to("comment-less-than-sign-bang-dash-dash")
                .and_continue(),

            // Anything else
            //
            // Reprendre dans l'état `comment-end-dash`.
            | _ => self.reconsume("comment-end-dash").and_continue(),
        }
    }

    pub(crate) fn handle_comment_less_than_sign_bang_dash_dash_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.stream.consume_next_input_character() {
            // U+003E GREATER-THAN SIGN (>)
            // EOF
            //
            // Reprendre à l'état `comment-end`.
            | Some('-') | None => {
                self.reconsume("comment-end").and_continue()
            }

            // Anything else
            //
            // Il s'agit d'une erreur d'analyse de type `nested-comment`.
            // Reprendre dans l'état `comment-end`.
            | Some(_) => self
                .reconsume("comment-end")
                .and_continue_with_error("nested-comment"),
        }
    }

    pub(crate) fn handle_comment_start_dash_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.stream.consume_next_input_character() {
            // U+002D HYPHEN-MINUS (-)
            //
            // Passer à l'état final du commentaire.
            | Some('-') => {
                self.switch_state_to("comment-end").and_continue()
            }

            // U+003E GREATER-THAN SIGN (>)
            //
            // Il s'agit d'une erreur d'analyse
            // abrupt-closing-of-empty-comment. Passer à l'état de données.
            // Émettre le jeton de commentaire actuel.
            | Some('>') => self
                .switch_state_to("data")
                .and_emit_with_error("abrupt-closing-of-empty-comment"),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-comment`.
            // Émettre le jeton `comment` actuel. Émettre un jeton `end of
            // file`.
            | None => self
                .and_emit_current_token()
                .set_token(HTMLToken::EOF)
                .and_emit_with_error("eof-in-comment"),

            // Anything else
            //
            // Ajouter un caractère U+002D HYPHEN-MINUS (-) aux données du
            // jeton `comment`. Reprendre l'état `comment`.
            | Some(_) => self
                .change_current_token(|comment_tok| {
                    comment_tok.append_character('-');
                })
                .reconsume("comment")
                .and_continue(),
        }
    }

    pub(crate) fn handle_comment_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.stream.consume_next_input_character() {
            // U+003C LESS-THAN SIGN (<)
            //
            // Ajouter le caractère actuel aux données du jeton `comment`.
            // Passer à l'état `comment-less-than-sign`.
            | Some(ch @ '<') => self
                .change_current_token(|comment_tok| {
                    comment_tok.append_character(ch)
                })
                .switch_state_to("comment-less-than-sign")
                .and_continue(),

            // U+002D HYPHEN-MINUS (-)
            //
            // Passer à l'état `comment-end-dash`.
            | Some('-') => {
                self.switch_state_to("comment-end-dash").and_continue()
            }

            // U+0000 NULL
            //
            // Il s'agit d'une erreur d'analyse de type
            // `unexpected-null-character`. Ajouter un caractère U+FFFD
            // `REPLACEMENT_CHARACTER` aux données du jeton `comment`.
            | Some('\0') => self
                .change_current_token(|comment_tok| {
                    comment_tok
                        .append_character(char::REPLACEMENT_CHARACTER);
                })
                .and_continue_with_error("unexpected-null-character"),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-comment`.
            // Émettre le jeton `comment` actuel. Émettre un jeton `end of
            // file`.
            | None => self
                .and_emit_current_token()
                .set_token(HTMLToken::EOF)
                .and_emit_with_error("eof-in-comment"),

            // Anything else
            //
            // Ajouter le caractère actuel aux données du jeton `comment`.
            | Some(ch) => self
                .change_current_token(|comment_tok| {
                    comment_tok.append_character(ch);
                })
                .and_continue(),
        }
    }

    pub(crate) fn handle_comment_end_dash_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.stream.consume_next_input_character() {
            // U+002D HYPHEN-MINUS (-)
            //
            // Passer à l'état `comment-end`.
            | Some('-') => {
                self.switch_state_to("comment-end").and_continue()
            }

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-comment`.
            // Émettre le jeton `comment` actuel. Émettre un jeton ` end of
            // file`.
            | None => self
                .and_emit_current_token()
                .set_token(HTMLToken::EOF)
                .and_emit_with_error("eof-in-comment"),

            // Anything else
            //
            // Ajouter un caractère U+002D HYPHEN-MINUS (-) aux données du
            // jeton `comment`. Reprendre l'état `comment`.
            | Some(_) => self
                .change_current_token(|comment_tok| {
                    comment_tok.append_character('-');
                })
                .reconsume("comment")
                .and_continue(),
        }
    }

    pub(crate) fn handle_comment_end_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.stream.consume_next_input_character() {
            // U+003E GREATER-THAN SIGN (>)
            //
            // Passer à l'état `data`. Émettre le jeton `comment` actuel.
            | Some('>') => self.switch_state_to("data").and_emit(),

            // U+0021 EXCLAMATION MARK (!)
            //
            // Passer à l'état `comment-end-bang`.
            | Some('!') => {
                self.switch_state_to("comment-end-bang").and_continue()
            }

            // U+002D HYPHEN-MINUS (-)
            //
            // Ajouter un caractère U+002D HYPHEN-MINUS (-) aux données du
            // jeton `comment` actuel.
            | Some(ch @ '-') => self
                .change_current_token(|comment_tok| {
                    comment_tok.append_character(ch);
                })
                .and_continue(),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-comment`.
            // Émettre le jeton `comment` actuel. Émettre un jeton `end of
            // file`.
            | None => self
                .and_emit_current_token()
                .set_token(HTMLToken::EOF)
                .and_emit_with_error("eof-in-comment"),

            // Anything else
            //
            // Ajouter deux caractères U+002D HYPHEN-MINUS (-) aux données
            // du jeton `comment`. Reprendre l'état `comment`.
            | Some(_) => self
                .change_current_token(|comment_tok| {
                    comment_tok.append_character('-');
                    comment_tok.append_character('-');
                })
                .reconsume("comment")
                .and_continue(),
        }
    }

    pub(crate) fn handle_comment_end_bang_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.stream.consume_next_input_character() {
            // U+002D HYPHEN-MINUS (-)
            //
            // Ajouter deux caractères U+002D HYPHEN-MINUS (-) et un
            // caractère U+0021 EXCLAMATION MARK (!) aux données du jeton
            // `comment`. Passer à l'état `comment-end-dash`.
            | Some('-') => self
                .change_current_token(|comment_tok| {
                    comment_tok.append_character('-');
                    comment_tok.append_character('-');
                    comment_tok.append_character('!');
                })
                .switch_state_to("comment-end-dash")
                .and_continue(),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Il s'agit d'une erreur d'analyse de type
            // `incorrectly-closed-comment`. Passer à l'état `data`.
            // Émettre le jeton `comment` actuel.
            | Some('>') => self
                .switch_state_to("data")
                .and_emit_with_error("incorrectly-closed-comment"),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-comment`.
            // Émettre le jeton `comment` actuel. Émettre un jeton `end of
            // file`.
            | None => self
                .and_emit_current_token()
                .set_token(HTMLToken::EOF)
                .and_emit_with_error("eof-in-comment"),

            // Anything else
            //
            // Ajouter deux caractères U+002D HYPHEN-MINUS (-) et un
            // caractère U+0021 EXCLAMATION MARK (!) aux données du jeton
            // de commentaire. Reprendre dans l'état `comment`.
            | Some(_) => self
                .change_current_token(|comment_tok| {
                    comment_tok.append_character('-');
                    comment_tok.append_character('-');
                    comment_tok.append_character('!');
                })
                .reconsume("comment")
                .and_continue(),
        }
    }
}
