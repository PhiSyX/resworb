/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{
    character_data::CharacterData,
    document::Document,
    node::{Node, NodeType},
};

pub struct Comment {
    character_data: CharacterData,
}

impl Comment {
    pub fn new(document: &Document, data: String) -> Self {
        let character_data =
            CharacterData::new(document, NodeType::COMMENT_NODE, data);
        Self { character_data }
    }

    pub fn node(&self) -> &Node {
        self.character_data.node()
    }
}
