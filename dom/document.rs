/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::node::Node;

// --------- //
// Structure //
// --------- //

/// Chaque document XML et HTML dans une UA HTML est représenté par un
/// objet Document. [DOM](https://dom.spec.whatwg.org/)
#[derive(Clone)]
pub struct HTMLDocument {
    quirks_mode: QuirksMode,
}

// ----------- //
// Énumération //
// ----------- //

#[derive(Clone)]
pub enum QuirksMode {
    No,
    Yes,
    Limited,
}

// -------------- //
// Implémentation //
// -------------- //

impl HTMLDocument {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            quirks_mode: QuirksMode::Yes,
        }
    }
}

impl HTMLDocument {
    // todo: fixme
    pub fn append_child(&mut self, node: &Node) {
        todo!()
    }

    pub fn set_quirks_mode(&mut self, mode: QuirksMode) -> &mut Self {
        self.quirks_mode = mode;
        self
    }
}
