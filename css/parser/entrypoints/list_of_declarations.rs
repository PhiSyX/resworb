/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{declaration::CSSDeclarationList, CSSParser};

impl CSSParser {
    /// Analyse une liste de déclarations
    pub fn list_of_declarations(&mut self) -> CSSDeclarationList {
        self.consume_list_of_declarations()
    }
}

// ---- //
// Test //
// ---- //

#[cfg(test)]
mod tests {
    use crate::{
        declaration::CSSDeclaration, style_blocks_content::CSSStyleBlock,
        test_the_str, tokenization::CSSToken,
    };

    #[test]
    fn test_parse_a_list_of_declarations() {
        let mut parser = test_the_str!(
            "
            color: red;
            background-color: blue;
            "
        );

        assert_eq!(
            parser.list_of_declarations(),
            [
                CSSStyleBlock::Declaration(
                    CSSDeclaration::default()
                        .with_name(&CSSToken::Ident("color".into()))
                        .with_values([CSSToken::Ident("red".into())])
                ),
                CSSStyleBlock::Declaration(
                    CSSDeclaration::default()
                        .with_name(&CSSToken::Ident(
                            "background-color".into()
                        ))
                        .with_values([CSSToken::Ident("blue".into())])
                ),
            ]
        );
    }
}
