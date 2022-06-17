/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use infra::primitive::string::DOMString;

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub struct Attr {
    name: DOMString,
    value: DOMString,
}
