/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{component_value::CSSComponentValue, tokenization::CSSToken};

// --------- //
// Structure //
// --------- //

/// Une fonction possède un nom et une valeur constituée d'une liste
/// de valeurs de composants.
#[derive(Debug)]
#[derive(Default)]
#[derive(PartialEq, Eq)]
pub struct CSSFunction {
    name: String,
    value: Vec<CSSComponentValue>,
}

// -------------- //
// Implémentation //
// -------------- //

impl CSSFunction {
    pub(crate) fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }
}

impl CSSFunction {
    pub(crate) fn append(&mut self, value: CSSComponentValue) {
        self.value.push(value);
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl From<CSSToken> for CSSFunction {
    fn from(token: CSSToken) -> Self {
        match token {
            | CSSToken::Function(fn_name) => Self::new(fn_name),
            | _ => panic!("Jeton `CSSToken::Function` attendu."),
        }
    }
}
