/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod document;
mod metadata;
mod scripting;
mod text_level;

pub mod interface;
mod sections;
mod tags;

pub use self::{
    document::*,
    metadata::*,
    scripting::*,
    sections::*,
    tags::{tag_attributes, tag_names},
    text_level::*,
};
