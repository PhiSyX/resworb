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

#[derive(Debug)]
enum State {
    /// 13.2.5.1 Data state
    Data,

    /// 13.2.5.6 Tag open state
    TagOpen,

    /// 13.2.5.7 End tag open state
    EndTagOpen,

    /// 13.2.5.8 Tag name state
    TagName,

    /// 13.2.5.32 Before attribute name state
    BeforeAttributeName,

    /// 13.2.5.40 Self-closing start tag state
    SelfClosingStartTag,

    /// 13.2.5.41 Bogus comment state
    BogusComment,

    /// 13.2.5.42 Markup declaration open state
    MarkupDeclarationOpen,

    /// 13.2.5.72 Character reference state
    CharacterReference,
}

enum StateIterator {
    Continue,
    Break,
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

    pub fn next_token(&mut self) -> Option<HTMLToken> {
        self.next()
    }

    fn pop_token(&mut self) -> Option<HTMLToken> {
        self.list.pop_front()
    }

    fn reconsume(&mut self, state: State) {
        self.stream.rollback();
        self.state.current = state;
    }

    // fn reset(&mut self) {
    // self.token = None;
    // self.state = HTMLState::default();
    // }
}

// ---------------------- //
// Implémentation | State //
// ---------------------- //

impl<C> Tokenizer<C>
where
    C: Iterator<Item = char>,
{
    fn handle_data_state(&mut self) -> StateIterator {
        match self.stream.next_input_char() {
            // U+0026 AMPERSAND (&)
            //
            // Définir l'état de retour à l'état de données. Passer à
            // l'état de référence de caractère.
            | Some('&') => {
                self.state.returns = Some(State::Data);
                self.state.current = State::CharacterReference;
                StateIterator::Continue
            }

            // U+003C LESS-THAN SIGN (<)
            //
            // Passer à l'état de balise ouverte.
            | Some('<') => {
                self.state.current = State::TagOpen;
                StateIterator::Continue
            }

            // U+0000 NULL
            //
            // Il s'agit d'une erreur d'analyse de caractère NULL et
            // inattendu. Émettre le caractère d'entrée actuel comme un
            // jeton de caractère.
            | Some('\0') => {
                emit_html_error!(HTMLParserError::UnexpectedNullCharacter);
                StateIterator::Break
            }

            // EOF
            //
            // Émettre un jeton de fin de fichier.
            | None => {
                self.token = Some(HTMLToken::EOF);
                StateIterator::Break
            }

            // Anything else
            //
            // Émettre le caractère actuel comme un jeton de caractère.
            | Some(_) => {
                self.token = self.stream.current.map(HTMLToken::Character);
                StateIterator::Break
            }
        }
    }

    fn handle_tag_open_state(&mut self) -> StateIterator {
        match self.stream.next_input_char() {
            // U+0021 EXCLAMATION MARK (!)
            //
            // Passer à l'état ouvert de la déclaration de balisage.
            | Some('!') => {
                self.state.current = State::MarkupDeclarationOpen;
                StateIterator::Continue
            }

            // U+002F SOLIDUS (/)
            //
            // Passer à l'état ouvert de la balise de fin.
            | Some('/') => {
                self.state.current = State::EndTagOpen;
                StateIterator::Continue
            }

            // ASCII alpha
            //
            // Créer un nouveau jeton de balise de départ, définir son nom
            // de balise à la chaîne vide. Reprendre dans l'état de nom de
            // balise.
            | Some(ch) if ch.is_ascii_alphabetic() => {
                self.token = Some(HTMLToken::new_start_tag(String::new()));
                self.reconsume(State::TagName);
                StateIterator::Continue
            }

            // U+003F QUESTION MARK (?)
            //
            // Il s'agit d'une erreur d'analyse
            // unexpected-question-mark-instead-of-tag-name. Créer un jeton
            // de commentaire dont les données sont une chaîne vide.
            // Reprendre dans l'état de faux commentaire.
            | Some('?') => {
                emit_html_error!(
                            HTMLParserError::UnexpectedQuestionMarkInsteadOfTagName
                        );

                self.token = Some(HTMLToken::new_comment(String::new()));

                self.reconsume(State::BogusComment);
                StateIterator::Continue
            }

            // EOF
            //
            // Ceci est une erreur d'analyse eof-before-tag-name. Émettre
            // un jeton de caractère U+003C LESS-THAN SIGN et un jeton de
            // fin de fichier.
            | None => {
                emit_html_error!(HTMLParserError::EofBeforeTagName);

                self.list.push_front(HTMLToken::Character('<'));
                self.token = Some(HTMLToken::EOF);

                StateIterator::Break
            }

            // Anything else
            //
            // Il s'agit d'une erreur d'analyse de type
            // invalid-first-character-of-tag-name. Émettre un jeton de
            // caractère U+003C LESS-THAN SIGN. Reprendre dans l'état de
            // données.
            | Some(_) => {
                emit_html_error!(
                    HTMLParserError::InvalidFirstCharacterOfTagName
                );
                self.list.push_front(HTMLToken::Character('<'));
                self.reconsume(State::Data);
                StateIterator::Continue
            }
        }
    }

    fn handle_end_tag_open_state(&mut self) -> StateIterator {
        match self.stream.next_input_char() {
            // ASCII alpha
            //
            // Créer un nouveau jeton de balise de fin, définir son nom de
            // balise à la chaîne vide. Reprendre l'état de nom de balise.
            | Some(ch) if ch.is_ascii_alphabetic() => {
                self.token = Some(HTMLToken::new_end_tag(String::new()));
                self.reconsume(State::TagName);
                StateIterator::Continue
            }

            // U+003E GREATER-THAN SIGN (>)
            //
            // Il s'agit d'une erreur d'analyse missing-end-tag-name.
            // Passer à l'état de données.
            | Some('>') => {
                emit_html_error!(HTMLParserError::MissingEndTagName);
                self.state.current = State::Data;
                StateIterator::Continue
            }

            // EOF
            //
            // Ceci est une erreur d'analyse eof-before-tag-name. Émettre
            // un jeton de caractère U+003C LESS-THAN SIGN, un jeton de
            // caractère U+002F SOLIDUS et un jeton de fin de fichier.
            | None => {
                emit_html_error!(HTMLParserError::EofBeforeTagName);

                self.list.push_front(HTMLToken::Character('<'));
                self.list.push_front(HTMLToken::Character('/'));

                StateIterator::Break
            }

            // Anything else
            //
            // Il s'agit d'une erreur de parse
            // invalid-first-character-of-tag-name. Créer un jeton de
            // commentaire dont les données sont la chaîne vide. Reprendre
            // l'état de faux commentaire.
            | Some(_) => {
                emit_html_error!(
                    HTMLParserError::InvalidFirstCharacterOfTagName
                );

                self.token = Some(HTMLToken::new_comment(String::new()));
                self.reconsume(State::BogusComment);

                StateIterator::Continue
            }
        }
    }

    fn handle_tag_name_state(&mut self) -> StateIterator {
        match self.stream.next_input_char() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Si le jeton de balise de fin actuel est un jeton de balise
            // de fin approprié, passez à l'état before du nom de
            // l'attribut. Sinon, traitez-le comme indiqué dans l'entrée
            // "Anything else" ci-dessous.
            | Some(ch) if ch.is_ascii_whitespace() && ch != '\r' => {
                self.state.current = State::BeforeAttributeName;
                StateIterator::Continue
            }

            // U+002F SOLIDUS (/)
            //
            // Si le jeton de fin actuel est un jeton de fin approprié, il
            // faut passer à l'état de balise de début à fermeture
            // automatique. Sinon, traitez-le comme dans l'entrée
            // "Anything else" ci-dessous.
            | Some('/') => {
                self.state.current = State::SelfClosingStartTag;
                StateIterator::Continue
            }

            // U+003E GREATER-THAN SIGN (>)
            //
            // Passer à l'état de données. Émettre le jeton de balise
            // actuel.
            | Some('>') => {
                self.state.current = State::Data;
                StateIterator::Break
            }

            // ASCII upper alpha
            //
            // Ajoute la version en minuscules du caractère d'entrée actuel
            // (ajouter 0x0020 au point de code du caractère) au nom de
            // balise du jeton de balise actuel.
            | Some(ch) if ch.is_ascii_uppercase() => {
                if let Some(ref mut tag) = self.token {
                    tag.append_character(ch.to_ascii_lowercase());
                }
                StateIterator::Continue
            }

            // U+0000 NULL
            //
            // Il s'agit d'une erreur d'analyse de caractère NULL et
            // inattendu. Ajouter un caractère U+FFFD REPLACEMENT
            // CHARACTER au nom de balise du jeton de balise actuel.
            | Some('\0') => {
                emit_html_error!(HTMLParserError::UnexpectedNullCharacter);

                if let Some(ref mut tag) = self.token {
                    tag.append_character(char::REPLACEMENT_CHARACTER);
                }

                StateIterator::Continue
            }

            // EOF
            //
            // C'est une erreur d'analyse eof-in-tag. Émettre un jeton de
            // fin de fichier.
            | None => {
                emit_html_error!(HTMLParserError::EofInTag);
                self.token = Some(HTMLToken::EOF);
                StateIterator::Break
            }

            // Anything else
            //
            // Ajoute le caractère d'entrée actuel au nom de balise du
            // jeton de balise actuel.
            | Some(_) => {
                if let Some(ref mut tag) = self.token {
                    let ch =
                        self.stream.current.expect("Le caractère courant");
                    tag.append_character(ch);
                }
                StateIterator::Continue
            }
        }
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
        if !self.list.is_empty() {
            return self.pop_token();
        }

        loop {
            let state = match self.state.current {
                | State::Data => self.handle_data_state(),
                | State::TagOpen => self.handle_tag_open_state(),
                | State::EndTagOpen => self.handle_end_tag_open_state(),
                | State::TagName => self.handle_tag_name_state(),
                | _ => return None,
            };

            match state {
                | StateIterator::Continue => continue,
                | StateIterator::Break => break,
            }
        }

        self.current_token()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn get_tokenizer_html(
        input: &'static str,
    ) -> HTMLTokenizer<impl Iterator<Item = char>> {
        let stream = InputStreamPreprocessor::new(input.chars());
        HTMLTokenizer::new(stream)
    }

    #[test]
    fn test_simple_div() {
        let mut html_tok =
            get_tokenizer_html(include_str!("crashtests/simple_tag.html"));

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::new_start_tag("div".into()))
        );

        assert_eq!(html_tok.next_token(), Some(HTMLToken::Character('H')));
        assert_eq!(html_tok.next_token(), Some(HTMLToken::Character('e')));
        assert_eq!(html_tok.next_token(), Some(HTMLToken::Character('l')));
        assert_eq!(html_tok.next_token(), Some(HTMLToken::Character('l')));
        assert_eq!(html_tok.next_token(), Some(HTMLToken::Character('o')));
        assert_eq!(html_tok.next_token(), Some(HTMLToken::Character(' ')));
        assert_eq!(html_tok.next_token(), Some(HTMLToken::Character('W')));
        assert_eq!(html_tok.next_token(), Some(HTMLToken::Character('o')));
        assert_eq!(html_tok.next_token(), Some(HTMLToken::Character('r')));
        assert_eq!(html_tok.next_token(), Some(HTMLToken::Character('l')));
        assert_eq!(html_tok.next_token(), Some(HTMLToken::Character('d')));

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::new_end_tag("div".into()))
        );

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::Character('\n'))
        );
        assert_eq!(html_tok.next_token(), Some(HTMLToken::EOF));
    }
}
