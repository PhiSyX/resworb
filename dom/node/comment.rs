/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use infra::structure::tree::TreeNode;

use super::{CharacterData, Node, NodeData, NodeType};

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
pub struct Comment {
    character_data: CharacterData,
}

// -------------- //
// Implémentation //
// -------------- //

impl Comment {
    /// Les étapes du constructeur du nouveau commentaire consistent
    /// à définir les données de ce dernier comme étant des [CharacterData]
    /// et le nœud du document de ce dernier comme étant le [Document]
    /// associé à l'objet global actuel.
    pub fn new(data: Option<String>) -> Self {
        let character_data = CharacterData::new(data.unwrap_or_default());
        Self { character_data }
    }

    pub fn into_tree(self, document: &TreeNode<Node>) -> TreeNode<Node> {
        let tree = TreeNode::from(self);
        tree.set_document(document);
        tree
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl From<Comment> for TreeNode<Node> {
    fn from(comment: Comment) -> Self {
        Self::new(Node::new(
            NodeData::Comment(comment),
            NodeType::COMMENT_NODE,
        ))
    }
}
