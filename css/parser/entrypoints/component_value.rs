/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use parser::StreamIteratorInterface;

use crate::{
    component_value::{CSSComponentValue, CSSComponentValueError},
    CSSParser,
};

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
}

// ---- //
// Test //
// ---- //

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        function::CSSFunction,
        test_the_str,
        tokenization::{CSSToken, DimensionUnit, NumberFlag},
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
}
