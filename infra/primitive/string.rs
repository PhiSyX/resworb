/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use core::fmt;
use std::{
    hash,
    sync::{RwLock, RwLockReadGuard},
};

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
#[derive(Default)]
pub struct DOMString {
    inner: RwLock<String>,
}

// -------------- //
// Implémentation //
// -------------- //

impl DOMString {
    pub fn new(s: impl ToString) -> Self {
        Self {
            inner: RwLock::new(s.to_string()),
        }
    }

    pub fn read(&self) -> RwLockReadGuard<String> {
        self.inner.read().unwrap()
    }

    pub fn write(&self, data: Self) {
        self.inner
            .write()
            .unwrap()
            .clone_from(&data.inner.read().unwrap());
    }

    pub fn eq_ignore_ascii_case(&self, other: &Self) -> bool {
        (self.inner.read().unwrap())
            .to_ascii_lowercase()
            .eq_ignore_ascii_case(
                &other.inner.read().unwrap().to_ascii_lowercase(),
            )
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl<S> From<S> for DOMString
where
    S: AsRef<str>,
{
    fn from(s: S) -> Self {
        Self::new(s.as_ref())
    }
}

impl PartialEq for DOMString {
    fn eq(&self, other: &Self) -> bool {
        *self.read() == *other.read()
    }
}

impl Eq for DOMString {}

impl hash::Hash for DOMString {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.read().hash(state);
    }
}

impl Clone for DOMString {
    fn clone(&self) -> Self {
        let s = self.read().clone();
        Self {
            inner: RwLock::new(s),
        }
    }
}

impl fmt::Display for DOMString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.read())
    }
}
