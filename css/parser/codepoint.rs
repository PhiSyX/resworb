/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use infra::primitive::codepoint::CodePoint;

// --------- //
// Interface //
// --------- //

pub trait CSSCodePoint: Copy {
    /// Chiffre
    ///
    /// Un point de code compris entre U+0030 DIGIT ZERO (0) et U+0039
    /// DIGIT NINE (9) inclus.
    fn is_css_digit(self) -> bool;

    /// Espace blanc.
    ///
    /// Un saut de ligne, une TABULATION DE CARACTÈRES U+0009 ou un ESPACE
    /// U+0020.
    fn is_css_whitespace(self) -> bool;

    /// ident code point
    ///
    /// Un point de code ident de début, un chiffre ou U+002D HYPHEN-MINUS
    /// (-).
    fn is_ident_codepoint(self) -> bool;

    /// ident-start code point
    ///
    /// Une lettre, un point de code non ASCII ou U+005F LOW LINE (_).
    fn is_ident_start_codepoint(self) -> bool;

    /// Lettre
    ///
    /// Une lettre majuscule ou une lettre minuscule.
    fn is_letter(self) -> bool;

    /// Lettre minuscule
    ///
    /// Une lettre minuscule.
    fn is_lowercase_letter(self) -> bool;

    /// Lettre majuscule
    ///
    /// Une lettre majuscule.
    fn is_uppercase_letter(self) -> bool;

    /// Saut de ligne.
    ///
    /// Note: U+000D CARRIAGE RETURN et U+000C FORM FEED ne sont pas inclus
    /// dans cette définition, car ils sont convertis en U+000A LINE FEED
    /// lors du prétraitement.
    fn is_newline(self) -> bool;
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl CSSCodePoint for CodePoint {
    fn is_css_digit(self) -> bool {
        self.is_ascii_digit()
    }

    fn is_css_whitespace(self) -> bool {
        self.is_newline() || matches!(self, '\t' | ' ')
    }

    fn is_ident_codepoint(self) -> bool {
        self.is_ident_start_codepoint()
            || self.is_css_digit()
            || self == '-'
    }

    fn is_ident_start_codepoint(self) -> bool {
        self.is_letter()
    }

    fn is_letter(self) -> bool {
        self.is_lowercase_letter() || self.is_uppercase_letter()
    }

    fn is_lowercase_letter(self) -> bool {
        self.is_ascii_lowercase()
    }

    fn is_uppercase_letter(self) -> bool {
        self.is_ascii_uppercase()
    }

    fn is_newline(self) -> bool {
        self == '\n'
    }
}
