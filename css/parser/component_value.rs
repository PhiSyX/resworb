/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{
    function::CSSFunction,
    preserved_tokens::{CSSPreservedToken, CSSPreservedTokenError},
    simple_block::CSSSimpleBlock,
    tokenization::CSSToken,
};

// ---- //
// Type //
// ---- //

pub type CSSComponentValuesList = Vec<CSSComponentValue>;

// --------- //
// Structure //
// --------- //

/// Une valeur de composant est l'un des [jetons
/// conservés](CSSPreservedToken), une [fonction](CSSFunction) ou un
/// [bloc simple](CSSSimpleBlock).
#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq, Eq)]
pub enum CSSComponentValue {
    Preserved(CSSPreservedToken),
    Function(CSSFunction),
    SimpleBlock(CSSSimpleBlock),
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq, Eq)]
pub enum CSSComponentValueError {
    ConsumedToken,
    SyntaxError,
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl CSSComponentValue {
    pub(crate) fn simple_block(&self) -> Option<&CSSSimpleBlock> {
        match self {
            | Self::SimpleBlock(simple_block) => Some(simple_block),
            | _ => None,
        }
    }

    pub(crate) fn simple_block_unchecked(&self) -> &CSSSimpleBlock {
        self.simple_block().expect("Simple bloc")
    }
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

impl TryFrom<CSSToken> for CSSComponentValue {
    type Error = CSSComponentValueError;

    fn try_from(token: CSSToken) -> Result<Self, Self::Error> {
        match token {
            | CSSToken::EOF
            | CSSToken::Ident(_)
            | CSSToken::AtKeyword(_)
            | CSSToken::Hash(_, _)
            | CSSToken::String(_)
            | CSSToken::Url(_)
            | CSSToken::Delim(_)
            | CSSToken::Number(_, _)
            | CSSToken::Percentage(_)
            | CSSToken::Dimension(_, _, _)
            | CSSToken::Whitespace
            | CSSToken::CDO
            | CSSToken::CDC
            | CSSToken::Colon
            | CSSToken::Semicolon
            | CSSToken::Comma => token
                .try_into()
                .map(Self::Preserved)
                .map_err(|err| match err {
                    | CSSPreservedTokenError::ConsumedToken => {
                        Self::Error::ConsumedToken
                    }
                    | CSSPreservedTokenError::SyntaxError => {
                        Self::Error::SyntaxError
                    }
                }),

            | CSSToken::Function(_) => Ok(Self::Function(token.into())),

            | CSSToken::LeftCurlyBracket
            | CSSToken::LeftSquareBracket
            | CSSToken::LeftParenthesis => {
                Ok(Self::SimpleBlock(token.into()))
            }

            | CSSToken::BadString
            | CSSToken::BadUrl
            | CSSToken::RightCurlyBracket
            | CSSToken::RightSquareBracket
            | CSSToken::RightParenthesis => {
                Err(Self::Error::ConsumedToken)
            }
        }
    }
}
