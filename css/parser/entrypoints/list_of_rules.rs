/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{grammars::CSSRuleList, CSSParser};

impl<T> CSSParser<T> {
    /// Analyse une liste de règles
    pub fn list_of_rules(&mut self) -> CSSRuleList {
        self.consume_list_of_rules(false)
    }
}

// ---- //
// Test //
// ---- //

#[cfg(test)]
mod tests {
    use crate::{
        grammars::CSSRule,
        qualified_rule::CSSQualifiedRule,
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

        assert_eq!(
            parser.list_of_rules(),
            [
                CSSRule::QualifiedRule(
                    CSSQualifiedRule::default()
                        .with_prelude([
                            CSSToken::Hash("foo-1".into(), HashFlag::ID),
                            CSSToken::Whitespace
                        ])
                        .with_block(
                            CSSSimpleBlock::new(
                                CSSToken::LeftCurlyBracket
                            )
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
                ),
                CSSRule::QualifiedRule(
                    CSSQualifiedRule::default()
                        .with_prelude([
                            CSSToken::Hash("foo-2".into(), HashFlag::ID),
                            CSSToken::Whitespace
                        ])
                        .with_block(
                            CSSSimpleBlock::new(
                                CSSToken::LeftCurlyBracket
                            )
                            .set_values([
                                CSSToken::Whitespace,
                                CSSToken::Ident("color".into()),
                                CSSToken::Colon,
                                CSSToken::Whitespace,
                                CSSToken::Ident("blue".into()),
                                CSSToken::Semicolon,
                                CSSToken::Whitespace,
                            ])
                        )
                )
            ]
        );
    }
}
