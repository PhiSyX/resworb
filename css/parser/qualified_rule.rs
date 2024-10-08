/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{
    component_value::{CSSComponentValue, CSSComponentValuesList},
    simple_block::CSSSimpleBlock,
    tokenization::CSSToken,
};

// --------- //
// Structure //
// --------- //

/// Une règle qualifiée possède un prélude constitué d'une liste de
/// valeurs de composants, et un bloc constitué d'un simple bloc {}.
#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub struct CSSQualifiedRule {
    prelude: CSSComponentValuesList,
    block: CSSSimpleBlock,
}

// -------------- //
// Implémentation //
// -------------- //

impl CSSQualifiedRule {
    pub(super) fn with_prelude(
        mut self,
        prelude: impl IntoIterator<Item = CSSToken>,
    ) -> Self {
        self.prelude = prelude
            .into_iter()
            .filter_map(|token| token.try_into().ok())
            .collect();
        self
    }

    pub(super) fn with_block(mut self, block: CSSSimpleBlock) -> Self {
        self.block = block;
        self
    }
}

impl CSSQualifiedRule {
    pub(super) fn append(&mut self, value: CSSComponentValue) {
        self.prelude.push(value);
    }

    pub(super) fn set_block(&mut self, block: CSSSimpleBlock) {
        self.block = block;
    }
}

impl Default for CSSQualifiedRule {
    fn default() -> Self {
        Self {
            prelude: Default::default(),
            block: CSSSimpleBlock::new(CSSToken::LeftCurlyBracket),
        }
    }
}
