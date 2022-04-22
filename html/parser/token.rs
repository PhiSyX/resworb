/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

// ---- //
// Type //
// ---- //

pub type TagAttributeName = String;
pub type TagAttributeValue = String;

// --------- //
// Structure //
// --------- //

#[derive(Clone)]
#[non_exhaustive]
pub struct HTMLTokenTag {
    name: String,
    self_closing_flag: bool,
    attributes: HashMap<TagAttributeName, TagAttributeValue>,
}

// ----------- //
// Énumération //
// ----------- //

/// La sortie de l'étape de tokenisation est une série de zéro ou plus des
/// jetons suivants :
///   - DOCTYPE
///    - balise de début
///    - balise de fin
///    - commentaire
///    - caractère
///    - fin de fichier
#[derive(Clone)]
pub enum HTMLToken {
    /// Les jetons `DOCTYPE` ont :
    ///   - un nom
    ///   - un identifiant public
    ///   - un identifiant système
    ///   - un drapeau "force-quirks"
    DOCTYPE {
        name: Option<String>,
        public_identifier: Option<String>,
        system_identifier: Option<String>,
        force_quirks_flag: bool,
    },

    /// Les balises de début et de fin ont
    ///   - un nom de balise
    ///   - un drapeau de fermeture automatique
    ///   - une liste d'attributs: chacun d'entre eux ayant un nom et une
    ///     valeur.
    StartTag(HTMLTokenTag),

    /// Les balises de début et de fin ont
    ///   - un nom de balise
    ///   - un drapeau de fermeture automatique
    ///   - une liste d'attributs: chacun d'entre eux ayant un nom et une
    ///     valeur.
    EndTag(HTMLTokenTag),

    /// Les jetons de commentaire ont une chaîne de caractères.
    Comment(String),

    /// Les jetons de caractère ont un caractères.
    Character(char),

    EOF,
}

// -------------- //
// Implémentation //
// -------------- //

impl HTMLToken {
    /// Lorsqu'un jeton [DOCTYPE](HTMLToken::DOCTYPE) est créé, son nom,
    /// son identificateur public et son identificateur système doivent
    /// être marqués comme [manquants](None) (ce qui est un état distinct
    /// de la chaîne vide), et l'indicateur `force-quirks` doit être
    /// désactivé (son autre état est activé).
    pub fn new_doctype() -> Self {
        Self::DOCTYPE {
            name: None,
            public_identifier: None,
            system_identifier: None,
            force_quirks_flag: false,
        }
    }

    /// Lorsqu'un jeton de balise de début ou de fin est créé, son
    /// indicateur de fermeture automatique doit être désactivé (son
    /// autre état est qu'il soit activé), et sa liste d'attributs
    /// doit être vide.
    pub fn new_start_tag(name: String) -> Self {
        Self::StartTag(HTMLTokenTag {
            name,
            self_closing_flag: false,
            attributes: HashMap::default(),
        })
    }

    /// Lorsqu'un jeton de balise de début ou de fin est créé, son
    /// indicateur de fermeture automatique doit être désactivé (son
    /// autre état est qu'il soit activé), et sa liste d'attributs
    /// doit être vide.
    pub fn new_end_tag(name: String) -> Self {
        Self::EndTag(HTMLTokenTag {
            name,
            self_closing_flag: false,
            attributes: HashMap::default(),
        })
    }
}
