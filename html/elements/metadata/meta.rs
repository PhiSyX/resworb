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
pub struct HTMLMetaElement {}

// -------------- //
// Implémentation //
// -------------- //

impl HTMLMetaElement {
    pub const NAME: &'static str = "meta";
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl HTMLElementInterface for HTMLMetaElement {
    fn tag_name(&self) -> &'static str {
        Self::NAME
    }
}
