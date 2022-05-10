/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{cell::RefCell, collections::HashMap};

use infra::namespace::Namespace;

use crate::{element::HTMLElement, fragment::DocumentFragment};

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
pub struct Element {
    inner_data: HTMLElement,
    pub attributes: RefCell<HashMap<String, String>>,
    pub id: RefCell<Option<String>>,
}

// -------------- //
// Implémentation //
// -------------- //

impl Element {
    pub fn new(data: HTMLElement) -> Self {
        Self {
            inner_data: data,
            attributes: Default::default(),
            id: Default::default(),
        }
    }
}

impl Element {
    pub fn local_name(&self) -> String {
        self.inner_data.to_string()
    }

    pub fn content(&self) -> Option<DocumentFragment> {
        assert!(matches!(
            self.inner_data,
            HTMLElement::ScriptingTemplate(_)
        ));

        if let HTMLElement::ScriptingTemplate(el) = &self.inner_data {
            return Some(el.content.borrow().clone());
        }

        None
    }

    pub fn namespace(&self) -> String {
        self.inner_data.to_string()
    }

    pub fn is_in_html_namespace(&self) -> bool {
        self.namespace()
            .parse::<Namespace>()
            .ok()
            .filter(|ns| Namespace::HTML.eq(ns))
            .is_some()
    }

    // todo: fixme
    pub fn is_an_html_integration_point(&self) -> bool {
        false
    }

    pub fn set_attribute(&self, name: &str, value: &str) {
        if name == "id" {
            *self.id.borrow_mut() = value.to_owned().into();
            return;
        }

        self.attributes
            .borrow_mut()
            .insert(name.to_owned(), value.to_owned());
    }
}
