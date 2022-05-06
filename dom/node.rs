/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::document::Document;

#[allow(non_camel_case_types)]
#[repr(u16)]
pub enum NodeType {
    INVALID = 0,

    COMMENT_NODE = 8,

    DOCUMENT_TYPE_NODE = 10,
}

pub struct Node {
    document: Document,
    node_type: NodeType,
}

// -------------- //
// Implémentation //
// -------------- //

impl Node {
    pub fn new(document: &Document, node_type: NodeType) -> Self {
        Self {
            document: document.clone(),
            node_type,
        }
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl Default for NodeType {
    fn default() -> Self {
        Self::INVALID
    }
}
