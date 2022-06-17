/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use crate::interface::HTMLElementInterface;

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
#[derive(Default)]
#[derive(PartialEq, Eq)]
pub struct HTMLScriptElement<Document>
where
    Document: Clone,
{
    parser_document: RefCell<Document>,
    non_blocking: RefCell<bool>,
    already_started: RefCell<bool>,
}

// -------------- //
// Implémentation //
// -------------- //

impl<D> HTMLScriptElement<D>
where
    D: Clone,
{
    pub const NAME: &'static str = "script";

    pub fn set_already_started(&self, to: bool) -> &Self {
        *self.already_started.borrow_mut() = to;
        self
    }

    pub fn set_non_blocking(&self, to: bool) -> &Self {
        *self.non_blocking.borrow_mut() = to;
        self
    }

    pub fn set_parser_document(&self, parser_document: &D) -> &Self {
        *self.parser_document.borrow_mut() = parser_document.clone();
        self
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl<D> HTMLElementInterface for HTMLScriptElement<D>
where
    D: Clone,
{
    fn tag_name(&self) -> &'static str {
        Self::NAME
    }
}
