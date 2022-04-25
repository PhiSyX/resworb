/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// ---- //
// Type //
// ---- //

pub type HTMLTagAttributeName = String;
pub type HTMLTagAttributeValue = String;
pub type HTMLTagAttribute = (HTMLTagAttributeName, HTMLTagAttributeValue);

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
        attributes: Vec<HTMLTagAttribute>,
    },

    /// Les balises de fin ont :
    ///   - un nom de balise
    ///   - un drapeau de fermeture automatique
    ///   - une liste d'attributs: chacun d'entre eux ayant un nom et une
    ///     valeur.
    EndTag {
        name: String,
        self_closing_flag: bool,
        attributes: Vec<HTMLTagAttribute>,
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

    pub fn define_doctype_name(mut self, ch: char) -> Self {
        if let Self::DOCTYPE { ref mut name, .. } = self {
            let c = String::from(ch);
            *name = Some(c);
        }

        self
    }

    pub fn define_force_quirks_flag(mut self) -> Self {
        if let Self::DOCTYPE {
            ref mut force_quirks_flag,
            ..
        } = self
        {
            *force_quirks_flag = true;
        }

        self
    }

    /// Lorsqu'un jeton de [balise de début](HTMLToken::StartTag) est créé,
    /// son indicateur de fermeture automatique doit être désactivé
    /// (son autre état est qu'il soit activé), et sa liste d'attributs
    /// doit être vide.
    pub fn new_start_tag(name: String) -> Self {
        Self::StartTag {
            name,
            self_closing_flag: false,
            attributes: Vec::default(),
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
            attributes: Vec::default(),
        }
    }
}

impl HTMLToken {
    /// Ajoute un caractère au DOCTYPE, ou nom de la balise de début ou de
    /// fin ou a un commentaire.
    pub fn append_character(&mut self, ch: char) {
        assert!(matches!(
            self,
            Self::DOCTYPE { .. }
                | Self::StartTag { .. }
                | Self::EndTag { .. }
                | Self::Comment(_)
        ));

        if let Self::DOCTYPE {
            name: Some(name), ..
        }
        | Self::StartTag { name, .. }
        | Self::EndTag { name, .. }
        | Self::Comment(name) = self
        {
            name.push(ch);
        }
    }

    pub fn append_character_to_attribute_name(&mut self, ch: char) {
        assert!(matches!(
            self,
            Self::StartTag { .. } | Self::EndTag { .. }
        ));

        if let Self::StartTag { attributes, .. }
        | Self::EndTag { attributes, .. } = self
        {
            let attr = attributes.iter_mut().last().unwrap();
            attr.0.push(ch);
        }
    }

    pub fn append_character_to_attribute_value(&mut self, ch: char) {
        assert!(matches!(
            self,
            Self::StartTag { .. } | Self::EndTag { .. }
        ));

        if let Self::StartTag { attributes, .. }
        | Self::EndTag { attributes, .. } = self
        {
            let attr = attributes.iter_mut().last().unwrap();
            attr.1.push(ch);
        }
    }

    pub fn append_character_to_public_identifier(&mut self, ch: char) {
        assert!(matches!(
            self,
            Self::DOCTYPE {
                public_identifier: Some(_),
                ..
            }
        ));

        if let Self::DOCTYPE {
            public_identifier: Some(public_identifier),
            ..
        } = self
        {
            public_identifier.push(ch);
        }
    }

    pub fn append_character_to_system_identifier(&mut self, ch: char) {
        assert!(matches!(
            self,
            Self::DOCTYPE {
                system_identifier: Some(_),
                ..
            }
        ));

        if let Self::DOCTYPE {
            system_identifier: Some(system_identifier),
            ..
        } = self
        {
            system_identifier.push(ch);
        }
    }

    pub fn define_tag_attributes(&mut self, attribute: HTMLTagAttribute) {
        assert!(matches!(
            self,
            Self::StartTag { .. } | Self::EndTag { .. }
        ));

        if let Self::StartTag { attributes, .. }
        | Self::EndTag { attributes, .. } = self
        {
            attributes.push(attribute);
        }
    }

    pub fn set_force_quirks_flag(&mut self, to: bool) {
        assert!(matches!(self, Self::DOCTYPE { .. }));

        if let Self::DOCTYPE {
            force_quirks_flag, ..
        } = self
        {
            *force_quirks_flag = to;
        }
    }

    pub fn set_public_identifier(&mut self, pi: String) {
        assert!(matches!(
            self,
            Self::DOCTYPE {
                public_identifier: None,
                ..
            }
        ));

        if let Self::DOCTYPE {
            public_identifier, ..
        } = self
        {
            *public_identifier = Some(pi);
        }
    }

    pub fn set_system_identifier(&mut self, si: String) {
        assert!(matches!(
            self,
            Self::DOCTYPE {
                system_identifier: None,
                ..
            }
        ));

        if let Self::DOCTYPE {
            system_identifier, ..
        } = self
        {
            *system_identifier = Some(si);
        }
    }
}
