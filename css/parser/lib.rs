/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod codepoint;

/// 4. Tokenization
mod tokenization;

/// 5. Parsing
mod at_rule;
mod component_value;
mod declaration;
mod function;
mod preserved_tokens;
mod qualified_rule;
mod simple_block;

/// 5.3 Parser Entry Points
mod entrypoints;

/// 8 Defining Grammars for Rules and Other Values
mod grammars;

use std::marker::PhantomData;

use infra::primitive::codepoint::CodePointIterator;
use parser::{StreamInputInterface, StreamIteratorInterface};

use self::{
    at_rule::CSSAtRule, component_value::CSSComponentValue,
    declaration::CSSDeclaration, function::CSSFunction,
    grammars::CSSRuleList, preserved_tokens::CSSPreservedToken,
    qualified_rule::CSSQualifiedRule, simple_block::CSSSimpleBlock,
};
use crate::tokenization::{CSSToken, CSSTokenStream};

// --------- //
// Structure //
// --------- //

/// 5. Parsing
pub struct CSSParser<Token> {
    tokens: CSSTokenStream,
    toplevel_flag: bool,
    _marker: PhantomData<Token>,
}

// -------------- //
// Implémentation //
// -------------- //

impl<T> CSSParser<T>
where
    T: StreamInputInterface,
{
    pub fn new<C>(input: C) -> Self
    where
        C: CodePointIterator,
    {
        let tokens = CSSTokenStream::new(input);
        Self {
            tokens,
            toplevel_flag: Default::default(),
            _marker: Default::default(),
        }
    }
}

impl<T> CSSParser<T> {
    fn consume_at_rule(&mut self) -> CSSAtRule {
        self.consume_next_input_token();

        let mut at_rule =
            CSSAtRule::default().with_name(self.current_input_token());

        loop {
            match self.consume_next_input_token() {
                // <semicolon-token>
                //
                // Retourner la règle.
                | CSSToken::Semicolon => break,

                // <EOF-token>
                //
                // Il s'agit d'une erreur de syntaxe. Retourner la règle
                // TODO(css): gérer les erreurs.
                | CSSToken::EOF => break,

                // <{-token>
                //
                // Consommer le bloc simple à partir de l'entrée et
                // assigner la valeur de retour à la règle. Retourner la
                // règle.
                | CSSToken::LeftCurlyBracket => {
                    at_rule.set_block(self.consume_simple_block());
                    break;
                }

                // Anything else
                //
                // Re-consommer le jeton d'entrée actuel. Consommer une
                // valeur de composant. Ajouter la valeur de composant au
                // prélude de la règle.
                | _ => {
                    self.tokens.reconsume_current_input();
                    at_rule.append(self.consume_component_value());
                }
            }
        }

        at_rule
    }

    fn consume_component_value(&mut self) -> CSSComponentValue {
        self.consume_next_input_token();

        match self.current_input_token().clone() {
            // <{-token>
            // <[-token>
            // <(-token>
            //
            // Consommer le bloc simple et le retourner.
            | CSSToken::LeftCurlyBracket
            | CSSToken::LeftSquareBracket
            | CSSToken::LeftParenthesis => {
                self.consume_simple_block().into()
            }

            // <function-token>
            //
            // Consommer une fonction et la retourner.
            | CSSToken::Function(name) => {
                self.consume_function(name).into()
            }

            // Anything else
            //
            // Retourner le jeton d'entrée actuel.
            | current_token => {
                let preserved_token: CSSPreservedToken =
                    current_token.into();
                let component_value: CSSComponentValue =
                    preserved_token.into();
                component_value
            }
        }
    }

    fn consume_declaration(&mut self) -> Option<CSSDeclaration> {
        self.consume_next_input_token();

        let mut declaration = CSSDeclaration::default()
            .with_name(self.current_input_token());

        self.tokens.advance_as_long_as_possible(
            |token| *token == CSSToken::Whitespace,
            None,
        );

        if self.next_input_token() != CSSToken::Colon {
            // TODO(css): gérer les erreurs.
            return None;
        }

        self.consume_next_input_token();

        self.tokens.advance_as_long_as_possible(
            |token| *token == CSSToken::Whitespace,
            None,
        );

        while self.next_input_token() != CSSToken::EOF {
            declaration.append(self.consume_component_value());
        }

        let last_2_tokens: Vec<_> = declaration.last_n_tokens(2).collect();
        let cond_1 = CSSToken::Delim('!').eq(last_2_tokens[0]);
        let cond_2 = if let CSSToken::Ident(name) = last_2_tokens[1] {
            name.eq_ignore_ascii_case("important")
        } else {
            false
        };
        if cond_1 && cond_2 {
            declaration.remove_last_n_values(2);
            declaration.set_important_flag(true);
        }

        while let Some(CSSToken::Whitespace) = declaration.last_token() {
            declaration.remove_last_n_values(1);
        }

        Some(declaration)
    }

    fn consume_function(&mut self, name_fn: String) -> CSSFunction {
        let mut function = CSSFunction::new(name_fn);
        loop {
            match self.consume_next_input_token() {
                // <)-token>
                //
                // Retourner la fonction.
                | CSSToken::RightParenthesis => break,

                // <EOF-token>
                //
                // Il s'agit d'un erreur d'analyse. Retourner la fonction.
                // TODO(css): gérer les erreurs.
                | CSSToken::EOF => break,

                // Anything else
                //
                // Re-consommer le jeton d'entrée actuel. Consommer la
                // valeur de composant et l'ajouter à la
                // liste de valeurs de composants
                // de la fonction.
                | _ => {
                    self.tokens.reconsume_current_input();
                    function.append(self.consume_component_value());
                }
            }
        }
        function
    }

    fn consume_list_of_rules(
        &mut self,
        toplevel_flag: bool,
    ) -> CSSRuleList {
        self.toplevel_flag = toplevel_flag;

        let mut rules: CSSRuleList = Vec::new();

        loop {
            match self.consume_next_input_token() {
                // <whitespace-token>
                //
                // Ne rien faire.
                | CSSToken::Whitespace => continue,

                // <EOF-token>
                //
                // Retourner la liste des règles.
                | CSSToken::EOF => break,

                // <CDO-token>
                // <CDC-token>
                //
                // Si le drapeau top-level est défini, ne rien faire.
                | CSSToken::CDO | CSSToken::CDC if self.toplevel_flag => {
                    continue
                }

                // <CDO-token>
                // <CDC-token>
                //
                // Re-consommer le jeton courant. Consommer une
                // règle qualifiée. Si un élément est retourné, il est
                // ajouté à la liste des règles.
                | CSSToken::CDO | CSSToken::CDC if !self.toplevel_flag => {
                    self.tokens.reconsume_current_input();
                    if let Some(qualified_rule) =
                        self.consume_qualified_rule()
                    {
                        rules.push(qualified_rule.into());
                    }
                }

                // <at-keyword-token>
                //
                // Re-consommer le jeton courant. Consommer une règle
                // at-rule, et l'ajouter à la liste des règles.
                | CSSToken::AtKeyword(_) => {
                    self.tokens.reconsume_current_input();
                    let at_rule = self.consume_at_rule();
                    rules.push(at_rule.into());
                }

                // Anything else
                //
                // Re-consommer le jeton courant. Consommer une règle
                // qualifiée. Si un élément est retourné, il est ajouté
                // à la liste des règles.
                | _ => {
                    self.tokens.reconsume_current_input();
                    if let Some(qualified_rule) =
                        self.consume_qualified_rule()
                    {
                        rules.push(qualified_rule.into());
                    }
                }
            };
        }

        rules
    }

    fn consume_qualified_rule(&mut self) -> Option<CSSQualifiedRule> {
        let mut qualified_rule = CSSQualifiedRule::default();

        loop {
            match self.consume_next_input_token() {
                // <EOF-token>
                //
                // Il s'agit d'une erreur d'analyse. Ne rien retourner.
                // TODO(css): gérer les erreurs.
                | CSSToken::EOF => {}

                // <{-token>
                //
                // Consommer un bloc simple et l'assigner à la règle
                // qualifiée. Retourner la règle qualifiée.
                | CSSToken::LeftCurlyBracket => {
                    let block = self.consume_simple_block();
                    qualified_rule.set_block(block);
                    break;
                }

                // TODO(css): cas <at-keyword-token>
                //            CSSComponentValue::SimpleBlock(..)

                // Anything else
                //
                // Re-consommer le jeton courant. Consommer une valeur de
                // composant. Ajouter la valeur retournée au prélude de
                // l'at-rule.
                | _ => {
                    self.tokens.reconsume_current_input();
                    qualified_rule.append(self.consume_component_value());
                }
            }
        }

        qualified_rule.into()
    }

    fn consume_simple_block(&mut self) -> CSSSimpleBlock {
        let ending_token = self.current_input_token().mirror();
        let mut simple_block = CSSSimpleBlock::new(ending_token.clone());

        loop {
            match self.consume_next_input_token() {
                // ending token
                //
                // Retourner le bloc.
                | token if token == ending_token => {
                    break;
                }

                // <EOF-token>
                //
                // Il s'agit d'une erreur d'analyse. Retourner le bloc
                // TODO(css): gérer les erreurs.
                | CSSToken::EOF => break,

                // Anything else
                //
                // Re-consommer le jeton courant. Consommer une valeur de
                // composant et l'ajouter à la valeur du bloc.
                | _ => {
                    self.tokens.reconsume_current_input();
                    simple_block.append(self.consume_component_value());
                }
            }
        }

        simple_block
    }

    pub fn consume_next_input_token(&mut self) -> CSSToken {
        self.tokens
            .consume_next_input()
            .expect("Il y a une c*ui**e dans le pâté?")
    }

    pub fn next_input_token(&mut self) -> CSSToken {
        self.tokens
            .next_input()
            .expect("Il y a une c*ui**e dans le pâté?")
    }

    pub(crate) fn current_input_token(&mut self) -> &CSSToken {
        self.tokens
            .current_input()
            .expect("Il y a une c*ui**e dans le pâté?")
    }
}
