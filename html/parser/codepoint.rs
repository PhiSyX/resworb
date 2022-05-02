/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use infra::primitive::codepoint::CodePoint;

// --------- //
// Interface //
// --------- //

pub(super) trait HTMLCodePoint: Copy {
    fn is_html_whitespace(self) -> bool;
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl HTMLCodePoint for CodePoint {
    fn is_html_whitespace(self) -> bool {
        self.is_ascii_whitespace() && self != '\r'
    }
}
