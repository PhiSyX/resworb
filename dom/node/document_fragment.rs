/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use core::ops;
use std::borrow::Borrow;

use infra::structure::tree::TreeNode;

use super::ShadowRoot;
use crate::node::{Node, NodeData, NodeType};

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub struct DocumentFragmentNode {
    tree: TreeNode<Node>,
}

#[derive(Debug)]
#[derive(Default)]
#[derive(Clone)]
#[derive(PartialEq, Eq)]
pub struct DocumentFragment {}

// -------------- //
// Implémentation //
// -------------- //

impl DocumentFragmentNode {
    pub(crate) fn document_fragment(
        fragment: DocumentFragment,
    ) -> NodeData {
        NodeData::DocumentFragment {
            fragment: fragment.into(),
            shadow_root: None,
        }
    }

    pub(crate) fn shadow_root(
        mut node_data: NodeData,
        sr: ShadowRoot,
    ) -> NodeData {
        assert!(matches!(
            node_data,
            NodeData::DocumentFragment {
                shadow_root: None,
                ..
            }
        ));

        if let NodeData::DocumentFragment {
            ref mut shadow_root,
            ..
        } = node_data
        {
            *shadow_root = sr.into();
        }

        node_data
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl Default for DocumentFragmentNode {
    fn default() -> Self {
        let tree = TreeNode::new(
            Node::builder()
                .set_data(NodeData::DocumentFragment {
                    fragment: None,
                    shadow_root: None,
                })
                .set_type(NodeType::DOCUMENT_FRAGMENT_NODE)
                .build(),
        );
        Self { tree }
    }
}

impl ops::Deref for DocumentFragmentNode {
    type Target = TreeNode<Node>;

    fn deref(&self) -> &Self::Target {
        self.tree.borrow()
    }
}
