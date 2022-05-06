/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{
    document::Document,
    node::{Node, NodeType},
};

// --------- //
// Structure //
// --------- //

pub struct DocumentType {
    node: Node,

    name: String,
    public_identifier: String,
    system_identifier: String,
}

// -------------- //
// Implémentation //
// -------------- //

impl DocumentType {
    pub fn new(document: &Document) -> Self {
        Self {
            node: Node::new(document, NodeType::DOCUMENT_TYPE_NODE),
            name: Default::default(),
            public_identifier: Default::default(),
            system_identifier: Default::default(),
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

    pub fn set_public_identifier(
        &mut self,
        maybe_pid: Option<&String>,
    ) -> &mut Self {
        self.public_identifier = if let Some(x) = maybe_pid {
            x.to_owned()
        } else {
            String::new()
        };
        self
    }

    pub fn set_system_identifier(
        &mut self,
        maybe_sid: Option<&String>,
    ) -> &mut Self {
        self.system_identifier = if let Some(x) = maybe_sid {
            x.to_owned()
        } else {
            String::new()
        };
        self
    }
}
