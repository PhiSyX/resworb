/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{borrow::Cow, collections::VecDeque};

use parser::preprocessor::InputStreamPreprocessor;

use super::{
    error::HTMLParserError,
    token::{HTMLTagAttribute, HTMLToken},
};
use crate::{emit_html_error, parser::token::HTMLTagAttributeName};

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

    /// 13.2.5.53 DOCTYPE state
    DOCTYPE,

    /// 13.2.5.54 Before DOCTYPE name state
    BeforeDOCTYPEName,

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
            // Il s'agit d'une erreur d'analyse eof-in-tag. Émettre un
            // jeton de fin de fichier.
            | None => {
                emit_html_error!(HTMLParserError::EofInTag);
                self.token = Some(HTMLToken::EOF);
                StateIterator::Break
            }

            // Anything else
            //
            // Ajoute le caractère d'entrée actuel au nom de balise du
            // jeton de balise actuel.
            | Some(ch) => {
                if let Some(ref mut tag) = self.token {
                    tag.append_character(ch);
                }
                StateIterator::Continue
            }
        }
    }

    fn handle_before_attribute_name_state(&mut self) -> StateIterator {
        match self.stream.next_input_char() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Ignorer le caractère
            | Some(ch) if ch.is_ascii_whitespace() && ch != '\r' => {
                StateIterator::Continue
            }

            // U+002F SOLIDUS (/)
            // U+003E GREATER-THAN SIGN (>)
            // EOF
            //
            // Reprendre dans l'état après le nom de l'attribut.
            | Some('/' | '>') | None => {
                self.reconsume(State::AfterAttributeName);
                StateIterator::Continue
            }

            // U+003D EQUALS SIGN (=)
            //
            // Il s'agit d'une erreur d'analyse
            // unexpected-equals-sign-before-attribute-name. Commencer un
            // nouvel attribut dans le jeton de balise actuel. Définir
            // le nom de cet attribut sur le caractère d'entrée actuel, et
            // sa valeur sur une chaîne vide. Passer à l'état de nom
            // d'attribut.
            | Some(ch @ '=') => {
                emit_html_error!(HTMLParserError::UnexpectedEqualsSignBeforeAttributeName);

                let mut attribute = HTMLTagAttribute::default();
                attribute.0 = HTMLTagAttributeName::from(ch);

                if let Some(ref mut tag) = self.token {
                    tag.define_tag_attributes(attribute);
                }

                self.state.current = State::AttributeName;

                StateIterator::Break
            }

            // Anything else
            //
            // Commence un nouvel attribut dans le jeton de balise actuel.
            // Définissez le nom et la valeur de cet attribut à la chaîne
            // vide. Reprendre l'état du nom de l'attribut.
            | Some(_) => {
                let attribute = HTMLTagAttribute::default();

                if let Some(ref mut tag) = self.token {
                    tag.define_tag_attributes(attribute);
                }

                self.reconsume(State::AttributeName);

                StateIterator::Continue
            }
        }
    }

    /// Lorsque l'agent utilisateur quitte l'état du nom de l'attribut (et
    /// avant d'émettre le jeton de balise, le cas échéant), le nom de
    /// l'attribut complet doit être comparé aux autres attributs du même
    /// jeton ; s'il existe déjà un attribut du jeton portant exactement le
    /// même nom, il s'agit d'une erreur d'analyse d'attribut en double et
    /// le nouvel attribut doit être retiré du jeton.
    ///
    /// Note: si un attribut est donc retiré d'un token, il n'est plus
    /// jamais utilisé par l'analyseur syntaxique, de même que la valeur
    /// qui lui est associée, le cas échéant, et il est donc effectivement
    /// mis au rebut. Le retrait de l'attribut de cette manière ne modifie
    /// pas son statut d'"attribut actuel" pour les besoins du tokenizer,
    /// cependant.
    fn handle_attribute_name_state(&mut self) -> StateIterator {
        match self.stream.next_input_char() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            // U+002F SOLIDUS (/)
            // U+003E GREATER-THAN SIGN (>)
            // EOF
            //
            // Reprendre dans l'état après le nom de l'attribut.
            | Some(ch) if ch.is_ascii_whitespace() && ch != '\r' => {
                self.reconsume(State::AfterAttributeName);
                StateIterator::Continue
            }
            | None | Some('/' | '>') => {
                self.reconsume(State::AfterAttributeName);
                StateIterator::Continue
            }

            // U+003D EQUALS SIGN (=)
            //
            // Passer à l'état de la valeur de l'attribut avant.
            | Some('=') => {
                self.state.current = State::BeforeAttributeValue;
                StateIterator::Continue
            }

            // ASCII upper alpha
            //
            // Ajoute la version en minuscules du caractère d'entrée actuel
            // (ajouter 0x0020 au point de code du caractère) au nom de
            // l'attribut actuel.
            | Some(ch) if ch.is_ascii_uppercase() => {
                if let Some(ref mut tag) = self.token {
                    tag.append_character_to_attribute_name(
                        ch.to_ascii_lowercase(),
                    );
                }
                StateIterator::Continue
            }

            // U+0000 NULL
            //
            // Il s'agit d'une erreur d'analyse unexpected-null-character.
            // Ajoute un caractère U+FFFD REPLACEMENT CHARACTER au nom de
            // l'attribut actuel.
            | Some('\0') => {
                emit_html_error!(HTMLParserError::UnexpectedNullCharacter);
                self.list.push_back(HTMLToken::Character(
                    char::REPLACEMENT_CHARACTER,
                ));
                StateIterator::Continue
            }

            // U+0022 QUOTATION MARK (")
            // U+0027 APOSTROPHE (')
            // U+003C LESS-THAN SIGN (<)
            //
            // Il s'agit d'une erreur d'analyse de type
            // unexpected-character-in-attribute-name. La traiter comme
            // l'entrée "Anything else" ci-dessous.
            //
            // Anything else
            //
            // Ajoute le caractère d'entrée actuel au nom de l'attribut
            // actuel.
            | Some(ch) => {
                if matches!(ch, '"' | '\'' | '<') {
                    emit_html_error!(
                        HTMLParserError::UnexpectedCharacterInAttributeName
                    );
                }

                if let Some(ref mut tag) = self.token {
                    tag.append_character_to_attribute_name(ch);
                }

                StateIterator::Continue
            }
        }
    }

    fn handle_after_attribute_name_state(&mut self) -> StateIterator {
        match self.stream.next_input_char() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Ignorer le caractère.
            | Some(ch) if ch.is_ascii_whitespace() && ch != '\r' => {
                StateIterator::Continue
            }

            // U+002F SOLIDUS (/)
            //
            // Passer à l'état de balise de démarrage auto-fermante.
            | Some('/') => {
                self.state.current = State::SelfClosingStartTag;
                StateIterator::Continue
            }

            // U+003D EQUALS SIGN (=)
            //
            // Passer à l'état d'avant la valeur de l'attribut.
            | Some('=') => {
                self.state.current = State::BeforeAttributeValue;
                StateIterator::Continue
            }

            // U+003E GREATER-THAN SIGN (>)
            //
            // Passer à l'état de données. Émettre le jeton actuel.
            | Some('>') => {
                self.state.current = State::Data;
                StateIterator::Break
            }

            // EOF
            //
            // Il s'agit d'une erreur d'analyse eof-in-tag. Émettre un
            // jeton de fin de fichier.
            | None => {
                emit_html_error!(HTMLParserError::EofInTag);
                self.token = Some(HTMLToken::EOF);
                StateIterator::Break
            }

            // Anything else
            //
            // Commence un nouvel attribut dans le jeton de balise actuel.
            // Définissez le nom et la valeur de cet attribut à la chaîne
            // vide. Reprendre l'état du nom de l'attribut.
            | Some(_) => {
                let attribute = HTMLTagAttribute::default();

                if let Some(ref mut tag) = self.token {
                    tag.define_tag_attributes(attribute);
                }

                self.reconsume(State::AttributeName);

                StateIterator::Continue
            }
        }
    }

    fn handle_before_attribute_value_state(&mut self) -> StateIterator {
        match self.stream.next_input_char() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Ignorer le caractère.
            | Some(ch) if ch.is_ascii_whitespace() && ch != '\r' => {
                StateIterator::Continue
            }

            // U+0022 QUOTATION MARK (")
            //
            // Passer à l'état de valeur d'attribut (double guillemets).
            | Some('"') => {
                self.state.current = State::AttributeValueDoubleQuoted;
                StateIterator::Continue
            }

            // U+0027 APOSTROPHE (')
            //
            // Passer à l'état de valeur d'attribut (simple guillemet).
            | Some('\'') => {
                self.state.current = State::AttributeValueSimpleQuoted;
                StateIterator::Continue
            }

            // U+003E GREATER-THAN SIGN (>)
            //
            // Il s'agit d'une erreur d'analyse missing-attribute-value.
            // Passer à l'état de données. Émettre le jeton de balise
            // actuel.
            | Some('>') => {
                emit_html_error!(HTMLParserError::MissingAttributeValue);
                StateIterator::Break
            }

            // Anything else
            //
            // Reprendre à l'état de la valeur de l'attribut (unquoted).
            | _ => {
                self.reconsume(State::AttributeValueUnquoted);
                StateIterator::Continue
            }
        }
    }

    fn handle_attribute_value_quoted_state(
        &mut self,
        quote: char,
    ) -> StateIterator {
        match self.stream.next_input_char() {
            // U+0022 QUOTATION MARK (")
            //
            // Passer à l'état de la valeur d'après attribut (quoted).
            | Some('"') if quote == '"' => {
                self.state.current = State::AfterAttributeValueQuoted;
                StateIterator::Continue
            }

            // U+0027 APOSTROPHE (')
            //
            // Passer à l'état de la valeur d'après attribut (quoted).
            | Some('\'') if quote == '\'' => {
                self.state.current = State::AfterAttributeValueQuoted;
                StateIterator::Continue
            }

            // U+0026 AMPERSAND (&)
            //
            // Définir l'état de retour à l'état de la valeur de l'attribut
            // (entre guillemets). Passer à l'état de référence du
            // caractère.
            | Some('&') => {
                self.state.returns = Some(State::CharacterReference);
                StateIterator::Continue
            }

            // U+0000 NULL
            //
            // Il s'agit d'une erreur d'analyse unexpected-null-character.
            // Ajouter un caractère U+FFFD REPLACEMENT CHARACTER
            // à la valeur de l'attribut actuel.
            | Some('\0') => {
                emit_html_error!(HTMLParserError::UnexpectedNullCharacter);
                if let Some(ref mut html_tok) = self.token {
                    html_tok.append_character_to_attribute_value(
                        char::REPLACEMENT_CHARACTER,
                    );
                }
                StateIterator::Continue
            }

            // EOF
            //
            // Il s'agit d'une erreur d'analyse eof-in-tag. Émettre un
            // jeton de fin de fichier.
            | None => {
                emit_html_error!(HTMLParserError::EofInTag);
                self.token = Some(HTMLToken::EOF);
                StateIterator::Break
            }

            // Anything else
            //
            // Ajoute le caractère d'entrée actuel à la valeur de
            // l'attribut actuel.
            | Some(ch) => {
                if let Some(ref mut html_tok) = self.token {
                    html_tok.append_character_to_attribute_value(ch);
                }
                StateIterator::Continue
            }
        }
    }

    fn handle_attribute_value_unquoted_state(&mut self) -> StateIterator {
        match self.stream.next_input_char() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Passer à l'état avant le nom d'attribut.
            | Some(ch) if ch.is_ascii_whitespace() && ch != '\r' => {
                self.state.current = State::BeforeAttributeName;
                StateIterator::Continue
            }

            // U+0026 AMPERSAND (&)
            //
            // Définir l'état de retour à l'état de la valeur de l'attribut
            // (entre guillemets). Passer à l'état de référence du
            // caractère.
            | Some('&') => {
                self.state.returns = Some(State::AttributeValueUnquoted);
                self.state.current = State::CharacterReference;
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

            // U+0000 NULL
            //
            // Il s'agit d'une erreur d'analyse unexpected-null-character.
            // Ajouter un caractère REPLACEMENT CHARACTER U+FFFD à la
            // valeur de l'attribut actuel.
            | Some('\0') => {
                emit_html_error!(HTMLParserError::UnexpectedNullCharacter);

                if let Some(ref mut tag) = self.token {
                    tag.append_character_to_attribute_value(
                        char::REPLACEMENT_CHARACTER,
                    );
                }

                StateIterator::Continue
            }

            // EOF
            //
            // Il s'agit d'une erreur d'analyse eof-in-tag. Émettre un
            // jeton de fin de fichier.
            | None => {
                emit_html_error!(HTMLParserError::UnexpectedNullCharacter);
                self.token = Some(HTMLToken::EOF);
                StateIterator::Break
            }

            // U+0022 QUOTATION MARK (")
            // U+0027 APOSTROPHE (')
            // U+003C LESS-THAN SIGN (<)
            // U+003D EQUALS SIGN (=)
            // U+0060 GRAVE ACCENT (`)
            //
            // Il s'agit d'une erreur d'analyse de type
            // unexpected-character-in-unquoted-attribute-value. La traiter
            // comme l'entrée "Anything else" ci-dessous.

            // Anything else
            //
            // Append the current input character to the current
            // attribute's value.
            | Some(ch) => {
                if matches!(ch, '"' | '\'' | '<' | '=' | '`') {
                    emit_html_error!(
                        HTMLParserError::UnexpectedCharacterInUnquotedAttributeValue
                    );
                }

                if let Some(ref mut html_tok) = self.token {
                    let current_ch =
                        self.stream.current.expect("Le caractère actuel");
                    html_tok
                        .append_character_to_attribute_value(current_ch);
                }

                StateIterator::Continue
            }
        }
    }

    fn handle_after_attribute_value_quoted_state(
        &mut self,
    ) -> StateIterator {
        match self.stream.next_input_char() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Passer à l'état avant le nom d'attribut.
            | Some(ch) if ch.is_ascii_whitespace() && ch != '\r' => {
                self.state.current = State::BeforeAttributeName;
                StateIterator::Continue
            }

            // U+002F SOLIDUS (/)
            //
            // Passer à l'état de balise de début à fermeture automatique.
            | Some('/') => {
                self.state.current = State::SelfClosingStartTag;
                StateIterator::Continue
            }

            // U+003E GREATER-THAN SIGN (>)
            //
            // Passer à l'état des données. Émettez le jeton de balise
            // actuel.
            | Some('>') => {
                self.state.current = State::Data;
                StateIterator::Break
            }

            // EOF
            //
            // Il s'agit d'une erreur d'analyse eof-in-tag. Émettre un
            // jeton de fin de fichier.
            | None => {
                emit_html_error!(HTMLParserError::EofInTag);
                self.token = Some(HTMLToken::EOF);
                StateIterator::Break
            }

            // Anything else
            //
            // Il s'agit d'une erreur d'analyse
            // missing-whitespace-between-attributes. Reprendre l'état
            // avant le nom d'attribut.
            | Some(_) => {
                emit_html_error!(
                    HTMLParserError::MissingWhitespaceBetweenAttributes
                );
                self.reconsume(State::BeforeAttributeName);
                StateIterator::Continue
            }
        }
    }

    fn handle_markup_declaration_open_state(&mut self) -> StateIterator {
        let mut f = false;

        // Two U+002D HYPHEN-MINUS characters (-)
        //
        // Consommer ces deux caractères, créer un jeton de commentaire
        // dont les données sont la chaîne vide, et passer à l'état de
        // début de commentaire.
        if let Cow::Borrowed("--") = self.stream.slice_until(2) {
            f = true;

            self.stream.advance(2);
            self.token = Some(HTMLToken::new_comment(String::new()));
            self.state.current = State::CommentStart;
        } else if let Cow::Owned(word) = self.stream.slice_until(7) {
            f = false;

            // Correspondance ASCII insensible à la casse pour le mot
            // "DOCTYPE".
            //
            // Consommer ces caractères et passer à l'état DOCTYPE.
            if word.to_ascii_lowercase() == "doctype" {
                f = true;

                self.state.current = State::DOCTYPE;
                self.stream.advance(7);
            }
            // La chaîne "[CDATA[" (les cinq lettres majuscules "CDATA"
            // avec un caractère U+005B LEFT SQUARE BRACKET avant et après)
            //
            // Consommer ces caractères. S'il existe un noeud courant
            // ajusté et qu'il ne s'agit pas d'un élément de l'espace de
            // noms HTML, alors passer à l'état de section CDATA. Sinon, il
            // s'agit d'une erreur d'analyse cdata-in-html-content. Créer
            // un jeton de commentaire dont les données sont la chaîne
            // "[CDATA[". Passer à l'état de commentaire fictif.
            else if word == "[CDATA[" {
                f = true;

                // todo: adjusted current node
                emit_html_error!(HTMLParserError::CDATAInHtmlContent);
                self.token = Some(HTMLToken::new_comment(word));
                self.state.current = State::BogusComment;
                self.stream.advance(7);
            }
        }

        // Anything else
        //
        // Il s'agit d'une erreur d'analyse incorrectly-opened-comment.
        // Créer un jeton de commentaire dont les données sont une
        // chaîne vide. Passer à l'état de commentaire fictif
        // (ne consommez rien dans l'état actuel).
        if !f {
            emit_html_error!(HTMLParserError::IncorrectlyOpenedComment);
            self.token = Some(HTMLToken::new_comment(String::new()));
            self.state.current = State::BogusComment;
        }

        StateIterator::Continue
    }

    fn handle_bogus_comment_state(&mut self) -> StateIterator {
        match self.stream.next_input_char() {
            // U+003E GREATER-THAN SIGN (>)
            //
            // Passer à l'état de données. Émettre le jeton de commentaire
            // actuel.
            | Some('>') => {
                self.state.current = State::Data;
                StateIterator::Break
            }

            // EOF
            //
            // Émettre le commentaire. Émettre un jeton de fin de fichier.
            | None => {
                if let Some(token) = self.current_token() {
                    self.list.push_back(token);
                }

                self.token = Some(HTMLToken::EOF);

                StateIterator::Break
            }

            // U+0000 NULL
            //
            // Il s'agit d'une erreur d'analyse de type
            // unexpected-null-character. Ajouter un caractère U+FFFD
            // REPLACEMENT CHARACTER aux données du jeton de commentaire.
            | Some('\0') => {
                emit_html_error!(HTMLParserError::UnexpectedNullCharacter);

                if let Some(ref mut html_tok) = self.token {
                    html_tok.append_character(char::REPLACEMENT_CHARACTER);
                }

                StateIterator::Continue
            }

            // Anything else
            //
            // Ajouter le caractère d'entrée actuel aux données du jeton de
            // commentaire.
            | Some(ch) => {
                if let Some(ref mut html_tok) = self.token {
                    html_tok.append_character(ch);
                }

                StateIterator::Continue
            }
        }
    }

    fn handle_doctype_state(&mut self) -> StateIterator {
        match self.stream.next_input_char() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Passer à l'état d'avant nom du DOCTYPE.
            | Some(ch) if ch.is_ascii_whitespace() && ch != '\r' => {
                self.state.current = State::BeforeDOCTYPEName;
                StateIterator::Continue
            }

            // ASCII upper alpha
            //
            // Reprendre l'état d'avant le nom du DOCTYPE.
            | Some(ch) if ch.is_ascii_uppercase() => {
                self.reconsume(State::BeforeDOCTYPEName);
                StateIterator::Continue
            }

            // Il s'agit d'une erreur d'analyse de type eof-in-doctype.
            // Créer un nouveau jeton DOCTYPE. Mettre son drapeau
            // force-quirks à vrai. Émettre le jeton actuel. Émettre un
            // jeton de fin de fichier.
            | None => {
                emit_html_error!(HTMLParserError::EofInDOCTYPE);
                let doctype_tok = HTMLToken::new_doctype(String::new())
                    .define_force_quirks_flag();
                self.list.push_back(doctype_tok);
                self.token = Some(HTMLToken::EOF);
                StateIterator::Break
            }

            | Some(_) => {
                emit_html_error!(
                    HTMLParserError::MissingWhitespaceBeforeDOCTYPEName
                );
                self.reconsume(State::BeforeDOCTYPEName);
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
            let state = match &self.state.current {
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
                | State::MarkupDeclarationOpen => {
                    self.handle_markup_declaration_open_state()
                }
                | State::BogusComment => self.handle_bogus_comment_state(),
                | State::DOCTYPE => self.handle_doctype_state(),

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
    #[ignore = "ok"]
    fn test_simple_tag() {
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

    #[test]
    #[ignore = "ok"]
    fn test_simple_tag_attributes() {
        let mut html_tok = get_tokenizer_html(include_str!(
            "crashtests/simple_tag_attributes.html"
        ));

        let attributes: [(String, String); 3] = [
            (String::from("id"), String::from("un-id")),
            (
                String::from("class"),
                String::from("css-class_1 css-class-2"),
            ),
            (String::from("href"), String::from("#")),
        ];

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::StartTag {
                name: "a".into(),
                self_closing_flag: false,
                attributes: attributes.to_vec()
            })
        );
    }

    #[test]
    fn test_doctype() {
        let mut html_tok =
            get_tokenizer_html(include_str!("crashtests/doctype.html"));

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::DOCTYPE {
                name: None,
                public_identifier: None,
                system_identifier: None,
                force_quirks_flag: false
            })
        );
    }
}
