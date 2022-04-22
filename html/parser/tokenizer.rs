/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::VecDeque;

use parser::preprocessor::InputStreamPreprocessor;

use super::{error::HTMLParserError, token::HTMLToken};
use crate::emit_html_error;

// ---- //
// Type //
// ---- //

pub(crate) type Tokenizer<C> = HTMLTokenizer<C>;

// --------- //
// Structure //
// --------- //

pub struct HTMLTokenizer<Chars>
where
    Chars: Iterator<Item = char>,
{
    stream: InputStreamPreprocessor<Chars, Chars::Item>,
    token: Option<HTMLToken>,
    state: HTMLState,
    list: VecDeque<HTMLToken>,
}

pub struct HTMLState {
    current: State,
    returns: Option<State>,
}

// ----------- //
// Énumération //
// ----------- //

enum State {
    Data,
    TagOpen,
    CharacterReference,
}

// -------------- //
// Implémentation //
// -------------- //

impl<C> Tokenizer<C>
where
    C: Iterator<Item = char>,
{
    pub fn new(iter: C) -> Self {
        let stream = InputStreamPreprocessor::new(iter);
        Self {
            stream,
            token: None,
            state: HTMLState::default(),
            list: VecDeque::default(),
        }
    }
}

impl<C> Tokenizer<C>
where
    C: Iterator<Item = char>,
{
    pub fn current_token(&mut self) -> Option<HTMLToken> {
        if let Some(token) = self.token.clone() {
            self.list.push_back(token);
        }
        self.pop_token()
    }

    pub fn pop_token(&mut self) -> Option<HTMLToken> {
        self.list.pop_front()
    }

    pub fn next_token(&mut self) -> Option<HTMLToken> {
        self.next()
    }

    pub fn reset(&mut self) {
        self.token = None;
        self.state = HTMLState::default();
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl<C> Iterator for Tokenizer<C>
where
    C: Iterator<Item = char>,
{
    type Item = HTMLToken;

    fn next(&mut self) -> Option<Self::Item> {
        match &self.state.current {
            | State::Data => {
                // Consomme le prochain caractère du flux.
                match self.stream.next_input_char() {
                    // U+0026 AMPERSAND (&)
                    //
                    // Définir l'état de retour à l'état de données. Passer
                    // à l'état de référence de caractère.
                    | Some('&') => {
                        self.state.current = State::CharacterReference;
                        self.state.returns = Some(State::Data);
                    }

                    // U+003C LESS-THAN SIGN (<)
                    //
                    // Passez à l'état de balise ouverte.
                    | Some('<') => {
                        self.state.current = State::TagOpen;
                    }

                    // U+0000 NULL
                    //
                    // Il s'agit d'une erreur d'analyse de caractère nul et
                    // inattendu. Émettre le caractère d'entrée actuel
                    // comme un jeton de caractère.
                    | Some('\0') => {
                        emit_html_error!(
                            HTMLParserError::UnexpectedNullCharacter
                        );
                        return self.current_token();
                    }

                    // EOF
                    //
                    // Émettre un jeton de fin de fichier.
                    | None => {
                        self.token = Some(HTMLToken::EOF);
                        return self.current_token();
                    }

                    // Anything else
                    //
                    // Émet le caractère actuel comme un jeton de
                    // caractère.
                    | Some(_) => {
                        self.token =
                            self.stream.current.map(HTMLToken::Character);
                        return self.current_token();
                    }
                }

                None
            }
            | _ => None,
        }
    }
}

// -------------- //
// Implémentation // -> Default
// -------------- //

impl Default for HTMLState {
    fn default() -> Self {
        Self {
            current: State::Data,
            returns: None,
        }
    }
}
