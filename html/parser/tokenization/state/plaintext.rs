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
    pub(crate) fn handle_plaintext_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
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
            // Émettre un jeton `end-of-file`.
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
