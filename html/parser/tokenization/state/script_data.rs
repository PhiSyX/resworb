/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use infra::primitive::codepoint::CodePoint;

use crate::tokenization::{
    tokenizer::{
        HTMLTokenizerProcessInterface, HTMLTokenizerProcessResult,
    },
    HTMLToken, HTMLTokenizer,
};

impl<C> HTMLTokenizer<C>
where
    C: Iterator<Item = CodePoint>,
{
    pub(crate) fn handle_script_data_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.stream.next_input_char() {
            // U+003C LESS-THAN SIGN (<)
            //
            // Passer à l'état `script-data-less-than-sign`.
            | Some('<') => self
                .switch_state_to("script-data-less-than-sign")
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
}
