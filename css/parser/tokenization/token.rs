/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// ----------- //
// Énumération //
// ----------- //

#[derive(Debug)]
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
    Dimension(f64, NumberFlag, String),
    Whitespace,

    /// Suite de points de code "<!--"
    Cdo,
    /// Suite de points de code "-->"
    Cdc,

    /// Caractère ':'
    Colon,
    /// Caractère ';'
    Semicolon,
    /// Caractère ','
    Comma,
    /// Caractère '['
    LeftBracket,
    /// Caractère ']'
    RightBracket,
    /// Caractère '('
    LeftParenthesis,
    /// Caractère ')'
    RightParenthesis,
    /// Caractère '{'
    LeftBrace,
    /// Caractère '}'
    RightBrace,

    EOF,
}

#[derive(Debug)]
#[derive(Default)]
#[derive(PartialEq, Eq)]
pub enum HashFlag {
    ID,

    /// Le drapeau de type prend par défaut la valeur "unrestricted".
    #[default]
    Unrestricted,
}

#[derive(Debug)]
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
    pub(crate) fn define_hash_flag(mut self, flag: HashFlag) -> Self {
        assert!(matches!(self, CSSToken::Hash(_, _)));

        if let CSSToken::Hash(_, ref mut current_flag) = self {
            *current_flag = flag;
        }

        self
    }

    pub(crate) fn define_hash_value(
        mut self,
        value: impl ToString,
    ) -> Self {
        assert!(matches!(self, CSSToken::Hash(_, _)));

        if let CSSToken::Hash(ref mut current_value, _) = self {
            *current_value = value.to_string();
        }

        self
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl Eq for CSSToken {}
