/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::CharacterData;
use crate::{
    document::HTMLDocument,
    node::{Node, NodeType},
};

// --------- //
// Structure //
// --------- //

pub struct Comment {
    character_data: CharacterData,
}

// -------------- //
// Implémentation //
// -------------- //

impl Comment {
    /// Les étapes du constructeur du nouveau Commentaire(data) consistent
    /// à définir les données de ce dernier comme étant des [CharacterData]
    /// et le nœud du document de ce dernier comme étant le [Document]
    /// associé à l'objet global actuel.
    pub fn new(document: &HTMLDocument, data: String) -> Self {
        let character_data =
            CharacterData::new(document, NodeType::COMMENT_NODE, data);
        Self { character_data }
    }

    pub fn node(&self) -> &Node {
        self.character_data.node()
    }
}
