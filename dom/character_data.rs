/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{
    document::Document,
    node::{Node, NodeType},
};

pub struct CharacterData {
    data: String,
    node: Node,
}

impl CharacterData {
    pub fn new(
        document: &Document,
        node_type: NodeType,
        data: String,
    ) -> Self {
        Self {
            data,
            node: Node::new(document, node_type),
        }
    }

    pub fn node(&self) -> &Node {
        &self.node
    }
}
