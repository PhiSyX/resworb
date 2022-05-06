/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod character_data;
mod comment;

/// 4.6. Interface DocumentType
mod doctype;

/// 4.9. Interface Element
mod element;

pub use character_data::CharacterData;
pub use comment::Comment;
pub use doctype::DocumentType;
pub use element::Element;

use crate::document::HTMLDocument;

// --------- //
// Structure //
// --------- //

/// 4.4. Interface Node
pub struct Node {
    document: HTMLDocument,
    node_type: NodeType,
}

// ----------- //
// Énumération //
// ----------- //

#[allow(non_camel_case_types)]
#[repr(u16)]
pub enum NodeType {
    INVALID = 0,
    ELEMENT_NODE = 1,
    ATTRIBUTE_NODE = 2,
    TEXT_NODE = 3,
    CDATA_SECTION_NODE = 4,
    PROCESSING_INSTRUCTION_NODE = 7,
    COMMENT_NODE = 8,
    DOCUMENT_NODE = 9,
    DOCUMENT_TYPE_NODE = 10,
    DOCUMENT_FRAGMENT_NODE = 11,
}

// -------------- //
// Implémentation //
// -------------- //

impl Node {
    pub fn new(document: &HTMLDocument, node_type: NodeType) -> Self {
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
