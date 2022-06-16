/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use parser::{StreamIterator, StreamTokenIterator};

use crate::{
    at_rule::CSSAtRule, qualified_rule::CSSQualifiedRule, CSSParser,
};

#[cfg(test)]
#[macro_export]
macro_rules! test_the_str {
    ($str:literal) => {{
        use $crate::CSSParser;
        let s = $str;
        let parser: CSSParser = CSSParser::new(s.chars());
        parser
    }};
}

// ---- //
// Type //
// ---- //

/// la production `<rule-list>` représente une liste de règles, et ne peut
/// être utilisée dans les grammaires que comme seule valeur dans un bloc.
/// Elle indique que le contenu du bloc doit être analysé à l'aide de
/// l'algorithme consume a list of rules.
// NOTE(phisyx): peut-être améliorer ce type.
pub type CSSRuleList = Vec<CSSRule>;

/// la production `<stylesheet>` représente une liste de règles. Elle est
/// identique à `<rule-list>`, sauf que les blocs qui l'utilisent acceptent
/// par défaut toutes les règles qui ne sont pas autrement limitées à un
/// contexte particulier.
// NOTE(phisyx): peut-être améliorer ce type.
pub type CSSStyleSheet = CSSRuleList;

// ----------- //
// Énumération //
// ----------- //

/// Voir le tableau <https://www.w3.org/TR/css-syntax-3/#declaration-rule-list>
#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub enum CSSRule {
    QualifiedRule(CSSQualifiedRule),
    AtRule(CSSAtRule),
}

#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub enum CSSRuleError {
    SyntaxError,
}

// ----------- //
// Entry Point //
// ----------- //

impl CSSParser {
    /// Analyse une liste de règles
    pub fn list_of_rules(&mut self) -> CSSRuleList {
        self.consume_list_of_rules(false)
    }

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
            .next_token()
            .filter(|token| token.is_eof())
            .and(rule)
            .ok_or(CSSRuleError::SyntaxError) /* ------------------------------- ^ */
    }

    /// Analyse d'une feuille de style.
    pub fn stylesheet(&mut self) -> CSSStyleSheet {
        self.consume_list_of_rules(true)
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl From<CSSQualifiedRule> for CSSRule {
    fn from(qualified_rule: CSSQualifiedRule) -> Self {
        Self::QualifiedRule(qualified_rule)
    }
}

impl From<CSSAtRule> for CSSRule {
    fn from(at_rule: CSSAtRule) -> Self {
        Self::AtRule(at_rule)
    }
}

// ---- //
// Test //
// ---- //

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        simple_block::CSSSimpleBlock,
        test_the_str,
        tokenization::{CSSToken, HashFlag},
    };

    #[test]
    fn test_parse_a_list_of_rules() {
        let mut parser = test_the_str!(
            "
            #foo-1 { color: red; }
            #foo-2 { color: blue; }
            "
        );

        let mut block = CSSSimpleBlock::new(CSSToken::LeftCurlyBracket)
            .set_values([
                CSSToken::Whitespace,
                CSSToken::Ident("color".into()),
                CSSToken::Colon,
                CSSToken::Whitespace,
                CSSToken::Ident("red".into()),
                CSSToken::Semicolon,
                CSSToken::Whitespace,
                CSSToken::Whitespace,
                CSSToken::Hash("foo-2".into(), HashFlag::ID),
                CSSToken::Whitespace,
            ]);

        block.append(
            CSSSimpleBlock::new(CSSToken::LeftCurlyBracket).set_values([
                CSSToken::Whitespace,
                CSSToken::Ident("color".into()),
                CSSToken::Colon,
                CSSToken::Whitespace,
                CSSToken::Ident("blue".into()),
                CSSToken::Semicolon,
                CSSToken::Whitespace,
                CSSToken::Whitespace,
            ]),
        );

        assert_eq!(
            parser.list_of_rules(),
            [CSSRule::QualifiedRule(
                CSSQualifiedRule::default()
                    .with_prelude([
                        CSSToken::Hash("foo-1".into(), HashFlag::ID),
                        CSSToken::Whitespace,
                    ])
                    .with_block(block)
            ),],
        );
    }

    #[test]
    fn test_parse_a_rule() {
        let mut parser = test_the_str!(r#"@charset "utf-8""#);
        assert_eq!(
            parser.rule(),
            Ok(CSSRule::AtRule(
                CSSAtRule::default().with_name("charset").with_prelude([
                    CSSToken::Whitespace, // <-- ???
                    CSSToken::String("utf-8".into())
                ])
            ))
        );
    }

    #[test]
    fn test_parse_a_stylesheet() {
        let mut parser = test_the_str!("#foo { color: red; }");

        assert_eq!(
            parser.stylesheet(),
            [CSSRule::QualifiedRule(
                CSSQualifiedRule::default()
                    .with_prelude([
                        CSSToken::Hash("foo".into(), HashFlag::ID),
                        CSSToken::Whitespace
                    ])
                    .with_block(
                        CSSSimpleBlock::new(CSSToken::LeftCurlyBracket)
                            .set_values([
                                CSSToken::Whitespace,
                                CSSToken::Ident("color".into()),
                                CSSToken::Colon,
                                CSSToken::Whitespace,
                                CSSToken::Ident("red".into()),
                                CSSToken::Semicolon,
                                CSSToken::Whitespace,
                            ])
                    )
            )]
        );
    }
}
