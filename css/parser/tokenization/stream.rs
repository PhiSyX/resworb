/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops;

use infra::primitive::codepoint::CodePointIterator;
use parser::stream::TokenStream;

use super::{CSSToken, CSSTokenizer};

// --------- //
// Structure //
// --------- //

/// Pour tokeniser un flux de points de code en un flux de jetons
/// CSS en entrée, nous devons consommer de manière répétée un jeton
/// en entrée jusqu'à ce qu'un <EOF-token> soit atteint, en poussant
/// chacun des jetons retournés dans un flux.
#[derive(Debug)]
pub struct CSSTokenStream(TokenStream<CSSToken>);

// -------------- //
// Implémentation //
// -------------- //

impl CSSTokenStream {
    pub fn new<C>(input: C) -> Self
    where
        C: CodePointIterator,
    {
        let tokenizer = CSSTokenizer::new(input);
        let tokens = TokenStream::new(tokenizer);
        Self(tokens)
    }
}

// -------------- //
// Implémentation // - Interface
// -------------- //

impl ops::Deref for CSSTokenStream {
    type Target = TokenStream<CSSToken>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for CSSTokenStream {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
