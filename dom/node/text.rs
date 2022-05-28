/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{borrow::Borrow, ops};

use infra::structure::tree::TreeNode;

use super::{
    character_data::CharacterDataInner, CharacterData, DocumentNode, Node,
    NodeData, NodeType,
};

// --------- //
// Structure //
// --------- //

pub struct TextNode {
    tree: TreeNode<Node>,
}

#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub struct Text {
    data: String,
}

// -------------- //
// Implémentation //
// -------------- //

impl TextNode {
    pub fn new(document: &DocumentNode, data: String) -> Self {
        let text = Text::new(data);
        let tree = TreeNode::new(
            Node::builder()
                .set_data(NodeData::CharacterData(CharacterData::new(
                    CharacterDataInner::Text(text),
                )))
                .set_type(NodeType::TEXT_NODE)
                .build(),
        );
        tree.set_document(document);
        Self { tree }
    }
}

impl Text {
    pub fn new(data: String) -> Self {
        Self { data }
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl ops::Deref for TextNode {
    type Target = TreeNode<Node>;

    fn deref(&self) -> &Self::Target {
        self.tree.borrow()
    }
}
