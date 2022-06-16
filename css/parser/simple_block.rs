/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{
    component_value::{CSSComponentValue, CSSComponentValuesList},
    tokenization::CSSToken,
};

// --------- //
// Structure //
// --------- //

/// Un bloc simple a un jeton associé (soit un
/// <[\[-token](CSSToken::LeftSquareBracket)>,
/// <[(-token](CSSToken::LeftParenthesis)>, ou
/// <[{-token](CSSToken::LeftCurlyBracket)>) et une valeur constituée d'une
/// liste de [valeurs de composants](CSSComponentValue).
///
/// {}-block, []-block, et ()-block font spécifiquement
/// référence à un bloc simple avec le jeton associé correspondant.
#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq, Eq)]
pub struct CSSSimpleBlock {
    pub token: CSSToken,
    value: CSSComponentValuesList,
}

// -------------- //
// Implémentation //
// -------------- //

impl CSSSimpleBlock {
    pub(super) fn new(token: CSSToken) -> Self {
        Self {
            token,
            value: vec![],
        }
    }

    pub(super) fn set_values(
        mut self,
        value: impl IntoIterator<Item = impl TryInto<CSSComponentValue>>,
    ) -> Self {
        self.value = value
            .into_iter()
            .filter_map(|cv| cv.try_into().ok())
            .collect();
        self
    }
}

impl CSSSimpleBlock {
    pub(super) fn append(
        &mut self,
        value: impl TryInto<CSSComponentValue>,
    ) {
        if let Ok(cv) = value.try_into() {
            self.value.push(cv);
        } else {
            eprintln!("Tentative d'ajout d'une valeur invalide");
        }
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl From<CSSToken> for CSSSimpleBlock {
    fn from(token: CSSToken) -> Self {
        match token {
            | token @ (CSSToken::LeftSquareBracket
            | CSSToken::LeftParenthesis
            | CSSToken::LeftCurlyBracket) => CSSSimpleBlock::new(token),
            | _ => panic!("Impossible de convertir le jeton {token:?} en CSSSimpleBlock."),
        }
    }
}
