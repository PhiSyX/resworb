/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod codepoint;

/// 4. Tokenization
mod tokenization;

use infra::primitive::codepoint::CodePoint;

use self::tokenization::CSSTokenizer;

// --------- //
// Structure //
// --------- //

pub struct CSSParser<C> {
    tokenizer: CSSTokenizer<C>,
}

// -------------- //
// Implémentation //
// -------------- //

impl<C> CSSParser<C>
where
    C: Iterator<Item = CodePoint>,
{
    pub fn new(input: C) -> Self {
        Self {
            tokenizer: CSSTokenizer::new(input),
        }
    }
}
