/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::{Node, NodeType};
use crate::document::HTMLDocument;

// --------- //
// Structure //
// --------- //

/// Les doctypes ont un nom associé, un ID public et un ID système.
pub struct DocumentType {
    node: Node,

    name: String,
    public_id: String,
    system_id: String,
}

// -------------- //
// Implémentation //
// -------------- //

impl DocumentType {
    /// Lorsqu'un doctype est créé, son nom est toujours donné. À moins
    /// qu'ils ne soient explicitement donnés lors de la création d'un
    /// doctype, son ID public et son ID système sont une chaîne de
    /// caractères vide.
    pub fn new(document: &HTMLDocument, name: Option<&String>) -> Self {
        Self {
            node: Node::new(document, NodeType::DOCUMENT_TYPE_NODE),
            name: if let Some(x) = name {
                x.to_owned()
            } else {
                String::new()
            },
            public_id: String::new(),
            system_id: String::new(),
        }
    }
}

impl DocumentType {
    pub fn node(&self) -> &Node {
        &self.node
    }

    pub fn set_name(&mut self, maybe_name: Option<&String>) -> &mut Self {
        self.name = if let Some(x) = maybe_name {
            x.to_owned()
        } else {
            String::new()
        };
        self
    }

    pub fn set_public_id(
        &mut self,
        maybe_pid: Option<&String>,
    ) -> &mut Self {
        self.public_id = if let Some(x) = maybe_pid {
            x.to_owned()
        } else {
            String::new()
        };
        self
    }

    pub fn set_system_id(
        &mut self,
        maybe_sid: Option<&String>,
    ) -> &mut Self {
        self.system_id = if let Some(x) = maybe_sid {
            x.to_owned()
        } else {
            String::new()
        };
        self
    }
}
