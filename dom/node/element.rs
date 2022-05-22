/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use core::{fmt, str};
use std::{
    collections::HashMap,
    sync::{RwLock, RwLockReadGuard},
};

use html_elements::{
    interface::{HTMLElementInterface, IsOneOfTagsInterface},
    tag_names, HTMLScriptElement,
};
use infra::{namespace::Namespace, primitive::string::DOMString};

use super::{document_fragment::DocumentFragmentNode, DocumentNode};

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
pub struct Element {
    inner: HTMLElement,
    // todo: changer en NamedNodeMap (cf. https://dom.spec.whatwg.org/#namednodemap)
    pub attributes: RwLock<HashMap<DOMString, DOMString>>,
    pub id: RwLock<Option<String>>,
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

    pub fn tag_name(&self) -> tag_names {
        self.local_name()
            .parse()
            .expect("Devrait être un nom de balise valide")
    }

    pub fn content(
        &self,
    ) -> Option<RwLockReadGuard<DocumentFragmentNode>> {
        assert!(matches!(self.inner, HTMLElement::ScriptingTemplate(_)));

        if let HTMLElement::ScriptingTemplate(el) = &self.inner {
            return el.content.read().ok();
        }

        None
    }

    pub fn namespace(&self) -> Option<Namespace> {
        self.inner.to_string().parse().ok()
    }

    pub fn is_in_html_namespace(&self) -> bool {
        self.namespace() == Some(Namespace::HTML)
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

    pub fn has_attribute(&self, name: &str) -> bool {
        let dom_string = DOMString::new(name);
        self.attributes.read().unwrap().contains_key(&dom_string)
    }

    pub fn set_attribute(&self, name: &str, value: &str) {
        if name == "id" {
            *self.id.write().unwrap() = value.to_owned().into();
            return;
        }

        self.attributes
            .write()
            .unwrap()
            .insert(DOMString::new(name), DOMString::new(value));
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl PartialEq for Element {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
            && *self.attributes.read().unwrap()
                == *other.attributes.read().unwrap()
            && *self.id.read().unwrap() == *other.id.read().unwrap()
    }
}

impl str::FromStr for HTMLElement {
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
