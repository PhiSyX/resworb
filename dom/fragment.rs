/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use infra::structure::tree::TreeNode;

use crate::node::{Node, NodeData, NodeType};

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
#[derive(Default)]
#[derive(Clone)]
pub struct DocumentFragment {}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl From<DocumentFragment> for TreeNode<Node> {
    fn from(fragment: DocumentFragment) -> Self {
        Self::new(Node::new(
            NodeData::DocumentFragment(fragment),
            NodeType::DOCUMENT_FRAGMENT_NODE,
        ))
    }
}
