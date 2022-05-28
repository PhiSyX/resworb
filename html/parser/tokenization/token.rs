/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom::node::QuirksMode;
use html_elements::{
    interface::IsOneOfAttributesInterface, tag_attributes, tag_names,
};
use infra::{namespace::Namespace, primitive::codepoint::CodePoint};

// ---- //
// Type //
// ---- //

pub type HTMLTagAttributeName = String;
pub type HTMLTagAttributeValue = String;

// --------- //
// Structure //
// --------- //

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
    pub(crate) name: String,
    pub(crate) self_closing_flag: bool,
    pub(crate) self_closing_flag_acknowledge: bool,
    pub(crate) attributes: Vec<HTMLTagAttribute>,
    pub(crate) is_end: bool,
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(Default)]
#[derive(PartialEq)]
pub struct HTMLTagAttribute {
    pub(crate) name: HTMLTagAttributeName,
    pub(crate) value: HTMLTagAttributeValue,
    pub(crate) prefix: Option<String>,
    pub(crate) namespace_uri: Option<Namespace>,
}

// ----------- //
// Énumération //
// ----------- //

#[derive(Debug)]
#[derive(Copy, Clone)]
#[derive(PartialEq)]
pub enum ForceQuirksFlag {
    On = 1,
    Off = 0,
}

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
#[allow(clippy::upper_case_acronyms)]
pub enum HTMLToken {
    /// Les jetons `DOCTYPE` ont :
    ///   - un nom ;
    ///   - un identifiant public ;
    ///   - un identifiant système ;
    ///   - un drapeau "force-quirks".
    DOCTYPE {
        name: Option<String>,
        public_identifier: Option<String>,
        system_identifier: Option<String>,
        force_quirks_flag: ForceQuirksFlag,
    },

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
            Self::DOCTYPE { .. } | Self::Tag(_) | Self::Comment(_)
        ));

        if let Self::DOCTYPE {
            name: Some(name), ..
        }
        | Self::Tag(HTMLTagToken { name, .. })
        | Self::Comment(name) = self
        {
            name.push(ch);
        }
    }

    pub(crate) const fn name(&self) -> &String {
        match self {
            | Self::DOCTYPE {
                name: Some(name), ..
            } => name,
            | Self::Tag(HTMLTagToken { name, .. }) => name,
            | Self::Comment(name) => name,
            | _ => {
                panic!("Impossible d'obtenir le nom du jeton.");
            }
        }
    }

    pub fn is_eof(&self) -> bool {
        matches!(self, Self::EOF)
    }
}

impl HTMLToken {
    /// Défini un nom pour le [DOCTYPE](HTMLToken::DOCTYPE).
    pub fn with_name(mut self, new_name: impl ToString) -> Self {
        assert!(matches!(self, Self::DOCTYPE { name: None, .. }));
        if let Self::DOCTYPE { ref mut name, .. } = self {
            *name = Some(new_name.to_string());
        }
        self
    }
}

// ------------- //
// Jeton DOCTYPE //
// ------------- //

// Self
impl HTMLToken {
    /// Lorsqu'un jeton [DOCTYPE](HTMLToken::DOCTYPE) est créé, son nom,
    /// son identificateur public et son identificateur système doivent
    /// être marqués comme [manquants](None) (ce qui est un état distinct
    /// de la chaîne de caractères vide), et le drapeau `force-quirks`
    /// doit être désactivé (son autre état est activé).
    pub(crate) const fn new_doctype() -> Self {
        Self::DOCTYPE {
            name: None,
            public_identifier: None,
            system_identifier: None,
            force_quirks_flag: ForceQuirksFlag::Off,
        }
    }

    /// Défini un identificateur public pour le
    /// [DOCTYPE](HTMLToken::DOCTYPE).
    pub fn with_public_identifier(mut self, pid: impl ToString) -> Self {
        assert!(matches!(self, Self::DOCTYPE { .. }));
        if let Self::DOCTYPE {
            ref mut public_identifier,
            ..
        } = self
        {
            *public_identifier = pid.to_string().into();
        }
        self
    }

    /// Défini un identificateur système pour le
    /// [DOCTYPE](HTMLToken::DOCTYPE).
    pub fn with_system_identifier(mut self, sid: impl ToString) -> Self {
        assert!(matches!(self, Self::DOCTYPE { .. }));
        if let Self::DOCTYPE {
            ref mut system_identifier,
            ..
        } = self
        {
            *system_identifier = sid.to_string().into();
        }
        self
    }

    /// Active le drapeau `force-quirks` pour le
    /// [DOCTYPE](HTMLToken::DOCTYPE).
    pub fn with_quirks_mode(mut self) -> Self {
        assert!(matches!(self, Self::DOCTYPE { .. }));
        if let Self::DOCTYPE {
            ref mut force_quirks_flag,
            ..
        } = self
        {
            *force_quirks_flag = ForceQuirksFlag::On;
        }
        self
    }
}

// &mut Self

impl HTMLToken {
    /// Ajoute un caractère au jeton `DOCTYPE` à son identifiant public.
    pub(crate) fn append_character_to_public_identifier(
        &mut self,
        ch: CodePoint,
    ) {
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

    /// Ajoute un caractère au jeton `DOCTYPE` à son identifiant système.
    pub(crate) fn append_character_to_system_identifier(
        &mut self,
        ch: CodePoint,
    ) {
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

    pub(crate) fn set_force_quirks_flag(&mut self, to: ForceQuirksFlag) {
        assert!(matches!(self, Self::DOCTYPE { .. }));
        if let Self::DOCTYPE {
            force_quirks_flag, ..
        } = self
        {
            *force_quirks_flag = to;
        }
    }

    pub(crate) fn set_public_identifier(&mut self, pid: impl ToString) {
        assert!(matches!(self, Self::DOCTYPE { .. }));
        if let Self::DOCTYPE {
            ref mut public_identifier,
            ..
        } = self
        {
            *public_identifier = pid.to_string().into();
        }
    }

    pub(crate) fn set_system_identifier(&mut self, sid: impl ToString) {
        assert!(matches!(self, Self::DOCTYPE { .. }));
        if let Self::DOCTYPE {
            ref mut system_identifier,
            ..
        } = self
        {
            *system_identifier = sid.to_string().into();
        }
    }
}

// &Self

impl HTMLToken {
    pub(crate) const fn public_identifier(&self) -> Option<&String> {
        assert!(matches!(self, Self::DOCTYPE { .. }));
        if let Self::DOCTYPE {
            public_identifier, ..
        } = self
        {
            public_identifier.as_ref()
        } else {
            None
        }
    }

    pub(crate) const fn system_identifier(&self) -> Option<&String> {
        assert!(matches!(self, Self::DOCTYPE { .. }));
        if let Self::DOCTYPE {
            system_identifier, ..
        } = self
        {
            system_identifier.as_ref()
        } else {
            None
        }
    }

    pub(crate) fn is_html_name(&self) -> bool {
        assert!(matches!(self, Self::DOCTYPE { .. }));
        if let Self::DOCTYPE { name, .. } = self {
            name.as_ref().contains(&tag_names::html)
        } else {
            false
        }
    }

    pub(crate) const fn is_public_identifier_missing(&self) -> bool {
        assert!(matches!(self, Self::DOCTYPE { .. }));
        if let Self::DOCTYPE {
            public_identifier, ..
        } = self
        {
            public_identifier.is_none()
        } else {
            false
        }
    }

    pub(crate) const fn is_system_identifier_missing(&self) -> bool {
        assert!(matches!(self, Self::DOCTYPE { .. }));
        if let Self::DOCTYPE {
            system_identifier, ..
        } = self
        {
            system_identifier.is_none()
        } else {
            false
        }
    }

    pub(crate) fn is_about_legacy_compat(&self) -> bool {
        assert!(matches!(self, Self::DOCTYPE { .. }));
        if let Self::DOCTYPE {
            system_identifier, ..
        } = self
        {
            system_identifier.contains(&"about:legacy-compat")
        } else {
            false
        }
    }

    pub(crate) fn quirks_mode(&self) -> QuirksMode {
        assert!(matches!(self, Self::DOCTYPE { .. }));

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

        if let Self::DOCTYPE {
            force_quirks_flag, ..
        } = self
        {
            // Le drapeau force-quirks est activé
            if ForceQuirksFlag::On.eq(force_quirks_flag) {
                return QuirksMode::Yes;
            }
        }

        // Le nom du doctype n'est pas "html"
        if !self.is_html_name() {
            return QuirksMode::Yes;
        }

        if let Self::DOCTYPE {
            public_identifier,
            system_identifier,
            ..
        } = self
        {
            // L'identifieur publique est défini à l'une des entrées du
            // tableau [DOCTYPE::PUBLIC_ID_DEFINED_RULE_1]
            let is_pid = DOCTYPE::PUBLIC_ID_DEFINED_RULE_1
                .into_iter()
                .any(|x| is_eq(public_identifier, x));

            if is_pid {
                return QuirksMode::Yes;
            }

            // L'identifieur publique est défini à l'une des entrées du
            // tableau [DOCTYPE::SYSTEM_ID_DEFINED_RULE_1]
            let is_sid = DOCTYPE::SYSTEM_ID_DEFINED_RULE_1
                .into_iter()
                .any(|x| is_eq(system_identifier, x));

            if is_sid {
                return QuirksMode::Yes;
            }

            // L'identifiant public commence par l'une des entrées du
            // tableau [DOCTYPE::PUBLIC_ID_STARTS_WITH_RULE_1]
            let is_starts_with_pid = DOCTYPE::PUBLIC_ID_STARTS_WITH_RULE_1
                .into_iter()
                .any(|x| is_start_with(public_identifier, x));

            if is_starts_with_pid {
                return QuirksMode::Yes;
            }

            // L'identifiant système commence par l'une des entrées du
            // tableau [DOCTYPE::SYSTEM_ID_STARTS_WITH_RULE_1]
            let is_starts_with_sid = DOCTYPE::SYSTEM_ID_STARTS_WITH_RULE_1
                .into_iter()
                .any(|x| is_start_with(system_identifier, x));

            if is_starts_with_sid {
                return QuirksMode::Yes;
            }

            // L'identifiant public commence par l'une des entrées du
            // tableau [DOCTYPE::PUBLIC_ID_STARTS_WITH_RULE_2]
            let is_starts_with_pid = DOCTYPE::PUBLIC_ID_STARTS_WITH_RULE_2
                .into_iter()
                .any(|x| is_start_with(public_identifier, x));

            if is_starts_with_pid {
                return QuirksMode::Yes;
            }

            // L'identifiant système n'est pas manquant et l'identifier
            // public commence par l'une des entrées du tableau
            // [DOCTYPE::PUBLIC_ID_STARTS_WITH_RULE_2_1]
            if !self.is_system_identifier_missing() {
                let is_starts_with_pid =
                    DOCTYPE::PUBLIC_ID_STARTS_WITH_RULE_2_1
                        .into_iter()
                        .any(|x| is_start_with(public_identifier, x));

                if is_starts_with_pid {
                    return QuirksMode::Yes;
                }
            }
        }

        QuirksMode::No
    }
}

impl HTMLToken {
    pub(crate) fn as_doctype(&self) -> &HTMLToken {
        assert!(matches!(self, Self::DOCTYPE { .. }));
        self
    }
}

// --------- //
// Jeton tag //
// --------- //

impl HTMLToken {
    pub const fn as_tag(&self) -> &HTMLTagToken {
        assert!(matches!(self, Self::Tag(HTMLTagToken { .. })));
        if let Self::Tag(tag) = self {
            return tag;
        }
        unreachable!()
    }

    pub fn as_tag_mut(&mut self) -> &mut HTMLTagToken {
        assert!(matches!(self, Self::Tag(HTMLTagToken { .. })));
        if let Self::Tag(tag) = self {
            return tag;
        }
        unreachable!()
    }

    pub const fn is_start_tag(&self) -> bool {
        if let Self::Tag(HTMLTagToken {
            is_end: is_end_token,
            ..
        }) = self
        {
            !(*is_end_token)
        } else {
            false
        }
    }

    pub const fn is_end_tag(&self) -> bool {
        if let Self::Tag(HTMLTagToken {
            is_end: is_end_token,
            ..
        }) = self
        {
            *is_end_token
        } else {
            false
        }
    }
}

impl HTMLTagToken {
    /// Lorsqu'un jeton [start-tag](HTMLToken::Tag) est créé,
    /// son drapeau de fermeture automatique doit être désactivé
    /// (son autre état est qu'il soit activé), et sa liste d'attributs
    /// doit être vide.
    pub const fn start() -> Self {
        Self {
            name: String::new(),
            self_closing_flag: false,
            self_closing_flag_acknowledge: false,
            attributes: vec![],
            is_end: false,
        }
    }

    /// Lorsqu'un jeton de [end-tag](HTMLToken::Tag) est créé,
    /// son indicateur de fermeture automatique doit être désactivé
    /// (son autre état est qu'il soit activé), et sa liste d'attributs
    /// doit être vide.
    pub const fn end() -> Self {
        Self {
            name: String::new(),
            self_closing_flag: false,
            self_closing_flag_acknowledge: false,
            attributes: vec![],
            is_end: true,
        }
    }

    /// Définit le nom du jeton d'une balise [start-tag](HTMLToken::Tag).
    pub fn with_name(mut self, name: impl ToString) -> Self {
        self.name = name.to_string();
        self
    }

    /// Définit des attributs à un jeton d'une balise.
    pub fn with_attributes(
        mut self,
        attributes: impl IntoIterator<Item = impl Into<HTMLTagAttribute>>,
    ) -> Self {
        self.attributes =
            attributes.into_iter().map(|x| x.into()).collect();
        self
    }

    /// Définit le drapeau de fermeture automatique d'un jeton d'une balise
    /// [start-tag](HTMLToken::Tag).
    pub fn with_self_closing_flag(mut self) -> Self {
        self.self_closing_flag = true;
        self
    }
}

// &HTMLTagToken
impl HTMLTagToken {
    pub fn local_name(&self) -> &str {
        &self.name
    }

    pub fn tag_name(&self) -> tag_names {
        let Self { name, .. } = self;
        name.parse().expect("Devrait être un nom de balise valide")
    }
}

// &mut HTMLTagToken
impl HTMLTagToken {
    pub fn adjust_attribute_name(
        &mut self,
        old_name: impl ToString,
        new_name: impl ToString,
    ) {
        let old_name = old_name.to_string();
        let new_name = new_name.to_string();

        for attr in self.attributes.iter_mut() {
            if attr.name == old_name {
                attr.name = new_name.to_string();
            }
        }
    }

    pub fn adjust_foreign_attribute(
        &mut self,
        old_name: impl ToString,
        prefix: impl ToString,
        local_name: impl ToString,
        namespace: Namespace,
    ) {
        self.attributes.iter_mut().for_each(|attr| {
            if *attr.name == old_name.to_string() {
                attr.name = local_name.to_string();
                attr.prefix.replace(prefix.to_string());
                attr.namespace_uri.replace(namespace);
            }
        });
    }

    pub fn adjust_tag_name(
        &mut self,
        old_name: impl ToString,
        new_name: impl ToString,
    ) {
        if self.name == old_name.to_string() {
            self.name = new_name.to_string();
        }
    }

    /// Ajoute un caractère à un jeton `tag`, au nom d'un attribut
    /// (attr-name), le dernier attribut trouvé.
    ///
    /// attr-name="attr-value"
    pub fn append_character_to_attribute_name(&mut self, ch: CodePoint) {
        let Self { attributes, .. } = self;
        let attr = attributes.iter_mut().last().unwrap();
        attr.name.push(ch);
    }

    /// Ajoute un caractère à un jeton `tag`, au nom d'un attribut
    /// (attr-value), le dernier attribut trouvé.
    ///
    /// attr-name="attr-value"
    pub fn append_character_to_attribute_value(&mut self, ch: CodePoint) {
        let Self { attributes, .. } = self;
        let attr = attributes.iter_mut().last().unwrap();
        attr.value.push(ch);
    }

    pub fn append_tag_attributes(
        &mut self,
        attribute: impl Into<HTMLTagAttribute>,
    ) {
        let Self { attributes, .. } = self;
        attributes.push(attribute.into());
    }

    pub fn has_attributes(
        &self,
        attribute_names: impl IntoIterator<Item = tag_attributes> + Copy,
    ) -> bool {
        let Self { attributes, .. } = self;
        attributes
            .iter()
            .any(|attribute| attribute.name.is_one_of(attribute_names))
    }

    pub fn set_self_closing_tag(&mut self, to: bool) {
        let Self {
            self_closing_flag, ..
        } = self;
        *self_closing_flag = to;
    }

    pub fn set_acknowledge_self_closing_flag(&mut self) {
        let Self {
            self_closing_flag,
            self_closing_flag_acknowledge,
            ..
        } = self;
        if *self_closing_flag {
            *self_closing_flag_acknowledge = true;
        }
    }
}

impl HTMLTagAttribute {
    /// Crée un attribut de balise.
    pub fn new(name: impl ToString, value: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            value: value.to_string(),
            prefix: None,
            namespace_uri: None,
        }
    }
}

// ------------- //
// Jeton comment //
// ------------- //

impl HTMLToken {
    /// Crée un nouveau jeton (comment)(HTMLToken::Comment).
    pub fn new_comment(comment: impl ToString) -> Self {
        Self::Comment(comment.to_string())
    }
}

// --------------- //
// Jeton character //
// --------------- //

impl HTMLToken {
    /// Crée un nouveau jeton (character)(HTMLToken::Character).
    pub const fn new_character(ch: CodePoint) -> Self {
        Self::Character(ch)
    }

    pub const fn is_character(&self) -> bool {
        matches!(self, Self::Character(_))
    }

    pub const fn is_ascii_whitespace(&self) -> bool {
        if let Self::Character(ch) = self {
            ch.is_ascii_whitespace()
        } else {
            false
        }
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl<S1, S2> From<(S1, S2)> for HTMLTagAttribute
where
    S1: AsRef<str>,
    S2: AsRef<str>,
{
    fn from(nv: (S1, S2)) -> Self {
        Self::new(nv.0.as_ref(), nv.1.as_ref())
    }
}

mod DOCTYPE {
    pub(crate) const PUBLIC_ID_DEFINED_RULE_1: [&str; 3] = [
        "-//W3O//DTD W3 HTML Strict 3.0//EN//",
        "-/W3C/DTD HTML 4.0 Transitional/EN",
        "HTML",
    ];

    pub(crate) const SYSTEM_ID_DEFINED_RULE_1: [&str; 1] =
        ["http://www.ibm.com/data/dtd/v11/ibmxhtml1-transitional.dtd"];

    pub(crate) const PUBLIC_ID_STARTS_WITH_RULE_1: [&str; 55] = [
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
    pub(crate) const PUBLIC_ID_STARTS_WITH_RULE_2: [&str; 2] = [
        "-//W3C//DTD XHTML 1.0 Frameset//",
        "-//W3C//DTD XHTML 1.0 Transitional//",
    ];
    pub(crate) const PUBLIC_ID_STARTS_WITH_RULE_2_1: [&str; 2] = [
        "-//W3C//DTD HTML 4.01 Frameset//",
        "-//W3C//DTD HTML 4.01 Transitional//",
    ];

    pub(crate) const SYSTEM_ID_STARTS_WITH_RULE_1: [&str; 2] = [
        "-//W3C//DTD HTML 4.01 Frameset//",
        "-//W3C//DTD HTML 4.01 Transitional//",
    ];
}
