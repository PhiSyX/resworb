/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use infra::primitive::codepoint::CodePointIterator;

use crate::tokenization::{
    tokenizer::{
        HTMLTokenizer, HTMLTokenizerProcessInterface,
        HTMLTokenizerProcessResult,
    },
    HTMLToken,
};

impl<C> HTMLTokenizer<C>
where
    C: CodePointIterator,
{
    pub(crate) fn handle_data_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
            // U+0026 AMPERSAND (&)
            //
            // Définir l'état de retour à l'état `data`. Passer à l'état
            // `character-reference`.
            | Some('&') => self
                .switch_state_to("character-reference")
                .set_return_state_to("data")
                .and_continue(),

            // U+003C LESS-THAN SIGN (<)
            //
            // Passer à l'état `tag-open`.
            | Some('<') => self.switch_state_to("tag-open").and_continue(),

            // U+0000 NULL
            //
            // Il s'agit d'une erreur d'analyse de type
            // `unexpected-null-character. Émettre le caractère
            // actuel comme un jeton de caractère.
            | Some('\0') => {
                self.and_emit_with_error("unexpected-null-character")
            }

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
