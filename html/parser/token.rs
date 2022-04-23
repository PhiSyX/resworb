/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

// ---- //
// Type //
// ---- //

pub type TagAttributeName = String;
pub type TagAttributeValue = String;

// ----------- //
// Énumération //
// ----------- //

/// La sortie de l'étape de tokenisation est une série de zéro ou plus des
/// jetons suivants :
///   - DOCTYPE
///   - balise de début
///   - balise de fin
///   - commentaire
///   - caractère
///   - fin de fichier
#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
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

    /// Les balises de début ont :
    ///   - un nom de balise
    ///   - un drapeau de fermeture automatique
    ///   - une liste d'attributs: chacun d'entre eux ayant un nom et une
    ///     valeur.
    StartTag {
        name: String,
        self_closing_flag: bool,
        attributes: HashMap<TagAttributeName, TagAttributeValue>,
    },

    /// Les balises de fin ont :
    ///   - un nom de balise
    ///   - un drapeau de fermeture automatique
    ///   - une liste d'attributs: chacun d'entre eux ayant un nom et une
    ///     valeur.
    EndTag {
        name: String,
        self_closing_flag: bool,
        attributes: HashMap<TagAttributeName, TagAttributeValue>,
    },

    /// Le jeton de commentaire a une chaîne de caractères.
    Comment(String),

    /// Le jeton de caractère a un caractère.
    Character(char),

    EOF,
}

// -------------- //
// Implémentation //
// -------------- //

impl HTMLToken {
    /// Crée un nouveau jeton de commentaire.
    pub fn new_comment(comment: String) -> Self {
        Self::Comment(comment)
    }

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

    /// Lorsqu'un jeton de [balise de début](HTMLToken::StartTag) est créé,
    /// son indicateur de fermeture automatique doit être désactivé
    /// (son autre état est qu'il soit activé), et sa liste d'attributs
    /// doit être vide.
    pub fn new_start_tag(name: String) -> Self {
        Self::StartTag {
            name,
            self_closing_flag: false,
            attributes: HashMap::default(),
        }
    }

    /// Lorsqu'un jeton de [balise de début](HTMLToken::EndTag) est créé,
    /// son indicateur de fermeture automatique doit être désactivé
    /// (son autre état est qu'il soit activé), et sa liste d'attributs
    /// doit être vide.
    pub fn new_end_tag(name: String) -> Self {
        Self::EndTag {
            name,
            self_closing_flag: false,
            attributes: HashMap::default(),
        }
    }
}

impl HTMLToken {
    /// Ajoute un caractère au nom de la balise de début ou de fin
    /// ou a un commentaire.
    pub fn append_character(&mut self, ch: char) {
        assert!(matches!(
            self,
            Self::StartTag { .. } | Self::EndTag { .. } | Self::Comment(_)
        ));

        if let Self::StartTag { name, .. }
        | Self::EndTag { name, .. }
        | Self::Comment(name) = self
        {
            name.push(ch);
        }
    }
}
