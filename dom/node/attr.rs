/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use infra::primitive::string::DOMString;

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
pub struct Attr {
    name: DOMString,
    value: DOMString,
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl PartialEq for Attr {
    fn eq(&self, other: &Self) -> bool {
        *self.name.read() == *other.name.read()
            && *self.value.read() == *other.value.read()
    }
}

impl Eq for Attr {}
