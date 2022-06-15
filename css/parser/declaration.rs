/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::Deref;

use crate::{component_value::CSSComponentValue, tokenization::CSSToken};

// ---- //
// Type //
// ---- //

pub type CSSDeclarationList = Vec<CSSDeclaration>;

// --------- //
// Structure //
// --------- //

/// D'un point de vue conceptuel, les déclarations sont un exemple
/// particulier d'association d'un nom de propriété ou de descripteur à une
/// valeur. Sur le plan syntaxique, une déclaration comporte un nom, une
/// valeur constituée d'une liste de valeurs de composants et un drapeau
/// important qui est initialement désactivé.
#[derive(Debug)]
#[derive(Default)]
#[derive(PartialEq, Eq)]
pub struct CSSDeclaration {
    name: String,
    value: Vec<CSSComponentValue>,
    important_flag: bool,
}

// ----------- //
// Énumération //
// ----------- //

/// Les déclarations sont ensuite classées en déclarations de propriétés ou
/// en déclarations de descripteurs, les premières définissant les
/// propriétés CSS et apparaissant le plus souvent dans les règles
/// qualifiées et les secondes définissant les descripteurs CSS, qui
/// n'apparaissent que dans les at-rules. (Cette catégorisation
/// n'intervient pas au niveau de la syntaxe ; elle est plutôt le produit
/// de l'endroit où la déclaration apparaît, et est définie par les
/// spécifications respectives définissant la règle donnée).
///
/// TODO.
pub enum CSSDeclarationCategory {
    Property,
    Descriptor,
}

// -------------- //
// Implémentation //
// -------------- //

impl CSSDeclaration {
    pub(crate) fn with_name(mut self, token: &CSSToken) -> Self {
        self.name = token.name();
        self
    }

    pub(crate) fn with_values(
        mut self,
        prelude: impl IntoIterator<Item = CSSComponentValue>,
    ) -> Self {
        self.value = prelude.into_iter().collect();
        self
    }
}

impl CSSDeclaration {
    pub(crate) fn last_n_values(&self, n: usize) -> &[CSSComponentValue] {
        let size = self.value.len();
        let start = size.saturating_sub(n);
        let values = &self.value[start..];
        assert!(values.len() == n);
        values
    }

    pub(crate) fn last_n_tokens(
        &self,
        n: usize,
    ) -> impl Iterator<Item = &CSSToken> {
        self.last_n_values(n).iter().filter_map(|component_value| {
            match component_value {
                | CSSComponentValue::Preserved(preserve_token) => {
                    Some(preserve_token.deref())
                }
                | _ => None,
            }
        })
    }

    pub(crate) fn last_token(&self) -> Option<&CSSToken> {
        self.last_n_tokens(1).next()
    }
}

impl CSSDeclaration {
    pub(crate) fn append(&mut self, component_value: CSSComponentValue) {
        self.value.push(component_value);
    }

    pub(crate) fn remove_last_n_values(&mut self, n: usize) {
        let size = self.value.len();
        let start = size.saturating_sub(n);
        self.value.drain(start..);
    }

    pub(crate) fn set_important_flag(&mut self, important_flag: bool) {
        self.important_flag = important_flag;
    }
}
