/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::interface::HTMLElementInterface;

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
#[derive(Default)]
#[derive(PartialEq, Eq)]
pub struct HTMLOListElement {}

// -------------- //
// Implémentation //
// -------------- //

impl HTMLOListElement {
    pub const NAME: &'static str = "ol";
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl HTMLElementInterface for HTMLOListElement {
    fn tag_name(&self) -> &'static str {
        Self::NAME
    }
}
