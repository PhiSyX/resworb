/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use core::ops;
use std::borrow::Borrow;

use infra::structure::tree::TreeNode;

use super::{
    character_data::CharacterDataInner, CharacterData, DocumentNode, Node,
    NodeData, NodeType,
};

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
pub struct CommentNode {
    tree: TreeNode<Node>,
}

#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub struct Comment {
    data: String,
}

// -------------- //
// Implémentation //
// -------------- //

impl CommentNode {
    pub fn new(document: &DocumentNode, data: String) -> Self {
        let tree = TreeNode::new(
            Node::builder()
                .set_data(Self::character_data(Comment::new(data)))
                .set_type(NodeType::COMMENT_NODE)
                .build(),
        );
        tree.set_document(document);
        Self { tree }
    }

    fn character_data(comment: Comment) -> NodeData {
        NodeData::CharacterData(CharacterData::new(
            CharacterDataInner::Comment(comment),
        ))
    }
}

impl Comment {
    /// Les étapes du constructeur du nouveau commentaire consistent
    /// à définir les données de ce dernier comme étant des
    /// [CharacterData].
    fn new(data: String) -> Self {
        Self { data }
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl ops::Deref for CommentNode {
    type Target = TreeNode<Node>;

    fn deref(&self) -> &Self::Target {
        self.tree.borrow()
    }
}
