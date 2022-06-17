/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{html_element, interface::HTMLElementInterface};

html_element! {
    struct HTMLHeadingElement(?) {
        level: u8
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl HTMLElementInterface for HTMLHeadingElement {
    fn tag_name(&self) -> String {
        match self.level {
            | 1 => "h1",
            | 2 => "h2",
            | 3 => "h3",
            | 4 => "h4",
            | 5 => "h5",
            | 6 => "h6",
            | _ => "h1",
        }
        .to_owned()
    }
}
