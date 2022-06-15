/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{style_blocks_content::CSSStyleBlocksContents, CSSParser};

impl CSSParser {
    /// Analyse le contenu d'un bloc de style
    pub fn style_blocks_contents(&mut self) -> CSSStyleBlocksContents {
        self.consume_style_blocks_contents()
    }
}

// ---- //
// Test //
// ---- //

#[cfg(test)]
mod tests {
    use crate::test_the_str;

    #[test]
    fn test_parse_a_style_blocks_contents() {
        let mut parser = test_the_str!("#foo { color: red; }");
        assert_eq!(parser.style_blocks_contents(), vec![]);
    }
}
