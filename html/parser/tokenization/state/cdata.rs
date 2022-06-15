/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use infra::primitive::codepoint::CodePointIterator;

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
    pub(crate) fn handle_cdata_section_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.stream.consume_next_input_character() {
            // U+005D RIGHT SQUARE BRACKET (])
            //
            // Passer à l'état `cdata-section-bracket`.
            | Some(']') => self
                .switch_state_to("cdata-section-bracket")
                .and_continue(),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-cdata`.
            // Émettre un jeton `end-of-file`.
            | None => self
                .set_token(HTMLToken::EOF)
                .and_emit_with_error("eof-in-cdata"),

            // Anything else
            //
            // Émettre le caractère actuel comme un jeton de `character`.
            | Some(ch) => {
                self.set_token(HTMLToken::Character(ch)).and_emit()
            }
        }
    }

    pub(crate) fn handle_cdata_section_bracket_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.stream.consume_next_input_character() {
            // U+005D RIGHT SQUARE BRACKET (])
            //
            // Passer à l'état `cdata-section-end`.
            | Some(']') => {
                self.switch_state_to("cdata-section-end").and_continue()
            }

            // Anything else
            //
            // Émettre un jeton `character `U+005D RIGHT SQUARE BRACKET.
            // Reprendre dans l'état de `cdata-section`.
            | _ => self
                .emit_token(HTMLToken::Character(']'))
                .reconsume("cdata-section")
                .and_continue(),
        }
    }

    pub(crate) fn handle_cdata_section_end_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.stream.consume_next_input_character() {
            // U+005D RIGHT SQUARE BRACKET (])
            //
            // Émettre un jeton `character` U+005D RIGHT SQUARE BRACKET.
            | Some(ch @ ']') => {
                self.set_token(HTMLToken::Character(ch)).and_emit()
            }

            // U+003E GREATER-THAN SIGN character
            //
            // Passer à l'état `data`.
            | Some('>') => self.switch_state_to("data").and_continue(),

            // Anything else
            //
            // Émettre deux jetons `character` U+005D RIGHT SQUARE BRACKET.
            // Reprendre dans l'état `cdata-section`.
            | _ => self
                .emit_token(HTMLToken::Character(']'))
                .emit_token(HTMLToken::Character(']'))
                .reconsume("cdata-section")
                .and_continue(),
        }
    }
}
