/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;

use infra::primitive::codepoint::CodePoint;
use parser::preprocessor::InputStream;

use super::CSSToken;
use crate::{codepoint::CSSCodePoint, tokenization::token::HashFlag};

// ---- //
// Type //
// ---- //

pub(crate) type CSSInputStream<Iter> = InputStream<Iter, CodePoint>;

// --------- //
// Structure //
// --------- //

/// Pour tokeniser un flux de points de code en un flux de jetons CSS en
/// entrée, nous devons consommer de manière répétée un jeton en entrée
/// jusqu'à ce qu'un <EOF-token> soit atteint, en poussant chacun des
/// jetons retournés dans un flux.
pub struct CSSTokenizer<Chars> {
    pub(crate) stream: CSSInputStream<Chars>,
}

// -------------- //
// Implémentation //
// -------------- //

impl<C> CSSTokenizer<C>
where
    C: Iterator<Item = CodePoint>,
{
    /// Crée un nouveau [tokenizer](CSSTokenizer) à partir d'un flux de
    /// points de code.
    pub fn new(iter: C) -> Self {
        // Remplacer tous les points de code
        //   - U+000D CARRIAGE RETURN (CR),
        //   - U+000C FORM FEED (FF)
        //   - U+000D CARRIAGE RETURN (CR) suivis de U+000A LINE FEED (LF)
        // par un seul point de code U+000A LINE FEED (LF).
        //
        // Remplacer tout point de code U+0000 NULL ou de substitution en
        // entrée par U+FFFD REPLACEMENT CHARACTER (�).
        let stream =
            CSSInputStream::new(iter).with_pre_scan(|ch| match ch {
                | Some('\r' | '\n' | '\x0C') => Some('\n'),
                | Some('\0') => Some(CodePoint::REPLACEMENT_CHARACTER),
                | n => n,
            });

        Self { stream }
    }
}
