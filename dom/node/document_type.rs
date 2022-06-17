/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

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
    pub fn new(maybe_name: impl Into<String>) -> Self {
        Self {
            name: RefCell::new(maybe_name.into()),
            public_id: Default::default(),
            system_id: Default::default(),
        }
    }
}

impl DocumentType {
    pub fn set_name(
        &mut self,
        maybe_name: Option<impl Into<String>>,
    ) -> &mut Self {
        self.name =
            RefCell::new(maybe_name.map(Into::into).unwrap_or_default());
        self
    }

    pub fn set_public_id(
        &mut self,
        maybe_pid: Option<impl Into<String>>,
    ) -> &mut Self {
        self.public_id =
            RefCell::new(maybe_pid.map(Into::into).unwrap_or_default());
        self
    }

    pub fn set_system_id(
        &mut self,
        maybe_sid: Option<impl Into<String>>,
    ) -> &mut Self {
        self.system_id =
            RefCell::new(maybe_sid.map(Into::into).unwrap_or_default());
        self
    }
}
