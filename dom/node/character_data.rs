/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use super::{Comment, Text};

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
#[derive(PartialEq)]
pub struct CharacterData {
    inner: CharacterDataInner,
    data: RefCell<String>,
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum CharacterDataInner {
    Text(Text),
    Comment(Comment),
}

// -------------- //
// Implémentation //
// -------------- //

impl CharacterData {
    pub fn new(inner: CharacterDataInner) -> Self {
        Self {
            inner,
            data: Default::default(),
        }
    }

    pub fn set_data(&self, data: &str) {
        self.data.replace(data.to_owned());
    }
}
