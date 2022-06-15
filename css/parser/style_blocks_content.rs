/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{declaration::CSSDeclaration, grammars::CSSRule, CSSParser};

// ---- //
// Type //
// ---- //

pub type CSSStyleBlocksContents = Vec<CSSStyleBlock>;

// ----------- //
// Énumération //
// ----------- //

#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub enum CSSStyleBlock {
    Declaration(CSSDeclaration),
    Rule(CSSRule),
}

// ----------- //
// Entry Point //
// ----------- //

impl CSSParser {
    /// Analyse le contenu d'un bloc de style
    pub fn style_blocks_contents(&mut self) -> CSSStyleBlocksContents {
        self.consume_style_blocks_contents()
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl From<CSSDeclaration> for CSSStyleBlock {
    fn from(declaration: CSSDeclaration) -> Self {
        Self::Declaration(declaration)
    }
}

impl From<CSSRule> for CSSStyleBlock {
    fn from(rule: CSSRule) -> Self {
        Self::Rule(rule)
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
