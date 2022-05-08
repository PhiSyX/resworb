/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod character_data;
mod comment;

/// 4.6. Interface DocumentType
mod doctype;

/// 4.9. Interface Element
mod element;

use core::cell::RefCell;

pub use character_data::CharacterData;
pub use comment::Comment;
pub use doctype::DocumentType;
pub use element::Element;
use infra::structure::tree::{TreeNode, TreeNodeWeak};

use crate::document::Document;

// --------- //
// Structure //
// --------- //

/// 4.4. Interface Node
#[derive(Debug)]
pub struct Node {
    owner_document: RefCell<Option<TreeNodeWeak<Self>>>,
    data: Option<NodeData>,
    node_type: NodeType,
}

// ----------- //
// Énumération //
// ----------- //

#[derive(Debug)]
pub enum NodeData {
    Document(Document),
    Element(Element),
    Comment(Comment),
    Doctype(DocumentType),
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
#[repr(u8)]
pub enum NodeType {
    INVALID = 0,
    ELEMENT_NODE = 1,
    ATTRIBUTE_NODE = 2,
    TEXT_NODE = 3,
    CDATA_SECTION_NODE = 4,
    ENTITY_REFERENCE_NODE = 5,
    PROCESSING_INSTRUCTION_NODE = 7,
    COMMENT_NODE = 8,
    DOCUMENT_NODE = 9,
    DOCUMENT_TYPE_NODE = 10,
    DOCUMENT_FRAGMENT_NODE = 11,
    NOTATION_NODE = 12,
}

// -------------- //
// Implémentation //
// -------------- //

impl Node {
    pub fn new(data: NodeData, node_type: NodeType) -> Self {
        Self {
            data: data.into(),
            node_type,
            owner_document: Default::default(),
        }
    }

    pub fn is_in_html_namespace(&self) -> bool {
        let element = self.element_ref();
        element.is_in_html_namespace()
    }

    pub fn is_an_html_integration_point(&self) -> bool {
        let element = self.element_ref();
        element.is_an_html_integration_point()
    }

    pub fn element_ref(&self) -> &Element {
        match self.data.as_ref() {
            | Some(NodeData::Element(element)) => element,
            | _ => panic!("Élément attendu."),
        }
    }

    pub fn document_ref(&self) -> &Document {
        match self.data.as_ref() {
            | Some(NodeData::Document(ref d)) => d,
            | _ => panic!("Document attendu."),
        }
    }

    pub fn set_document(&self, document: &TreeNode<Node>) {
        let document_weak: TreeNodeWeak<Node> =
            TreeNodeWeak::from(document);
        self.owner_document.replace(Some(document_weak));
    }
}
