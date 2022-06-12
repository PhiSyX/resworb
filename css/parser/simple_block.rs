/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{component_value::CSSComponentValue, tokenization::CSSToken};

// --------- //
// Structure //
// --------- //

/// Un bloc simple a un jeton associé (soit un
/// <[\[-token](CSSToken::LeftSquareBracket)>,
/// <[(-token](CSSToken::LeftParenthesis)>, ou
/// <[{-token](CSSToken::LeftCurlyBracket)>) et une valeur constituée d'une
/// liste de [valeurs de composants](CSSComponentValue).
///
/// {}-block, []-block, et ()-block font spécifiquement
/// référence à un bloc simple avec le jeton associé correspondant.
#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub struct CSSSimpleBlock {
    token: CSSToken,
    value: Vec<CSSComponentValue>,
}

pub const CURLY_BRACKET_BLOCK: CSSSimpleBlock = CSSSimpleBlock {
    token: CSSToken::LeftCurlyBracket,
    value: vec![],
};

// -------------- //
// Implémentation //
// -------------- //

impl CSSSimpleBlock {
    pub(crate) fn new(token: CSSToken) -> Self {
        Self {
            token,
            value: vec![],
        }
    }

    pub(crate) fn set_values(
        mut self,
        value: impl IntoIterator<Item = impl Into<CSSComponentValue>>,
    ) -> Self {
        self.value = value.into_iter().map(Into::into).collect();
        self
    }
}

impl CSSSimpleBlock {
    pub(crate) fn append(&mut self, value: CSSComponentValue) {
        self.value.push(value);
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl From<CSSToken> for CSSSimpleBlock {
    fn from(token: CSSToken) -> Self {
        match token {
            | token @ (CSSToken::LeftSquareBracket
            | CSSToken::LeftParenthesis
            | CSSToken::LeftCurlyBracket
            | CSSToken::RightSquareBracket
            | CSSToken::RightParenthesis
            | CSSToken::RightCurlyBracket) => CSSSimpleBlock::new(token),
            | _ => panic!("Impossible de convertir le jeton {token:?} en CSSSimpleBlock."),
        }
    }
}
