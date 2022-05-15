/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use core::fmt;
use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    str::FromStr,
};

use html_elements::{
    interface::HTMLElementInterface, tag_names, HTMLScriptElement,
};
use infra::namespace::Namespace;

use super::{document_fragment::DocumentFragmentNode, DocumentNode};

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Element {
    inner: HTMLElement,
    pub attributes: RefCell<HashMap<String, String>>,
    pub id: RefCell<Option<String>>,
}

// ----------- //
// Énumération //
// ----------- //

/// 4 The elements of HTML
#[derive(Debug)]
#[derive(PartialEq)]
pub enum HTMLElement {
    // 4.1 The document element
    DocumentHtml(
        /// 4.1.1 The html element
        html_elements::HTMLHtmlElement,
    ),

    // 4.2 Document metadata
    MetadataHead(
        /// 4.2.1 The head element
        html_elements::HTMLHeadElement,
    ),
    MetadataTitle(
        /// 4.2.2 The title element
        html_elements::HTMLTitleElement,
    ),

    // 4.12 Scripting
    ScriptingScript(html_elements::HTMLScriptElement<DocumentNode>),
    ScriptingTemplate(
        /// 4.12.3 The template element
        html_elements::HTMLTemplateElement<DocumentFragmentNode>,
    ),
}

// -------------- //
// Implémentation //
// -------------- //

impl Element {
    pub fn new(data: HTMLElement) -> Self {
        Self {
            inner: data,
            attributes: Default::default(),
            id: Default::default(),
        }
    }
}

impl Element {
    pub fn local_name(&self) -> String {
        self.inner.to_string()
    }

    pub fn content(&self) -> Option<Ref<'_, DocumentFragmentNode>> {
        assert!(matches!(self.inner, HTMLElement::ScriptingTemplate(_)));

        if let HTMLElement::ScriptingTemplate(el) = &self.inner {
            return Some(el.content.borrow());
        }

        None
    }

    pub fn namespace(&self) -> String {
        self.inner.to_string()
    }

    pub fn is_in_html_namespace(&self) -> bool {
        self.namespace()
            .parse::<Namespace>()
            .ok()
            .filter(|ns| Namespace::HTML.eq(ns))
            .is_some()
    }

    // todo: fixme
    pub fn is_mathml_text_integration_point(&self) -> bool {
        false
    }

    // todo: fixme
    pub fn is_html_text_integration_point(&self) -> bool {
        false
    }

    pub fn script(&self) -> &HTMLScriptElement<DocumentNode> {
        match &self.inner {
            | HTMLElement::ScriptingScript(script) => script,
            | _ => panic!("N'est pas un élément HTMLScriptElement."),
        }
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

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl FromStr for HTMLElement {
    type Err = &'static str;

    fn from_str(local_name: &str) -> Result<Self, Self::Err> {
        let local_name =
            local_name.to_ascii_lowercase().parse::<tag_names>()?;

        Ok(match local_name {
            | tag_names::html => Self::DocumentHtml(
                html_elements::HTMLHtmlElement::default(),
            ),
            | tag_names::head => Self::MetadataHead(
                html_elements::HTMLHeadElement::default(),
            ),
            | tag_names::title => Self::MetadataTitle(
                html_elements::HTMLTitleElement::default(),
            ),
            | tag_names::template => Self::ScriptingTemplate(
                html_elements::HTMLTemplateElement::default(),
            ),
            | _ => {
                return Err("Element non pris en charge pour le moment.")
            }
        })
    }
}

impl fmt::Display for HTMLElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                | Self::DocumentHtml(el) => el.tag_name(),
                | Self::MetadataHead(el) => el.tag_name(),
                | Self::MetadataTitle(el) => el.tag_name(),
                | Self::ScriptingScript(el) => el.tag_name(),
                | Self::ScriptingTemplate(el) => el.tag_name(),
            }
        )
    }
}
