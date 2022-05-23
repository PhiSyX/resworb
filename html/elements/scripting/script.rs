/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::{
    atomic::{AtomicBool, Ordering},
    RwLock,
};

use crate::interface::HTMLElementInterface;

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
#[derive(Default)]
pub struct HTMLScriptElement<Document>
where
    Document: Clone,
{
    parser_document: RwLock<Document>,
    non_blocking: AtomicBool,
    already_started: AtomicBool,
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
        self.already_started.swap(to, Ordering::Relaxed);
        self
    }

    pub fn set_non_blocking(&self, to: bool) -> &Self {
        self.non_blocking.swap(to, Ordering::Relaxed);
        self
    }

    pub fn set_parser_document(&self, parser_document: &D) -> &Self {
        *self.parser_document.write().unwrap() = parser_document.clone();
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

impl<D> PartialEq for HTMLScriptElement<D>
where
    D: Clone + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        *self.parser_document.read().unwrap()
            == *other.parser_document.read().unwrap()
            && self.already_started.load(Ordering::Relaxed)
                == other.already_started.load(Ordering::Relaxed)
            && self.non_blocking.load(Ordering::Relaxed)
                == other.non_blocking.load(Ordering::Relaxed)
    }
}
