/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use parser::StreamIterator;

use crate::{
    function::CSSFunction,
    preserved_tokens::{CSSPreservedToken, CSSPreservedTokenError},
    simple_block::CSSSimpleBlock,
    tokenization::CSSToken,
    CSSParser,
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

// ----------- //
// Entry Point //
// ----------- //

impl CSSParser {
    /// Analyse d'une valeur de composant.
    pub fn component_value(
        &mut self,
    ) -> Result<CSSComponentValue, CSSComponentValueError> {
        self.tokens.advance_as_long_as_possible(
            |token| token.is_whitespace(),
            None,
        );

        if self.next_input_token().is_eof() {
            return Err(CSSComponentValueError::SyntaxError);
        }

        let value = self.consume_component_value();

        self.tokens.advance_as_long_as_possible(
            |token| token.is_whitespace(),
            None,
        );

        if self.next_input_token().is_eof() {
            value.ok_or(CSSComponentValueError::SyntaxError)
        } else {
            Err(CSSComponentValueError::SyntaxError)
        }
    }

    /// Analyse d'une liste de valeurs de composants
    pub fn list_of_component_values(&mut self) -> CSSComponentValuesList {
        let mut component_values: CSSComponentValuesList =
            CSSComponentValuesList::default();

        loop {
            match self.consume_component_value() {
                | None
                | Some(CSSComponentValue::Preserved(
                    CSSPreservedToken(CSSToken::EOF),
                )) => break,

                | Some(component_value) => {
                    component_values.push(component_value)
                }
            }
        }

        component_values
    }

    /// Analyse d'une liste de valeurs de composants séparés par des
    /// virgules.
    pub fn comma_separated_list_of_component_values(
        &mut self,
    ) -> CSSComponentValuesList {
        let mut list_of_cvls = CSSComponentValuesList::default();

        loop {
            match self.consume_component_value() {
                | None
                | Some(CSSComponentValue::Preserved(
                    CSSPreservedToken(CSSToken::EOF),
                )) => break,

                | Some(CSSComponentValue::Preserved(
                    CSSPreservedToken(CSSToken::Comma),
                )) => continue,

                | Some(component_value) => {
                    list_of_cvls.push(component_value)
                }
            }
        }

        list_of_cvls
    }
}

// -------------- //
// Implémentation //
// -------------- //

impl CSSComponentValue {
    pub(super) fn simple_block(&self) -> Option<&CSSSimpleBlock> {
        match self {
            | Self::SimpleBlock(simple_block) => Some(simple_block),
            | _ => None,
        }
    }

    pub(super) fn simple_block_unchecked(&self) -> &CSSSimpleBlock {
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

// ---- //
// Test //
// ---- //

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        test_the_str,
        tokenization::{DimensionUnit, NumberFlag},
    };

    #[test]
    fn test_parse_a_component_value() {
        let mut parser = test_the_str!("clamp(20px, 5vw, 50px)");
        assert_eq!(
            parser.component_value(),
            Ok(CSSComponentValue::Function(
                CSSFunction::new("clamp").with_values([
                    CSSToken::Dimension(
                        20.0,
                        NumberFlag::Integer,
                        DimensionUnit("px".into())
                    ),
                    CSSToken::Comma,
                    CSSToken::Whitespace,
                    CSSToken::Dimension(
                        5.0,
                        NumberFlag::Integer,
                        DimensionUnit("vw".into())
                    ),
                    CSSToken::Comma,
                    CSSToken::Whitespace,
                    CSSToken::Dimension(
                        50.0,
                        NumberFlag::Integer,
                        DimensionUnit("px".into())
                    ),
                ])
            ))
        );
    }

    #[test]
    fn test_parse_a_list_of_component_values() {
        let mut parser = test_the_str!(
            "
            url(img.png);
            @font-face {
                font-family: var(--font-family, 'Roboto');
                src: url(fonts/Roboto-Regular.ttf);
            }
            "
        );

        let mut block = CSSSimpleBlock::new(CSSToken::LeftCurlyBracket)
            .set_values([
                CSSToken::Whitespace,
                CSSToken::Ident("font-family".into()),
                CSSToken::Colon,
                CSSToken::Whitespace,
            ]);

        let var_fn = CSSFunction::new("var").with_values([
            CSSToken::Ident("--font-family".into()),
            CSSToken::Comma,
            CSSToken::Whitespace,
            CSSToken::String("Roboto".into()),
        ]);

        block.append(var_fn);
        block.append(CSSToken::Semicolon);
        block.append(CSSToken::Whitespace);
        block.append(CSSToken::Ident("src".into()));
        block.append(CSSToken::Colon);
        block.append(CSSToken::Whitespace);
        block.append(CSSToken::Url("fonts/Roboto-Regular.ttf".into()));
        block.append(CSSToken::Semicolon);
        block.append(CSSToken::Whitespace);
        block.append(CSSToken::Whitespace);

        assert_eq!(
            parser.list_of_component_values(),
            [
                CSSToken::Whitespace.try_into().unwrap(),
                CSSToken::Url("img.png".into()).try_into().unwrap(),
                CSSToken::Semicolon.try_into().unwrap(),
                CSSToken::Whitespace.try_into().unwrap(),
                CSSToken::AtKeyword("font-face".into())
                    .try_into()
                    .unwrap(),
                CSSToken::Whitespace.try_into().unwrap(),
                block.try_into().unwrap(),
            ]
        );
    }

    #[test]
    fn test_parse_a_comma_separated_list_of_component_values() {
        let mut parser = test_the_str!("url(img.png), url(img2.png);");

        assert_eq!(
            parser.comma_separated_list_of_component_values(),
            [
                CSSToken::Url("img.png".into()).try_into().unwrap(),
                CSSToken::Whitespace.try_into().unwrap(),
                CSSToken::Url("img2.png".into()).try_into().unwrap(),
                CSSToken::Semicolon.try_into().unwrap(),
            ]
        );
    }
}
