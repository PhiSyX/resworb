/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use serde::Deserialize;

// ---- //
// Type //
// ---- //

pub type NamedCharacterReferencesEntities =
    HashMap<String, NamedCharacterReferenceEntity>;

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
#[derive(Deserialize)]
pub struct NamedCharacterReferences(
    HashMap<String, NamedCharacterReferenceEntity>,
);

#[derive(Debug)]
#[derive(Deserialize)]
pub struct NamedCharacterReferenceEntity {
    pub codepoints: Vec<usize>,
    pub characters: String,
}

// -------------- //
// Implémentation //
// -------------- //

impl NamedCharacterReferences {
    pub fn entities() -> NamedCharacterReferencesEntities {
        let json = include_str!("entities.json");
        let named_character_references =
            serde_json::from_str::<NamedCharacterReferences>(json)
                .expect("Les entités références des caractères nommés");
        named_character_references.0
    }
}
