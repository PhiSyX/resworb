/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// --------- //
// Structure //
// --------- //

use infra::primitive::codepoint::CodePoint;
use parser::StreamInputInterface;

// ----------- //
// Énumération //
// ----------- //

/// La sortie de l'étape de tokenisation est un flux de zéro ou plus des
/// jetons suivants.
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
pub enum CSSToken {
    /// `<ident-token>`, `<function-token>`, `<at-keyword-token>`,
    /// <hash-token>`, `<string-token>` et `<url-token>` ont une valeur
    /// composée de zéro ou plus points de code.
    ///
    /// Identifiant (comme auto`, `disc`, etc., qui sont simplement écrit
    /// comme leur valeur).
    Ident(String),

    /// Nom de la fonction suivi du caractère `(`, comme `translate(`.
    Function(String),

    /// Forme d'un caractère `@` suivi de la valeur du jeton, comme
    /// `@media`.
    AtKeyword(String),

    /// Les jetons de hachage ont un indicateur de type défini sur "id" ou
    /// "unrestricted" en plus.
    Hash(String, HashFlag),

    String(String),
    BadString,
    Url(String),
    BadUrl,

    /// `<delim-token>` a une valeur composée d'un seul point de code.
    Delim(char),

    /// `<number-token>`, `<percentage-token>` et `<dimension-token>` ont
    /// une valeur numérique. `<number-token>` et `<dimension-token>` ont
    /// en outre un indicateur de type défini sur "integer" ou
    /// "number".
    Number(f64, NumberFlag),

    Percentage(f64),

    /// Les `<dimension-token>` ont en outre une unité composée d'un ou
    /// plusieurs points de code.
    Dimension(f64, NumberFlag, DimensionUnit),

    Whitespace,

    /// Suite de points de code "<!--"
    CDO,
    /// Suite de points de code "-->"
    CDC,

    /// Caractère ':'
    Colon,
    /// Caractère ';'
    Semicolon,
    /// Caractère ','
    Comma,
    /// Caractère '['
    LeftSquareBracket,
    /// Caractère ']'
    RightSquareBracket,
    /// Caractère '('
    LeftParenthesis,
    /// Caractère ')'
    RightParenthesis,
    /// Caractère '{'
    LeftCurlyBracket,
    /// Caractère '}'
    RightCurlyBracket,

    EOF,
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(Default)]
#[derive(PartialEq, Eq)]
pub struct DimensionUnit(pub String);

#[derive(Debug)]
#[derive(Copy, Clone)]
#[derive(Default)]
#[derive(PartialEq, Eq)]
pub enum HashFlag {
    ID,

    /// Le drapeau de type prend par défaut la valeur "unrestricted".
    #[default]
    Unrestricted,
}

#[derive(Debug)]
#[derive(Copy, Clone)]
#[derive(Default)]
#[derive(PartialEq, Eq)]
pub enum NumberFlag {
    /// Le drapeau de type prend par défaut la valeur "integer".
    #[default]
    Integer,

    Number,
}

// -------------- //
// Implémentation //
// -------------- //

impl CSSToken {
    /// Variante miroir d'un dès jetons `<(-token>, <[-token>, <{-token>`,
    ///
    /// Exemple:
    /// Pour le jeton `<(-token>`, la variante miroir est `<)-token>`.
    pub(crate) fn mirror(&self) -> Self {
        assert!(matches!(
            self,
            Self::LeftParenthesis
                | Self::LeftSquareBracket
                | Self::LeftCurlyBracket
        ));

        match self {
            | Self::LeftParenthesis => Self::RightParenthesis,
            | Self::LeftSquareBracket => Self::RightSquareBracket,
            | Self::LeftCurlyBracket => Self::RightCurlyBracket,
            | _ => {
                unreachable!("Impossible de faire un miroir de {:?}", self)
            }
        }
    }

    pub(crate) fn name(&self) -> String {
        match self {
            | Self::Ident(s)
            | Self::Function(s)
            | Self::AtKeyword(s)
            | Self::String(s) => s.to_owned(),
            | _ => panic!(
                "Impossible de récupérer le nom du jeton {:?}.",
                self
            ),
        }
    }
}

impl CSSToken {
    pub(crate) fn append_character(&mut self, ch: CodePoint) {
        match self {
            | Self::Ident(s)
            | Self::Function(s)
            | Self::AtKeyword(s)
            | Self::Hash(s, _)
            | Self::Url(s)
            | Self::String(s) => s.push(ch),
            | _ => (),
        }
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

// NOTE(phisyx): obligé de faire ceci à cause du type f64 dans notre
//               énumération.
impl Eq for CSSToken {}

impl StreamInputInterface for CSSToken {
    fn eof() -> Self {
        Self::EOF
    }
}
