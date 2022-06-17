/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use infra::primitive::codepoint::CodePoint;
use parser::{stream::TokenStream, StreamToken};

use super::CSSToken;
use crate::{
    component_value::CSSComponentValue,
    preserved_tokens::CSSPreservedToken, simple_block::CSSSimpleBlock,
};

// --------- //
// Structure //
// --------- //

/// Pour tokeniser un flux de points de code en un flux de jetons
/// CSS en entrée, nous devons consommer de manière répétée un jeton
/// en entrée jusqu'à ce qu'un <EOF-token> soit atteint, en poussant
/// chacun des jetons retournés dans un flux.
pub type CSSTokenStream = TokenStream<CSSTokenVariant>;

#[derive(Debug)]
#[derive(Clone)]
pub enum CSSTokenVariant {
    ComponentValue(CSSComponentValue),
    Token(CSSToken),
}

// -------------- //
// Implémentation //
// -------------- //

impl CSSTokenVariant {
    pub(crate) fn is_at_keyword(&self) -> bool {
        matches!(
            self,
            Self::Token(CSSToken::AtKeyword(_))
                | Self::ComponentValue(CSSComponentValue::Preserved(
                    CSSPreservedToken(CSSToken::AtKeyword(_))
                ))
        )
    }

    pub(crate) fn is_cdo(&self) -> bool {
        matches!(
            self,
            Self::Token(CSSToken::CDO)
                | Self::ComponentValue(CSSComponentValue::Preserved(
                    CSSPreservedToken(CSSToken::CDO)
                ))
        )
    }

    pub(crate) fn is_cdt(&self) -> bool {
        matches!(
            self,
            Self::Token(CSSToken::CDC)
                | Self::ComponentValue(CSSComponentValue::Preserved(
                    CSSPreservedToken(CSSToken::CDC)
                ))
        )
    }

    pub(crate) fn is_colon(&self) -> bool {
        matches!(
            self,
            Self::Token(CSSToken::Colon)
                | Self::ComponentValue(CSSComponentValue::Preserved(
                    CSSPreservedToken(CSSToken::Colon)
                ))
        )
    }

    pub(crate) fn is_delimiter(&self) -> bool {
        matches!(
            self,
            Self::Token(CSSToken::Delim(_))
                | Self::ComponentValue(CSSComponentValue::Preserved(
                    CSSPreservedToken(CSSToken::Delim(_))
                ))
        )
    }

    pub(crate) fn is_delimiter_with(&self, ch: CodePoint) -> bool {
        if !self.is_delimiter() {
            return false;
        }

        if let Some(CSSToken::Delim(delim)) = self.token() {
            ch.eq(delim)
        } else {
            false
        }
    }

    pub(crate) fn is_eof(&self) -> bool {
        matches!(
            self,
            Self::Token(CSSToken::EOF)
                | Self::ComponentValue(CSSComponentValue::Preserved(
                    CSSPreservedToken(CSSToken::EOF)
                ))
        )
    }

    pub(crate) fn is_function(&self) -> bool {
        matches!(
            self,
            Self::Token(CSSToken::Function(_))
                | Self::ComponentValue(CSSComponentValue::Function(_))
        )
    }

    pub(crate) fn is_ident(&self) -> bool {
        matches!(
            self,
            Self::Token(CSSToken::Ident(_))
                | Self::ComponentValue(CSSComponentValue::Preserved(
                    CSSPreservedToken(CSSToken::Ident(_))
                ))
        )
    }

    pub(crate) fn is_left_curly_bracket(&self) -> bool {
        matches!(
            self,
            Self::Token(CSSToken::LeftCurlyBracket)
                | Self::ComponentValue(CSSComponentValue::Preserved(
                    CSSPreservedToken(CSSToken::LeftCurlyBracket)
                ))
        )
    }

    pub(crate) fn is_left_square_bracket(&self) -> bool {
        matches!(
            self,
            Self::Token(CSSToken::LeftSquareBracket)
                | Self::ComponentValue(CSSComponentValue::Preserved(
                    CSSPreservedToken(CSSToken::LeftSquareBracket)
                ))
        )
    }

    pub(crate) fn is_left_parenthesis(&self) -> bool {
        matches!(
            self,
            Self::Token(CSSToken::LeftParenthesis)
                | Self::ComponentValue(CSSComponentValue::Preserved(
                    CSSPreservedToken(CSSToken::LeftParenthesis)
                ))
        )
    }

    pub(crate) fn is_mirror(&self, cmp_ending_token: &CSSToken) -> bool {
        if !(self.is_left_curly_bracket()
            || self.is_left_square_bracket()
            || self.is_left_parenthesis())
        {
            return false;
        }

        if self.is_right_curly_bracket()
            || self.is_right_square_bracket()
            || self.is_right_parenthesis()
        {
            return false;
        }

        let self_ending_token = self.token_unchecked();
        self_ending_token == cmp_ending_token
    }

    pub(crate) fn is_right_curly_bracket(&self) -> bool {
        matches!(
            self,
            Self::Token(CSSToken::RightCurlyBracket)
                | Self::ComponentValue(CSSComponentValue::Preserved(
                    CSSPreservedToken(CSSToken::RightCurlyBracket)
                ))
        )
    }

    pub(crate) fn is_right_square_bracket(&self) -> bool {
        matches!(
            self,
            Self::Token(CSSToken::RightSquareBracket)
                | Self::ComponentValue(CSSComponentValue::Preserved(
                    CSSPreservedToken(CSSToken::RightSquareBracket)
                ))
        )
    }

    pub(crate) fn is_right_parenthesis(&self) -> bool {
        matches!(
            self,
            Self::Token(CSSToken::RightParenthesis)
                | Self::ComponentValue(CSSComponentValue::Preserved(
                    CSSPreservedToken(CSSToken::RightParenthesis)
                ))
        )
    }

    pub(crate) fn is_semicolon(&self) -> bool {
        matches!(
            self,
            Self::Token(CSSToken::Semicolon)
                | Self::ComponentValue(CSSComponentValue::Preserved(
                    CSSPreservedToken(CSSToken::Semicolon)
                ))
        )
    }

    pub(crate) fn is_simple_block_with(
        &self,
        cmp_token: CSSToken,
    ) -> bool {
        match self {
            | CSSTokenVariant::ComponentValue(
                CSSComponentValue::SimpleBlock(CSSSimpleBlock {
                    token,
                    ..
                }),
            ) => cmp_token.eq(token),
            | _ => false,
        }
    }

    pub(crate) fn is_whitespace(&self) -> bool {
        matches!(
            self,
            Self::Token(CSSToken::Whitespace)
                | Self::ComponentValue(CSSComponentValue::Preserved(
                    CSSPreservedToken(CSSToken::Whitespace)
                ))
        )
    }

    pub(crate) fn component_value(&self) -> Option<CSSComponentValue> {
        match self {
            | Self::ComponentValue(component_value) => {
                Some(component_value).cloned()
            }
            | Self::Token(token) => token.to_owned().try_into().ok(),
        }
    }

    pub(crate) fn component_value_unchecked(&self) -> CSSComponentValue {
        self.component_value()
            .expect("impossible de récupérer une valeur de composant")
    }

    pub(crate) fn function_name(&self) -> String {
        assert!(self.is_function());

        match self {
            | Self::ComponentValue(CSSComponentValue::Function(
                function,
            )) => function.name().to_owned(),
            | Self::Token(token) => token.name(),
            | _ => unreachable!(),
        }
    }

    pub(crate) fn token(&self) -> Option<&CSSToken> {
        Some(match self {
            | Self::Token(token)
            | Self::ComponentValue(CSSComponentValue::Preserved(
                CSSPreservedToken(token),
            )) => token,
            | _ => return None,
        })
    }

    pub(crate) fn token_unchecked(&self) -> &CSSToken {
        self.token().expect("impossible de récupérer le jeton.")
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl StreamToken for CSSTokenVariant {
    fn eof() -> Self {
        CSSTokenVariant::Token(CSSToken::EOF)
    }
}

impl PartialEq for CSSTokenVariant {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            | (
                CSSTokenVariant::ComponentValue(c1),
                CSSTokenVariant::ComponentValue(c2),
            ) => c1 == c2,
            | (
                CSSTokenVariant::ComponentValue(c1),
                CSSTokenVariant::Token(t2),
            ) => {
                let maybe_t1: Result<CSSComponentValue, _> =
                    t2.to_owned().try_into();

                if let Ok(ref t1) = maybe_t1 {
                    c1 == t1
                } else {
                    false
                }
            }
            | (
                CSSTokenVariant::Token(t1),
                CSSTokenVariant::ComponentValue(c2),
            ) => {
                let maybe_t2: Result<CSSComponentValue, _> =
                    t1.to_owned().try_into();

                if let Ok(ref t2) = maybe_t2 {
                    c2 == t2
                } else {
                    false
                }
            }
            | (CSSTokenVariant::Token(t1), CSSTokenVariant::Token(t2)) => {
                t1 == t2
            }
        }
    }
}

impl Eq for CSSTokenVariant {}

impl From<CSSComponentValue> for CSSTokenVariant {
    fn from(component_value: CSSComponentValue) -> Self {
        Self::ComponentValue(component_value)
    }
}

impl From<CSSToken> for CSSTokenVariant {
    fn from(token: CSSToken) -> Self {
        Self::Token(token)
    }
}
