/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{borrow::Cow, collections::VecDeque};

use parser::preprocessor::InputStreamPreprocessor;

use super::{
    error::HTMLParserError,
    token::{HTMLTagAttribute, HTMLToken},
};
use crate::{
    named_characters::{
        NamedCharacterReferences, NamedCharacterReferencesEntities,
    },
    parser::token::HTMLTagAttributeName,
};

// ----- //
// Macro //
// ----- //

macro_rules! define_state {
    (
    $(
        #[$attr:meta]
        $enum:ident = $str:literal
    ),*
    ) => {
#[derive(Debug)]
#[derive(Clone)]
#[allow(clippy::upper_case_acronyms)]
pub enum State {
    $( #[$attr] $enum ),*
}

impl core::str::FromStr for State {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            $( | $str => Self::$enum, )*
            | _ => return Err("Nom de l'état inconnu."),
        })
    }
}

impl core::fmt::Display for State {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!( f, "{}", match self { $( | Self::$enum => $str, )* } )
    }
}
    };
}

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

    fn and_emit(&self) -> ResultHTMLStateIterator {
        Ok(HTMLStateIterator::Break)
    }

    fn and_emit_with_error(&self, err: &str) -> ResultHTMLStateIterator {
        let err = err.parse().unwrap();
        Err((err, HTMLStateIterator::Break))
    }
}

trait HTMLCharacterInterface {
    fn is_html_whitespace(&self) -> bool;
    fn is_noncharacter(&self) -> bool;
    fn is_surrogate(&self) -> bool;
}

// ---- //
// Type //
// ---- //

pub(crate) type Tokenizer<C> = HTMLTokenizer<C>;

type ResultHTMLStateIterator =
    Result<HTMLStateIterator, (HTMLParserError, HTMLStateIterator)>;

// --------- //
// Structure //
// --------- //

pub struct HTMLTokenizer<Chars>
where
    Chars: Iterator<Item = char>,
{
    stream: InputStreamPreprocessor<Chars, Chars::Item>,

    /// Le jeton courant.
    token: Option<HTMLToken>,

    state: HTMLState,

    /// La sortie de l'étape de tokenisation est une série de zéro ou plus
    /// des jetons.
    output_tokens: VecDeque<HTMLToken>,

    named_character_reference_code: NamedCharacterReferencesEntities,

    /// Certains états utilisent un tampon temporaire pour suivre leur
    /// progression.
    temporary_buffer: String,

    /// [L'état de référence du caractère](State::CharacterReference)
    /// utilise un [état de retour](HTMLState::returns) pour revenir à
    /// un état à partir duquel il a été invoqué.
    character_reference_code: u32,

    last_start_tag_token: Option<HTMLToken>,
}

#[derive(Clone)]
pub struct HTMLState {
    /// L'état courant.
    current: State,
    /// L'état de retour.
    returns: Option<State>,
}

// ----------- //
// Énumération //
// ----------- //

define_state! {
    /// 13.2.5.1 Data state
    Data = "data",

    /// 13.2.5.2 RCDATA state
    RCDATA = "rcdata",

    /// 13.2.5.3 RAWTEXT state
    RAWTEXT = "rawtext",

    /// 13.2.5.6 Tag open state
    TagOpen = "tag-open",

    /// 13.2.5.7 End tag open state
    EndTagOpen = "end-tag-open",

    /// 13.2.5.8 Tag name state
    TagName = "tag-name",

    /// 13.2.5.9 RCDATA less-than sign state
    RCDATALessThanSign = "rcdata-less-than-sign",

    /// 13.2.5.10 RCDATA end tag open state
    RCDATAEndTagOpen = "rcdata-end-tag-open",

    /// 13.2.5.11 RCDATA end tag name state
    RCDATAEndTagName = "rcdata-end-tag-name",

    /// 13.2.5.12 RAWTEXT less-than sign state
    RAWTEXTLessThanSign = "rawtext-less-than-sign",

    /// 13.2.5.32 Before attribute name state
    BeforeAttributeName = "before-attribute-name",

    /// 13.2.5.33 Attribute name state
    AttributeName = "attribute-name",

    /// 13.2.5.34 After attribute name state
    AfterAttributeName = "after-attribute-name",

    /// 13.2.5.35 Before attribute value state
    BeforeAttributeValue = "before-attribute-value",

    /// 13.2.5.36 Attribute value (double-quoted) state
    AttributeValueDoubleQuoted = "attribute-value-double-quoted",

    /// 13.2.5.37 Attribute value (single-quoted) state
    AttributeValueSingleQuoted = "attribute-value-single-quoted",

    /// 13.2.5.38 Attribute value (unquoted) state
    AttributeValueUnquoted = "attribute-value-unquoted",

    /// 13.2.5.39 After attribute value (quoted) state
    AfterAttributeValueQuoted = "after-attribute-value-quoted",

    /// 13.2.5.40 Self-closing start tag state
    SelfClosingStartTag = "self-closing-start-tag",

    /// 13.2.5.41 Bogus comment state
    BogusComment = "bogus-comment",

    /// 13.2.5.42 Markup declaration open state
    MarkupDeclarationOpen = "markup-declaration-open",

    /// 13.2.5.43 Comment start state
    CommentStart = "comment-start",

    /// 13.2.5.44 Comment start dash state
    CommentStartDash = "comment-start-dash",

    /// 13.2.5.45 Comment state
    Comment = "comment",

    /// 13.2.5.46 Comment less-than sign state
    CommentLessThanSign = "comment-less-than-sign",

    /// 13.2.5.47 Comment less-than sign bang state
    CommentLessThanSignBang = "comment-less-than-sign-bang",

    /// 13.2.5.48 Comment less-than sign bang dash state
    CommentLessThanSignBangDash = "comment-less-than-sign-bang-dash",

    /// 13.2.5.49 Comment less-than sign bang dash dash state
    CommentLessThanSignBangDashDash = "comment-less-than-sign-bang-dash-dash",

    /// 13.2.5.50 Comment end dash state
    CommentEndDash = "comment-end-dash",

    /// 13.2.5.51 Comment end state
    CommentEnd = "comment-end",

    /// 13.2.5.52 Comment end bang state
    CommentEndBang = "comment-end-bang",

    /// 13.2.5.53 DOCTYPE state
    DOCTYPE = "doctype",

    /// 13.2.5.54 Before DOCTYPE name state
    BeforeDOCTYPEName = "before-doctype-name",

    /// 13.2.5.55 DOCTYPE name state
    DOCTYPEName = "doctype-name",

    /// 13.2.5.56 After DOCTYPE name state
    AfterDOCTYPEName = "after-doctype-name",

    /// 13.2.5.57 After DOCTYPE public keyword state
    AfterDOCTYPEPublicKeyword = "after-doctype-public-keyword",

    /// 13.2.5.58 Before DOCTYPE public identifier state
    BeforeDOCTYPEPublicIdentifier = "before-doctype-public-identifier",

    /// 13.2.5.59 DOCTYPE public identifier (double-quoted) state
    DOCTYPEPublicIdentifierDoubleQuoted = "doctype-public-identifier-double-quoted",

    /// 13.2.5.60 DOCTYPE public identifier (single-quoted) state
    DOCTYPEPublicIdentifierSingleQuoted = "doctype-public-identifier-single-quoted",

    /// 13.2.5.61 After DOCTYPE public identifier state
    AfterDOCTYPEPublicIdentifier = "after-doctype-public-identifier",

    /// 13.2.5.62 Between DOCTYPE public and system identifiers state
    BetweenDOCTYPEPublicAndSystemIdentifiers = "between-doctype-public-and-system-identifiers",

    /// 13.2.5.63 After DOCTYPE system keyword state
    AfterDOCTYPESystemKeyword = "after-doctype-system-keyword",

    /// 13.2.5.64 Before DOCTYPE system identifier state
    BeforeDOCTYPESystemIdentifier = "before-doctype-system-identifier",

    /// 13.2.5.65 DOCTYPE system identifier (double-quoted) state
    DOCTYPESystemIdentifierDoubleQuoted = "doctype-system-identifier-double-quoted",

    /// 13.2.5.66 DOCTYPE system identifier (single-quoted) state
    DOCTYPESystemIdentifierSingleQuoted = "doctype-system-identifier-single-quoted",

    /// 13.2.5.67 After DOCTYPE system identifier state
    AfterDOCTYPESystemIdentifier = "after-doctype-system-identifier",

    /// 13.2.5.68 Bogus DOCTYPE state
    BogusDOCTYPE = "bogus-doctype",

    /// 13.2.5.72 Character reference state
    CharacterReference = "character-reference",

    /// 13.2.5.73 Named character reference state
    NamedCharacterReference = "named-character-reference",

    /// 13.2.5.74 Ambiguous ampersand state
    AmbiguousAmpersand = "ambiguous-ampersand",

    /// 13.2.5.75 Numeric character reference state
    NumericCharacterReference = "numeric-character-reference",

    /// 13.2.5.76 Hexadecimal character reference start state
    HexadecimalCharacterReferenceStart = "hexadecimal-character-reference-start",

    /// 13.2.5.77 Decimal character reference start state
    DecimalCharacterReferenceStart = "decimal-character-reference-start",

    /// 13.2.5.78 Hexadecimal character reference state
    HexadecimalCharacterReference = "hexadecimal-character-reference",

    /// 13.2.5.79 Decimal character reference state
    DecimalCharacterReference = "decimal-character-reference",

    /// 13.2.5.80 Numeric character reference end state
    NumericCharacterReferenceEnd = "numeric-character-reference-end"
}

enum HTMLStateIterator {
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
            output_tokens: VecDeque::default(),
            temporary_buffer: String::default(),
            named_character_reference_code:
                NamedCharacterReferences::entities(),
            character_reference_code: 0,
            last_start_tag_token: None,
        }
    }
}

impl<C> Tokenizer<C>
where
    C: Iterator<Item = char>,
{
    pub fn current_token(&mut self) -> Option<HTMLToken> {
        if let Some(token) = self.token.clone() {
            self.output_tokens.push_back(token);
        }
        self.pop_token()
    }

    pub fn next_token(&mut self) -> Option<HTMLToken> {
        self.next()
    }

    fn pop_token(&mut self) -> Option<HTMLToken> {
        self.output_tokens.pop_front()
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

    fn emit_each_characters_of_temporary_buffer(&mut self) -> &mut Self {
        self.temporary_buffer.chars().for_each(|ch| {
            self.output_tokens.push_back(HTMLToken::Character(ch));
        });
        self
    }

    fn emit_token(&mut self, token: HTMLToken) -> &mut Self {
        self.output_tokens.push_front(token);
        self
    }

    fn set_token(&mut self, token: HTMLToken) -> &mut Self {
        self.token = Some(token);
        self
    }

    /// Lorsqu'un état indique de reprendre (re-consommer) un caractère
    /// correspondant dans un état spécifié, cela signifie de passer à
    /// cet état, mais lorsqu'il tente de consommer le prochain caractère,
    /// de lui fournir le caractère actuel à la place.
    fn reconsume(&mut self, state: &str) -> &mut Self {
        self.stream.rollback();
        self.state.switch_to(state);
        self
    }

    fn switch_state_to(&mut self, state: &str) -> &mut Self {
        self.state.switch_to(state);
        self
    }

    fn set_temporary_buffer(
        &mut self,
        temporary_buffer: String,
    ) -> &mut Self {
        self.temporary_buffer = temporary_buffer;
        self
    }

    fn append_character_to_temporary_buffer(
        &mut self,
        ch: char,
    ) -> &mut Self {
        self.temporary_buffer.push(ch);
        self
    }

    fn flush_temporary_buffer(&mut self) -> &mut Self {
        if self.state.is_character_of_attribute() {
            self.temporary_buffer.clone().chars().for_each(|ch| {
                self.change_current_token(|tok| {
                    tok.append_character_to_attribute_value(ch);
                });
            });
        } else {
            self.temporary_buffer.chars().for_each(|c| {
                self.output_tokens.push_back(HTMLToken::Character(c))
            });
        }
        self
    }

    /// Un jeton `end-tag` approprié est un jeton de `end-tag` dont le nom
    /// de balise correspond au nom de balise de la dernière balise de
    /// début qui a été émise par ce tokenizer, le cas échéant.
    /// Si aucune balise de début n'a été émise par ce tokenizer, alors
    /// aucune balise de fin n'est appropriée.
    fn is_appropriate_end_tag(&self) -> bool {
        if let (
            Some(HTMLToken::EndTag {
                name: current_tag_name,
                ..
            }),
            Some(HTMLToken::EndTag {
                name: last_tag_name,
                ..
            }),
        ) = (self.token.as_ref(), self.last_start_tag_token.as_ref())
        {
            current_tag_name == last_tag_name
        } else {
            false
        }
    }
}

impl HTMLState {
    /// Change l'état actuel par un nouvel état.
    /// Terme `switch_to` venant de la spécification HTML "Switch to the
    /// ..."
    fn switch_to(&mut self, state: &str) -> &mut Self {
        let mut to: Cow<str> = Cow::default();

        if state == "return-state" {
            if let Some(return_state) = self.returns.clone() {
                to = Cow::from(return_state.to_string());
            }
        } else {
            to = Cow::from(state);
        }

        self.current = to.parse().unwrap();
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

    fn is_character_of_attribute(&self) -> bool {
        matches!(
            self.returns,
            Some(State::AttributeValueDoubleQuoted)
                | Some(State::AttributeValueSingleQuoted)
                | Some(State::AttributeValueUnquoted)
        )
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
                self.and_emit_with_error("unexpected-null-character")
            }

            // EOF
            //
            // Émettre un jeton `end of file`.
            | None => self.set_token(HTMLToken::EOF).and_emit(),

            // Anything else
            //
            // Émettre le caractère actuel comme un jeton `character`.
            | Some(ch) => {
                self.set_token(HTMLToken::Character(ch)).and_emit()
            }
        }
    }

    fn handle_rcdata_state(&mut self) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+0026 AMPERSAND (&)
            // Définir l'état de retour à l'état `rcdata`. Passez à l'état
            // `character-reference`.
            | Some('&') => self
                .state
                .set_return("rcdata")
                .switch_to("character-reference")
                .and_continue(),

            // U+003C LESS-THAN SIGN (<)
            //
            // Passer à l'état `rcdata-less-than-sign`.
            | Some('<') => self
                .switch_state_to("rcdata-less-than-sign")
                .and_continue(),

            // U+0000 NULL
            //
            // Il s'agit d'une erreur d'analyse de type
            // `unexpected-null-character`. Émettre un jeton `character`
            // U+FFFD REPLACEMENT CHARACTER.
            | Some('\0') => self
                .set_token(HTMLToken::Character(
                    char::REPLACEMENT_CHARACTER,
                ))
                .and_emit_with_error("unexpected-null-character"),

            // EOF
            //
            // Émettre un token `end of file`.
            | None => self.set_token(HTMLToken::EOF).and_emit(),

            // Anything else
            //
            // Émettre le caractère actuel comme un jeton `character`.
            | Some(ch) => {
                self.set_token(HTMLToken::Character(ch)).and_emit()
            }
        }
    }

    fn handle_rawtext_state(&mut self) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+003C LESS-THAN SIGN (<)
            // Passez à l'état `rawtext-less-than-sign`.
            | Some('<') => self
                .state
                .switch_to("rawtext-less-than-sign")
                .and_continue(),

            // U+0000 NULL
            //
            // Il s'agit d'une erreur d'analyse de type
            // `unexpected-null-character`. Émettre un jeton `character`
            // U+FFFD REPLACEMENT CHARACTER.
            | Some('\0') => self
                .set_token(HTMLToken::Character(
                    char::REPLACEMENT_CHARACTER,
                ))
                .and_emit_with_error("unexpected-null-character"),

            // EOF
            //
            // Émettre un jeton `end of file`
            | None => self.set_token(HTMLToken::EOF).and_emit(),

            // Anything else
            //
            // Émettre le caractère actuel comme un jeton `character`.
            | Some(ch) => {
                self.set_token(HTMLToken::Character(ch)).and_emit()
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
                .and_emit_with_error(
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
                .and_emit_with_error("eof-before-tag-name"),

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
                .and_emit_with_error("eof-before-tag-name"),

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
            | Some('>') => self.state.switch_to("data").and_emit(),

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
                .and_emit_with_error("eof-in-tag"),

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

    fn handle_rcdata_less_than_sign_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+002F SOLIDUS (/)
            //
            // Définir le tampon temporaire à une chaîne de caractères
            // vide. Passer à l'état `rcdata-end-tag-open`.
            | Some('/') => self
                .set_temporary_buffer(String::new())
                .switch_state_to("rcdata-end-tag-open")
                .and_continue(),

            // Anything else
            //
            // Émettre un jeton de caractère U+003C LESS-THAN SIGN.
            // Reprendre dans l'état `rcdata`.
            | _ => self
                .set_token(HTMLToken::Character('<'))
                .and_emit_current_token()
                .reconsume("rcdata")
                .and_continue(),
        }
    }

    fn handle_rcdata_end_tag_open_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // ASCII alpha
            //
            // Créer un nouveau jeton `end-tag`, définir son nom comme une
            // chaîne de caractères vide. Reprendre l'état
            // `rcdata-end-tag-name`.
            | Some(ch) if ch.is_ascii_alphabetic() => self
                .set_token(HTMLToken::new_end_tag(String::new()))
                .reconsume("rcdata-end-tag-name")
                .and_continue(),

            // Anything else
            //
            // Émettre un jeton `character` U+003C LESS-THAN SIGN et un
            // jeton de `character` U+002F SOLIDUS. Reprendre
            // dans l'état `rcdata`.
            | _ => self
                .emit_token(HTMLToken::Character('<'))
                .emit_token(HTMLToken::Character('/'))
                .reconsume("rcdata")
                .and_continue(),
        }
    }

    fn handle_rcdata_end_tag_name_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Si le jeton `end tag` actuel est un jeton `end tag`
            // approprié, alors passer à l'état `before-attribute-name`.
            // Sinon, traitez-le comme indiqué dans l'entrée
            // "Anything else" ci-dessous.
            | Some(ch)
                if ch.is_html_whitespace()
                    && self.is_appropriate_end_tag() =>
            {
                self.state
                    .switch_to("before-attribute-name")
                    .and_continue()
            }

            // U+002F SOLIDUS (/)
            //
            // Si le jeton `end tag` actuel est un jeton `end tag`
            // approprié, alors passer à l'état `self-closing-start-tag`.
            // Sinon, traitez-le comme indiqué dans l'entrée
            // "Anything else" ci-dessous.
            | Some('/') if self.is_appropriate_end_tag() => self
                .state
                .switch_to("self-closing-start-tag")
                .and_continue(),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Si le jeton `end tag` actuel est un jeton `end tag`
            // approprié, alors passer à l'état `data` et émettre le jeton
            // courant. Sinon, traitez-le comme indiqué dans l'entrée
            // "Anything else" ci-dessous.
            | Some('>') if self.is_appropriate_end_tag() => {
                self.state.switch_to("data").and_emit()
            }

            // ASCII upper alpha
            //
            // Ajouter la version en minuscules du caractère actuel
            // (ajouter 0x0020 au point de code du caractère) au nom
            // de balise du jeton `tag` actuel. Ajouter le caractère
            // actuel au tampon temporaire.
            | Some(ch) if ch.is_ascii_uppercase() => self
                .change_current_token(|tag_tok| {
                    tag_tok.append_character(ch.to_ascii_lowercase());
                })
                .append_character_to_temporary_buffer(ch)
                .and_continue(),

            // ASCII lower alpha
            //
            // Ajouter le caractère actuel au nom de balise du jeton de
            // `tag` actuel. Ajoute le caractère d'entrée actuel au tampon
            // temporaire.
            | Some(ch) if ch.is_ascii_lowercase() => self
                .change_current_token(|tag_tok| {
                    tag_tok.append_character(ch);
                })
                .append_character_to_temporary_buffer(ch)
                .and_continue(),

            // Anything else
            //
            // Émettre un jeton `character` U+003C LESS-THAN SIGN, un jeton
            // `character` U+002F SOLIDUS et un jeton `character` pour
            // chacun des caractères du tampon temporaire (dans l'ordre où
            // ils ont été ajoutés au tampon). Reprendre dans l'état
            // `RCDATA`.
            | _ => self
                .emit_token(HTMLToken::Character('<'))
                .emit_token(HTMLToken::Character('/'))
                .emit_each_characters_of_temporary_buffer()
                .reconsume("rcdata")
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
                .and_emit_with_error(
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
            | Some('>') => self.state.switch_to("data").and_emit(),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-tag`.
            // Émettre un jeton `end of file`.
            | None => self
                .set_token(HTMLToken::EOF)
                .and_emit_with_error("eof-in-tag"),

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
            // Passer à l'état `attribute-value-single-quoted`.
            | Some('\'') => self
                .state
                .switch_to("attribute-value-single-quoted")
                .and_continue(),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Il s'agit d'une erreur d'analyse de type
            // `missing-attribute-value`. Passer à l'état `data`. Émettre
            // le jeton `tag` actuel.
            | Some('>') => {
                self.and_emit_with_error("missing-attribute-value")
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
                .and_emit_with_error("eof-in-tag"),

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
            | Some('>') => self.state.switch_to("data").and_emit(),

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
                .and_emit_with_error("unexpected-null-character"),

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
            | Some('>') => self.state.switch_to("data").and_emit(),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-tag`.
            // Émettre un jeton `end of file`.
            | None => self
                .set_token(HTMLToken::EOF)
                .and_emit_with_error("eof-in-tag"),

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
                .and_emit(),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-tag`.
            // Émettre un jeton `end of file`.
            | None => self
                .set_token(HTMLToken::EOF)
                .and_emit_with_error("eof-in-tag"),

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
        if let Some(word) = self.stream.peek_until::<String>(7) {
            // Correspondance ASCII insensible à la casse pour le mot
            // "DOCTYPE".
            //
            // Consommer ces caractères et passer à l'état `doctype`.
            if word.to_ascii_lowercase() == "doctype" {
                self.stream.advance(7);
                return self.switch_state_to("doctype").and_continue();
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
                // todo: adjusted current node
                self.stream.advance(7);
                return self
                    .set_token(HTMLToken::new_comment(word))
                    .switch_state_to("bogus-comment")
                    .and_continue_with_error("cdata-in-html-content");
            }
        }

        // Two U+002D HYPHEN-MINUS characters (-)
        //
        // Consommer ces deux caractères, créer un jeton `comment`
        // dont les données sont une chaîne de caractères vide, passer à
        // l'état `comment-start`.
        if let Some(word) = self.stream.peek_until::<String>(2) {
            if word == "--" {
                self.stream.advance(2);
                return self
                    .set_token(HTMLToken::new_comment(String::new()))
                    .switch_state_to("comment-start")
                    .and_continue();
            }
        }

        // Anything else
        //
        // Il s'agit d'une erreur d'analyse de type
        // `incorrectly-opened-comment`. Créer un jeton `comment` dont les
        // données sont une chaîne de caractères vide. Passer à l'état
        // `bogus-comment` (ne pas consommer dans l'état actuel).
        self.set_token(HTMLToken::new_comment(String::new()))
            .switch_state_to("bogus-comment")
            .and_continue_with_error("incorrectly-opened-comment")
    }

    fn handle_bogus_comment_state(&mut self) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+003E GREATER-THAN SIGN (>)
            //
            // Passer à l'état `data`. Émettre le jeton `comment` actuel.
            | Some('>') => self.state.switch_to("data").and_emit(),

            // EOF
            //
            // Émettre `comment`. Émettre un jeton `end of file`.
            | None => self
                .and_emit_current_token()
                .set_token(HTMLToken::EOF)
                .and_emit(),

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
                .and_emit_with_error("abrupt-closing-of-empty-comment"),

            // Anything else
            //
            // Reprendre dans l'état de commentaire.
            | _ => self.reconsume("comment").and_continue(),
        }
    }

    fn handle_comment_less_than_sign_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+0021 EXCLAMATION MARK (!)
            //
            // Ajouter le caractère actuel aux données du jeton `comment`.
            // Passer à l'état `comment-less-than-sign-bang`.
            | Some(ch @ '!') => self
                .change_current_token(|comment_tok| {
                    comment_tok.append_character(ch)
                })
                .switch_state_to("comment-less-than-sign-bang")
                .and_continue(),

            // U+003C LESS-THAN SIGN (<)
            //
            // Ajoute le caractère actuel aux données du jeton `comment`.
            | Some(ch @ '<') => self
                .change_current_token(|comment_tok| {
                    comment_tok.append_character(ch)
                })
                .and_continue(),

            // Anything else
            //
            // Reprendre dans l'état `comment`.
            | _ => self.reconsume("comment").and_continue(),
        }
    }

    fn handle_comment_less_than_sign_bang_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+002D HYPHEN-MINUS (-)
            //
            // Passez à l'état `comment-less-than-sign-bang-dash`.
            | Some('-') => self
                .state
                .switch_to("comment-less-than-sign-bang-dash")
                .and_continue(),

            // Anything else
            //
            // Reprendre dans l'état `comment`.
            | _ => self.reconsume("comment").and_continue(),
        }
    }

    fn handle_comment_less_than_sign_bang_dash_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+002D HYPHEN-MINUS (-)
            //
            // Passez à l'état `comment-less-than-sign-bang-dash-dash`.
            | Some('-') => self
                .state
                .switch_to("comment-less-than-sign-bang-dash-dash")
                .and_continue(),

            // Anything else
            //
            // Reprendre dans l'état `comment-end-dash`.
            | _ => self.reconsume("comment-end-dash").and_continue(),
        }
    }

    fn handle_comment_less_than_sign_bang_dash_dash_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+003E GREATER-THAN SIGN (>)
            // EOF
            //
            // Reprendre à l'état `comment-end`.
            | Some('-') | None => {
                self.reconsume("comment-end").and_continue()
            }

            // Anything else
            //
            // Il s'agit d'une erreur d'analyse de type `nested-comment`.
            // Reprenez à l'état `comment-end`.
            | Some(_) => self
                .reconsume("comment-end")
                .and_continue_with_error("nested-comment"),
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
                .and_emit_with_error("abrupt-closing-of-empty-comment"),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-comment`.
            // Émettre le jeton `comment` actuel. Émettre un jeton `end of
            // file`.
            | None => self
                .and_emit_current_token()
                .set_token(HTMLToken::EOF)
                .and_emit_with_error("eof-in-comment"),

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

    fn handle_comment_state(&mut self) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+003C LESS-THAN SIGN (<)
            //
            // Ajouter le caractère actuel aux données du jeton `comment`.
            // Passer à l'état `comment-less-than-sign`.
            | Some(ch @ '<') => self
                .change_current_token(|comment_tok| {
                    comment_tok.append_character(ch)
                })
                .switch_state_to("comment-less-than-sign")
                .and_continue(),

            // U+002D HYPHEN-MINUS (-)
            //
            // Passer à l'état `comment-end-dash`.
            | Some('-') => {
                self.state.switch_to("comment-end-dash").and_continue()
            }

            // U+0000 NULL
            //
            // Il s'agit d'une erreur d'analyse de type
            // `unexpected-null-character`. Ajouter un caractère U+FFFD
            // `REPLACEMENT_CHARACTER` aux données du jeton `comment`.
            | Some('\0') => self
                .change_current_token(|comment_tok| {
                    comment_tok
                        .append_character(char::REPLACEMENT_CHARACTER);
                })
                .and_continue_with_error("unexpected-null-character"),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-comment`.
            // Émettre le jeton `comment` actuel. Émettre un jeton `end of
            // file`.
            | None => self
                .and_emit_current_token()
                .set_token(HTMLToken::EOF)
                .and_emit_with_error("eof-in-comment"),

            // Anything else
            //
            // Ajouter le caractère actuel aux données du jeton `comment`.
            | Some(ch) => self
                .change_current_token(|comment_tok| {
                    comment_tok.append_character(ch);
                })
                .and_continue(),
        }
    }

    fn handle_comment_end_dash_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+002D HYPHEN-MINUS (-)
            //
            // Passer à l'état `comment-end`.
            | Some('-') => {
                self.state.switch_to("comment-end").and_continue()
            }

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-comment`.
            // Émettre le jeton `comment` actuel. Émettre un jeton ` end of
            // file`.
            | None => self
                .and_emit_current_token()
                .set_token(HTMLToken::EOF)
                .and_emit_with_error("eof-in-comment"),

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

    fn handle_comment_end_state(&mut self) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+003E GREATER-THAN SIGN (>)
            //
            // Passer à l'état `data`. Émettre le jeton `comment` actuel.
            | Some('>') => self.state.switch_to("data").and_emit(),

            // U+0021 EXCLAMATION MARK (!)
            //
            // Passez à l'état `comment-end-bang`.
            | Some('!') => {
                self.state.switch_to("comment-end-bang").and_continue()
            }

            // U+002D HYPHEN-MINUS (-)
            //
            // Ajouter un caractère U+002D HYPHEN-MINUS (-) aux données du
            // jeton `comment` actuel.
            | Some(ch @ '-') => self
                .change_current_token(|comment_tok| {
                    comment_tok.append_character(ch);
                })
                .and_continue(),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-comment`.
            // Émettre le jeton `comment` actuel. Émettre un jeton `end of
            // file`.
            | None => self
                .and_emit_current_token()
                .set_token(HTMLToken::EOF)
                .and_emit_with_error("eof-in-comment"),

            // Anything else
            //
            // Ajouter deux caractères U+002D HYPHEN-MINUS (-) aux données
            // du jeton `comment`. Reprendre l'état `comment`.
            | Some(_) => self
                .change_current_token(|comment_tok| {
                    comment_tok.append_character('-');
                    comment_tok.append_character('-');
                })
                .reconsume("comment")
                .and_continue(),
        }
    }

    fn handle_comment_end_bang_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // U+002D HYPHEN-MINUS (-)
            //
            // Ajouter deux caractères U+002D HYPHEN-MINUS (-) et un
            // caractère U+0021 EXCLAMATION MARK (!) aux données du jeton
            // `comment`. Passer à l'état `comment-end-dash`.
            | Some('-') => self
                .change_current_token(|comment_tok| {
                    comment_tok.append_character('-');
                    comment_tok.append_character('-');
                    comment_tok.append_character('!');
                })
                .switch_state_to("comment-end-dash")
                .and_continue(),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Il s'agit d'une erreur d'analyse de type
            // `incorrectly-closed-comment`. Passez à l'état `data`.
            // Émettre le jeton `comment` actuel.
            | Some('>') => self
                .switch_state_to("data")
                .and_emit_with_error("incorrectly-closed-comment"),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-comment`.
            // Émettre le jeton `comment` actuel. Émettre un jeton `end of
            // file`.
            | None => self
                .and_emit_current_token()
                .set_token(HTMLToken::EOF)
                .and_emit_with_error("eof-in-comment"),

            // Anything else
            //
            // Ajouter deux caractères U+002D HYPHEN-MINUS (-) et un
            // caractère U+0021 EXCLAMATION MARK (!) aux données du jeton
            // de commentaire. Reprendre dans l'état `comment`.
            | Some(_) => self
                .change_current_token(|comment_tok| {
                    comment_tok.append_character('-');
                    comment_tok.append_character('-');
                    comment_tok.append_character('!');
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
                .and_emit_with_error("eof-in-doctype"),

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
                .and_emit_with_error("missing-doctype-name"),

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
                .and_emit_with_error("eof-in-doctype"),

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
            | Some('>') => self.state.switch_to("data").and_emit(),

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
                .and_emit_with_error("eof-in-doctype"),

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
            | Some('>') => self.state.switch_to("data").and_emit(),

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
                .and_emit_with_error("eof-in-doctype"),

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

                if let Some(word) = self.stream.peek_until::<String>(5) {
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
                .and_emit_with_error("missing-doctype-public-identifier"),

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
                .and_emit_with_error("eof-in-doctype"),

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
                .and_emit_with_error("missing-doctype-public-identifier"),

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
                .and_emit_with_error("eof-in-doctype"),

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
                .and_emit_with_error("abrupt-doctype-public-identifier"),

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
                .and_emit_with_error("eof-in-doctype"),

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
            | Some('>') => self.state.switch_to("data").and_emit(),

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
                .and_emit_with_error("eof-in-doctype"),

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
            | Some('>') => self.state.switch_to("data").and_emit(),

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
                .and_emit_with_error("eof-in-doctype"),

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
                .and_emit_with_error("missing-doctype-system-identifier"),

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
                .and_emit_with_error("eof-in-doctype"),

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
                .and_emit_with_error("missing-doctype-system-identifier"),

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
                .and_emit_with_error("eof-in-doctype"),

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
                .and_emit_with_error("abrupt-doctype-system-identifier"),

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
                .and_emit_with_error("eof-in-doctype"),

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
            | Some('>') => self.state.switch_to("data").and_emit(),

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
                .and_emit_with_error("eof-in-doctype"),

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
            | Some('>') => self.state.switch_to("data").and_emit(),

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
                .and_emit(),

            // Anything else
            //
            // Ignorer le caractère
            | Some(_) => self.ignore(),
        }
    }

    fn handle_character_reference_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        self.set_temporary_buffer(String::new())
            .append_character_to_temporary_buffer('&');

        match self.stream.next_input_char() {
            // ASCII alphanumeric
            //
            // Reprendre dans l'état `named-character-reference`.
            | Some(ch) if ch.is_ascii_alphanumeric() => {
                self.reconsume("named-character-reference").and_continue()
            }

            // U+0023 NUMBER SIGN (#)
            //
            // Ajouter le caractère actuel au tampon temporaire.
            // Passer à l'état `numeric-character-reference`.
            | Some(ch @ '#') => self
                .append_character_to_temporary_buffer(ch)
                .switch_state_to("numeric-character-reference")
                .and_continue(),

            // Anything else
            //
            // Flush code points consumed as a character reference.
            // Reconsume in the return state.
            | _ => self
                .flush_temporary_buffer()
                .reconsume("return-state")
                .and_continue(),
        }
    }

    /// Consomme le nombre maximum de caractères possible, où les
    /// caractères consommés sont l'un des identifiants de la première
    /// colonne de la table des références de caractères nommés. Ajouter
    /// chaque caractère au tampon temporaire lorsqu'il est consommé.
    fn handle_named_character_reference_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        let ch = self.stream.current.expect("le caractère actuel");
        let rest_of_chars = self.stream.peek_until_end::<String>();
        let full_str = format!("{ch}{rest_of_chars}");

        let entities = &self.named_character_reference_code;

        let (maybe_result, max_size) = entities.iter().fold(
            (None, 0),
            |(mut maybe_result, mut max_size), item| {
                let name = item.0;
                let size_name = name.len();

                if full_str.starts_with(name) && size_name > max_size {
                    max_size = size_name;
                    maybe_result = Some(item);
                }

                (maybe_result, max_size)
            },
        );

        match maybe_result {
            | Some((entity_name, entity)) => {
                // Consomme tous les caractères trouvés
                entity_name.chars().for_each(|ch| {
                    self.stream.next();
                    self.temporary_buffer.push(ch);
                });

                let mut maybe_err = None;
                if ch != ';' {
                    maybe_err =
                        "missing-semicolon-after-character-reference"
                            .into();

                    if let Some(ch) = full_str.chars().nth(max_size - 1) {
                        if (ch == '=' || ch.is_ascii_alphanumeric())
                            && self.state.is_character_of_attribute()
                        {
                            {
                                return self
                                    .flush_temporary_buffer()
                                    .switch_state_to("return-state")
                                    .and_continue();
                            }
                        }
                    }
                }

                self.temporary_buffer.clear();

                entity.codepoints.iter().for_each(|&cp| {
                    let ch = char::from_u32(cp).expect("un caractère");
                    self.temporary_buffer.push(ch);
                });

                self.flush_temporary_buffer()
                    .switch_state_to("return-state");
                if let Some(err) = maybe_err {
                    self.and_continue_with_error(err)
                } else {
                    self.and_continue()
                }
            }
            | None => self
                .flush_temporary_buffer()
                .switch_state_to("ambiguous-ampersand")
                .and_continue(),
        }
    }

    fn handle_ambiguous_ampersand_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // ASCII alphanumeric
            //
            // Si la référence de caractère a été consommée dans le cadre
            // d'un attribut, alors ajouter le caractère actuel à la valeur
            // de l'attribut actuel. Sinon, émettre le caractère actuel
            // comme un jeton `character`.
            | Some(ch) if ch.is_ascii_alphanumeric() => {
                if self.state.is_character_of_attribute() {
                    self.change_current_token(|tok| {
                        tok.append_character_to_attribute_value(ch);
                    })
                    .and_continue()
                } else {
                    self.set_token(HTMLToken::Character(ch)).and_emit()
                }
            }

            // U+003B SEMICOLON (;)
            //
            // Il s'agit d'une erreur d'analyse de type
            // `unknown-named-character-reference`. Reprendre dans l'état
            // `return-state`.
            | Some(';') => {
                self.reconsume("return-state").and_continue_with_error(
                    "unknown-named-character-reference",
                )
            }

            // Anything else
            //
            // Reprendre dans l'état `return-state`.
            | _ => self.reconsume("return-state").and_continue(),
        }
    }

    fn handle_numeric_character_reference_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        // Définir le code de référence du caractère à zéro (0).
        self.character_reference_code = 0;

        match self.stream.next_input_char() {
            // U+0078 LATIN SMALL LETTER X
            // U+0058 LATIN CAPITAL LETTER X
            //
            // Ajouter le caractère actuel au tampon temporaire.
            // Passer à l'état `hexadecimal-character-reference-start`.
            | Some(ch @ ('x' | 'X')) => self
                .append_character_to_temporary_buffer(ch)
                .switch_state_to("hexadecimal-character-reference-start")
                .and_continue(),

            // Anything else
            | _ => self
                .reconsume("decimal-character-reference-start")
                .and_continue(),
        }
    }

    fn handle_hexadecimal_character_reference_start_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // ASCII hex digit
            //
            // Reprendre dans l'état `hexadecimal-character-reference`.
            | Some(ch) if ch.is_ascii_hexdigit() => self
                .reconsume("hexadecimal-character-reference")
                .and_continue(),

            // Anything else
            //
            // Il s'agit d'une erreur d'analyse de type
            // `absence-of-digits-in-numeric-character-reference`. Videz
            // les points de code consommés comme référence de caractère.
            // Reprendre dans l'état `return-state`.
            | _ => self
                .flush_temporary_buffer()
                .reconsume("return-state")
                .and_continue_with_error(
                    "absence-of-digits-in-numeric-character-reference",
                ),
        }
    }

    fn handle_decimal_character_reference_start_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // ASCII digit
            //
            // Reprendre dans l'état `decimal-character-reference`.
            | Some(ch) if ch.is_ascii_digit() => self
                .reconsume("decimal-character-reference")
                .and_continue(),

            // Anything else
            //
            // Il s'agit d'une erreur d'analyse de type
            // `absence-of-digits-in-numeric-character-reference`. Videz
            // les points de code consommés comme référence de caractère.
            // Reprendre dans l'état `return-state`.
            | _ => self
                .flush_temporary_buffer()
                .reconsume("return-state")
                .and_continue_with_error(
                    "absence-of-digits-in-numeric-character-reference",
                ),
        }
    }

    fn handle_hexadecimal_character_reference_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // ASCII digit
            //
            // Multiplier le code de référence du caractère par 16. Ajouter
            // une version numérique du caractère actuel
            // (soustraire 0x0030 du point de code du caractère) au code de
            // référence du caractère.
            | Some(ch) if ch.is_ascii_digit() => {
                self.character_reference_code *= 16;
                self.character_reference_code +=
                    ((ch as u8) - 0x0030) as u32;
                self.and_continue()
            }

            // ASCII upper hex digit
            //
            // Multiplier le code de référence du caractère par 16. Ajouter
            // une version numérique du caractère actuel sous
            // forme de chiffre hexadécimal (soustraire 0x0037 du point de
            // code du caractère) au code de référence du caractère.
            | Some(ch)
                if ch.is_ascii_hexdigit() && ch.is_ascii_uppercase() =>
            {
                self.character_reference_code *= 16;
                self.character_reference_code +=
                    ((ch as u8) - 0x0037) as u32;
                self.and_continue()
            }

            // ASCII lower hex digit
            //
            // Multiplier le code de référence du caractère par 16. Ajouter
            // une version numérique du caractère actuel sous
            // forme de chiffre hexadécimal (soustraire 0x0057 du point de
            // code du caractère) au code de référence du caractère.
            | Some(ch)
                if ch.is_ascii_hexdigit() && ch.is_ascii_lowercase() =>
            {
                self.character_reference_code *= 16;
                self.character_reference_code +=
                    ((ch as u8) - 0x0057) as u32;
                self.and_continue()
            }

            // U+003B SEMICOLON
            //
            // Passer à l'état `numeric-character-reference-end`.
            | Some(';') => self
                .switch_state_to("numeric-character-reference-end")
                .and_continue(),

            // Anything else
            //
            // Il s'agit d'une erreur d'analyse de type
            // `missing-semicolon-after-character-reference`. Reprendre
            // dans l'état `numeric-character-reference-end`.
            | _ => self
                .reconsume("numeric-character-reference-end")
                .and_continue_with_error(
                    "missing-semicolon-after-character-reference",
                ),
        }
    }

    fn handle_decimal_character_reference_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        match self.stream.next_input_char() {
            // ASCII digit
            //
            // Multiplier le code de référence du caractère par 10. Ajouter
            // une version numérique du caractère actuel (soustraire 0x0030
            // du point de code du caractère) au code de référence du
            // caractère.
            | Some(ch) if ch.is_ascii_digit() => {
                self.character_reference_code *= 10;
                self.character_reference_code +=
                    ((ch as u8) - 0x0030) as u32;
                self.and_continue()
            }

            // U+003B SEMICOLON
            | Some(';') => self
                .switch_state_to("numeric-character-reference-end")
                .and_continue(),

            // Anything else
            //
            // Il s'agit d'une erreur d'analyse de type
            // `missing-semicolon-after-character-reference`. Reprendre
            // dans l'état `numeric-character-reference-end`.
            | _ => self
                .reconsume("numeric-character-reference-end")
                .and_continue_with_error(
                    "missing-semicolon-after-character-reference",
                ),
        }
    }

    fn handle_numeric_character_reference_end_state(
        &mut self,
    ) -> ResultHTMLStateIterator {
        let mut err: Option<&str> = None;

        match self.character_reference_code {
            // Si le nombre est 0x00, il s'agit d'une erreur d'analyse de
            // type `null-character-reference`. Définir le code de
            // référence du caractère à 0xFFFD.
            | 0x00 => {
                err = "null-character-reference".into();
                self.character_reference_code = 0xFFFD;
            }

            // Si le nombre est supérieur à 0x10FFFF, il s'agit d'une
            // erreur d'analyse de référence de caractère hors
            // de la plage unicode. Définissez le code de
            // référence du caractère à 0xFFFD.
            | crc if crc > 0x10FFFF => {
                err = "character-reference-outside-unicode-range".into();
                self.character_reference_code = 0xFFFD;
            }

            // Si le nombre est un substitut, il s'agit d'une erreur
            // d'analyse de type `surrogate-character-reference`.
            // Définir le code de référence du caractère à 0xFFFD.
            | crc if (crc as u8 as char).is_surrogate() => {
                err = "surrogate-character-reference".into();
                self.character_reference_code = 0xFFFD;
            }

            // Si le nombre n'est pas un caractère, il s'agit d'une erreur
            // d'analyse de type `noncharacter-character-reference`.
            | crc if (crc as u8 as char).is_noncharacter() => {
                err = "noncharacter-character-reference".into();
            }

            // Si le nombre est 0x0D, ou un contrôle qui n'est pas un
            // espace ASCII, il s'agit d'une erreur d'analyse de référence
            // de caractère de contrôle. Si le nombre est l'un des nombres
            // de la première colonne du tableau suivant, trouvez la ligne
            // avec ce nombre dans la première colonne, et définissez le
            // code de référence de caractère au nombre de la deuxième
            // colonne de cette ligne.
            | crc if crc == 0x0D
                || ((crc as u8 as char).is_control()
                    && !(crc as u8 as char).is_whitespace()) =>
            {
                err = "control-character-reference".into();
            }
            | _ => {}
        }

        let ch = char::from_u32(self.character_reference_code)
            .unwrap_or(char::REPLACEMENT_CHARACTER);
        self.temporary_buffer.clear();
        self.append_character_to_temporary_buffer(ch)
            .flush_temporary_buffer()
            .switch_state_to("return-state");

        if let Some(err) = err {
            self.and_continue_with_error(err)
        } else {
            self.and_continue()
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

    /// https://infra.spec.whatwg.org/#noncharacter
    fn is_noncharacter(&self) -> bool {
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

    fn is_surrogate(&self) -> bool {
        matches!(self, '\u{D_8000}'..='\u{D_FFFF}')
    }
}

impl<C> Iterator for Tokenizer<C>
where
    C: Iterator<Item = char>,
{
    type Item = HTMLToken;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.output_tokens.is_empty() {
            return self.pop_token();
        }

        loop {
            let state = match self.state.current {
                | State::Data => self.handle_data_state(),
                | State::RCDATA => self.handle_rcdata_state(),
                | State::RAWTEXT => self.handle_rawtext_state(),
                | State::TagOpen => self.handle_tag_open_state(),
                | State::EndTagOpen => self.handle_end_tag_open_state(),
                | State::TagName => self.handle_tag_name_state(),
                | State::RCDATALessThanSign => {
                    self.handle_rcdata_less_than_sign_state()
                }
                | State::RCDATAEndTagOpen => {
                    self.handle_rcdata_end_tag_open_state()
                }
                | State::RCDATAEndTagName => {
                    self.handle_rcdata_end_tag_name_state()
                }
                | State::RAWTEXTLessThanSign => todo!(),
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
                | State::AttributeValueSingleQuoted => {
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
                | State::CommentLessThanSign => {
                    self.handle_comment_less_than_sign_state()
                }
                | State::CommentLessThanSignBang => {
                    self.handle_comment_less_than_sign_bang_state()
                }
                | State::CommentLessThanSignBangDash => {
                    self.handle_comment_less_than_sign_bang_dash_state()
                }
                | State::CommentLessThanSignBangDashDash => {
                    self.handle_comment_less_than_sign_bang_dash_dash_state()
                }
                | State::CommentStartDash => {
                    self.handle_comment_start_dash_state()
                }
                | State::Comment => self.handle_comment_state(),
                | State::CommentEndDash => {
                    self.handle_comment_end_dash_state()
                }
                | State::CommentEnd => {
                    self.handle_comment_end_state()
                }
                | State::CommentEndBang => {
                    self.handle_comment_end_bang_state()
                }
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
                | State::CharacterReference => {
                    self.handle_character_reference_state()
                }
                | State::NamedCharacterReference => {
                    self.handle_named_character_reference_state()
                }
                | State::AmbiguousAmpersand => {
                    self.handle_ambiguous_ampersand_state()
                }
                | State::NumericCharacterReference => {
                    self.handle_numeric_character_reference_state()
                }
                | State::HexadecimalCharacterReferenceStart => {
                    self.handle_hexadecimal_character_reference_start_state()
                }
                | State::DecimalCharacterReferenceStart => {
                    self.handle_decimal_character_reference_start_state()
                }
                | State::HexadecimalCharacterReference => {
                    self.handle_hexadecimal_character_reference_state()
                }
                | State::DecimalCharacterReference => {
                    self.handle_decimal_character_reference_state()
                }
                | State::NumericCharacterReferenceEnd => {
                    self.handle_numeric_character_reference_end_state()
                }
                // | _ => return None,
            };

            match state {
                | Ok(HTMLStateIterator::Continue) => continue,
                | Ok(HTMLStateIterator::Break) => break,
                | Err((err, state)) => {
                    log::error!("[HTMLParserError]: {err}");
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
            Some(HTMLToken::Comment(" Hello World ".into()))
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

    // #[test]
    // fn test_site() {
    //     let html_tok =
    //         get_tokenizer_html(include_str!("crashtests/site.html.local"
    // ));
    //
    //     for tok in html_tok {
    //         if let HTMLToken::EOF = tok {
    //             break;
    //         }
    //         if let HTMLToken::Character(_) = tok {
    //             continue;
    //         }
    //
    //         println!("-> {tok:?}");
    //     }
    // }
}
