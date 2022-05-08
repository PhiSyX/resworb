/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::element::HTMLElementInterface;

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
#[derive(Default)]
pub struct HTMLHtmlElement {}

// -------------- //
// Implémentation //
// -------------- //

impl HTMLHtmlElement {
    pub const NAME: &'static str = "html";
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl HTMLElementInterface for HTMLHtmlElement {
    fn tag_name(&self) -> &'static str {
        Self::NAME
    }
}
