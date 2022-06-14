/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use parser::StreamIteratorInterface;

use crate::{
    declaration::CSSDeclaration, grammars::CSSRuleError,
    tokenization::CSSToken, CSSParser,
};

impl<T> CSSParser<T> {
    /// Analyse d'une déclaration
    pub fn declaration(&mut self) -> Result<CSSDeclaration, CSSRuleError> {
        self.tokens.advance_as_long_as_possible(
            |token| *token == CSSToken::Whitespace,
            None,
        );

        let declaration = match self.next_input_token() {
            | CSSToken::Ident(_) => self.consume_declaration(),
            | _ => None,
        };

        declaration.ok_or(CSSRuleError::SyntaxError)
    }
}

// ---- //
// Test //
// ---- //

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        component_value::CSSComponentValue, test_the_str,
        tokenization::CSSToken,
    };

    #[test]
    fn test_parse_declaration() {
        let mut parser = test_the_str!(r#"color: red;"#);
        assert_eq!(
            parser.declaration(),
            Ok(CSSDeclaration::default()
                .with_name(&CSSToken::Ident("color".into()))
                .with_values([
                    CSSComponentValue::Preserved(
                        CSSToken::Ident("red".into()).into()
                    ),
                    CSSComponentValue::Preserved(
                        CSSToken::Semicolon.into()
                    )
                ]))
        );
    }

    #[test]
    fn test_parse_declaration_is_not() {
        let mut parser = test_the_str!(r#".class {}"#);
        assert_eq!(parser.declaration(), Err(CSSRuleError::SyntaxError));
    }
}
