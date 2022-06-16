/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use infra::primitive::string::DOMString;

use super::{Comment, Text};

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
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
        let dom_string = DOMString::new(data);
        self.data.write(dom_string);
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl PartialEq for CharacterData {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
            && *self.data.read() == *other.data.read()
    }
}

impl Eq for CharacterData {}
