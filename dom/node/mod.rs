/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/// 4.5. Interface Document
mod document;

/// 4.6. Interface DocumentType
mod document_type;

/// 4.7. Interface DocumentFragment
mod document_fragment;

/// 4.8. Interface ShadowRoot
mod shadow_root;

/// 4.9. Interface Element
mod element;

/// 4.9.2. Interface Attr
mod attr;

/// 4.10. Interface CharacterData
mod character_data;

/// 4.11. Interface Text
mod text;

/// 4.14. Interface Comment
mod comment;

use std::cell::RefCell;

use html_elements::HTMLScriptElement;
use infra::structure::tree::{TreeNode, TreeNodeWeak};

pub use self::{
    attr::Attr,
    character_data::CharacterData,
    comment::{Comment, CommentNode},
    document::{CreateElementOptions, Document, DocumentNode, QuirksMode},
    document_fragment::{DocumentFragment, DocumentFragmentNode},
    document_type::DocumentType,
    element::Element,
    shadow_root::ShadowRoot,
    text::{Text, TextNode},
};

// --------- //
// Structure //
// --------- //

/// 4.4. Interface Node
#[derive(Debug)]
pub struct Node {
    owner_document: RefCell<Option<TreeNodeWeak<Self>>>,
    node_data: Option<NodeData>,
    node_type: NodeType,
}

pub(super) struct NodeBuilder {
    owner_document: RefCell<Option<TreeNodeWeak<Node>>>,
    node_data: Option<NodeData>,
    node_type: NodeType,
}

// ----------- //
// Énumération //
// ----------- //

#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub enum NodeData {
    Document(Document),
    DocumentType(DocumentType),
    DocumentFragment {
        fragment: Option<DocumentFragment>,
        shadow_root: Option<ShadowRoot>,
    },
    Element(Element),
    CharacterData(CharacterData),
    Attr(Attr),
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
#[derive(PartialEq, Eq)]
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
    pub(super) fn builder() -> NodeBuilder {
        NodeBuilder::new()
    }

    pub fn isin_html_namespace(&self) -> bool {
        let element = self.element_ref();
        element.isin_html_namespace()
    }

    pub fn is_html_text_integration_point(&self) -> bool {
        let element = self.element_ref();
        element.is_html_text_integration_point()
    }

    pub fn is_mathml_text_integration_point(&self) -> bool {
        let element = self.element_ref();
        element.is_mathml_text_integration_point()
    }

    /// Le noeud courant est un document de type [NodeType::COMMENT_NODE].
    pub fn is_comment(&self) -> bool {
        self.node_type == NodeType::COMMENT_NODE
    }

    /// Le noeud courant est un document de type
    /// [NodeType::DOCUMENT_TYPE_NODE].
    pub fn is_doctype(&self) -> bool {
        self.node_type == NodeType::DOCUMENT_TYPE_NODE
    }

    /// Le noeud courant est un document de type [NodeType::DOCUMENT_NODE].
    pub fn is_document(&self) -> bool {
        self.node_type == NodeType::DOCUMENT_NODE
    }

    /// Le noeud courant est un document de type [NodeType::TEXT_NODE].
    pub fn is_text(&self) -> bool {
        self.node_type == NodeType::TEXT_NODE
    }

    /// Retourne la donnée du noeud, qui est l'élément courant.
    // NOTE(phisyx): au lieux de panic comme un demeuré: mieux gérer les
    // erreurs.
    pub fn element_ref(&self) -> &Element {
        match self.node_data.as_ref() {
            | Some(NodeData::Element(element)) => element,
            | _ => panic!("Élément attendu."),
        }
    }

    /// Retourne la donnée du noeud, qui est le document courant.
    // NOTE(phisyx): au lieux de panic comme un demeuré: mieux gérer les
    // erreurs.
    pub fn document_ref(&self) -> &Document {
        match self.node_data.as_ref() {
            | Some(NodeData::Document(ref d)) => d,
            | _ => panic!("Document attendu."),
        }
    }

    /// Retourne la donnée du noeud, qui est l'élément script.
    // NOTE(phisyx): au lieux de panic comme un demeuré: mieux gérer les
    // erreurs.
    pub fn script_ref(&self) -> &HTMLScriptElement<DocumentNode> {
        match self.node_data.as_ref() {
            | Some(NodeData::Element(element)) => element.script(),
            | _ => panic!("Élément script attendu."),
        }
    }

    /// Défini une suite de caractères au noeud courant dans lequel nous
    /// pouvons définir des [données de caractères](CharacterData).
    // NOTE(phisyx): au lieux de panic comme un demeuré: mieux gérer les
    // erreurs.
    pub fn set_data(&self, data: &str) {
        match self.node_data.as_ref() {
            | Some(NodeData::CharacterData(cd)) => cd.set_data(data),
            | _ => panic!("N'est pas un noeud où l'on peut définir une suite de caractères"),
        };
    }

    pub fn set_document(&self, document: &TreeNode<Node>) {
        let document_weak: TreeNodeWeak<Node> =
            TreeNodeWeak::from(document);
        self.owner_document.borrow_mut().replace(document_weak);
    }
}

impl NodeBuilder {
    fn new() -> Self {
        Self {
            node_data: Default::default(),
            node_type: NodeType::INVALID,
            owner_document: Default::default(),
        }
    }

    fn set_data(mut self, node_data: NodeData) -> Self {
        self.node_data = node_data.into();
        self
    }

    fn set_type(mut self, node_type: NodeType) -> Self {
        self.node_type = node_type;
        self
    }

    fn build(self) -> Node {
        assert_ne!(self.node_data, None);
        assert_ne!(self.node_type, NodeType::INVALID);

        Node {
            owner_document: self.owner_document,
            node_data: self.node_data,
            node_type: self.node_type,
        }
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.node_type == other.node_type
            && self.node_data == other.node_data
    }
}

impl Eq for Node {}
