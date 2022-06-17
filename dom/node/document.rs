/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use core::ops;
use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
};

use html_elements::tag_names;
use infra::{namespace::Namespace, structure::tree::TreeNode};

use super::{comment::CommentNode, element::HTMLElement, Element};
use crate::{
    exception::DOMException,
    node::{DocumentType, Node, NodeData, NodeType},
};

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq, Eq)]
pub struct DocumentNode {
    tree: TreeNode<Node>,
}

/// Chaque document XML et HTML dans une UA HTML est représenté par un
/// objet Document. [DOM](https://dom.spec.whatwg.org/)
#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub struct Document {
    doctype: RefCell<Option<DocumentType>>,
    pub quirks_mode: RefCell<QuirksMode>,
}

// ----------- //
// Énumération //
// ----------- //

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq, Eq)]
pub enum QuirksMode {
    No,
    Yes,
    Limited,
}

pub struct CreateElementOptions {
    pub is: Option<String>,
    pub namespace: Option<Namespace>,
}

// -------------- //
// Implémentation //
// -------------- //

impl Document {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            doctype: Default::default(),
            quirks_mode: RefCell::new(QuirksMode::Yes),
        }
    }

    pub fn create_element(
        local_name: impl AsRef<str>,
        options: Option<CreateElementOptions>,
    ) -> Result<TreeNode<Node>, DOMException> {
        // 1) Si localName ne correspond pas à la production de Name, une
        // DOMException "InvalidCharacterError" est levée.
        if !tag_names::is_valid_name(&local_name) {
            return Err(DOMException::InvalidCharacterError);
        }

        // 2) S'il s'agit d'un document HTML, définir localName en
        // minuscules ASCII.
        let maybe_element = local_name.as_ref().parse::<HTMLElement>();

        // 3) Laisser `is` être null.
        // 4) Si options est un dictionnaire et que options["is"] existe,
        // alors `is` lui est attribué.
        let is =
            options.as_ref().and_then(|options| options.is.to_owned());

        // 5) Que namespace soit l'espace de noms HTML, si c'est un
        // document HTML ou si le type de contenu est
        // "application/xhtml+xml" ; sinon null
        let namespace = if let Some(options) = options.as_ref() {
            options.namespace
        } else {
            Some(Namespace::HTML)
        };

        // 6) Renvoie le résultat de la création d'un élément avec this,
        // localName, namespace, null, is, et avec l'indicateur d'éléments
        // personnalisés synchrones activé.

        maybe_element
            .map(|element| {
                TreeNode::new(
                    Node::builder()
                        .set_data(NodeData::Element(Element::new(
                            element,
                            is,
                            namespace.unwrap(),
                        )))
                        .set_type(NodeType::ELEMENT_NODE)
                        .build(),
                )
            })
            .map_err(|_| DOMException::InvalidNodeTypeError)
    }
}

impl Document {
    pub fn get_doctype(&self) -> Option<DocumentType> {
        self.doctype.borrow().clone()
    }

    pub fn isin_quirks_mode(&self) -> bool {
        matches!(*self.quirks_mode.borrow(), QuirksMode::Yes)
    }
}

// &mut Self
impl Document {
    pub fn set_doctype(&self, doctype: DocumentType) -> &Self {
        *self.doctype.borrow_mut() = doctype.into();
        self
    }

    pub fn set_quirks_mode(&self, mode: QuirksMode) -> &Self {
        *self.quirks_mode.borrow_mut() = mode;
        self
    }
}

impl DocumentNode {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self) -> &Document {
        self.document_ref()
    }

    pub fn get_mut(&self) -> &Document {
        self.document_ref().borrow_mut()
    }

    pub fn insert_comment(&self, text: String) {
        let comment_node = CommentNode::new(self, text).to_owned();
        self.append_child(comment_node);
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl Default for DocumentNode {
    fn default() -> Self {
        Self {
            tree: TreeNode::new(
                Node::builder()
                    .set_data(NodeData::Document(Document::new()))
                    .set_type(NodeType::DOCUMENT_NODE)
                    .build(),
            ),
        }
    }
}

impl ops::Deref for DocumentNode {
    type Target = TreeNode<Node>;

    fn deref(&self) -> &Self::Target {
        self.tree.borrow()
    }
}
