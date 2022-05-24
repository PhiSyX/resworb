/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{interface::HTMLElementInterface, tag_names};

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
#[derive(PartialEq)]
pub struct HTMLHeadingElement {
    level: u8,
}

// -------------- //
// Implémentation //
// -------------- //

impl HTMLHeadingElement {
    pub const fn new(tag_name: tag_names) -> Self {
        let level = match tag_name {
            | tag_names::h1 => 1,
            | tag_names::h2 => 2,
            | tag_names::h3 => 3,
            | tag_names::h4 => 4,
            | tag_names::h5 => 5,
            | tag_names::h6 => 6,
            | _ => panic!(
                "Nom de balise invalide, attendue : h1, h2, h3, h4, h5, h6"
            ),
        };
        Self { level }
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl HTMLElementInterface for HTMLHeadingElement {
    fn tag_name(&self) -> &'static str {
        match self.level {
            | 1 => "h1",
            | 2 => "h2",
            | 3 => "h3",
            | 4 => "h4",
            | 5 => "h5",
            | 6 => "h6",
            | _ => "h1",
        }
    }
}
