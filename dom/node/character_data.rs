/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use infra::primitive::string::DOMString;

use super::{Comment, Text};

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub struct CharacterData {
    inner: CharacterDataInner,
    data: DOMString,
}

#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub enum CharacterDataInner {
    Text(Text),
    Comment(Comment),
}

// -------------- //
// Implémentation //
// -------------- //

impl CharacterData {
    pub(crate) fn new(inner: CharacterDataInner) -> Self {
        Self {
            inner,
            data: Default::default(),
        }
    }

    pub(crate) fn set_data(&self, data: &str) {
        *self.data.borrow_mut() = data.to_owned();
    }
}
