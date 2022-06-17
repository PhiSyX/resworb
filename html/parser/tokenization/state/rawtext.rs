/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use infra::primitive::codepoint::CodePointIterator;

use crate::{
    codepoint::HTMLCodePoint,
    tokenization::{
        tokenizer::{
            HTMLTokenizerProcessInterface, HTMLTokenizerProcessResult,
        },
        HTMLToken, HTMLTokenizer,
    },
};

impl<C> HTMLTokenizer<C>
where
    C: CodePointIterator,
{
    pub(crate) fn handle_rawtext_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
            // U+003C LESS-THAN SIGN (<)
            //
            // Passer à l'état `rawtext-less-than-sign`.
            | Some('<') => self
                .switch_state_to("rawtext-less-than-sign")
                .and_continue(),

            // U+0000 NULL
            //
            // Il s'agit d'une erreur d'analyse de type
            // `unexpected-null-character`. Émettre un jeton `character`
            // U+FFFD REPLACEMENT CHARACTER.
            | Some('\0') => self
                .set_token(HTMLToken::Character(
                    char::REPLACEMENT_CHARACTER,
                ))
                .and_emit_with_error("unexpected-null-character"),

            // EOF
            //
            // Émettre un jeton `end-of-file`
            | None => self.set_token(HTMLToken::EOF).and_emit(),

            // Anything else
            //
            // Émettre le caractère actuel comme un jeton `character`.
            | Some(ch) => {
                self.set_token(HTMLToken::Character(ch)).and_emit()
            }
        }
    }

    pub(crate) fn handle_rawtext_less_than_sign_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
            // U+002F SOLIDUS (/)
            //
            // Définir le tampon temporaire à une chaîne de caractères
            // vide. Passer à l'état `rawtext-end-tag-open`.
            | Some('/') => self
                .set_temporary_buffer(String::new())
                .switch_state_to("rawtext-end-tag-open")
                .and_continue(),

            // Anything else
            //
            // Émettre un jeton `character` U+003C LESS-THAN SIGN.
            // Reprendre dans l'état `rawtext`
            | _ => self
                .emit_token(HTMLToken::Character('<'))
                .reconsume("rawtext")
                .and_continue(),
        }
    }

    pub(crate) fn handle_rawtext_end_tag_open_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
            // ASCII alpha
            //
            // Créer un nouveau jeton `end-tag`, définir son nom de
            // balise en une chaîne de caractères vide. Reprendre dans
            // l'état `rawtext-end-tag-name`.
            | Some(ch) if ch.is_ascii_alphabetic() => self
                .set_token(HTMLToken::new_end_tag())
                .reconsume("rawtext-end-tag-name")
                .and_continue(),

            // Anything else
            //
            // Émettre un jeton `character` U+003C LESS-THAN SIGN et un
            // jeton `character` U+002F SOLIDUS. Reprendre dans
            // l'état `rawtext`.
            | _ => self
                .emit_token(HTMLToken::Character('<'))
                .emit_token(HTMLToken::Character('/'))
                .reconsume("rawtext")
                .and_continue(),
        }
    }

    pub(crate) fn handle_rawtext_end_tag_name_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Si le jeton `end-tag` actuel est un jeton `end-tag`
            // approprié, alors passer à l'état `before-attribute-name`.
            // Sinon, le traiter comme indiqué dans l'entrée "Anything
            // else" ci-dessous.
            | Some(ch)
                if ch.is_html_whitespace()
                    && self.is_appropriate_end_tag() =>
            {
                self.switch_state_to("before-attribute-name")
                    .and_continue()
            }

            // U+002F SOLIDUS (/)
            //
            // Si le jeton `end-tag` actuel est un jeton `end-tag`
            // approprié, passer à l'état à `self-closing-start-tag`.
            // Sinon, le traiter comme indiqué dans l'entrée
            // "Anything else" ci-dessous.
            | Some('/') if self.is_appropriate_end_tag() => self
                .switch_state_to("self-closing-start-tag")
                .and_continue(),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Si le jeton `end-tag` actuel est un jeton `end-tag`
            // approprié, passer à l'état `data` et émettre
            // le jeton `end-tag` actuel. Sinon, le traiter comme
            // indiqué dans l'entrée "Anything else" ci-dessous.
            | Some('>') if self.is_appropriate_end_tag() => {
                self.switch_state_to("data").and_emit()
            }

            // ASCII upper alpha
            //
            // Ajouter la version en minuscules du caractère actuel
            // (ajouter 0x0020 au point de code du caractère) au nom
            // de la balise du jeton `*-tag` actuel. Ajouter le caractère
            // actuel au tampon temporaire.
            | Some(ch) if ch.is_ascii_uppercase() => self
                .change_current_token(|token| {
                    token.append_character(ch.to_ascii_lowercase())
                })
                .append_character_to_temporary_buffer(ch)
                .and_continue(),

            // ASCII lower alpha
            //
            // Ajouter le caractère actuel au nom de la balise du jeton
            // `*-tag` actuel. Ajouter le caractère actuel au tampon
            // temporaire.
            | Some(ch) if ch.is_ascii_lowercase() => self
                .change_current_token(|token| token.append_character(ch))
                .append_character_to_temporary_buffer(ch)
                .and_continue(),

            // Anything else
            //
            // Émettre un jeton `character` U+003C LESS-THAN SIGN, un jeton
            // `character` U+002F SOLIDUS et un jeton `character` pour
            // chacun des caractères du tampon temporaire (dans l'ordre où
            // ils ont été ajoutés au tampon). Reprendre dans l'état
            // `rawtext`.
            | _ => self
                .emit_token(HTMLToken::Character('<'))
                .emit_token(HTMLToken::Character('/'))
                .emit_each_characters_of_temporary_buffer()
                .reconsume("rawtext")
                .and_continue(),
        }
    }
}
