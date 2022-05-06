/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use infra::namespace::Namespaces;

// --------- //
// Structure //
// --------- //

#[derive(Default)]
pub struct Element {
    qualified_name: String,
}

// -------------- //
// Implémentation //
// -------------- //

impl Element {
    pub fn namespace(&self) -> &str {
        self.qualified_name.as_ref()
    }

    pub fn is_in_html_namespace(&self) -> bool {
        self.namespace()
            .parse::<Namespaces>()
            .ok()
            .filter(|ns| Namespaces::HTML.eq(ns))
            .is_some()
    }

    // todo: fixme
    pub fn is_an_html_integration_point(&self) -> bool {
        false
    }
}
