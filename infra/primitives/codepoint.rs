/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// ---- //
// Type //
// ---- //

/// Un point de code est un point de code Unicode et est représenté par
/// "U+" suivi de quatre à six chiffres hexadécimaux supérieurs ASCII,
/// compris entre U+0000 et U+10FFFF, inclus. La valeur d'un point de code
/// est son numéro sous-jacent.
///
/// Un point de code peut être suivi de son nom, de sa forme rendue entre
/// parenthèses lorsqu'il n'est pas U+0028 ou U+0029, ou des deux. Les
/// documents utilisant la norme Infra sont encouragés à suivre les points
/// de code par leur nom lorsqu'ils ne peuvent pas être rendus ou qu'ils
/// sont U+0028 ou U+0029 ; sinon, il faut les suivre par leur forme rendue
/// entre parenthèses, pour des raisons de lisibilité.
pub type CodePoint = char;

// --------- //
// Interface //
// --------- //

pub trait CodePointInterface: Copy {
    /// Un point de code ASCII est un point de code situé dans la plage
    /// U+0000 NULL à U+007F DELETE, inclusivement.
    fn is_ascii_code_point(self) -> bool;

    // est U+0009 TAB, U+000A LF, ou U+000D CR.
    fn is_ascii_tab_or_newline(self) -> bool;

    // Un C0 control est un point de code dans la gamme U+0000 NULL to
    // U+001F INFORMATION SEPARATOR ONE, inclusive.
    fn is_c0_control(self) -> bool;

    // A C0 control or space is a C0 control or U+0020 SPACE.
    fn is_c0_control_or_space(self) -> bool;

    /// Un non-caractère est un point de code qui se trouve dans
    /// l'intervalle des caractères. U+FDD0 à U+FDEF, inclus,
    /// ou U+FFFE, U+FFFF, U+1FFFE, U+1FFFF, U+2FFFE, U+2FFFF, U+3FFFE,
    /// U+3FFFF, U+4FFFE, U+4FFFF, U+5FFFE, U+5FFFF, U+6FFFE, U+6FFFF,
    /// U+7FFFE, U+7FFFF, U+8FFFE, U+8FFFF, U+9FFFE, U+9FFFF, U+AFFFE,
    /// U+AFFFF, U+BFFFE, U+BFFFF, U+CFFFE, U+CFFFF, U+DFFFE, U+DFFFF,
    /// U+EFFFE, U+EFFFF, U+FFFFE, U+FFFFF, U+10FFFE, ou U+10FFFF.
    fn is_noncharacter(self) -> bool;

    /// Une valeur scalaire est un point de code qui n'est pas un
    /// substitut.
    fn is_scalar_value(self) -> bool {
        !self.is_surrogate()
    }

    /// Un substitut est un point de code qui se trouve dans la plage
    /// U+D800 à U+DFFF, inclus.
    fn is_surrogate(self) -> bool;
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl CodePointInterface for CodePoint {
    fn is_ascii_code_point(self) -> bool {
        matches!(self, '\0'..='\u{007F}')
    }

    fn is_ascii_tab_or_newline(self) -> bool {
        matches!(self, '\t' | '\n' | '\r')
    }

    fn is_c0_control(self) -> bool {
        matches!(self, '\0'..='\u{001F}')
    }

    fn is_c0_control_or_space(self) -> bool {
        self.is_c0_control() || self == ' '
    }

    fn is_noncharacter(self) -> bool {
        matches!(self,
            | '\u{FDD0}'..='\u{FDEF}'
            | '\u{FFFE}'..='\u{FFFF}'
            | '\u{1_FFFE}'..='\u{1_FFFF}'
            | '\u{2_FFFE}'..='\u{2_FFFF}'
            | '\u{3_FFFE}'..='\u{3_FFFF}'
            | '\u{4_FFFE}'..='\u{4_FFFF}'
            | '\u{5_FFFE}'..='\u{5_FFFF}'
            | '\u{6_FFFE}'..='\u{6_FFFF}'
            | '\u{7_FFFE}'..='\u{7_FFFF}'
            | '\u{8_FFFE}'..='\u{8_FFFF}'
            | '\u{9_FFFE}'..='\u{9_FFFF}'
            | '\u{A_FFFE}'..='\u{A_FFFF}'
            | '\u{B_FFFE}'..='\u{B_FFFF}'
            | '\u{C_FFFE}'..='\u{C_FFFF}'
            | '\u{D_FFFE}'..='\u{D_FFFF}'
            | '\u{E_FFFE}'..='\u{E_FFFF}'
            | '\u{F_FFFE}'..='\u{F_FFFF}'
            | '\u{10_FFFE}'..='\u{10_FFFF}')
    }

    fn is_surrogate(self) -> bool {
        matches!(self, '\u{D_8000}'..='\u{D_FFFF}')
    }
}
