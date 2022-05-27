/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{borrow::Cow, collections::VecDeque};

use infra::{
    primitive::codepoint::{CodePoint, CodePointInterface},
    structure::lists::peekable::PeekableInterface,
};
use macros::dd;
use named_character_references::{
    NamedCharacterReferences, NamedCharacterReferencesEntities,
};
use parser::preprocessor::InputStream;

use super::{HTMLTagToken, HTMLToken};
use crate::error::HTMLParserError;

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

pub(crate) trait HTMLTokenizerProcessInterface {
    fn ignore(&self) -> HTMLTokenizerProcessResult {
        Ok(HTMLTokenizerProcessControlFlow::Continue)
    }

    fn and_continue(&self) -> HTMLTokenizerProcessResult {
        Ok(HTMLTokenizerProcessControlFlow::Continue)
    }

    fn and_continue_with_error(
        &self,
        err: &str,
    ) -> HTMLTokenizerProcessResult {
        let err = err.parse().unwrap();
        Err((err, HTMLTokenizerProcessControlFlow::Continue))
    }

    fn and_emit(&self) -> HTMLTokenizerProcessResult {
        Ok(HTMLTokenizerProcessControlFlow::Emit)
    }

    fn and_emit_with_error(
        &self,
        err: &str,
    ) -> HTMLTokenizerProcessResult {
        let err = err.parse().unwrap();
        Err((err, HTMLTokenizerProcessControlFlow::Emit))
    }
}

pub(crate) enum HTMLTokenizerProcessControlFlow {
    Continue,
    Emit,
}

pub(crate) type HTMLInputStream<Iter> = InputStream<Iter, CodePoint>;

pub(crate) type HTMLTokenizerProcessResult = Result<
    HTMLTokenizerProcessControlFlow,
    (HTMLParserError, HTMLTokenizerProcessControlFlow),
>;

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
pub struct HTMLTokenizer<Chars>
where
    Chars: Iterator<Item = CodePoint>,
{
    pub(crate) stream: HTMLInputStream<Chars>,

    /// Le jeton courant.
    token: Option<HTMLToken>,

    state: HTMLTokenizerState,

    /// La sortie de l'étape de tokenisation est une série de zéro ou plus
    /// des jetons.
    output_tokens: VecDeque<HTMLToken>,

    named_character_reference_code: NamedCharacterReferencesEntities,

    /// Certains états utilisent un tampon temporaire pour suivre leur
    /// progression.
    pub(crate) temporary_buffer: String,

    /// [L'état de référence du caractère](State::CharacterReference)
    /// utilise un [état de retour](HTMLTokenizerState::returns) pour
    /// revenir à un état à partir duquel il a été invoqué.
    character_reference_code: u32,

    last_start_tag_token: Option<HTMLToken>,
}

type Tokenizer<C> = HTMLTokenizer<C>;

#[derive(Debug)]
#[derive(Clone)]
pub struct HTMLTokenizerState {
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

    /// 13.2.5.4 Script data state
    ScriptData = "script-data",

    /// 13.2.5.5 PLAINTEXT state
    PLAINTEXT = "plaintext",

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

    /// 13.2.5.13 RAWTEXT end tag open state
    RAWTEXTEndTagOpen = "rawtext-end-tag-open",

    /// 13.2.5.14 RAWTEXT end tag name state
    RAWTEXTEndTagName = "rawtext-end-tag-name",

    /// 13.2.5.15 Script data less-than sign state
    ScriptDataLessThanSign = "script-data-less-than-sign",

    /// 13.2.5.16 Script data end tag open state
    ScriptDataEndTagOpen = "script-data-end-tag-open",

    /// 13.2.5.17 Script data end tag name state
    ScriptDataEndTagName = "script-data-end-tag-name",

    /// 13.2.5.18 Script data escape start state
    ScriptDataEscapeStart = "script-data-escape-start",

    /// 13.2.5.19 Script data escape start dash state
    ScriptDataEscapeStartDash = "script-data-escape-start-dash",

    /// 13.2.5.20 Script data escaped state
    ScriptDataEscaped = "script-data-escaped",

    /// 13.2.5.21 Script data escaped dash state
    ScriptDataEscapedDash = "script-data-escaped-dash",

    /// 13.2.5.22 Script data escaped dash dash state
    ScriptDataEscapedDashDash = "script-data-escaped-dash-dash",

    /// 13.2.5.23 Script data escaped less-than sign state
    ScriptDataEscapedLessThanSign = "script-data-escaped-less-than-sign",

    /// 13.2.5.24 Script data escaped end tag open state
    ScriptDataEscapedEndTagOpen = "script-data-escaped-end-tag-open",

    /// 13.2.5.25 Script data escaped end tag name state
    ScriptDataEscapedEndTagName = "script-data-escaped-end-tag-name",

    /// 13.2.5.26 Script data double escape start state
    ScriptDataDoubleEscapeStart = "script-data-double-escape-start",

    /// 13.2.5.27 Script data double escaped state
    ScriptDataDoubleEscapedState = "script-data-double-escaped",

    /// 13.2.5.28 Script data double escaped dash state
    ScriptDataDoubleEscapedDash = "script-data-double-escaped-dash",

    /// 13.2.5.29 Script data double escaped dash dash state
    ScriptDataDoubleEscapedDashDash = "script-data-double-escaped-dash-dash",

    /// 13.2.5.30 Script data double escaped less-than sign state
    ScriptDataDoubleEscapedLessThanSign = "script-data-double-escaped-less-than-sign",

    /// 13.2.5.31 Script data double escape end state
    ScriptDataDoubleEscapeEnd = "script-data-double-escape-end",

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

    /// 13.2.5.69 CDATA section state
    CDATASection = "cdata-section",

    /// 13.2.5.70 CDATA section bracket state
    CDATASectionBracket = "cdata-section-bracket",

    /// 13.2.5.71 CDATA section end state
    CDATASectionEnd = "cdata-section-end",

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

// -------------- //
// Implémentation //
// -------------- //

impl<C> Tokenizer<C>
where
    C: Iterator<Item = CodePoint>,
{
    pub fn new(iter: C) -> Self {
        let stream = HTMLInputStream::new(iter);
        Self {
            stream,
            token: None,
            state: HTMLTokenizerState::default(),
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
    C: Iterator<Item = CodePoint>,
{
    pub fn current_token(&mut self) -> Option<HTMLToken> {
        if let Some(token) = self.token.to_owned() {
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

    pub(crate) fn change_current_token<F: FnOnce(&mut HTMLToken)>(
        &mut self,
        callback: F,
    ) -> &mut Self {
        if let Some(ref mut token) = self.token {
            callback(token);
        }
        self
    }

    pub(crate) fn and_emit_current_token(&mut self) -> &mut Self {
        if let Some(token) = self.current_token() {
            self.emit_token(token);
        }
        self
    }

    pub(crate) fn emit_each_characters_of_temporary_buffer(
        &mut self,
    ) -> &mut Self {
        self.temporary_buffer.chars().for_each(|ch| {
            self.output_tokens.push_back(HTMLToken::Character(ch));
        });
        self
    }

    pub(crate) fn emit_token(&mut self, token: HTMLToken) -> &mut Self {
        if matches!(token, HTMLToken::Character('<' | '/')) {
            self.last_start_tag_token = self.token.clone();
        }

        self.output_tokens.push_front(token);
        self
    }

    pub(crate) fn set_token(&mut self, token: HTMLToken) -> &mut Self {
        self.token.replace(token);
        self
    }

    /// Lorsqu'un état indique de reprendre (re-consommer) un caractère
    /// correspondant dans un état spécifié, cela signifie de passer à
    /// cet état, mais lorsqu'il tente de consommer le prochain caractère,
    /// de lui fournir le caractère actuel à la place.
    pub(crate) fn reconsume(&mut self, state: &str) -> &mut Self {
        self.stream.rollback();
        self.switch_state_to(state);
        self
    }

    pub(crate) fn set_return_state_to(
        &mut self,
        state: impl AsRef<str>,
    ) -> &mut Self {
        self.state.set_return(state.as_ref());
        self
    }

    pub(crate) fn switch_state_to(
        &mut self,
        state: impl AsRef<str>,
    ) -> &mut Self {
        self.state.switch_to(state.as_ref());
        self
    }

    pub(crate) fn set_temporary_buffer(
        &mut self,
        temporary_buffer: String,
    ) -> &mut Self {
        self.temporary_buffer = temporary_buffer;
        self
    }

    pub(crate) fn append_character_to_temporary_buffer(
        &mut self,
        ch: CodePoint,
    ) -> &mut Self {
        self.temporary_buffer.push(ch);
        self
    }

    pub(crate) fn flush_temporary_buffer(&mut self) -> &mut Self {
        if self.state.is_character_of_attribute() {
            self.temporary_buffer.to_owned().chars().for_each(|ch| {
                self.change_current_token(|token| {
                    token
                        .as_tag_mut()
                        .append_character_to_attribute_value(ch);
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
    pub(crate) fn is_appropriate_end_tag(&self) -> bool {
        if let (
            Some(HTMLToken::Tag(HTMLTagToken {
                name: current_tag_name,
                is_end: true,
                ..
            })),
            Some(HTMLToken::Tag(HTMLTagToken {
                name: last_tag_name,
                is_end: true,
                ..
            })),
        ) = (self.token.as_ref(), self.last_start_tag_token.as_ref())
        {
            current_tag_name == last_tag_name
        } else {
            false
        }
    }
}

impl HTMLTokenizerState {
    /// Change l'état actuel par un nouvel état.
    /// Terme `switch_to` venant de la spécification HTML "Switch to the
    /// ..."
    fn switch_to(&mut self, state: &str) -> &mut Self {
        let mut to: Cow<str> = Cow::default();

        if state == "return-state" {
            if let Some(return_state) = self.returns.to_owned() {
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

    const fn is_character_of_attribute(&self) -> bool {
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
    C: Iterator<Item = CodePoint>,
{
    fn handle_character_reference_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
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
    ) -> HTMLTokenizerProcessResult {
        let ch = self.stream.current.expect("le caractère actuel");
        let rest_of_chars =
            self.stream.meanwhile().peek_until_end::<String>();
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
                    let ch =
                        CodePoint::from_u32(cp).expect("un caractère");
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
    ) -> HTMLTokenizerProcessResult {
        match self.stream.next_input_char() {
            // ASCII alphanumeric
            //
            // Si la référence de caractère a été consommée dans le cadre
            // d'un attribut, alors ajouter le caractère actuel à la valeur
            // de l'attribut actuel. Sinon, émettre le caractère actuel
            // comme un jeton `character`.
            | Some(ch) if ch.is_ascii_alphanumeric() => {
                if self.state.is_character_of_attribute() {
                    self.change_current_token(|token| {
                        token
                            .as_tag_mut()
                            .append_character_to_attribute_value(ch);
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
    ) -> HTMLTokenizerProcessResult {
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
    ) -> HTMLTokenizerProcessResult {
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
            // `absence-of-digits-in-numeric-character-reference`. Vider
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
    ) -> HTMLTokenizerProcessResult {
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
            // `absence-of-digits-in-numeric-character-reference`. Vider
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
    ) -> HTMLTokenizerProcessResult {
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
    ) -> HTMLTokenizerProcessResult {
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
    ) -> HTMLTokenizerProcessResult {
        let mut err: Option<&str> = None;

        let cp = self.character_reference_code as u8 as CodePoint;

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
            // de la plage unicode. Définir le code de référence du
            // caractère à 0xFFFD.
            | crc if crc > 0x10FFFF => {
                err = "character-reference-outside-unicode-range".into();
                self.character_reference_code = 0xFFFD;
            }

            // Si le nombre est un substitut, il s'agit d'une erreur
            // d'analyse de type `surrogate-character-reference`.
            // Définir le code de référence du caractère à 0xFFFD.
            | _ if cp.is_surrogate() => {
                err = "surrogate-character-reference".into();
                self.character_reference_code = 0xFFFD;
            }

            // Si le nombre n'est pas un caractère, il s'agit d'une erreur
            // d'analyse de type `noncharacter-character-reference`.
            | _ if cp.is_noncharacter() => {
                err = "noncharacter-character-reference".into();
            }

            // Si le nombre est 0x0D, ou un contrôle qui n'est pas un
            // espace ASCII, il s'agit d'une erreur d'analyse de référence
            // de caractère de contrôle. Si le nombre est l'un des nombres
            // de la première colonne du tableau suivant, trouver la ligne
            // avec ce nombre dans la première colonne, et définir le
            // code de référence de caractère au nombre de la deuxième
            // colonne de cette ligne.
            | crc if crc == 0x0D
                || (cp.is_control() && !cp.is_ascii_whitespace()) =>
            {
                err = "control-character-reference".into();
            }
            | _ => {}
        }

        let ch = CodePoint::from_u32(self.character_reference_code)
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

impl<C> HTMLTokenizerProcessInterface for Tokenizer<C> where
    C: Iterator<Item = CodePoint>
{
}

impl HTMLTokenizerProcessInterface for HTMLTokenizerState {}

impl<C> Iterator for Tokenizer<C>
where
    C: Iterator<Item = CodePoint>,
{
    type Item = HTMLToken;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.output_tokens.is_empty() {
            return self.pop_token();
        }

        loop {
            let state = match dd!(&self.state.current) {
                | State::Data => self.handle_data_state(),
                | State::RCDATA => self.handle_rcdata_state(),
                | State::RAWTEXT => self.handle_rawtext_state(),
                | State::ScriptData => self.handle_script_data_state(),
                | State::PLAINTEXT => self.handle_plaintext_state(),
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
                | State::RAWTEXTLessThanSign => {
                    self.handle_rawtext_less_than_sign_state()
                }
                | State::RAWTEXTEndTagOpen => {
                    self.handle_rawtext_end_tag_open_state()
                }
                | State::RAWTEXTEndTagName => {
                    self.handle_rawtext_end_tag_name_state()
                }
                | State::ScriptDataLessThanSign => {
                    self.handle_script_data_less_than_sign_state()
                }
                | State::ScriptDataEndTagOpen => {
                    self.handle_script_data_end_tag_open_state()
                }
                | State::ScriptDataEndTagName => {
                    self.handle_script_data_end_tag_name_state()
                }
                | State::ScriptDataEscapeStart => {
                    self.handle_script_data_escape_start_state()
                }
                | State::ScriptDataEscapeStartDash => {
                    self.handle_script_data_escape_start_dash_state()
                }
                | State::ScriptDataEscaped => {
                    self.handle_script_data_escaped_state()
                }
                | State::ScriptDataEscapedDash => {
                    self.handle_script_data_escaped_dash_state()
                }
                | State::ScriptDataEscapedDashDash => {
                    self.handle_script_data_escaped_dash_dash_state()
                }
                | State::ScriptDataEscapedLessThanSign => {
                    self.handle_script_data_escaped_less_than_sign_state()
                }
                | State::ScriptDataEscapedEndTagOpen => {
                    self.handle_script_data_escaped_end_tag_open_state()
                }
                | State::ScriptDataEscapedEndTagName => {
                    self.handle_script_data_escaped_end_tag_name_state()
                }
                | State::ScriptDataDoubleEscapeStart => {
                    self.handle_script_data_double_escape_start_state()
                }
                | State::ScriptDataDoubleEscapedState => {
                    self.handle_script_data_double_escaped_state()
                }
                | State::ScriptDataDoubleEscapedDash => {
                    self.handle_script_data_double_escaped_dash_state()
                }
                | State::ScriptDataDoubleEscapedDashDash => {
                    self.handle_script_data_double_escaped_dash_dash_state()
                }
                | State::ScriptDataDoubleEscapedLessThanSign => {
                    self.handle_script_data_double_escaped_less_than_sign_state()
                }
                | State::ScriptDataDoubleEscapeEnd => {
                    self.handle_script_data_double_escape_end_state()
                }
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
                | State::CDATASection => self.handle_cdata_section_state(),
                | State::CDATASectionBracket => {
                    self.handle_cdata_section_bracket_state()
                }
                | State::CDATASectionEnd => {
                    self.handle_cdata_section_end_state()
                }
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
                | Ok(HTMLTokenizerProcessControlFlow::Continue) => {
                    continue
                }
                | Ok(HTMLTokenizerProcessControlFlow::Emit) => break,
                | Err((err, state)) => {
                    log::error!("[HTMLParserError]: {err}");
                    match state {
                        | HTMLTokenizerProcessControlFlow::Continue => {
                            continue
                        }
                        | HTMLTokenizerProcessControlFlow::Emit => break,
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

impl Default for HTMLTokenizerState {
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
    ) -> HTMLTokenizer<impl Iterator<Item = CodePoint>> {
        let stream = InputStream::new(input.chars());
        HTMLTokenizer::new(stream)
    }

    #[test]
    fn test_ambiguous_ampersand() {
        let mut token = get_tokenizer_html(include_str!(
            "../crashtests/tag/ambiguous_ampersand.html"
        ));

        let attr_name = "href";
        let attr_value =
            "?a=b&c=d&a0b=c&copy=1&noti=n&not=in&notin=&notin;&not;&;& &";

        assert_eq!(
            token.next_token(),
            Some(HTMLToken::Tag(
                HTMLTagToken::start()
                    .with_name("a")
                    .with_attributes([(attr_name, attr_value)])
            )),
        );
    }

    #[test]
    fn test_comment() {
        let mut token = get_tokenizer_html(include_str!(
            "../crashtests/comment/comment.html"
        ));

        assert_eq!(
            token.next_token(),
            Some(HTMLToken::Comment(" Hello World ".into()))
        );
    }

    #[test]
    fn test_tag() {
        let mut token =
            get_tokenizer_html(include_str!("../crashtests/tag/tag.html"));

        assert_eq!(
            token.next_token(),
            Some(HTMLToken::Tag(
                HTMLTagToken::start()
                    .with_name("div")
                    .with_attributes([("id", "foo")])
            )),
        );

        // Hello World</div> ...
        token.nth(12);

        assert_eq!(
            token.next_token(),
            Some(HTMLToken::Tag(
                HTMLTagToken::start()
                    .with_name("input")
                    .with_attributes([("value", "Hello World")])
                    .with_self_closing_flag()
            ))
        );
    }
}
