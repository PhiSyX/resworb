/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use core::ops;
use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
};

use infra::{namespace::Namespace, structure::tree::TreeNode};

use crate::{
    element::HTMLElement,
    exception::DOMException,
    node::{Comment, DocumentType, Element, Node, NodeData, NodeType},
};

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
pub struct DocumentNode {
    tree: TreeNode<Node>,
}

/// Chaque document XML et HTML dans une UA HTML est représenté par un
/// objet Document. [DOM](https://dom.spec.whatwg.org/)
#[derive(Debug)]
pub struct Document {
    doctype: RefCell<Option<DocumentType>>,
    quirks_mode: RefCell<QuirksMode>,
}

// ----------- //
// Énumération //
// ----------- //

#[derive(Debug)]
#[derive(Clone)]
pub enum QuirksMode {
    No,
    Yes,
    Limited,
}

#[non_exhaustive]
pub struct CreateElementOptions {
    is: String,
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

    /// todo: FIXME
    pub fn create_element(
        local_name: impl AsRef<str>,
        options: Option<CreateElementOptions>,
    ) -> Result<TreeNode<Node>, DOMException> {
        // 1) Si localName ne correspond pas à la production de Name, une
        // DOMException "InvalidCharacterError" est levée.
        if !Self::is_valid_name(&local_name) {
            return Err(DOMException::InvalidCharacterError);
        }

        // 2) S'il s'agit d'un document HTML, définir localName en
        // minuscules ASCII.
        let mut production_name = local_name.as_ref().to_owned();
        let maybe_element = local_name.as_ref().parse::<HTMLElement>();
        if maybe_element.is_ok() {
            production_name = production_name.to_ascii_lowercase();
        }

        // 3) Laisser `is` être null.
        // 4) Si options est un dictionnaire et que options["is"] existe,
        // alors `is` lui est attribué.
        let is = if let Some(options) = options {
            Some(options.is)
        } else {
            None
        };

        // 5) Que namespace soit l'espace de noms HTML, si c'est un
        // document HTML ou si le type de contenu est
        // "application/xhtml+xml" ; sinon null
        let namespace = Some(Namespace::HTML);

        // 6) Renvoie le résultat de la création d'un élément avec this,
        // localName, namespace, null, is, et avec l'indicateur d'éléments
        // personnalisés synchrones activé.

        maybe_element
            .map(|element| {
                TreeNode::new(Node::new(
                    NodeData::Element(Element::new(element)),
                    NodeType::ELEMENT_NODE,
                ))
            })
            .map_err(|_| DOMException::InvalidNodeTypeError)
    }

    fn is_valid_name(name: impl AsRef<str>) -> bool {
        let name = name.as_ref();

        if name.is_empty() {
            return false;
        }

        /*
        NameStartChar ::= ":" | [A-Z]     | "_" | [a-z]     | [#xC0-#xD6]
                        | [#xD8-#xF6]     | [#xF8-#x2FF]    | [#x370-#x37D]
                        | [#x37F-#x1FFF]  | [#x200C-#x200D] | [#x2070-#x218F]
                        | [#x2C00-#x2FEF] | [#x3001-#xD7FF] | [#xF900-#xFDCF]
                        | [#xFDF0-#xFFFD] | [#x10000-#xEFFFF]
        */
        fn name_start_char(ch: char) -> bool {
            ch.is_ascii_alphabetic()
                || matches!(ch, | ':' | '_'
                 | '\u{00C0}'..='\u{00D6}' | '\u{00D8}'..='\u{00F6}'
                 | '\u{00F8}'..='\u{02FF}' | '\u{0370}'..='\u{037D}'
                 | '\u{037F}'..='\u{1FFF}' | '\u{200C}'..='\u{200D}'
                 | '\u{2070}'..='\u{218F}' | '\u{2C00}'..='\u{2FEF}'
                 | '\u{3001}'..='\u{D7FF}' | '\u{F901}'..='\u{FDCF}'
                 | '\u{FDF0}'..='\u{FFFD}' | '\u{10000}'..='\u{EFFFF}'
                )
        }

        /*
        NameChar :: = NameStartChar   | "-" | "." | [0-9] | #xB7
                    | [#x0300-#x036F] | [#x203F-#x2040]
        */
        fn name_char(ch: char) -> bool {
            name_start_char(ch)
                || ch.is_ascii_alphanumeric()
                || matches!(ch, '-' | '.'
                 | '\u{00B7}'
                 | '\u{0300}'..='\u{036F}'
                 | '\u{203F}'..='\u{2040}'
                )
        }

        let mut chars = name.chars();

        let next_ch = chars.next().unwrap();
        if !name_start_char(next_ch) {
            return false;
        }

        chars.any(name_char)
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
        self.append_child(Comment::new(text).into_tree(self));
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl Default for DocumentNode {
    fn default() -> Self {
        Self {
            tree: TreeNode::new(Node::new(
                NodeData::Document(Document::new()),
                NodeType::DOCUMENT_NODE,
            )),
        }
    }
}

impl ops::Deref for DocumentNode {
    type Target = TreeNode<Node>;

    fn deref(&self) -> &Self::Target {
        self.tree.borrow()
    }
}
