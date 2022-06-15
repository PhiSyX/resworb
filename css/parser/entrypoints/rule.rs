/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use parser::StreamIteratorInterface;

use crate::{
    grammars::{CSSRule, CSSRuleError},
    CSSParser,
};

impl CSSParser {
    /// Analyse d'une règle
    pub fn rule(&mut self) -> Result<CSSRule, CSSRuleError> {
        self.tokens.advance_as_long_as_possible(
            |token| token.is_whitespace(),
            None,
        );

        let rule = match self.next_input_token() {
            // <EOF-token>
            //
            // Retourner une erreur de syntaxe.
            | variant if variant.is_eof() => None, /* <--------------------------------------- v |> Erreur de syntaxe */

            // <at-keyword-token>
            //
            // Consommer une règle at-rule à partir de l'entrée, et
            // assigner  la valeur de retour à la règle.
            | variant if variant.is_at_keyword() => {
                CSSRule::AtRule(self.consume_at_rule()).into()
            }

            // Anything else
            //
            // Consommer une règle qualifiée à partir de l'entrée et
            // assigner la valeur de retour à la règle. Si rien n'est
            // retourné, nous devons retourner une erreur de syntaxe.
            | _ => {
                self.consume_qualified_rule().map(CSSRule::QualifiedRule) /* --- v |> Erreur de syntaxe si None */
            }
        };

        self.tokens.advance_as_long_as_possible(
            |token| token.is_whitespace(),
            None,
        );

        self.tokens
            .next_input()
            .filter(|token| token.is_eof())
            .and(rule)
            .ok_or(CSSRuleError::SyntaxError) /* ------------------------------- ^ */
    }
}

// ---- //
// Test //
// ---- //

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        at_rule::CSSAtRule, test_the_str, tokenization::CSSToken,
    };

    #[test]
    fn test_parse_rule() {
        let mut parser = test_the_str!(r#"@charset "utf-8""#);
        assert_eq!(
            parser.rule(),
            Ok(CSSRule::AtRule(
                CSSAtRule::default()
                    .with_name(&CSSToken::AtKeyword("charset".into()))
                    .with_prelude([
                        CSSToken::Whitespace, // <-- ???
                        CSSToken::String("utf-8".into())
                    ])
            ))
        );
    }
}
