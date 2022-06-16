/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{
    component_value::{CSSComponentValue, CSSComponentValuesList},
    simple_block::CSSSimpleBlock,
};

// --------- //
// Structure //
// --------- //

/// Une at-rule possède un nom, un prélude constitué d'une liste de
/// valeurs de composants, et un bloc optionnel constitué d'un simple bloc
/// {}.
///
/// NOTE(css): la plupart des règles qualifiées seront des règles de style,
/// où le prélude est un sélecteur <https://www.w3.org/TR/selectors-3>
/// et le bloc une liste de déclarations.
#[derive(Debug)]
#[derive(Default)]
#[derive(PartialEq, Eq)]
pub struct CSSAtRule {
    name: String,
    prelude: CSSComponentValuesList,
    block: Option<CSSSimpleBlock>,
}

// -------------- //
// Implémentation //
// -------------- //

impl CSSAtRule {
    pub(crate) fn with_name(mut self, token_name: impl ToString) -> Self {
        self.name = token_name.to_string();
        self
    }

    pub(crate) fn with_prelude(
        mut self,
        prelude: impl IntoIterator<Item = impl TryInto<CSSComponentValue>>,
    ) -> Self {
        self.prelude = prelude
            .into_iter()
            .filter_map(|token| token.try_into().ok())
            .collect();
        self
    }
}

impl CSSAtRule {
    pub(crate) fn append(&mut self, component_value: CSSComponentValue) {
        self.prelude.push(component_value);
    }

    pub(crate) fn set_block(&mut self, block: CSSSimpleBlock) {
        self.block = Some(block);
    }
}
