/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{
    component_value::CSSComponentValue, simple_block::CSSSimpleBlock,
    tokenization::CSSToken,
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
    prelude: Vec<CSSComponentValue>,
    block: Option<CSSSimpleBlock>,
}

// -------------- //
// Implémentation //
// -------------- //

impl CSSAtRule {
    pub(crate) fn with_name(mut self, token: &CSSToken) -> Self {
        if let CSSToken::AtKeyword(name) = token {
            self.name = name.to_owned();
        }
        self
    }
}
