/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{borrow::Cow, collections::VecDeque, str::FromStr};

use parser::preprocessor::InputStreamPreprocessor;

use super::{
    error::HTMLParserError,
    token::{HTMLTagAttribute, HTMLToken},
};
use crate::{emit_html_error, parser::token::HTMLTagAttributeName};

// --------- //
// Interface //
// --------- //

trait HTMLStateIteratorInterface {
    fn ignore(&self) -> ResultHTMLStateIterator {
        Ok(HTMLStateIterator::Continue)
    }

    fn and_continue(&self) -> ResultHTMLStateIterator {
        Ok(HTMLStateIterator::Continue)
    }

    fn and_continue_with_error(
        &self,
        err: &str,
    ) -> ResultHTMLStateIterator {
        let err = err.parse().unwrap();
        Err((err, HTMLStateIterator::Continue))
    }

    fn and_break(&self) -> ResultHTMLStateIterator {
        Ok(HTMLStateIterator::Break)
    }

    fn and_break_with_error(&self, err: &str) -> ResultHTMLStateIterator {
        let err = err.parse().unwrap();
        Err((err, HTMLStateIterator::Break))
    }
}

trait HTMLCharacterInterface {
    fn is_html_whitespace(&self) -> bool;
}

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
    temp: VecDeque<HTMLToken>,
}

pub struct HTMLState {
    current: State,
    returns: Option<State>,
}

// ----------- //
// Énumération //
// ----------- //

#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
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

    /// 13.2.5.33 Attribute name state
    AttributeName,

    /// 13.2.5.34 After attribute name state
    AfterAttributeName,

    /// 13.2.5.35 Before attribute value state
    BeforeAttributeValue,

    /// 13.2.5.36 Attribute value (double-quoted) state
    AttributeValueDoubleQuoted,

    /// 13.2.5.37 Attribute value (single-quoted) state
    AttributeValueSimpleQuoted,

    /// 13.2.5.38 Attribute value (unquoted) state
    AttributeValueUnquoted,

    /// 13.2.5.39 After attribute value (quoted) state
    AfterAttributeValueQuoted,

    /// 13.2.5.40 Self-closing start tag state
    SelfClosingStartTag,

    /// 13.2.5.41 Bogus comment state
    BogusComment,

    /// 13.2.5.42 Markup declaration open state
    MarkupDeclarationOpen,

    /// 13.2.5.43 Comment start state
    CommentStart,

    /// 13.2.5.44 Comment start dash state
    CommentStartDash,

    /// 13.2.5.45 Comment state
    Comment,

    /// 13.2.5.50 Comment end dash state
    CommentEndDash,

    /// 13.2.5.51 Comment end state
    CommentEnd,

    /// 13.2.5.53 DOCTYPE state
    DOCTYPE,

    /// 13.2.5.54 Before DOCTYPE name state
    BeforeDOCTYPEName,

    /// 13.2.5.55 DOCTYPE name state
    DOCTYPEName,

    /// 13.2.5.56 After DOCTYPE name state
    AfterDOCTYPEName,

    /// 13.2.5.57 After DOCTYPE public keyword state
    AfterDOCTYPEPublicKeyword,

    /// 13.2.5.58 Before DOCTYPE public identifier state
    BeforeDOCTYPEPublicIdentifier,

    /// 13.2.5.59 DOCTYPE public identifier (double-quoted) state
    DOCTYPEPublicIdentifierDoubleQuoted,

    /// 13.2.5.60 DOCTYPE public identifier (single-quoted) state
    DOCTYPEPublicIdentifierSingleQuoted,

    /// 13.2.5.61 After DOCTYPE public identifier state
    AfterDOCTYPEPublicIdentifier,

    /// 13.2.5.62 Between DOCTYPE public and system identifiers state
    BetweenDOCTYPEPublicAndSystemIdentifiers,

    /// 13.2.5.63 After DOCTYPE system keyword state
    AfterDOCTYPESystemKeyword,

    /// 13.2.5.64 Before DOCTYPE system identifier state
    BeforeDOCTYPESystemIdentifier,

    /// 13.2.5.65 DOCTYPE system identifier (double-quoted) state
    DOCTYPESystemIdentifierDoubleQuoted,

    /// 13.2.5.66 DOCTYPE system identifier (single-quoted) state
    DOCTYPESystemIdentifierSingleQuoted,

    /// 13.2.5.67 After DOCTYPE system identifier state
    AfterDOCTYPESystemIdentifier,

    /// 13.2.5.68 Bogus DOCTYPE state
    BogusDOCTYPE,

    /// 13.2.5.72 Character reference state
    CharacterReference,
}

enum HTMLStateIterator {
    Continue,
    Break,
}

type ResultHTMLStateIterator =
    Result<HTMLStateIterator, (HTMLParserError, HTMLStateIterator)>;

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
            temp: VecDeque::default(),
        }
    }
}

impl<C> Tokenizer<C>
where
    C: Iterator<Item = char>,
{
    pub fn current_token(&mut self) -> Option<HTMLToken> {
        if let Some(token) = self.token.clone() {
            self.temp.push_back(token);
        }
        self.pop_token()
    }

    pub fn next_token(&mut self) -> Option<HTMLToken> {
        self.next()
    }

    fn pop_token(&mut self) -> Option<HTMLToken> {
        self.temp.pop_front()
    }

    fn change_current_token<F: FnOnce(&mut HTMLToken)>(
        &mut self,
        callback: F,
    ) -> &mut Self {
        if let Some(ref mut token) = self.token {
            callback(token);
        }
        self
    }

    fn and_emit_current_token(&mut self) -> &mut Self {
        if let Some(token) = self.current_token() {
            self.emit_token(token);
        }
        self
    }

    fn emit_token(&mut self, token: HTMLToken) -> &mut Self {
        self.temp.push_front(token);
        self
    }

    fn set_token(&mut self, token: HTMLToken) -> &mut Self {
        self.token = Some(token);
        self
    }

    fn reconsume(&mut self, state: &str) -> &mut Self {
        self.stream.rollback();
        self.state.switch_to(state);
        self
    }

    fn switch_state_to(&mut self, state: &str) -> &mut Self {
        self.state.switch_to(state);
        self
    }

    // fn reset(&mut self) {
    // self.token = None;
    // self.state = HTMLState::default();
    // }
}

impl HTMLState {
    /// Change l'état actuel par un nouvel état.
    /// Terme `switch_to` venant de la spécification HTML "Switch to the
    /// ..."
    fn switch_to(&mut self, state: &str) -> &mut Self {
        let state = state.parse().unwrap();
        self.current = state;
        self
    }

    /// Change l'état de retour par un nouvel état.
    /// Terme `set_return` venant de spécification HTML "Set the return
    /// state to the ..."
    fn set_return(&mut self, state: &str) -> &mut Self {
        let state = state.parse().unwrap();
        self.returns = Some(state);
        self
    }
}

// ---------------------- //
// Implémentation | State //
// ---------------------- //

impl<C> Tokenizer<C>
where
    C: Iterator<Item = char>,
{
    fn handle_data_state(&mut self) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+0026 AMPERSAND (&)
            //
            // Définir l'état de retour à l'état `data`. Passer à l'état
            // `character-reference`.
            | Some('&') => self
                .state
                .set_return("data")
                .switch_to("character-reference")
                .and_continue(),

            // U+003C LESS-THAN SIGN (<)
            //
            // Passer à l'état `tag-open`.
            | Some('<') => self.state.switch_to("tag-open").and_continue(),

            // U+0000 NULL
            //
            // Il s'agit d'une erreur d'analyse de type
            // `unexpected-null-character. Émettre le caractère
            // actuel comme un jeton de caractère.
            | Some('\0') => {
                self.and_break_with_error("unexpected-null-character")
            }

            // EOF
            //
            // Émettre un jeton `end of file`.
            | None => self.set_token(HTMLToken::EOF).and_break(),

            // Anything else
            //
            // Émettre le caractère actuel comme un jeton `character`.
            | Some(ch) => {
                self.set_token(HTMLToken::Character(ch)).and_break()
            }
        }
    }

    fn handle_tag_open_state(&mut self) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+0021 EXCLAMATION MARK (!)
            //
            // Passer à l'état `markup-declaration-open`.
            | Some('!') => self
                .state
                .switch_to("markup-declaration-open")
                .and_continue(),

            // U+002F SOLIDUS (/)
            //
            // Passer à l'état `end-tag-open`.
            | Some('/') => {
                self.state.switch_to("end-tag-open").and_continue()
            }

            // ASCII alpha
            //
            // Créer un nouveau jeton `start tag`, et définir son nom
            // en une chaîne de caractères vide. Reprendre dans `tag-name`.
            | Some(ch) if ch.is_ascii_alphabetic() => self
                .set_token(HTMLToken::new_start_tag(String::new()))
                .reconsume("tag-name")
                .and_continue(),

            // U+003F QUESTION MARK (?)
            //
            // Il s'agit d'une erreur d'analyse de type
            // `unexpected-question-mark-instead-of-tag-name`. Créer un
            // jeton de `comment` dont les données sont une chaîne de
            // caractères vide. Reprendre dans l'état de `bogus-comment`.
            | Some('?') => self
                .set_token(HTMLToken::new_comment(String::new()))
                .reconsume("bogus-comment")
                .and_break_with_error(
                    "unexpected-question-mark-instead-of-tag-name",
                ),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type
            // `eof-before-tag-name`. Émettre un jeton de
            // `character` U+003C LESS-THAN SIGN et un jeton de
            // `end of file`.
            | None => self
                .emit_token(HTMLToken::Character('<'))
                .set_token(HTMLToken::EOF)
                .and_break_with_error("eof-before-tag-name"),

            // Anything else
            //
            // Il s'agit d'une erreur d'analyse de type
            // `invalid-first-character-of-tag-name`. Émettre un jeton de
            // `character` U+003C LESS-THAN SIGN. Reprendre dans l'état
            // `data`.
            | Some(_) => self
                .emit_token(HTMLToken::Character('<'))
                .reconsume("data")
                .and_continue_with_error(
                    "invalid-first-character-of-tag-name",
                ),
        }
    }

    fn handle_end_tag_open_state(&mut self) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // ASCII alpha
            //
            // Créer un nouveau jeton `end tag`, et lui définir son nom
            // comme une chaîne de caractères vide. Reprendre l'état
            // `tag-name`.
            | Some(ch) if ch.is_ascii_alphabetic() => self
                .set_token(HTMLToken::new_end_tag(String::new()))
                .reconsume("tag-name")
                .and_continue(),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Il s'agit d'une erreur d'analyse de type
            // `missing-end-tag-name`. Passer à l'état `data`.
            | Some('>') => self
                .state
                .switch_to("data")
                .and_continue_with_error("missing-end-tag-name"),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type
            // `eof-before-tag-name`. Émettre un jeton `character`
            // U+003C LESS-THAN SIGN, un jeton de `character` U+002F
            // SOLIDUS et un jeton `end of file`.
            | None => self
                .emit_token(HTMLToken::Character('<'))
                .emit_token(HTMLToken::Character('/'))
                .and_break_with_error("eof-before-tag-name"),

            // Anything else
            //
            // Il s'agit d'une erreur d'analyse de type
            // `invalid-first-character-of-tag-name`. Créer un jeton
            // `comment` dont les données sont chaîne une chaîne de
            // caractères vide. Reprendre l'état de `bogus-comment`.
            | Some(_) => self
                .set_token(HTMLToken::new_comment(String::new()))
                .reconsume("bogus-comment")
                .and_continue_with_error(
                    "invalid-first-character-of-tag-name",
                ),
        }
    }

    fn handle_tag_name_state(&mut self) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Si le jeton `end tag` actuel est un jeton `end tag`
            // approprié, passez à l'état `before-attribute-name`. Sinon,
            // traitez-le comme indiqué dans l'entrée "Anything
            // else" ci-dessous.
            | Some(ch) if ch.is_html_whitespace() => self
                .state
                .switch_to("before-attribute-name")
                .and_continue(),

            // U+002F SOLIDUS (/)
            //
            // Si le jeton `end tag` actuel est un jeton `end tag`
            // approprié, passez l'état de balise de
            // `self-closing-start-tag`. Sinon, traitez-le comme dans
            // l'entrée "Anything else" ci-dessous.
            | Some('/') => self
                .state
                .switch_to("self-closing-start-tag")
                .and_continue(),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Passer à l'état `data`. Émettre le jeton `tag` actuel.
            | Some('>') => self.state.switch_to("data").and_break(),

            // ASCII upper alpha
            //
            // Ajouter la version en minuscules du caractère actuel
            // (ajouter 0x0020 au point de code du caractère) au nom du
            // jeton `tag` actuel.
            | Some(ch) if ch.is_ascii_uppercase() => self
                .change_current_token(|tag_tok| {
                    tag_tok.append_character(ch.to_ascii_lowercase());
                })
                .and_continue(),

            // U+0000 NULL
            //
            // Il s'agit d'une erreur d'analyse de type
            // `unexpected-null-character`. Ajouter un caractère U+FFFD
            // `REPLACEMENT_CHARACTER` au nom du jeton `tag` actuel.
            | Some('\0') => self
                .change_current_token(|tag_tok| {
                    tag_tok.append_character(char::REPLACEMENT_CHARACTER);
                })
                .and_continue_with_error("unexpected-null-character"),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type ` eof-in-tag`.
            // Émettre un jeton `end of file`.
            | None => self
                .set_token(HTMLToken::EOF)
                .and_break_with_error("eof-in-tag"),

            // Anything else
            //
            // Ajouter le caractère actuel au nom du jeton `tag` actuel.
            | Some(ch) => self
                .change_current_token(|tag_tok| {
                    tag_tok.append_character(ch);
                })
                .and_continue(),
        }
    }

    fn handle_before_attribute_name_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Ignorer le caractère.
            | Some(ch) if ch.is_html_whitespace() => self.ignore(),

            // U+002F SOLIDUS (/)
            // U+003E GREATER-THAN SIGN (>)
            // EOF
            //
            // Reprendre dans l'état `after-attribute-name`.
            | Some('/' | '>') | None => {
                self.reconsume("after-attribute-name").and_continue()
            }

            // U+003D EQUALS SIGN (=)
            //
            // Il s'agit d'une erreur d'analyse de type
            // `unexpected-equals-sign-before-attribute-name`. Commencer un
            // nouvel attribut dans le jeton `tag` actuel. Définir le nom
            // de cet attribut sur le caractère actuel, et sa valeur une
            // chaîne de caractères vide. Passer à l'état `attribute-name`.
            | Some(ch @ '=') => self
                .change_current_token(|tag_tok| {
                    let mut attribute = HTMLTagAttribute::default();
                    attribute.0 = HTMLTagAttributeName::from(ch);
                    tag_tok.define_tag_attributes(attribute);
                })
                .switch_state_to("attribute-name")
                .and_break_with_error(
                    "unexpected-equals-sign-before-attribute-name",
                ),

            // Anything else
            //
            // Commencer un nouvel attribut dans le jeton `tag` actuel.
            // Le nom et la valeur de cet attribut ont pour valeur une
            // chaîne de caractères vide.
            // Reprendre l'état `attribute-name`.
            | Some(_) => self
                .change_current_token(|tag_tok| {
                    let attribute = HTMLTagAttribute::default();
                    tag_tok.define_tag_attributes(attribute);
                })
                .reconsume("attribute-name")
                .and_continue(),
        }
    }

    fn handle_attribute_name_state(&mut self) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            // U+002F SOLIDUS (/)
            // U+003E GREATER-THAN SIGN (>)
            // EOF
            //
            // Reprendre dans l'état `after-attribute-name`.
            | Some(ch) if ch.is_html_whitespace() => {
                self.reconsume("after-attribute-name").and_continue()
            }
            | None | Some('/' | '>') => {
                self.reconsume("after-attribute-name").and_continue()
            }

            // U+003D EQUALS SIGN (=)
            //
            // Passer à l'état `before-attribute-value`.
            | Some('=') => self
                .state
                .switch_to("before-attribute-value")
                .and_continue(),

            // ASCII upper alpha
            //
            // Ajouter la version en minuscules du caractère actuel
            // (ajouter 0x0020 au point de code du caractère) au nom de
            // l'attribut actuel.
            | Some(ch) if ch.is_ascii_uppercase() => self
                .change_current_token(|tag_tok| {
                    tag_tok.append_character_to_attribute_name(
                        ch.to_ascii_lowercase(),
                    );
                })
                .and_continue(),

            // U+0000 NULL
            //
            // Il s'agit d'une erreur d'analyse de type
            // `unexpected-null-character`. Ajouter un caractère U+FFFD
            // `REPLACEMENT_CHARACTER` au nom de l'attribut actuel.
            | Some('\0') => self
                .emit_token(HTMLToken::Character(
                    char::REPLACEMENT_CHARACTER,
                ))
                .and_continue_with_error("unexpected-null-character"),

            // U+0022 QUOTATION MARK (")
            // U+0027 APOSTROPHE (')
            // U+003C LESS-THAN SIGN (<)
            //
            // Il s'agit d'une erreur d'analyse de type
            // `unexpected-character-in-attribute-name`. La traiter comme
            // l'entrée "Anything else" ci-dessous.
            //
            // Anything else
            //
            // Ajouter le caractère actuel au nom de l'attribut
            // actuel.
            | Some(ch) => {
                self.change_current_token(|tag_tok| {
                    tag_tok.append_character_to_attribute_name(ch);
                });

                if matches!(ch, '"' | '\'' | '<') {
                    self.and_continue_with_error(
                        "unexpected-character-in-attribute-name",
                    )
                } else {
                    self.and_continue()
                }
            }
        }
    }

    fn handle_after_attribute_name_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Ignorer le caractère.
            | Some(ch) if ch.is_html_whitespace() => self.ignore(),

            // U+002F SOLIDUS (/)
            //
            // Passer à l'état `self-closing-start-tag`.
            | Some('/') => self
                .state
                .switch_to("self-closing-start-tag")
                .and_continue(),

            // U+003D EQUALS SIGN (=)
            //
            // Passer à l'état `before-attribute-value`.
            | Some('=') => self
                .state
                .switch_to("before-attribute-value")
                .and_continue(),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Passer à l'état `data`. Émettre le jeton actuel.
            | Some('>') => self.state.switch_to("data").and_break(),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-tag`.
            // Émettre un jeton `end of file`.
            | None => self
                .set_token(HTMLToken::EOF)
                .and_break_with_error("eof-in-tag"),

            // Anything else
            //
            // Commencer un nouvel attribut dans le jeton `tag` actuel.
            // Définir le nom et la valeur de cet attribut à une chaîne de
            // caractères vide. Reprendre l'état `attribute-name`.
            | Some(_) => self
                .change_current_token(|tag_tok| {
                    let attribute = HTMLTagAttribute::default();
                    tag_tok.define_tag_attributes(attribute);
                })
                .reconsume("attribute-name")
                .and_continue(),
        }
    }

    fn handle_before_attribute_value_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Ignorer le caractère.
            | Some(ch) if ch.is_html_whitespace() => self.ignore(),

            // U+0022 QUOTATION MARK (")
            //
            // Passer à l'état `attribute-value-double-quoted`.
            | Some('"') => self
                .state
                .switch_to("attribute-value-double-quoted")
                .and_continue(),

            // U+0027 APOSTROPHE (')
            //
            // Passer à l'état `attribute-value-simple-quoted`.
            | Some('\'') => self
                .state
                .switch_to("attribute-value-simple-quoted")
                .and_continue(),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Il s'agit d'une erreur d'analyse de type
            // `missing-attribute-value`. Passer à l'état `data`. Émettre
            // le jeton `tag` actuel.
            | Some('>') => {
                self.and_break_with_error("missing-attribute-value")
            }

            // Anything else
            //
            // Reprendre dans l'état `attribute-value-unquoted`.
            | _ => {
                self.reconsume("attribute-value-unquoted").and_continue()
            }
        }
    }

    fn handle_attribute_value_quoted_state(
        &mut self,
        quote: char,
    ) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+0022 QUOTATION MARK (")
            //
            // Passer à l'état `after-attribute-value-quoted`.
            | Some('"') if quote == '"' => self
                .state
                .switch_to("after-attribute-value-quoted")
                .and_continue(),

            // U+0027 APOSTROPHE (')
            //
            // Passer à l'état `after-attribute-value-quoted`.
            | Some('\'') if quote == '\'' => self
                .state
                .switch_to("after-attribute-value-quoted")
                .and_continue(),

            // U+0026 AMPERSAND (&)
            //
            // Définir l'état de retour à l'état
            // `attribute-value-double-quoted`. Passer à l'état
            // `character-reference`.
            | Some('&') => self
                .state
                .set_return("attribute-value-double-quoted")
                .switch_to("character-reference")
                .and_continue(),

            // U+0000 NULL
            //
            // Il s'agit d'une erreur d'analyse de type
            // `unexpected-null-character`. Ajouter un caractère U+FFFD
            // `REPLACEMENT_CHARACTER` à la valeur de l'attribut actuel.
            | Some('\0') => self
                .change_current_token(|html_tok| {
                    html_tok.append_character_to_attribute_value(
                        char::REPLACEMENT_CHARACTER,
                    );
                })
                .and_continue_with_error("unexpected-null-character"),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-tag`.
            // Émettre un jeton `end of file`.
            | None => self
                .set_token(HTMLToken::EOF)
                .and_break_with_error("eof-in-tag"),

            // Anything else
            //
            // Ajouter le caractère actuel à la valeur de l'attribut
            // actuel.
            | Some(ch) => self
                .change_current_token(|html_tok| {
                    html_tok.append_character_to_attribute_value(ch);
                })
                .and_continue(),
        }
    }

    fn handle_attribute_value_unquoted_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Passer à l'état `before-attribute-name`.
            | Some(ch) if ch.is_html_whitespace() => self
                .state
                .switch_to("before-attribute-name")
                .and_continue(),

            // U+0026 AMPERSAND (&)
            //
            // Définir l'état de retour à `attribute-value-unquoted`.
            // Passer à l'état `character-reference`.
            | Some('&') => self
                .state
                .set_return("attribute-value-unquoted")
                .switch_to("character-reference")
                .and_continue(),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Passer à l'état `data`. Émettre le jeton `tag` actuel.
            | Some('>') => self.state.switch_to("data").and_break(),

            // U+0000 NULL
            //
            // Il s'agit d'une erreur d'analyse de type
            // `unexpected-null-character`. Ajouter un caractère
            // `REPLACEMENT_CHARACTER` U+FFFD à la valeur de l'attribut
            // actuel.
            | Some('\0') => self
                .change_current_token(|tag_tok| {
                    tag_tok.append_character_to_attribute_value(
                        char::REPLACEMENT_CHARACTER,
                    );
                })
                .and_continue_with_error("unexpected-null-character"),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-tag`.
            // Émettre un jeton `end of file`.
            | None => self
                .set_token(HTMLToken::EOF)
                .and_break_with_error("unexpected-null-character"),

            // U+0022 QUOTATION MARK (")
            // U+0027 APOSTROPHE (')
            // U+003C LESS-THAN SIGN (<)
            // U+003D EQUALS SIGN (=)
            // U+0060 GRAVE ACCENT (`)
            //
            // Il s'agit d'une erreur d'analyse de type
            // `unexpected-character-in-unquoted-attribute-value`. La
            // traiter comme l'entrée "Anything else" ci-dessous.
            //
            // Anything else
            //
            // Ajouter le caractère actuel à la valeur de l'attribut
            // actuel.
            | Some(ch) => {
                self.change_current_token(|html_tok| {
                    html_tok.append_character_to_attribute_value(ch);
                });

                if matches!(ch, '"' | '\'' | '<' | '=' | '`') {
                    self.and_continue_with_error(
                        "unexpected-character-in-unquoted-attribute-value",
                    )
                } else {
                    self.and_continue()
                }
            }
        }
    }

    fn handle_after_attribute_value_quoted_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Passer à l'état `before-attribute-name`.
            | Some(ch) if ch.is_html_whitespace() => self
                .state
                .switch_to("before-attribute-name")
                .and_continue(),

            // U+002F SOLIDUS (/)
            //
            // Passer à l'état `self-closing-start-tag`.
            | Some('/') => self
                .state
                .switch_to("self-closing-start-tag")
                .and_continue(),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Passer à l'état `data`. Émettez le jeton `tag` actuel.
            | Some('>') => self.state.switch_to("data").and_break(),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-tag`.
            // Émettre un jeton `end of file`.
            | None => self
                .set_token(HTMLToken::EOF)
                .and_break_with_error("eof-in-tag"),

            // Anything else
            //
            // Il s'agit d'une erreur d'analyse de type
            // `missing-whitespace-between-attributes`. Reprendre l'état
            // `before-attribute-name`.
            | Some(_) => self
                .reconsume("before-attribute-name")
                .and_continue_with_error(
                    "missing-whitespace-between-attributes",
                ),
        }
    }

    fn handle_self_closing_start_tag_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+003E GREATER-THAN SIGN (>)
            //
            // Définir le drapeau `self-closing` au jeton `tag` actuel sur
            // vrai. Passer à l'état `data`. Émettre le jeton actuel.
            | Some('>') => self
                .change_current_token(|tag_tok| {
                    tag_tok.set_self_closing_tag(true);
                })
                .switch_state_to("data")
                .and_break(),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-tag`.
            // Émettre un jeton `end of file`.
            | None => self
                .set_token(HTMLToken::EOF)
                .and_break_with_error("eof-in-tag"),

            // Anything else
            //
            // Il s'agit d'une erreur d'analyse de type
            // `unexpected-solidus-in-tag`. Reprendre dans l'état
            // `before-attribute-name`.
            | Some(_) => self
                .reconsume("before-attribute-name")
                .and_continue_with_error("unexpected-solidus-in-tag"),
        }
    }

    fn handle_markup_declaration_open_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        let mut f = false;

        // Two U+002D HYPHEN-MINUS characters (-)
        //
        // Consommer ces deux caractères, créer un jeton `comment`
        // dont les données sont une chaîne de caractères vide, passer à
        // l'état `comment-start`.
        if let Cow::Borrowed("--") = self.stream.slice_until(2) {
            f = true;

            self.stream.advance(2);
            self.set_token(HTMLToken::new_comment(String::new()))
                .switch_state_to("comment-start");
        } else if let Cow::Owned(word) = self.stream.slice_until(7) {
            f = false;

            // Correspondance ASCII insensible à la casse pour le mot
            // "DOCTYPE".
            //
            // Consommer ces caractères et passer à l'état `doctype`.
            if word.to_ascii_lowercase() == "doctype" {
                f = true;

                self.switch_state_to("doctype");
                self.stream.advance(7);
            }
            // La chaîne "[CDATA[" (les cinq lettres majuscules "CDATA"
            // avec un caractère U+005B LEFT SQUARE BRACKET avant et après)
            //
            // Consommer ces caractères. S'il existe un noeud courant
            // ajusté et qu'il ne s'agit pas d'un élément de l'espace de
            // noms HTML, alors passer à l'état de section CDATA. Sinon, il
            // s'agit d'une erreur d'analyse `cdata-in-html-content`. Créer
            // un jeton `comment` dont les données sont une chaîne de
            // caractères "[CDATA[". Passer à l'état `bogus-comment`.
            else if word == "[CDATA[" {
                f = true;

                // todo: adjusted current node
                // HTMLParserError::CDATAInHtmlContent;
                self.set_token(HTMLToken::new_comment(word))
                    .switch_state_to("bogus-comment");
                self.stream.advance(7);
            }
        }

        // Anything else
        //
        // Il s'agit d'une erreur d'analyse de type
        // `incorrectly-opened-comment`. Créer un jeton `comment` dont les
        // données sont une chaîne de caractères vide. Passer à l'état
        // `bogus-comment` (ne pas consommer dans l'état actuel).
        if !f {
            self.set_token(HTMLToken::new_comment(String::new()))
                .switch_state_to("bogus-comment")
                .and_continue_with_error("incorrectly-opened-comment")
        } else {
            self.and_continue()
        }
    }

    fn handle_bogus_comment_state(&mut self) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+003E GREATER-THAN SIGN (>)
            //
            // Passer à l'état `data`. Émettre le jeton `comment` actuel.
            | Some('>') => self.state.switch_to("data").and_break(),

            // EOF
            //
            // Émettre `comment`. Émettre un jeton `end of file`.
            | None => self
                .and_emit_current_token()
                .set_token(HTMLToken::EOF)
                .and_break(),

            // U+0000 NULL
            //
            // Il s'agit d'une erreur d'analyse de type
            // `unexpected-null-character`. Ajouter un caractère U+FFFD
            // `REPLACEMENT_CHARACTER` aux données du jeton `comment`.
            | Some('\0') => self
                .change_current_token(|html_tok| {
                    html_tok.append_character(char::REPLACEMENT_CHARACTER);
                })
                .and_continue_with_error("unexpected-null-character"),

            // Anything else
            //
            // Ajouter le caractère actuel aux données du jeton `comment`.
            | Some(ch) => self
                .change_current_token(|html_tok| {
                    html_tok.append_character(ch);
                })
                .and_continue(),
        }
    }

    fn handle_comment_start_state(&mut self) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+002D HYPHEN-MINUS (-)
            //
            // Passer à l'état `comment-start-dash`.
            | Some('-') => {
                self.state.switch_to("comment-start-dash").and_continue()
            }

            // U+003E GREATER-THAN SIGN (>)
            //
            // Il s'agit d'une erreur d'analyse de type
            // `abrupt-closing-of-empty-comment`. Passer à l'état `data`.
            // Émettre le jeton `comment` actuel.
            | Some('>') => self
                .state
                .switch_to("data")
                .and_break_with_error("abrupt-closing-of-empty-comment"),

            // Anything else
            //
            // Reprendre dans l'état de commentaire.
            | _ => self.reconsume("comment").and_continue(),
        }
    }

    fn handle_comment_start_dash_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+002D HYPHEN-MINUS (-)
            //
            // Passez à l'état final du commentaire.
            | Some('-') => {
                self.state.switch_to("comment-end").and_continue()
            }

            // U+003E GREATER-THAN SIGN (>)
            //
            // Il s'agit d'une erreur d'analyse
            // abrupt-closing-of-empty-comment. Passer à l'état de données.
            // Émettre le jeton de commentaire actuel.
            | Some('>') => self
                .state
                .switch_to("data")
                .and_break_with_error("abrupt-closing-of-empty-comment"),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-comment`.
            // Émettre le jeton `comment` actuel. Émettre un jeton `end of
            // file`.
            | None => self
                .and_emit_current_token()
                .set_token(HTMLToken::EOF)
                .and_break_with_error("eof-in-comment"),

            // Anything else
            //
            // Ajouter un caractère U+002D HYPHEN-MINUS (-) aux données du
            // jeton `comment`. Reprendre l'état `comment`.
            | Some(_) => self
                .change_current_token(|comment_tok| {
                    comment_tok.append_character('-');
                })
                .reconsume("comment")
                .and_continue(),
        }
    }

    fn handle_doctype_state(&mut self) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Passer à l'état `before-doctype-name`.
            | Some(ch) if ch.is_html_whitespace() => {
                self.state.switch_to("before-doctype-name").and_continue()
            }

            // ASCII upper alpha
            //
            // Reprendre l'état `before-doctype-name`.
            | Some(ch) if ch.is_ascii_uppercase() => {
                self.reconsume("before-doctype-name").and_continue()
            }

            // Il s'agit d'une erreur d'analyse de type `eof-in-doctype`.
            // Créer un nouveau jeton `doctype`. Mettre son drapeau
            // force-quirks à vrai. Émettre le jeton actuel. Émettre un
            // jeton `end of file`.
            | None => self
                .emit_token(
                    HTMLToken::new_doctype().define_force_quirks_flag(),
                )
                .set_token(HTMLToken::EOF)
                .and_break_with_error("eof-in-doctype"),

            // Anything else
            //
            // Il s'agit d'une erreur d'analyse de type
            // `missing-whitespace-before-doctype-name`. Reprendre dans
            // l'état `before-doctype-name`.
            | Some(_) => self
                .reconsume("before-doctype-name")
                .and_continue_with_error(
                    "missing-whitespace-before-doctype-name",
                ),
        }
    }

    fn handle_before_doctype_name_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Ignorer le caractère.
            | Some(ch) if ch.is_html_whitespace() => self.ignore(),

            // ASCII upper alpha
            //
            // Créer un nouveau jeton `doctype`. Définir le nom du jeton
            // comme la version en minuscules du caractère actuel
            // (ajoutez 0x0020 au point de code du caractère). Passer à
            // l'état `doctype-name`.
            | Some(ch) if ch.is_ascii_uppercase() => self
                .set_token(
                    HTMLToken::new_doctype()
                        .define_doctype_name(ch.to_ascii_lowercase()),
                )
                .switch_state_to("doctype-name")
                .and_continue(),

            // U+0000 NULL
            //
            // Il s'agit d'une erreur d'analyse de type
            // `unexpected-null-character`. Créer un nouveau jeton
            // `doctype`. Définir le nom du jeton sur un caractère U+FFFD
            // `REPLACEMENT_CHARACTER`. Passer à l'état `doctype-name`.
            | Some('\0') => self
                .set_token(
                    HTMLToken::new_doctype()
                        .define_doctype_name(char::REPLACEMENT_CHARACTER),
                )
                .switch_state_to("doctype-name")
                .and_continue_with_error("unexpected-null-character"),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Il s'agit d'une erreur d'analyse de type
            // missing-doctype-name`. Créer un nouveau jeton `doctype`.
            // Mettre son drapeau force-quirks à vrai. Passer à l'état
            // `data`. Émettre le jeton actuel.
            | Some('>') => self
                .set_token(
                    HTMLToken::new_doctype().define_force_quirks_flag(),
                )
                .switch_state_to("data")
                .and_break_with_error("missing-doctype-name"),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-doctype`.
            // Créer un nouveau jeton `doctype`. Mettre son drapeau
            // force-quirks à vrai. Émettre le jeton actuel. Émettre un
            // jeton de `end of file`.
            | None => self
                .emit_token(
                    HTMLToken::new_doctype().define_force_quirks_flag(),
                )
                .set_token(HTMLToken::EOF)
                .and_break_with_error("eof-in-doctype"),

            // Anything else
            //
            // Créer un nouveau jeton `doctype`. Définir le nom du jeton
            // sur le caractère actuel. Passer à l'état `doctype-name`.
            | Some(ch) => self
                .set_token(
                    HTMLToken::new_doctype().define_doctype_name(ch),
                )
                .switch_state_to("doctype-name")
                .and_continue(),
        }
    }

    fn handle_doctype_name_state(&mut self) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Passer à l'état `after-doctype-name`.
            | Some(ch) if ch.is_html_whitespace() => {
                self.state.switch_to("after-doctype-name").and_continue()
            }

            // U+003E GREATER-THAN SIGN (>)
            //
            // Passer à l'état `data`. Émettre le jeton `doctype` actuel.
            | Some('>') => self.state.switch_to("data").and_break(),

            // ASCII upper alpha
            //
            // Ajouter la version en minuscules du caractère actuel
            // (ajouter 0x0020 au point de code du caractère) au nom
            // du jeton `doctype` actuel.
            | Some(ch) if ch.is_ascii_uppercase() => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.append_character(ch.to_ascii_lowercase());
                })
                .and_continue(),

            // U+0000 NULL
            //
            // Il s'agit d'une erreur d'analyse de type
            // `unexpected-null-character`. Ajouter un caractère U+FFFD
            // REPLACEMENT CHARACTER au nom du jeton `doctype`
            // actuel.
            | Some('\0') => self
                .change_current_token(|doctype_tok| {
                    doctype_tok
                        .append_character(char::REPLACEMENT_CHARACTER);
                })
                .and_continue_with_error("unexpected-null-character"),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-doctype`.
            // Définir le drapeau force-quirks du jeton `doctype` actuel
            // sur vrai. Émettre le jeton `doctype` actuel. Émettre d'un
            // jeton de `end of file`.
            | None => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.set_force_quirks_flag(true);
                })
                .and_emit_current_token()
                .set_token(HTMLToken::EOF)
                .and_break_with_error("eof-in-doctype"),

            // Anything else
            //
            // Ajouter le caractère actuel au nom du jeton `doctype`
            // actuel.
            | Some(ch) => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.append_character(ch);
                })
                .and_continue(),
        }
    }

    fn handle_after_doctype_name_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Ignorer le caractère.
            | Some(ch) if ch.is_html_whitespace() => self.ignore(),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Passer à l'état `data`. Émettre le jeton `doctype` actuel.
            | Some('>') => self.state.switch_to("data").and_break(),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-doctype`.
            // Définir le drapeau force-quirks du jeton `doctype` actuel
            // sur vrai. Émettre le jeton `doctype` actuel. Émettre un
            // jeton `end of file`.
            | None => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.set_force_quirks_flag(true);
                })
                .and_emit_current_token()
                .set_token(HTMLToken::EOF)
                .and_break_with_error("eof-in-doctype"),

            // Anything else
            //
            // Si les six caractères à partir du caractère actuel sont une
            // correspondance ASCII insensible à la casse pour le
            // mot "PUBLIC", consommez ces caractères et passez à l'état
            // `after-doctype-public-keyword`.
            //
            // Sinon, si les six caractères à partir du caractère d'entrée
            // actuel sont une correspondance ASCII insensible à la casse
            // pour le mot "SYSTEM", consommez ces caractères et passez à
            // l'état `after-doctype-system-keyword`.
            //
            // Sinon, il s'agit d'une erreur d'analyse de type
            // `invalid-character-sequence-after-doctype-name`. Mettre
            // le drapeau force-quirks du jeton actuel à vrai. Reprendre
            // dans l'état `bogus-doctype`.
            | Some(ch) => {
                let mut f = false;

                if let Cow::Owned(word) = self.stream.slice_until(5) {
                    f = false;

                    let word =
                        format!("{ch}{}", word.to_ascii_uppercase());

                    if word == "PUBLIC" {
                        f = true;

                        self.state
                            .switch_to("after-doctype-public-keyword");
                        self.stream.advance(6);
                    } else if word == "SYSTEM" {
                        f = true;

                        self.state
                            .switch_to("after-doctype-system-keyword");
                        self.stream.advance(6);
                    }
                }

                if !f {
                    self.change_current_token(|doctype_tok| {
                        doctype_tok.set_force_quirks_flag(true);
                    })
                    .reconsume("bogus-doctype")
                    .and_continue_with_error(
                        "invalid-character-sequence-after-doctype-name",
                    )
                } else {
                    self.and_continue()
                }
            }
        }
    }

    fn handle_after_doctype_public_keyword_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Passer à l'état `before-doctype-public-identifier`.
            | Some(ch) if ch.is_html_whitespace() => self
                .state
                .switch_to("before-doctype-public-identifier")
                .and_continue(),

            // U+0022 QUOTATION MARK (")
            //
            // Il s'agit d'une erreur d'analyse de type
            // `missing-whitespace-after-doctype-public-keyword`. Donner à
            // l'identifiant public du jeton `doctype` actuel la valeur de
            // la chaîne de caractères vide, passer à l'état
            // `doctype-public-identifier-double-quoted`.
            | Some('"') => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.set_public_identifier(String::new());
                })
                .switch_state_to("doctype-public-identifier-double-quoted")
                .and_continue_with_error(
                    "missing-whitespace-after-doctype-public-keyword",
                ),

            // U+0027 APOSTROPHE (')
            //
            // Il s'agit d'une erreur d'analyse de type
            // `missing-whitespace-after-doctype-public-keyword`. Donner à
            // l'identifiant public du jeton `doctype` actuel la valeur de
            // la chaîne de caractères vide, passer à l'état
            // `doctype-public-identifier-single-quoted`.
            | Some('\'') => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.set_public_identifier(String::new());
                })
                .switch_state_to("doctype-public-identifier-single-quoted")
                .and_continue_with_error(
                    "missing-whitespace-after-doctype-public-keyword",
                ),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Il s'agit d'une erreur d'analyse de type
            // `missing-doctype-public-identifier`. Définir le drapeau
            // force-quirks du jeton `doctype` actuel sur vrai. Passer à
            // l'état `data`. Émettre le jeton `doctype` actuel.
            | Some('>') => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.set_force_quirks_flag(true);
                })
                .switch_state_to("data")
                .and_break_with_error("missing-doctype-public-identifier"),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-doctype`.
            // Définir le drapeau force-quirks du jeton `doctype` actuel
            // sur vrai. Émettre le jeton `doctype` actuel. Émettre d'un
            // jeton `end of file`.
            | None => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.set_force_quirks_flag(true);
                })
                .and_emit_current_token()
                .set_token(HTMLToken::EOF)
                .and_break_with_error("eof-in-doctype"),

            // Anything else
            //
            // Il s'agit d'une erreur d'analyse de type
            // `missing-quote-before-doctype-public-identifier`. Définir le
            // drapeau force-quirks du jeton `doctype` actuel sur vrai.
            // Reprendre dans l'état `bogus-doctype`.
            | Some(_) => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.set_force_quirks_flag(true);
                })
                .reconsume("bogus-doctype")
                .and_continue_with_error(
                    "missing-quote-before-doctype-public-identifier",
                ),
        }
    }

    fn handle_before_doctype_public_identifier_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Ignorer le caractère.
            | Some(ch) if ch.is_html_whitespace() => self.ignore(),

            // U+0022 QUOTATION MARK (")
            //
            // Définir l'identifiant public du jeton `doctype` actuel à une
            // chaîne de caractères vide, passer à l'état
            // `doctype-public-identifier-double-quoted`.
            | Some('"') => self
                .set_token(
                    HTMLToken::new_doctype().define_doctype_name('\0'),
                )
                .switch_state_to("doctype-public-identifier-double-quoted")
                .and_continue(),

            // U+0027 APOSTROPHE (')
            //
            // Définir l'identifiant public du jeton `doctype` actuel à une
            // chaîne de caractères vide, passer à l'état
            // `doctype-public-identifier-single-quoted`.
            | Some('\'') => self
                .set_token(
                    HTMLToken::new_doctype().define_doctype_name('\0'),
                )
                .switch_state_to("doctype-public-identifier-single-quoted")
                .and_continue(),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Il s'agit d'une erreur d'analyse de type
            // `missing-doctype-public-identifier`. Définir le drapeau
            // force-quirks du jeton `doctype` actuel sur vrai. Passer à
            // l'état `data`. Émettre le jeton `doctype` actuel.
            | Some('>') => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.set_force_quirks_flag(true);
                })
                .switch_state_to("data")
                .and_break_with_error("missing-doctype-public-identifier"),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-doctype`.
            // Définir le drapeau force-quirks du jeton `doctype` actuel
            // sur vrai. Émettre le jeton `doctype` actuel. Émettre un
            // jeton `end of file`.
            | None => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.set_force_quirks_flag(true);
                })
                .and_emit_current_token()
                .set_token(HTMLToken::EOF)
                .and_break_with_error("eof-in-doctype"),

            // Anything else
            //
            // Il s'agit d'une erreur d'analyse de type
            // `missing-quote-before-doctype-public-identifier`. Définir le
            // drapeau force-quirks du jeton `doctype` actuel sur vrai.
            // Reprendre à l'état `bogus-doctype`.
            | Some(_) => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.set_force_quirks_flag(true);
                })
                .reconsume("bogus-doctype")
                .and_continue_with_error(
                    "missing-quote-before-doctype-public-identifier",
                ),
        }
    }

    fn handle_doctype_public_identifier_quoted(
        &mut self,
        quote: char,
    ) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+0022 QUOTATION MARK (")
            //
            // Passer à l'état `after-doctype-public-identifier`.
            | Some('"') if quote == '"' => self
                .state
                .switch_to("after-doctype-public-identifier")
                .and_continue(),

            // U+0027 APOSTROPHE (')
            //
            // Passer à l'état `after-doctype-public-identifier`.
            | Some('\'') if quote == '\'' => self
                .state
                .switch_to("after-doctype-public-identifier")
                .and_continue(),

            // U+0000 NULL
            //
            // Il s'agit d'une erreur d'analyse de type
            // `unexpected-null-character`. Ajouter un caractère U+FFFD
            // `REPLACEMENT_CHARACTER` à l'identifiant public du jeton
            // `doctype` actuel.
            | Some('\0') => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.append_character_to_public_identifier(
                        char::REPLACEMENT_CHARACTER,
                    );
                })
                .and_continue_with_error("unexpected-null-character"),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Il s'agit d'une erreur d'analyse de type
            // `abrupt-doctype-public-identifier`. Définir le drapeau
            // force-quirks du jeton `doctype` actuel sur vrai. Passer à
            // l'état `data`. Émettre le jeton `doctype` actuel.
            | Some('>') => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.set_force_quirks_flag(true);
                })
                .switch_state_to("data")
                .and_break_with_error("abrupt-doctype-public-identifier"),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-doctype`.
            // Définir le drapeau force-quirks du jeton `doctype` actuel
            // sur vrai. Émettre le jeton `doctype` actuel. Émettre un
            // jeton `end of file`.
            | None => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.set_force_quirks_flag(true);
                })
                .and_emit_current_token()
                .set_token(HTMLToken::EOF)
                .and_break_with_error("eof-in-doctype"),

            // Anything else
            //
            // Ajouter le caractère actuel à l'identifiant public du jeton
            // `doctype` actuel.
            | Some(ch) => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.append_character_to_public_identifier(ch);
                })
                .and_continue(),
        }
    }

    fn handle_after_doctype_public_identifier_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Passer à l'état
            // `between-doctype-public-and-system-identifiers`.
            | Some(ch) if ch.is_html_whitespace() => self
                .state
                .switch_to("between-doctype-public-and-system-identifiers")
                .and_continue(),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Passer à l'état `data`. Émettre le jeton `DOCTYPE` actuel.
            | Some('>') => self.state.switch_to("data").and_break(),

            // U+0022 QUOTATION MARK (")
            // U+0027 APOSTROPHE (')
            //
            // Il s'agit d'une erreur d'analyse de type
            // `missing-whitespace-between-doctype-public-and-system-identifiers`.
            // Définir l'identifiant système du jeton `doctype` actuel
            // sur une chaîne de caractères vide, passer à l'état
            // `doctype-system-identifier-double-quoted` ou
            // `doctype-system-identifier-single-quoted`.
            | Some(ch @ ('"' | '\'')) => {
                let err = "missing-whitespace-between-doctype-public-and-system-identifiers";
                self.change_current_token(|doctype_tok| {
                    doctype_tok.set_system_identifier(String::new());
                })
                .switch_state_to(if ch == '"' {
                    "doctype-system-identifier-double-quoted"
                } else {
                    "doctype-system-identifier-single-quoted"
                })
                .and_continue_with_error(err)
            }

            // EOF
            //
            // Il s'agit d'une erreur d'analyse type `eof-in-doctype`.
            // Définir le drapeau force-quirks du jeton `doctype` actuel
            // sur vrai. Émettre le jeton `doctype` actuel. Émettre d'un
            // jeton `end of file`.
            | None => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.set_force_quirks_flag(true);
                })
                .and_emit_current_token()
                .set_token(HTMLToken::EOF)
                .and_break_with_error("eof-in-doctype"),

            // Anything else
            //
            // Il s'agit d'une erreur d'analyse de type
            // `missing-quote-before-doctype-system-identifier`. Définir le
            // drapeau force-quirks du jeton `doctype` actuel. Reprendre
            // dans l'état `bogus-doctype`.
            | Some(_) => {
                self.reconsume("bogus-doctype").and_continue_with_error(
                    "missing-quote-before-doctype-system-identifier",
                )
            }
        }
    }

    fn handle_between_doctype_public_and_system_identifiers_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Ignorer le caractère.
            | Some(ch) if ch.is_html_whitespace() => self.ignore(),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Passer à l'état `data`. Émettre le jeton `doctype` actuel.
            | Some('>') => self.state.switch_to("data").and_break(),

            // U+0022 QUOTATION MARK (")
            // U+0027 APOSTROPHE (')
            //
            // Définir l'identifiant système du jeton `doctype` actuel à
            // la chaîne de caractères vide, puis passer à l'état
            // `doctype-system-identifier-double-quoted` ou
            // `doctype-system-identifier-single-quoted`.
            | Some(ch @ ('"' | '\'')) => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.set_system_identifier(String::new());
                })
                .switch_state_to(if ch == '"' {
                    "doctype-system-identifier-double-quoted"
                } else {
                    "doctype-system-identifier-single-quoted"
                })
                .and_continue(),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse type `eof-in-doctype`.
            // Définir le drapeau force-quirks du jeton `doctype` actuel
            // sur vrai. Émettre le jeton `doctype` actuel. Émettre un
            // jeton `end of file`.
            | None => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.set_force_quirks_flag(true);
                })
                .and_emit_current_token()
                .set_token(HTMLToken::EOF)
                .and_break_with_error("eof-in-doctype"),

            // Anything else
            //
            // Il s'agit d'une erreur d'analyse de type
            // `missing-quote-before-doctype-system-identifier`. Définir le
            // drapeau force-quirks du jeton `doctype` actuel. Reprendre
            // dans l'état `bogus-doctype`.
            | Some(_) => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.set_force_quirks_flag(true);
                })
                .reconsume("bogus-doctype")
                .and_continue_with_error(
                    "missing-quote-before-doctype-system-identifier",
                ),
        }
    }

    fn handle_after_doctype_system_keyword_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Passer à l'état `before-doctype-system-identifier`.
            | Some(ch) if ch.is_html_whitespace() => self
                .state
                .switch_to("before-doctype-system-identifier")
                .and_continue(),

            // U+0022 QUOTATION MARK (")
            //
            // Il s'agit d'une erreur d'analyse de type
            // `missing-whitespace-after-doctype-system-keyword`.
            // Définir l'identifiant système du jeton `doctype` actuel
            // à une chaîne de caractères vide, passer à l'état
            // `doctype-system-identifier-double-quoted`.
            | Some('"') => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.set_system_identifier(String::new());
                })
                .switch_state_to("doctype-system-identifier-double-quoted")
                .and_continue_with_error(
                    "missing-whitespace-after-doctype-system-keyword",
                ),

            // U+0027 APOSTROPHE (')
            //
            // Il s'agit d'une erreur d'analyse de type
            // missing-whitespace-after-doctype-system-keyword.
            // Définir l'identifiant système du jeton `doctype` actuel
            // à une chaîne de caractères vide, passer à l'état
            // `doctype-system-identifier-single-quoted`.
            | Some('\'') => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.set_system_identifier(String::new());
                })
                .switch_state_to("doctype-system-identifier-single-quoted")
                .and_continue_with_error(
                    "missing-whitespace-after-doctype-system-keyword",
                ),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Il s'agit d'une erreur d'analyse de type
            // `missing-doctype-system-identifier`. Définir le drapeau
            // force-quirks du jeton `doctype` actuel sur vrai. Passer à
            // l'état `data`. Émettre le jeton `doctype` actuel.
            | Some('>') => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.set_force_quirks_flag(true);
                })
                .switch_state_to("data")
                .and_break_with_error("missing-doctype-system-identifier"),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse type `eof-in-doctype`.
            // Définir le drapeau force-quirks du jeton `doctype` actuel
            // sur vrai. Émettre le jeton `doctype` actuel. Émettre un
            // jeton `end of file`.
            | None => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.set_force_quirks_flag(true);
                })
                .and_emit_current_token()
                .set_token(HTMLToken::EOF)
                .and_break_with_error("eof-in-doctype"),

            // Anything else
            //
            // Il s'agit d'une erreur d'analyse de type
            // `missing-quote-before-doctype-system-identifier`. Définir le
            // drapeau force-quirks du jeton `doctype` actuel sur vrai.
            // Reprendre dans l'état `bogus-doctype`.
            | Some(_) => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.set_force_quirks_flag(true);
                })
                .set_token(HTMLToken::EOF)
                .reconsume("bogus-doctype")
                .and_continue_with_error(
                    "missing-quote-before-doctype-system-identifier",
                ),
        }
    }

    fn handle_before_doctype_system_identifier_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Ignorer le caractère.
            | Some(ch) if ch.is_html_whitespace() => self.ignore(),

            // U+0022 QUOTATION MARK (")
            //
            // Définir l'identifiant système du jeton `doctype` actuel à la
            // chaîne de caractères vide, passer à l'état
            // `doctype-system-identifier-double-quoted`.
            | Some('"') => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.set_system_identifier(String::new());
                })
                .switch_state_to("doctype-system-identifier-double-quoted")
                .and_continue(),

            // U+0027 APOSTROPHE (')
            //
            // Définir l'identifiant système du jeton `doctype` actuel à la
            // chaîne de caractères vide, passer à  l'état
            // `doctype-system-identifier-single-quoted`
            | Some('\'') => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.set_system_identifier(String::new());
                })
                .switch_state_to("doctype-system-identifier-single-quoted")
                .and_continue(),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Il s'agit d'une erreur d'analyse de type
            // `missing-doctype-system-identifier`. Définir le drapeau
            // force-quirks du jeton `doctype` actuel sur vrai. Passer à
            // l'état `data`. Émettre le jeton `doctype` actuel.
            | Some('>') => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.set_force_quirks_flag(true);
                })
                .switch_state_to("data")
                .and_break_with_error("missing-doctype-system-identifier"),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse type `eof-in-doctype`.
            // Définir le drapeau force-quirks du jeton `doctype` actuel
            // sur vrai. Émettre le jeton `doctype` actuel. Émettre un
            // jeton `end of file`.
            | None => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.set_force_quirks_flag(true);
                })
                .and_emit_current_token()
                .set_token(HTMLToken::EOF)
                .and_break_with_error("eof-in-doctype"),

            // Anything else
            //
            // Il s'agit d'une erreur d'analyse de type
            // `missing-quote-before-doctype-system-identifier`. Définir
            // le drapeau force-quirks du jeton `doctype` actuel sur vrai.
            // Reprendre dans l'état `bogus-doctype`.
            | Some(_) => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.set_force_quirks_flag(true);
                })
                .reconsume("bogus-doctype")
                .and_continue_with_error(
                    "missing-quote-before-doctype-system-identifier",
                ),
        }
    }

    fn handle_doctype_system_identifier_quoted_state(
        &mut self,
        quote: char,
    ) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+0022 QUOTATION MARK (")
            // U+0027 APOSTROPHE (')
            //
            // Passez à l'état `after-doctype-system-identifier`.
            | Some(ch) if ch == quote => self
                .state
                .switch_to("after-doctype-system-identifier")
                .and_continue(),

            // U+0000 NULL
            //
            // Il s'agit d'une erreur d'analyse de type
            // unexpected-null-character. Ajouter un caractère U+FFFD
            // `REPLACEMENT_CHARACTER` à l'identifiant système du jeton
            // DOCTYPE actuel.
            | Some('\0') => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.append_character_to_system_identifier(
                        char::REPLACEMENT_CHARACTER,
                    );
                })
                .and_continue(),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Il s'agit d'une erreur d'analyse
            // abrupt-doctype-system-identifier. Définir le drapeau
            // force-quirks du jeton `doctype` actuel. Passer à l'état de
            // données. Émettre le jeton `doctype` actuel.
            | Some('>') => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.set_force_quirks_flag(true);
                })
                .switch_state_to("data")
                .and_break_with_error("abrupt-doctype-system-identifier"),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse type `eof-in-doctype`.
            // Définir le drapeau force-quirks du jeton
            // `doctype` actuel sur vrai. Émettre le jeton
            // `doctype` actuel. Émission d'un jeton de
            // fin de fichier.
            | None => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.set_force_quirks_flag(true);
                })
                .and_emit_current_token()
                .set_token(HTMLToken::EOF)
                .and_break_with_error("eof-in-doctype"),

            // Anything else
            //
            // Ajouter le caractère actuel à l'identifiant système du jeton
            // DOCTYPE actuel.
            | Some(ch) => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.append_character_to_system_identifier(ch);
                })
                .and_continue(),
        }
    }

    fn handle_after_doctype_system_identifier_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Ignorer le caractère.
            | Some(ch) if ch.is_html_whitespace() => self.ignore(),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Passer à l'état `data`. Émettre le jeton `doctype` actuel.
            | Some('>') => self.state.switch_to("data").and_break(),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse type `eof-in-doctype`.
            // Définir le drapeau force-quirks du jeton `doctype` actuel
            // sur vrai. Émettre le jeton `doctype` actuel. Émettre un
            // jeton `end of file`.
            | None => self
                .change_current_token(|doctype_tok| {
                    doctype_tok.set_force_quirks_flag(true);
                })
                .and_emit_current_token()
                .set_token(HTMLToken::EOF)
                .and_break_with_error("eof-in-doctype"),

            // Anything else
            //
            // Il s'agit d'une erreur de parse
            // `unexpected-character-after-doctype-system-identifier`.
            // Reprendre dans l'état `bogus-doctype` (cela n'active pas
            // le drapeau force-quirks du jeton `doctype` actuel).
            | Some(_) => {
                self.reconsume("bogus-doctype").and_continue_with_error(
                    "unexpected-character-after-doctype-system-identifier",
                )
            }
        }
    }

    fn handle_bogus_doctype_state(&mut self) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+003E GREATER-THAN SIGN (>)
            //
            // Passer à l'état `data`. Émettez le jeton `doctype`.
            | Some('>') => self.state.switch_to("data").and_break(),

            // U+0000 NULL
            //
            // Il s'agit d'une erreur d'analyse de type
            // `unexpected-null-character`.
            //
            // Ignorer le caractère.
            | Some('\0') => {
                self.and_continue_with_error("unexpected-null-character")
            }

            // EOF
            //
            // Émettre le jeton `doctype`. Émettre un jeton de `end of
            // file`.
            | None => self
                .and_emit_current_token()
                .set_token(HTMLToken::EOF)
                .and_break(),

            // Anything else
            //
            // Ignorer le caractère
            | Some(_) => self.ignore(),
        }
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl<C> HTMLStateIteratorInterface for Tokenizer<C> where
    C: Iterator<Item = char>
{
}

impl HTMLStateIteratorInterface for HTMLState {}

impl HTMLCharacterInterface for char {
    fn is_html_whitespace(&self) -> bool {
        self.is_ascii_whitespace() && '\r'.ne(self)
    }
}

impl<C> Iterator for Tokenizer<C>
where
    C: Iterator<Item = char>,
{
    type Item = HTMLToken;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.temp.is_empty() {
            return self.pop_token();
        }

        loop {
            let state = match self.state.current {
                | State::Data => self.handle_data_state(),
                | State::TagOpen => self.handle_tag_open_state(),
                | State::EndTagOpen => self.handle_end_tag_open_state(),
                | State::TagName => self.handle_tag_name_state(),
                | State::BeforeAttributeName => {
                    self.handle_before_attribute_name_state()
                }
                | State::AttributeName => {
                    self.handle_attribute_name_state()
                }
                | State::AfterAttributeName => {
                    self.handle_after_attribute_name_state()
                }
                | State::BeforeAttributeValue => {
                    self.handle_before_attribute_value_state()
                }
                | State::AttributeValueDoubleQuoted => {
                    self.handle_attribute_value_quoted_state('"')
                }
                | State::AttributeValueSimpleQuoted => {
                    self.handle_attribute_value_quoted_state('\'')
                }
                | State::AttributeValueUnquoted => {
                    self.handle_attribute_value_unquoted_state()
                }
                | State::AfterAttributeValueQuoted => {
                    self.handle_after_attribute_value_quoted_state()
                }
                | State::SelfClosingStartTag => {
                    self.handle_self_closing_start_tag_state()
                }
                | State::MarkupDeclarationOpen => {
                    self.handle_markup_declaration_open_state()
                }
                | State::BogusComment => self.handle_bogus_comment_state(),
                | State::CommentStart => self.handle_comment_start_state(),
                | State::CommentStartDash => self.handle_comment_start_dash_state(),
                | State::DOCTYPE => self.handle_doctype_state(),
                | State::BeforeDOCTYPEName => {
                    self.handle_before_doctype_name_state()
                }
                | State::DOCTYPEName => self.handle_doctype_name_state(),
                | State::AfterDOCTYPEName => {
                    self.handle_after_doctype_name_state()
                }
                | State::AfterDOCTYPEPublicKeyword => {
                    self.handle_after_doctype_public_keyword_state()
                }
                | State::BeforeDOCTYPEPublicIdentifier => {
                    self.handle_before_doctype_public_identifier_state()
                }
                | State::DOCTYPEPublicIdentifierDoubleQuoted => {
                    self.handle_doctype_public_identifier_quoted('"')
                }
                | State::DOCTYPEPublicIdentifierSingleQuoted => {
                    self.handle_doctype_public_identifier_quoted('\'')
                }
                | State::AfterDOCTYPEPublicIdentifier => {
                    self.handle_after_doctype_public_identifier_state()
                }
                | State::BetweenDOCTYPEPublicAndSystemIdentifiers => {
                    self.handle_between_doctype_public_and_system_identifiers_state()
                }
                | State::AfterDOCTYPESystemKeyword => {
                    self.handle_after_doctype_system_keyword_state()
                }
                | State::BeforeDOCTYPESystemIdentifier => {
                    self.handle_before_doctype_system_identifier_state()
                }
                | State::DOCTYPESystemIdentifierDoubleQuoted => {
                    self.handle_doctype_system_identifier_quoted_state('"')
                }
                | State::DOCTYPESystemIdentifierSingleQuoted => {
                    self.handle_doctype_system_identifier_quoted_state('\'')
                }
                | State::AfterDOCTYPESystemIdentifier => {
                    self.handle_after_doctype_system_identifier_state()
                }
                | State::BogusDOCTYPE => self.handle_bogus_doctype_state(),

                | _ => return None,
            };

            match state {
                | Ok(HTMLStateIterator::Continue) => continue,
                | Ok(HTMLStateIterator::Break) => break,
                | Err((x, state)) => {
                    emit_html_error!(x);

                    match state {
                        | HTMLStateIterator::Continue => continue,
                        | HTMLStateIterator::Break => break,
                    }
                }
            }
        }

        self.current_token()
    }
}

impl FromStr for State {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            | "data" => Self::Data,
            | "tag-open" => Self::TagOpen,
            | "end-tag-open" => Self::EndTagOpen,
            | "tag-name" => Self::TagName,
            | "before-attribute-name" => Self::BeforeAttributeName,
            | "attribute-name" => Self::AttributeName,
            | "after-attribute-name" => Self::AfterAttributeName,
            | "before-attribute-value" => Self::BeforeAttributeValue,
            | "attribute-value-double-quoted" => {
                Self::AttributeValueDoubleQuoted
            }
            | "attribute-value-simple-quoted" => {
                Self::AttributeValueSimpleQuoted
            }
            | "attribute-value-unquoted" => Self::AttributeValueUnquoted,
            | "after-attribute-value-quoted" => {
                Self::AfterAttributeValueQuoted
            }
            | "self-closing-start-tag" => Self::SelfClosingStartTag,
            | "bogus-comment" => Self::BogusComment,
            | "markup-declaration-open" => Self::MarkupDeclarationOpen,
            | "comment-start" => Self::CommentStart,
            | "doctype" => Self::DOCTYPE,
            | "before-doctype-name" => Self::BeforeDOCTYPEName,
            | "doctype-name" => Self::DOCTYPEName,
            | "after-doctype-name" => Self::AfterDOCTYPEName,
            | "after-doctype-public-keyword" => {
                Self::AfterDOCTYPEPublicKeyword
            }
            | "before-doctype-public-identifier" => {
                Self::BeforeDOCTYPEPublicIdentifier
            }
            | "doctype-public-identifier-double-quoted" => {
                Self::DOCTYPEPublicIdentifierDoubleQuoted
            }
            | "doctype-public-identifier-single-quoted" => {
                Self::DOCTYPEPublicIdentifierSingleQuoted
            }
            | "after-doctype-public-identifier" => {
                Self::AfterDOCTYPEPublicIdentifier
            }
            | "between-doctype-public-and-system-identifiers" => {
                Self::BetweenDOCTYPEPublicAndSystemIdentifiers
            }
            | "after-doctype-system-keyword" => {
                Self::AfterDOCTYPESystemKeyword
            }
            | "before-doctype-system-identifier" => {
                Self::BeforeDOCTYPESystemIdentifier
            }
            | "doctype-system-identifier-double-quoted" => {
                Self::DOCTYPESystemIdentifierDoubleQuoted
            }
            | "doctype-system-identifier-single-quoted" => {
                Self::DOCTYPESystemIdentifierSingleQuoted
            }
            | "after-doctype-system-identifier" => {
                Self::AfterDOCTYPESystemIdentifier
            }
            | "bogus-doctype" => Self::BogusDOCTYPE,
            | "character-reference" => Self::CharacterReference,
            | _ => return Err("!!!!! Nom d'état inconnu !!!!!"),
        })
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

// ---- //
// Test //
// ---- //

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
    fn test_comment() {
        let mut html_tok = get_tokenizer_html(include_str!(
            "crashtests/comment/comment.html"
        ));

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::Comment("-- Hello World --".into()))
        );
    }

    #[test]
    fn test_tag() {
        let mut html_tok =
            get_tokenizer_html(include_str!("crashtests/tag/tag.html"));

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::StartTag {
                name: "div".into(),
                self_closing_flag: false,
                attributes: vec![("id".into(), "foo".into())]
            })
        );

        // Hello World</div> ...
        html_tok.nth(12);

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::StartTag {
                name: "input".into(),
                self_closing_flag: true,
                attributes: vec![("value".into(), "Hello World".into())]
            })
        );
    }
}
