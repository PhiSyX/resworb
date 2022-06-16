/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::Deref;

use parser::StreamIterator;

use crate::{
    component_value::{CSSComponentValue, CSSComponentValuesList},
    grammars::CSSRuleError,
    style_blocks_content::CSSStyleBlock,
    tokenization::CSSToken,
    CSSParser,
};

// ---- //
// Type //
// ---- //

pub type CSSDeclarationList = Vec<CSSStyleBlock>;

// --------- //
// Structure //
// --------- //

/// D'un point de vue conceptuel, les déclarations sont un exemple
/// particulier d'association d'un nom de propriété ou de descripteur à une
/// valeur. Sur le plan syntaxique, une déclaration comporte un nom, une
/// valeur constituée d'une liste de valeurs de composants et un drapeau
/// important qui est initialement désactivé.
#[derive(Debug)]
#[derive(Default)]
#[derive(PartialEq, Eq)]
pub struct CSSDeclaration {
    name: String,
    value: CSSComponentValuesList,
    important_flag: bool,
}

// ----------- //
// Énumération //
// ----------- //

/// Les déclarations sont ensuite classées en déclarations de propriétés ou
/// en déclarations de descripteurs, les premières définissant les
/// propriétés CSS et apparaissant le plus souvent dans les règles
/// qualifiées et les secondes définissant les descripteurs CSS, qui
/// n'apparaissent que dans les at-rules. (Cette catégorisation
/// n'intervient pas au niveau de la syntaxe ; elle est plutôt le produit
/// de l'endroit où la déclaration apparaît, et est définie par les
/// spécifications respectives définissant la règle donnée).
///
/// TODO.
pub enum CSSDeclarationCategory {
    Property,
    Descriptor,
}

// ----------- //
// Entry Point //
// ----------- //

impl CSSParser {
    /// Analyse d'une déclaration
    pub fn declaration(&mut self) -> Result<CSSDeclaration, CSSRuleError> {
        self.tokens.advance_as_long_as_possible(
            |token| token.is_whitespace(),
            None,
        );

        let declaration = match self.next_input_token() {
            | variant if variant.is_ident() => self.consume_declaration(),
            | _ => None,
        };

        declaration.ok_or(CSSRuleError::SyntaxError)
    }

    /// Analyse une liste de déclarations
    pub fn list_of_declarations(&mut self) -> CSSDeclarationList {
        self.consume_list_of_declarations()
    }
}

// -------------- //
// Implémentation //
// -------------- //

impl CSSDeclaration {
    pub(super) fn with_name(mut self, token_name: impl ToString) -> Self {
        self.name = token_name.to_string();
        self
    }

    pub(super) fn with_values(
        mut self,
        prelude: impl IntoIterator<Item = impl TryInto<CSSComponentValue>>,
    ) -> Self {
        self.value = prelude
            .into_iter()
            .filter_map(|v| v.try_into().ok())
            .collect();
        self
    }
}

impl CSSDeclaration {
    pub(super) fn last_n_values(&self, n: usize) -> &[CSSComponentValue] {
        let size = self.value.len();
        let start = size.checked_sub(n);
        if let Some(start) = start {
            &self.value[start..]
        } else {
            &[]
        }
    }

    pub(super) fn last_n_tokens(
        &self,
        n: usize,
    ) -> impl Iterator<Item = &CSSToken> {
        self.last_n_values(n).iter().filter_map(|component_value| {
            match component_value {
                | CSSComponentValue::Preserved(preserve_token) => {
                    Some(preserve_token.deref())
                }
                | _ => None,
            }
        })
    }

    pub(super) fn last_token(&self) -> Option<&CSSToken> {
        self.last_n_tokens(1).next()
    }
}

impl CSSDeclaration {
    pub(super) fn append(&mut self, component_value: CSSComponentValue) {
        self.value.push(component_value);
    }

    pub(super) fn remove_last_n_values(&mut self, n: usize) {
        let size = self.value.len();
        let start = size.saturating_sub(n);
        self.value.drain(start..);
    }

    pub(super) fn set_important_flag(&mut self, important_flag: bool) {
        self.important_flag = important_flag;
    }
}

// ---- //
// Test //
// ---- //

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_the_str;

    #[test]
    fn test_parse_declaration() {
        let mut parser = test_the_str!(r#"color: red;"#);
        assert_eq!(
            parser.declaration(),
            Ok(CSSDeclaration::default().with_name("color").with_values(
                [
                    CSSComponentValue::Preserved(
                        CSSToken::Ident("red".into()).try_into().unwrap()
                    ),
                    CSSComponentValue::Preserved(
                        CSSToken::Semicolon.try_into().unwrap()
                    )
                ]
            ))
        );
    }

    #[test]
    fn test_parse_declaration_is_not() {
        let mut parser = test_the_str!(r#".class {}"#);
        assert_eq!(parser.declaration(), Err(CSSRuleError::SyntaxError));
    }

    #[test]
    fn test_parse_a_list_of_declarations() {
        let mut parser = test_the_str!(
            "
            color: red;
            background-color: blue;
            "
        );

        assert_eq!(
            parser.list_of_declarations(),
            [
                CSSStyleBlock::Declaration(
                    CSSDeclaration::default()
                        .with_name("color")
                        .with_values([CSSToken::Ident("red".into())])
                ),
                CSSStyleBlock::Declaration(
                    CSSDeclaration::default()
                        .with_name("background-color")
                        .with_values([CSSToken::Ident("blue".into())])
                ),
            ]
        );
    }
}
