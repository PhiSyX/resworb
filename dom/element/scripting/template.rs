/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use crate::{element::HTMLElementInterface, fragment::DocumentFragment};

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
#[derive(Default)]
pub struct HTMLTemplateElement {
    pub(crate) content: RefCell<DocumentFragment>,
}

// -------------- //
// Implémentation //
// -------------- //

impl HTMLTemplateElement {
    pub const NAME: &'static str = "template";

    pub fn new(content: DocumentFragment) -> Self {
        Self {
            content: RefCell::new(content),
        }
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl HTMLElementInterface for HTMLTemplateElement {
    fn tag_name(&self) -> &'static str {
        Self::NAME
    }
}
