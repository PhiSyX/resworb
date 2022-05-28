/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use infra::primitive::string::DOMString;

// --------- //
// Structure //
// --------- //

/// Les doctypes ont un nom associé, un ID public et un ID système.
#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub struct DocumentType {
    pub name: DOMString,
    pub public_id: DOMString,
    pub system_id: DOMString,
}

// -------------- //
// Implémentation //
// -------------- //

impl DocumentType {
    /// Lorsqu'un doctype est créé, son nom est toujours donné. À moins
    /// qu'ils ne soient explicitement donnés lors de la création d'un
    /// doctype, son ID public et son ID système sont une chaîne de
    /// caractères vide.
    pub fn new(name: Option<&String>) -> Self {
        Self {
            name: if let Some(x) = name {
                DOMString::new(x)
            } else {
                Default::default()
            },
            public_id: Default::default(),
            system_id: Default::default(),
        }
    }
}

impl DocumentType {
    pub fn set_name(&mut self, maybe_name: Option<&String>) -> &mut Self {
        self.name = if let Some(x) = maybe_name {
            DOMString::new(x)
        } else {
            Default::default()
        };
        self
    }

    pub fn set_public_id(
        &mut self,
        maybe_pid: Option<&String>,
    ) -> &mut Self {
        self.public_id = if let Some(x) = maybe_pid {
            DOMString::new(x)
        } else {
            Default::default()
        };
        self
    }

    pub fn set_system_id(
        &mut self,
        maybe_sid: Option<&String>,
    ) -> &mut Self {
        self.system_id = if let Some(x) = maybe_sid {
            DOMString::new(x)
        } else {
            Default::default()
        };
        self
    }
}
