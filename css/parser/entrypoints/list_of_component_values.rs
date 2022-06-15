/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{
    component_value::{CSSComponentValue, CSSComponentValuesList},
    preserved_tokens::CSSPreservedToken,
    tokenization::CSSToken,
    CSSParser,
};

impl CSSParser {
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
}

// ---- //
// Test //
// ---- //

#[cfg(test)]
mod tests {
    use crate::{
        function::CSSFunction, simple_block::CSSSimpleBlock, test_the_str,
        tokenization::CSSToken,
    };

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
}
