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

/// Répertorie les noms de référence des caractères pris en charge par
/// HTML, ainsi que les points de code auxquels ils font référence.
#[derive(Debug)]
#[derive(Deserialize)]
pub struct NamedCharacterReferences(NamedCharacterReferencesEntities);

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Deserialize)]
pub struct NamedCharacterReferenceEntity {
    pub codepoints: Vec<u32>,
    pub characters: String,
}

// -------------- //
// Implémentation //
// -------------- //

impl NamedCharacterReferences {
    /// Dé-sérialise les entités références des caractères nommés vers
    /// [NamedCharacterReferencesEntities] et nous le retourne.
    pub fn entities() -> NamedCharacterReferencesEntities {
        // Ce JSON provient de `https://html.spec.whatwg.org/entities.json`
        let json_entities: &'static str = include_str!("entities.json");

        let named_character_references: NamedCharacterReferences =
            serde_json::from_str(json_entities)
                .expect("Les entités références des caractères nommés");

        named_character_references.0
    }
}

// ---- //
// Test //
// ---- //

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_use_case() {
        let entities = NamedCharacterReferences::entities();

        let entity = entities.get("&AElig");

        assert_eq!(
            entity,
            Some(&NamedCharacterReferenceEntity {
                characters: String::from('\u{00C6}'),
                codepoints: vec![198]
            })
        );

        let entity = entities.get("&test");
        assert_eq!(entity, None);
    }
}
