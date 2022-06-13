/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{grammars::CSSStyleSheet, CSSParser};

impl<T> CSSParser<T> {
    /// Analyse d'une feuille de style.
    pub fn stylesheet(&mut self) -> CSSStyleSheet {
        self.consume_list_of_rules(true)
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
                        CSSSimpleBlock::new(CSSToken::RightCurlyBracket)
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
