/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom::document::QuirksMode;
use infra::primitive::codepoint::CodePoint;

// ---- //
// Type //
// ---- //

pub type HTMLTagAttributeName = String;
pub type HTMLTagAttributeValue = String;
pub type HTMLTagAttribute = (HTMLTagAttributeName, HTMLTagAttributeValue);

// --------- //
// Structure //
// --------- //

/// Les jetons `DOCTYPE` ont :
///   - un nom ;
///   - un identifiant public ;
///   - un identifiant système ;
///   - un drapeau "force-quirks".
#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
pub struct HTMLDoctypeToken {
    pub name: Option<String>,
    pub public_identifier: Option<String>,
    pub system_identifier: Option<String>,
    pub force_quirks_flag: bool,
}

/// Les jetons `start-tag` (ou `tag`) ont :
/// Les jetons `end-tag`   (ou `tag`) ont :
///   - un nom, un nom de balise ;
///   - un drapeau permettant de savoir s'il s'agit d'une balise
///     auto-fermante ;
///   - une liste d'attributs: chacun d'entre eux ayant un nom et une
///     valeur.
#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
pub struct HTMLTagToken {
    pub name: String,
    pub self_closing_flag: bool,
    pub attributes: Vec<HTMLTagAttribute>,
    pub is_end_token: bool,
}

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
    DOCTYPE(HTMLDoctypeToken),

    Tag(HTMLTagToken),

    /// Le jeton `comment` contient une chaîne de caractères.
    /// Pour cet exemple : `<!-- Hello World -->`. La chaîne de caractères
    /// ` Hello World ` sera stockée.
    Comment(String),

    /// Le jeton `character`, contient un caractère.
    Character(CodePoint),

    /// Le jeton `end of file`
    EOF,
}

// -------------- //
// Implémentation //
// -------------- //

impl HTMLToken {
    /// Ajoute un caractère au nom du jeton `DOCTYPE`, ou au nom du jeton
    /// `tag` ou à un jeton `comment`.
    pub fn append_character(&mut self, ch: CodePoint) {
        assert!(matches!(
            self,
            Self::DOCTYPE(_) | Self::Tag(_) | Self::Comment(_)
        ));

        if let Self::DOCTYPE(HTMLDoctypeToken {
            name: Some(name), ..
        })
        | Self::Tag(HTMLTagToken { name, .. })
        | Self::Comment(name) = self
        {
            name.push(ch);
        }
    }
}

// ------------- //
// Jeton DOCTYPE //
// ------------- //

impl HTMLToken {
    pub fn into_doctype(&mut self) -> &mut HTMLDoctypeToken {
        assert!(matches!(self, Self::DOCTYPE(_)));
        if let Self::DOCTYPE(doctype) = self {
            return doctype;
        }
        unreachable!()
    }
}

impl HTMLDoctypeToken {
    const PUBLIC_ID_DEFINED_RULE_1: [&'static str; 3] = [
        "-//W3O//DTD W3 HTML Strict 3.0//EN//",
        "-/W3C/DTD HTML 4.0 Transitional/EN",
        "HTML",
    ];
    const PUBLIC_ID_STARTS_WITH_RULE_1: [&'static str; 55] = [
        "+//Silmaril//dtd html Pro v0r11 19970101//",
        "-//AS//DTD HTML 3.0 asWedit + extensions//",
        "-//AdvaSoft Ltd//DTD HTML 3.0 asWedit + extensions//",
        "-//IETF//DTD HTML 2.0 Level 1//",
        "-//IETF//DTD HTML 2.0 Level 2//",
        "-//IETF//DTD HTML 2.0 Strict Level 1//",
        "-//IETF//DTD HTML 2.0 Strict Level 2//",
        "-//IETF//DTD HTML 2.0 Strict//",
        "-//IETF//DTD HTML 2.0//",
        "-//IETF//DTD HTML 2.1E//",
        "-//IETF//DTD HTML 3.0//",
        "-//IETF//DTD HTML 3.2 Final//",
        "-//IETF//DTD HTML 3.2//",
        "-//IETF//DTD HTML 3//",
        "-//IETF//DTD HTML Level 0//",
        "-//IETF//DTD HTML Level 1//",
        "-//IETF//DTD HTML Level 2//",
        "-//IETF//DTD HTML Level 3//",
        "-//IETF//DTD HTML Strict Level 0//",
        "-//IETF//DTD HTML Strict Level 1//",
        "-//IETF//DTD HTML Strict Level 2//",
        "-//IETF//DTD HTML Strict Level 3//",
        "-//IETF//DTD HTML Strict//",
        "-//IETF//DTD HTML//",
        "-//Metrius//DTD Metrius Presentational//",
        "-//Microsoft//DTD Internet Explorer 2.0 HTML Strict//",
        "-//Microsoft//DTD Internet Explorer 2.0 HTML//",
        "-//Microsoft//DTD Internet Explorer 2.0 Tables//",
        "-//Microsoft//DTD Internet Explorer 3.0 HTML Strict//",
        "-//Microsoft//DTD Internet Explorer 3.0 HTML//",
        "-//Microsoft//DTD Internet Explorer 3.0 Tables//",
        "-//Netscape Comm. Corp.//DTD HTML//",
        "-//Netscape Comm. Corp.//DTD Strict HTML//",
        "-//O'Reilly and Associates//DTD HTML 2.0//",
        "-//O'Reilly and Associates//DTD HTML Extended 1.0//",
        "-//O'Reilly and Associates//DTD HTML Extended Relaxed 1.0//",
        "-//SQ//DTD HTML 2.0 HoTMetaL + extensions//",
        "-//SoftQuad Software//DTD HoTMetaL PRO 6.0::19990601::extensions to HTML 4.0//",
        "-//SoftQuad//DTD HoTMetaL PRO 4.0::19971010::extensions to HTML 4.0//",
        "-//Spyglass//DTD HTML 2.0 Extended//",
        "-//Sun Microsystems Corp.//DTD HotJava HTML//",
        "-//Sun Microsystems Corp.//DTD HotJava Strict HTML//",
        "-//W3C//DTD HTML 3 1995-03-24//",
        "-//W3C//DTD HTML 3.2 Draft//",
        "-//W3C//DTD HTML 3.2 Final//",
        "-//W3C//DTD HTML 3.2//",
        "-//W3C//DTD HTML 3.2S Draft//",
        "-//W3C//DTD HTML 4.0 Frameset//",
        "-//W3C//DTD HTML 4.0 Transitional//",
        "-//W3C//DTD HTML Experimental 19960712//",
        "-//W3C//DTD HTML Experimental 970421//",
        "-//W3C//DTD W3 HTML//",
        "-//W3O//DTD W3 HTML 3.0//",
        "-//WebTechs//DTD Mozilla HTML 2.0//",
        "-//WebTechs//DTD Mozilla HTML//",
    ];
    const PUBLIC_ID_STARTS_WITH_RULE_2: [&'static str; 2] = [
        "-//W3C//DTD XHTML 1.0 Frameset//",
        "-//W3C//DTD XHTML 1.0 Transitional//",
    ];
    const PUBLIC_ID_STARTS_WITH_RULE_2_1: [&'static str; 2] = [
        "-//W3C//DTD HTML 4.01 Frameset//",
        "-//W3C//DTD HTML 4.01 Transitional//",
    ];
    const SYSTEM_ID_DEFINED_RULE_1: [&'static str; 1] =
        ["http://www.ibm.com/data/dtd/v11/ibmxhtml1-transitional.dtd"];
    const SYSTEM_ID_STARTS_WITH_RULE_1: [&'static str; 2] = [
        "-//W3C//DTD HTML 4.01 Frameset//",
        "-//W3C//DTD HTML 4.01 Transitional//",
    ];

    /// Lorsqu'un jeton [DOCTYPE](HTMLToken::DOCTYPE) est créé, son nom,
    /// son identificateur public et son identificateur système doivent
    /// être marqués comme [manquants](None) (ce qui est un état distinct
    /// de la chaîne de caractères vide), et le drapeau `force-quirks`
    /// doit être désactivé (son autre état est activé).
    pub const fn new() -> Self {
        Self {
            name: None,
            public_identifier: None,
            system_identifier: None,
            force_quirks_flag: false,
        }
    }

    // &Self

    pub fn is_html_name(&self) -> bool {
        self.name == Some("html".to_owned())
    }

    pub fn is_public_identifier_missing(&self) -> bool {
        self.public_identifier.is_none()
    }

    pub fn is_system_identifier_missing(&self) -> bool {
        self.system_identifier.is_none()
    }

    pub fn is_about_legacy_compat(&self) -> bool {
        self.system_identifier == Some("about:legacy-compat".to_owned())
    }

    pub fn quirks_mode(&self) -> QuirksMode {
        fn is_eq(maybe_id: &Option<String>, xid: &str) -> bool {
            match maybe_id {
                | Some(p) if p.eq_ignore_ascii_case(xid) => true,
                | _ => false,
            }
        }

        fn is_start_with(maybe_id: &Option<String>, xid: &str) -> bool {
            match maybe_id {
                | Some(p)
                    if p.to_lowercase()
                        .starts_with(&xid.to_lowercase()) =>
                {
                    true
                }
                | _ => false,
            }
        }

        // Le drapeau force-quirks est activé
        if self.force_quirks_flag {
            return QuirksMode::Yes;
        }

        // Le nom du doctype n'est pas "html"
        if !self.is_html_name() {
            return QuirksMode::Yes;
        }

        // L'identifieur publique est défini à l'une des entrées du tableau
        // [HTMLDoctypeToken::PUBLIC_ID_DEFINED_RULE_1]
        let is_pid = Self::PUBLIC_ID_DEFINED_RULE_1
            .into_iter()
            .any(|x| is_eq(&self.public_identifier, x));

        if is_pid {
            return QuirksMode::Yes;
        }

        // L'identifieur publique est défini à l'une des entrées du tableau
        // [HTMLDoctypeToken::SYSTEM_ID_DEFINED_RULE_1]
        let is_sid = Self::SYSTEM_ID_DEFINED_RULE_1
            .into_iter()
            .any(|x| is_eq(&self.system_identifier, x));

        if is_sid {
            return QuirksMode::Yes;
        }

        // L'identifiant public commence par l'une des entrées du tableau
        // [HTMLDoctypeToken::PUBLIC_ID_STARTS_WITH_RULE_1]
        let is_starts_with_pid = Self::PUBLIC_ID_STARTS_WITH_RULE_1
            .into_iter()
            .any(|x| is_start_with(&self.public_identifier, x));

        if is_starts_with_pid {
            return QuirksMode::Yes;
        }

        // L'identifiant système commence par l'une des entrées du tableau
        // [HTMLDoctypeToken::SYSTEM_ID_STARTS_WITH_RULE_1]
        let is_starts_with_sid = Self::SYSTEM_ID_STARTS_WITH_RULE_1
            .into_iter()
            .any(|x| is_start_with(&self.system_identifier, x));

        if is_starts_with_sid {
            return QuirksMode::Yes;
        }

        // L'identifiant public commence par l'une des entrées du tableau
        // [HTMLDoctypeToken::PUBLIC_ID_STARTS_WITH_RULE_2]
        let is_starts_with_pid = Self::PUBLIC_ID_STARTS_WITH_RULE_2
            .into_iter()
            .any(|x| is_start_with(&self.public_identifier, x));

        if is_starts_with_pid {
            return QuirksMode::Yes;
        }

        // L'identifiant système n'est pas manquant et l'identifier public
        // commence par l'une des entrées du tableau
        // [HTMLDoctypeToken::PUBLIC_ID_STARTS_WITH_RULE_2_1]
        if !self.is_system_identifier_missing() {
            let is_starts_with_pid = Self::PUBLIC_ID_STARTS_WITH_RULE_2_1
                .into_iter()
                .any(|x| is_start_with(&self.public_identifier, x));

            if is_starts_with_pid {
                return QuirksMode::Yes;
            }
        }

        QuirksMode::No
    }

    // &mut Self

    /// Définie le drapeau force-quirks du jeton `DOCTYPE` à true.
    /// À priori le drapeau est désactivé ; à posteriori le drapeau est
    /// activé.
    pub fn define_force_quirks_flag(mut self) -> Self {
        assert!(matches!(
            self,
            Self {
                force_quirks_flag: false,
                ..
            }
        ));

        let Self {
            ref mut force_quirks_flag,
            ..
        } = self;
        *force_quirks_flag = true;

        self
    }

    /// Définie une chaîne de caractères au nom du jeton `DOCTYPE`.
    /// À priori son nom est équivalent à `None`, à posteriori son nom
    /// doit être équivalent à `Some(String)`.
    pub fn define_doctype_name(mut self, ch: CodePoint) -> Self {
        assert!(matches!(self, Self { name: None, .. }));

        let Self { ref mut name, .. } = self;
        *name = Some(ch.into());

        self
    }

    // &mut Self

    /// Ajoute un caractère au jeton `DOCTYPE` à son identifiant public.
    pub fn append_character_to_public_identifier(
        &mut self,
        ch: CodePoint,
    ) {
        assert!(matches!(
            self,
            Self {
                public_identifier: Some(_),
                ..
            }
        ));

        if let Self {
            public_identifier: Some(public_identifier),
            ..
        } = self
        {
            public_identifier.push(ch);
        }
    }

    /// Ajoute un caractère à l'identifiant système d'un DOCTYPE.
    pub fn append_character_to_system_identifier(
        &mut self,
        ch: CodePoint,
    ) {
        if let Self {
            system_identifier: Some(system_identifier),
            ..
        } = self
        {
            system_identifier.push(ch);
        }
    }

    pub fn set_force_quirks_flag(&mut self, to: bool) {
        let Self {
            force_quirks_flag, ..
        } = self;
        *force_quirks_flag = to;
    }

    pub fn set_public_identifier(&mut self, pi: String) {
        let Self {
            public_identifier, ..
        } = self;
        *public_identifier = Some(pi);
    }

    pub fn set_system_identifier(&mut self, si: String) {
        let Self {
            system_identifier, ..
        } = self;
        *system_identifier = Some(si);
    }
}

// --------- //
// Jeton tag //
// --------- //

impl HTMLToken {
    pub fn into_start_tag(&mut self) -> &mut HTMLTagToken {
        assert!(matches!(
            self,
            Self::Tag(HTMLTagToken {
                is_end_token: false,
                ..
            })
        ));
        if let Self::Tag(tag) = self {
            return tag;
        }
        unreachable!()
    }

    pub fn into_end_tag(&mut self) -> &mut HTMLTagToken {
        assert!(matches!(
            self,
            Self::Tag(HTMLTagToken {
                is_end_token: true,
                ..
            })
        ));
        if let Self::Tag(tag) = self {
            return tag;
        }
        unreachable!()
    }
}

impl HTMLTagToken {
    /// Lorsqu'un jeton [start-tag](HTMLToken::StartTag) est créé,
    /// son drapeau de fermeture automatique doit être désactivé
    /// (son autre état est qu'il soit activé), et sa liste d'attributs
    /// doit être vide.
    pub fn start() -> Self {
        Self {
            name: String::new(),
            self_closing_flag: false,
            attributes: vec![],
            is_end_token: false,
        }
    }

    /// Lorsqu'un jeton de [balise de début](HTMLToken::EndTag) est créé,
    /// son indicateur de fermeture automatique doit être désactivé
    /// (son autre état est qu'il soit activé), et sa liste d'attributs
    /// doit être vide.
    pub fn end() -> Self {
        Self {
            name: String::new(),
            self_closing_flag: false,
            attributes: vec![],
            is_end_token: true,
        }
    }

    /// Ajoute un caractère à un jeton `tag`, au nom d'un attribut
    /// (attr-name), le dernier attribut trouvé.
    ///
    /// attr-name="attr-value"
    pub fn append_character_to_attribute_name(&mut self, ch: CodePoint) {
        let Self { attributes, .. } = self;
        let attr = attributes.iter_mut().last().unwrap();
        attr.0.push(ch);
    }

    /// Ajoute un caractère à un jeton `tag`, au nom d'un attribut
    /// (attr-value), le dernier attribut trouvé.
    ///
    /// attr-name="attr-value"
    pub fn append_character_to_attribute_value(&mut self, ch: CodePoint) {
        let Self { attributes, .. } = self;
        let attr = attributes.iter_mut().last().unwrap();
        attr.1.push(ch);
    }

    pub fn define_tag_attributes(&mut self, attribute: HTMLTagAttribute) {
        let Self { attributes, .. } = self;
        attributes.push(attribute);
    }

    pub fn set_self_closing_tag(&mut self, to: bool) {
        let Self {
            self_closing_flag, ..
        } = self;
        *self_closing_flag = to;
    }
}

// ------------- //
// Jeton comment //
// ------------- //

impl HTMLToken {
    /// Crée un nouveau jeton (comment)(HTMLToken::Comment).
    pub fn new_comment(comment: String) -> Self {
        Self::Comment(comment)
    }
}

// --------------- //
// Jeton character //
// --------------- //

impl HTMLToken {
    /// Crée un nouveau jeton (character)(HTMLToken::Character).
    pub fn new_character(ch: CodePoint) -> Self {
        Self::Character(ch)
    }

    pub fn is_character(&self) -> bool {
        matches!(self, Self::Character(_))
    }

    pub fn is_ascii_whitespace(&self) -> bool {
        if let Self::Character(ch) = self {
            ch.is_ascii_whitespace()
        } else {
            false
        }
    }
}
