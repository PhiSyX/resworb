/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod document;
mod metadata;
mod scripting;
mod text_level;

pub mod interface;
mod tags;

pub use self::{
    document::HTMLHtmlElement,
    metadata::{HTMLHeadElement, HTMLTitleElement},
    scripting::{HTMLScriptElement, HTMLTemplateElement},
    tags::{attributes::tag_attributes, names::tag_names},
    text_level::HTMLAnchorElement,
};
