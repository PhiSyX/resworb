/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub mod declaration;
pub mod list_of_declarations;
pub mod list_of_rules;
pub mod rule;
pub mod style_blocks_contents;
pub mod stylesheet;

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
