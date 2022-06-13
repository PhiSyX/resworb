/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use parser::StreamIteratorInterface;

use crate::{
    grammars::{CSSRule, CSSRuleError},
    tokenization::CSSToken,
    CSSParser,
};

impl<T> CSSParser<T> {
    /// Analyse une règle
    pub fn rule(&mut self) -> Result<CSSRule, CSSRuleError> {
        self.tokens.advance_as_long_as_possible(
            |token| *token == CSSToken::Whitespace,
            None,
        );

        let rule = match self.consume_next_input_token() {
            // <EOF-token>
            //
            // Retourner une erreur de syntaxe.
            | CSSToken::EOF => {
                return Err(CSSRuleError::SyntaxError);
            }

            // <at-keyword-token>
            //
            // Consommer une règle at-rule à partir de l'entrée, et
            // assigner  la valeur de retour à la règle.
            | CSSToken::AtKeyword(_) => {
                CSSRule::AtRule(self.consume_at_rule())
            }

            // Anything else
            //
            // Consommer une règle qualifiée à partir de l'entrée et
            // assigner la valeur de retour à la règle. Si rien n'est
            // retourné, nous devons retourner une erreur de syntaxe.
            | _ => {
                if let Some(qualified_rule) = self.consume_qualified_rule()
                {
                    CSSRule::QualifiedRule(qualified_rule)
                } else {
                    return Err(CSSRuleError::SyntaxError);
                }
            }
        };

        self.tokens.advance_as_long_as_possible(
            |token| *token == CSSToken::Whitespace,
            None,
        );

        if self.next_input_token() == CSSToken::EOF {
            Ok(rule)
        } else {
            Err(CSSRuleError::SyntaxError)
        }
    }
}

// ---- //
// Test //
// ---- //

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{at_rule::CSSAtRule, tokenization::CSSToken};

    macro_rules! test_the_str {
        ($str:literal) => {{
            let s = $str;
            let parser: CSSParser<CSSToken> = CSSParser::new(s.chars());
            parser
        }};
    }

    #[test]
    fn test_parse_rule() {
        let mut parser = test_the_str!(r#"@charset "utf-8""#);
        assert_eq!(
            parser.rule(),
            Ok(CSSRule::AtRule(
                CSSAtRule::default()
                    .with_name(&CSSToken::AtKeyword("charset".into()))
                    .with_prelude([CSSToken::String("utf-8".into())])
            ))
        );
    }
}
