/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{declaration::CSSDeclaration, grammars::CSSRule};

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
