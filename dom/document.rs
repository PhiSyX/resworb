/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::node::Node;

// --------- //
// Structure //
// --------- //

#[derive(Clone)]
pub struct Document {
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

impl Document {
    pub fn new() -> Self {
        Self {
            quirks_mode: QuirksMode::Yes,
        }
    }
}

impl Document {
    pub fn append_child(&mut self, node: &Node) {}

    pub fn set_quirks_mode(&mut self, mode: QuirksMode) -> &mut Self {
        self.quirks_mode = mode;
        self
    }
}
