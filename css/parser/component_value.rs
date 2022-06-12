/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{
    function::CSSFunction, preserved_tokens::CSSPreservedToken,
    simple_block::CSSSimpleBlock, tokenization::CSSToken,
};

// --------- //
// Structure //
// --------- //

/// Une valeur de composant est l'un des [jetons
/// conservés](CSSPreservedToken), une [fonction](CSSFunction) ou un
/// [bloc simple](CSSSimpleBlock).
#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub(crate) enum CSSComponentValue {
    Preserved(CSSPreservedToken),
    Function(CSSFunction),
    SimpleBlock(CSSSimpleBlock),
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl From<CSSPreservedToken> for CSSComponentValue {
    fn from(token: CSSPreservedToken) -> Self {
        Self::Preserved(token)
    }
}

impl From<CSSFunction> for CSSComponentValue {
    fn from(function: CSSFunction) -> Self {
        Self::Function(function)
    }
}

impl From<CSSSimpleBlock> for CSSComponentValue {
    fn from(simple_block: CSSSimpleBlock) -> Self {
        Self::SimpleBlock(simple_block)
    }
}

impl From<CSSToken> for CSSComponentValue {
    fn from(token: CSSToken) -> Self {
        match token {
            | CSSToken::EOF
            | CSSToken::Ident(_)
            | CSSToken::AtKeyword(_)
            | CSSToken::Hash(_, _)
            | CSSToken::String(_)
            | CSSToken::BadString
            | CSSToken::Url(_)
            | CSSToken::BadUrl
            | CSSToken::Delim(_)
            | CSSToken::Number(_, _)
            | CSSToken::Percentage(_)
            | CSSToken::Dimension(_, _, _)
            | CSSToken::Whitespace
            | CSSToken::CDO
            | CSSToken::CDC
            | CSSToken::Colon
            | CSSToken::Semicolon
            | CSSToken::Comma => Self::Preserved(token.into()),

            | CSSToken::Function(_) => Self::Function(token.into()),

            | CSSToken::LeftSquareBracket
            | CSSToken::RightSquareBracket
            | CSSToken::LeftParenthesis
            | CSSToken::RightParenthesis
            | CSSToken::LeftCurlyBracket
            | CSSToken::RightCurlyBracket => {
                Self::SimpleBlock(token.into())
            }
        }
    }
}
