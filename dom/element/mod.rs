/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod document;
mod metadata;
mod scripting;

use core::fmt;
use std::str::FromStr;

pub use document::HTMLHtmlElement;
pub use metadata::{HTMLHeadElement, HTMLTitleElement};
pub use scripting::HTMLTemplateElement;

// --------- //
// Interface //
// --------- //

pub trait HTMLElementInterface {
    fn tag_name(&self) -> &'static str;
}

// ----------- //
// Énumération //
// ----------- //

#[derive(Debug)]
/// 4 The elements of HTML
pub enum HTMLElement {
    // 4.1 The document element
    DocumentHtml(
        /// 4.1.1 The html element
        document::HTMLHtmlElement,
    ),

    // 4.2 Document metadata
    MetadataHead(
        /// 4.2.1 The head element
        metadata::HTMLHeadElement,
    ),
    MetadataTitle(
        /// 4.2.2 The title element
        metadata::HTMLTitleElement,
    ),

    // 4.12 Scripting
    ScriptingTemplate(
        /// 4.12.3 The template element
        scripting::HTMLTemplateElement,
    ),
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl FromStr for HTMLElement {
    type Err = &'static str;

    fn from_str(local_name: &str) -> Result<Self, Self::Err> {
        let local_name = local_name.to_ascii_lowercase();
        Ok(match local_name.as_ref() {
            | "html" => {
                Self::DocumentHtml(document::HTMLHtmlElement::default())
            }
            | "head" => {
                Self::MetadataHead(metadata::HTMLHeadElement::default())
            }
            | "title" => {
                Self::MetadataTitle(metadata::HTMLTitleElement::default())
            }
            | "template" => Self::ScriptingTemplate(
                scripting::HTMLTemplateElement::default(),
            ),
            | _ => return Err("Element inconnu"),
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
                | Self::ScriptingTemplate(el) => el.tag_name(),
            }
        )
    }
}
