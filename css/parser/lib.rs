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

/// 8 Defining Grammars for Rules and Other Values
mod grammars;
mod style_blocks_content;

use infra::primitive::codepoint::CodePointIterator;
use parser::StreamIteratorInterface;

use self::{
    at_rule::CSSAtRule,
    component_value::CSSComponentValue,
    declaration::{CSSDeclaration, CSSDeclarationList},
    function::CSSFunction,
    grammars::{CSSRule, CSSRuleList},
    qualified_rule::CSSQualifiedRule,
    simple_block::CSSSimpleBlock,
    style_blocks_content::{CSSStyleBlock, CSSStyleBlocksContents},
    tokenization::{CSSTokenStream, CSSTokenVariant, CSSTokenizer},
};
use crate::tokenization::CSSToken;

// --------- //
// Structure //
// --------- //

/// 5. Parsing
pub struct CSSParser {
    tokens: CSSTokenStream,
    toplevel_flag: bool,
}

// -------------- //
// Implémentation //
// -------------- //

impl CSSParser {
    pub fn new<C>(input: C) -> Self
    where
        C: CodePointIterator,
    {
        let tokenizer = CSSTokenizer::new(input);
        let tokens = CSSTokenStream::new(tokenizer.stream());
        Self {
            tokens,
            toplevel_flag: Default::default(),
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_iter<Iter>(input: Iter) -> Self
    where
        Iter: Iterator<Item = CSSTokenVariant>,
    {
        let tokens = CSSTokenStream::from_iter(input);
        Self {
            tokens,
            toplevel_flag: Default::default(),
        }
    }
}

impl CSSParser {
    pub fn consume_next_input_token(&mut self) -> CSSTokenVariant {
        self.tokens
            .consume_next_input()
            .expect("Il y a une c*ui**e dans le pâté?")
    }

    pub fn next_input_token(&mut self) -> CSSTokenVariant {
        self.tokens
            .next_input()
            .expect("Il y a une c*ui**e dans le pâté?")
    }

    pub(crate) fn current_input_token(&mut self) -> &CSSTokenVariant {
        self.tokens
            .current_input()
            .expect("Il y a une c*ui**e dans le pâté?")
    }
}

impl CSSParser {
    fn consume_at_rule(&mut self) -> CSSAtRule {
        let current_variant = self.consume_next_input_token();
        assert!(current_variant.is_at_keyword());

        let current_token = current_variant.token_unchecked();

        let mut at_rule = CSSAtRule::default().with_name(current_token);

        loop {
            match self.consume_next_input_token() {
                // <semicolon-token>
                //
                // Retourner la règle.
                | variant if variant.is_semicolon() => break,

                // <EOF-token>
                //
                // Il s'agit d'une erreur de syntaxe. Retourner la règle.
                | variant if variant.is_eof() => {
                    // TODO(css): gérer les erreurs.
                    break;
                }

                // <{-token>
                //
                // Consommer le bloc simple à partir de l'entrée et
                // assigner la valeur de retour à la règle. Retourner la
                // règle.
                | variant if variant.is_left_curly_bracket() => {
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
                    if let Some(component_value) =
                        self.consume_component_value()
                    {
                        at_rule.append(component_value);
                    }
                }
            }
        }

        at_rule
    }

    fn consume_component_value(&mut self) -> Option<CSSComponentValue> {
        match self.consume_next_input_token() {
            // <{-token>
            // <[-token>
            // <(-token>
            //
            // Consommer le bloc simple et le retourner.
            | variant
                if variant.is_left_curly_bracket()
                    || variant.is_left_square_bracket()
                    || variant.is_left_parenthesis() =>
            {
                Some(self.consume_simple_block().into())
            }

            // <function-token>
            //
            // Consommer une fonction et la retourner.
            | variant if variant.is_function() => {
                let name = variant.function_name();
                Some(self.consume_function(name).into())
            }

            // Anything else
            //
            // Retourner le jeton d'entrée actuel.
            | current_token => current_token.component_value(),
        }
    }

    fn consume_declaration(&mut self) -> Option<CSSDeclaration> {
        self.consume_next_input_token();

        let current_variant = self.current_input_token();
        let current_token = current_variant.token_unchecked();

        let mut declaration =
            CSSDeclaration::default().with_name(current_token);

        self.tokens.advance_as_long_as_possible(
            |token| token.is_whitespace(),
            None,
        );

        // Si le prochain élément d'entrée n'est pas un <colon-token>,
        // il s'agit d'une erreur d'analyse. Ne rien retourner.
        if !self.next_input_token().is_colon() {
            // TODO(css): gérer les erreurs.
            return None;
        }

        self.consume_next_input_token();

        self.tokens.advance_as_long_as_possible(
            |token| token.is_whitespace(),
            None,
        );

        while !self.next_input_token().is_eof() {
            if let Some(component_value) = self.consume_component_value() {
                declaration.append(component_value);
            }
        }

        let last_2_tokens: Vec<_> = declaration.last_n_tokens(2).collect();
        let cond_1 = last_2_tokens
            .get(0)
            .filter(|token| CSSToken::Delim('!').eq(**token));

        let cond_2 = last_2_tokens.get(1).filter(|token| {
            if let CSSToken::Ident(name) = token {
                name.eq_ignore_ascii_case("important")
            } else {
                false
            }
        });

        if cond_1.is_some() && cond_2.is_some() {
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
                | variant if variant.is_right_parenthesis() => break,

                // <EOF-token>
                //
                // Il s'agit d'un erreur d'analyse. Retourner la fonction.
                // TODO(css): gérer les erreurs.
                | variant if variant.is_eof() => break,

                // Anything else
                //
                // Re-consommer le jeton d'entrée actuel. Consommer la
                // valeur de composant et l'ajouter à la
                // liste de valeurs de composants
                // de la fonction.
                | _ => {
                    self.tokens.reconsume_current_input();
                    if let Some(component_value) =
                        self.consume_component_value()
                    {
                        function.append(component_value);
                    }
                }
            }
        }
        function
    }

    fn consume_list_of_declarations(&mut self) -> CSSDeclarationList {
        let mut list_of_declarations = CSSDeclarationList::default();
        loop {
            match self.consume_next_input_token() {
                // <whitespace-token>
                // <semicolon-token>
                //
                // Ne rien faire.
                | variant
                    if variant.is_whitespace()
                        || variant.is_semicolon() =>
                {
                    continue;
                }

                // <EOF-token>
                //
                // Retourner la liste de déclarations.
                | variant if variant.is_eof() => break,

                // <at-keyword-token>
                //
                // Re-consommer le jeton d'entrée actuel. Consommer une
                // règle at-rule. Ajouter la règle à la liste de
                // déclarations.
                | variant if variant.is_at_keyword() => {
                    self.tokens.reconsume_current_input();
                    let at_rule = self.consume_at_rule();
                    let rule: CSSRule = at_rule.into();
                    let declaration = rule.into();
                    list_of_declarations.push(declaration);
                }

                // <ident-token>
                //
                // Initialiser une liste temporaire initialement remplie
                // avec le jeton d'entrée actuel.
                // Tant que le prochain jeton n'est pas un
                // <semicolon-token>, ou un <EOF-token>,
                // consommer une valeur de composant et l'ajouter à la
                // liste temporaire. Consommer une déclaration à partir de
                // la liste temporaire. Si quelque chose est retourné,
                // l'ajouter à la liste de déclarations.
                | variant if variant.is_ident() => {
                    let current_variant = self.current_input_token();
                    let current_token =
                        current_variant.token_unchecked().clone();

                    let mut temporary_list: Vec<CSSTokenVariant> =
                        vec![current_token.into()];

                    while !(self.next_input_token().is_semicolon()
                        || self.next_input_token().is_eof())
                    {
                        if let Some(component_value) =
                            self.consume_component_value()
                        {
                            temporary_list.push(component_value.into());
                        }
                    }

                    let mut stream =
                        CSSParser::from_iter(temporary_list.into_iter());
                    if let Some(declaration) = stream.consume_declaration()
                    {
                        list_of_declarations.push(declaration.into());
                    }
                }

                // Anything else
                //
                // Il s'agit d'une erreur d'analyse. Re-consommer le jeton
                // d'entrée actuel. Tant que le prochain token d'entrée est
                // autre chose qu'un <semicolon-token> ou <EOF-token>, nous
                // devons consommer une valeur de composant et jeter la
                // valeur retournée.
                // TODO(css): gérer les erreurs.
                | _ => {
                    self.tokens.reconsume_current_input();
                    while !(self.next_input_token().is_semicolon()
                        || self.next_input_token().is_eof())
                    {
                        self.consume_component_value();
                    }
                }
            }
        }

        list_of_declarations
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
                | variant if variant.is_whitespace() => continue,

                // <EOF-token>
                //
                // Retourner la liste des règles.
                | variant if variant.is_eof() => break,

                // <CDO-token>
                // <CDC-token>
                //
                // Si le drapeau top-level est défini, ne rien faire.
                | variant
                    if (variant.is_cdo() || variant.is_cdt())
                        && self.toplevel_flag =>
                {
                    continue
                }

                // <CDO-token>
                // <CDC-token>
                //
                // Re-consommer le jeton courant. Consommer une
                // règle qualifiée. Si un élément est retourné, il est
                // ajouté à la liste des règles.
                | variant
                    if (variant.is_cdo() || variant.is_cdt())
                        && !self.toplevel_flag =>
                {
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
                | variant if variant.is_at_keyword() => {
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
                | variant if variant.is_eof() => {}

                // <{-token>
                //
                // Consommer un bloc simple et l'assigner à la règle
                // qualifiée. Retourner la règle qualifiée.
                | variant if variant.is_left_curly_bracket() => {
                    let block = self.consume_simple_block();
                    qualified_rule.set_block(block);
                    break;
                }

                // simple block with an associated token of <{-token>
                //
                // Assigner le bloc au bloc de la règle qualifiée.
                // Retourner la règle qualifiée.
                | variant
                    if variant.is_simple_block_with(
                        CSSToken::LeftCurlyBracket,
                    ) =>
                {
                    qualified_rule.set_block(
                        variant
                            .component_value_unchecked()
                            .simple_block_unchecked()
                            .to_owned(),
                    );
                }

                // Anything else
                //
                // Re-consommer le jeton courant. Consommer une valeur de
                // composant. Ajouter la valeur retournée au prélude de
                // l'at-rule.
                | _ => {
                    self.tokens.reconsume_current_input();
                    if let Some(component_value) =
                        self.consume_component_value()
                    {
                        qualified_rule.append(component_value);
                    }
                }
            }
        }

        qualified_rule.into()
    }

    fn consume_simple_block(&mut self) -> CSSSimpleBlock {
        let current_variant = self.current_input_token();
        let current_token = current_variant.token_unchecked();
        let ending_token = current_token.mirror();

        let mut simple_block =
            CSSSimpleBlock::new(current_token.to_owned());

        loop {
            match self.consume_next_input_token() {
                // ending token
                //
                // Retourner le bloc.
                | variant if variant.is_mirror(&ending_token) => {
                    break;
                }

                // <EOF-token>
                //
                // Il s'agit d'une erreur d'analyse. Retourner le bloc
                // TODO(css): gérer les erreurs.
                | variant if variant.is_eof() => break,

                // Anything else
                //
                // Re-consommer le jeton courant. Consommer une valeur de
                // composant et l'ajouter à la valeur du bloc.
                | _ => {
                    self.tokens.reconsume_current_input();
                    if let Some(component_value) =
                        self.consume_component_value()
                    {
                        simple_block.append(component_value);
                    }
                }
            }
        }

        simple_block
    }

    fn consume_style_blocks_contents(&mut self) -> CSSStyleBlocksContents {
        let mut decls = CSSStyleBlocksContents::default();
        let mut rules = CSSStyleBlocksContents::default();

        loop {
            match self.consume_next_input_token() {
                // <whitespace-token>
                // <semicolon-token>
                //
                // Ne rien faire.
                | variant
                    if variant.is_whitespace()
                        || variant.is_semicolon() =>
                {
                    continue
                }

                // <EOF-token>
                //
                // Étendre les déclarations avec des règles, puis retourner
                // les déclarations.
                | variant if variant.is_eof() => {
                    decls.extend(rules);
                    break;
                }

                // <at-keyword-token>
                //
                // Re-consommer le jeton courant. Consommer une règle
                // at-rule, et l'ajouter à la liste des règles.
                | variant if variant.is_at_keyword() => {
                    self.tokens.reconsume_current_input();
                    let at_rule = self.consume_at_rule();
                    let rule: CSSRule = at_rule.into();
                    rules.push(rule.into());
                }

                // <ident-token>
                //
                // Initialiser une liste temporaire initialement remplie
                // avec le jeton d'entrée actuel.
                // Tant que le prochain jeton n'est pas un
                // <semicolon-token>, ou un <EOF-token>,
                // consommer une valeur de composant et l'ajouter à la
                // liste temporaire. Consommer une déclaration à partir de
                // la liste temporaire. Si quelque chose est retourné,
                // l'ajouter à decls.
                | variant if variant.is_ident() => {
                    let current_variant = self.current_input_token();
                    let current_token =
                        current_variant.token_unchecked().clone();

                    let mut temporary_list: Vec<CSSTokenVariant> =
                        vec![current_token.into()];

                    while !(self.next_input_token().is_semicolon()
                        || self.next_input_token().is_eof())
                    {
                        if let Some(component_value) =
                            self.consume_component_value()
                        {
                            temporary_list.push(component_value.into());
                        }
                    }

                    let mut stream =
                        CSSParser::from_iter(temporary_list.into_iter());
                    if let Some(decl) = stream.consume_declaration() {
                        decls.push(decl.into());
                    }
                }

                // <delim-token> with a value of "&" (U+0026 AMPERSAND)
                //
                // Re-consommer le jeton courant. Consommer une règle
                // qualifiée. Si un élément est retourné, l'ajouter aux
                // règles.
                | variant if variant.is_delimiter_with('&') => {
                    self.tokens.reconsume_current_input();
                    if let Some(qualified_rule) =
                        self.consume_qualified_rule()
                    {
                        let rule: CSSRule = qualified_rule.into();
                        let style_block: CSSStyleBlock = rule.into();
                        rules.push(style_block);
                    }
                }

                // Anything else
                //
                // Il s'agit d'une erreur d'analyse. Re-consommer le jeton
                // actuel. Tant que le prochain jeton n'est pas un
                // <semicolon-token>, ou un <EOF-token>, consommer une
                // valeur de composant et jeter la valeur retournée.
                // TODO(css): gérer les erreurs.
                | _ => {
                    self.tokens.reconsume_current_input();
                    while !(self.next_input_token().is_semicolon()
                        || self.next_input_token().is_eof())
                    {
                        self.consume_component_value();
                    }
                }
            }
        }

        decls
    }
}
