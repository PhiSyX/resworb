/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{borrow::Cow, collections::VecDeque};

use dom::node::DocumentNode;
use infra::primitive::codepoint::CodePoint;
use macros::dd;
use named_character_references::{
    NamedCharacterReferences, NamedCharacterReferencesEntities,
};
use parser::preprocessor::InputStream;

use super::{state::State, HTMLToken};
use crate::{
    error::HTMLParserError, tree_construction::HTMLTreeConstruction,
};

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
    pub(crate) tree_construction: HTMLTreeConstruction,

    /// Le jeton courant.
    token: Option<HTMLToken>,

    pub(crate) state: HTMLTokenizerState,

    /// La sortie de l'étape de tokenisation est une série de zéro ou plus
    /// des jetons.
    output_tokens: VecDeque<HTMLToken>,

    pub(crate) named_character_reference_code:
        NamedCharacterReferencesEntities,

    /// Certains états utilisent un tampon temporaire pour suivre leur
    /// progression.
    pub(crate) temporary_buffer: String,

    /// [L'état de référence du caractère](State::CharacterReference)
    /// utilise un [état de retour](HTMLTokenizerState::returns) pour
    /// revenir à un état à partir duquel il a été invoqué.
    pub(crate) character_reference_code: u32,

    last_start_tag_token: Option<HTMLToken>,
}

#[derive(Debug)]
#[derive(Clone)]
pub(crate) struct HTMLTokenizerState {
    /// L'état courant.
    current: State,
    /// L'état de retour.
    returns: Option<State>,
}

// ----------- //
// Énumération //
// ----------- //

// -------------- //
// Implémentation //
// -------------- //

impl<C> HTMLTokenizer<C>
where
    C: Iterator<Item = CodePoint>,
{
    pub fn new(document: DocumentNode, iter: C) -> Self {
        let stream = HTMLInputStream::new(iter);
        Self {
            stream,
            tree_construction: HTMLTreeConstruction::new(document),
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

impl<C> HTMLTokenizer<C>
where
    C: Iterator<Item = CodePoint>,
{
    /// Le jeton actuel.
    pub fn current_token(&mut self) -> Option<HTMLToken> {
        if let Some(token) = self.token.to_owned() {
            self.output_tokens.push_back(token);
        }
        self.pop_token()
    }

    /// Le jeton suivant.
    pub fn next_token(&mut self) -> Option<HTMLToken> {
        self.next()
    }

    /// Extrait le premier jeton.
    fn pop_token(&mut self) -> Option<HTMLToken> {
        self.output_tokens.pop_front()
    }

    /// Change l'état d'un jeton via une fonction de retour.
    pub(crate) fn change_current_token<F: FnOnce(&mut HTMLToken)>(
        &mut self,
        callback: F,
    ) -> &mut Self {
        if let Some(ref mut token) = self.token {
            callback(token);
        }
        self
    }

    /// Émet le jeton actuel.
    pub(crate) fn and_emit_current_token(&mut self) -> &mut Self {
        if let Some(token) = self.current_token() {
            self.emit_token(token);
        }
        self
    }

    /// Émet chaque caractère du tampon temporaire.
    pub(crate) fn emit_each_characters_of_temporary_buffer(
        &mut self,
    ) -> &mut Self {
        self.temporary_buffer.chars().for_each(|ch| {
            self.output_tokens.push_back(HTMLToken::Character(ch));
        });
        self
    }

    /// Émet le jeton actuel.
    pub(crate) fn emit_token(&mut self, token: HTMLToken) -> &mut Self {
        if matches!(token, HTMLToken::Character('<' | '/')) {
            self.last_start_tag_token = self.token.clone();
        }

        self.output_tokens.push_front(token);
        self
    }

    /// Remplace le jeton actuel par un nouveau jeton.
    pub(crate) fn set_token(&mut self, new_token: HTMLToken) -> &mut Self {
        self.token.replace(new_token);
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
                    token.append_character_to_attribute_value(ch);
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
            Some(HTMLToken::Tag {
                name: current_tag_name,
                is_end: true,
                ..
            }),
            Some(HTMLToken::Tag {
                name: last_tag_name,
                is_end: true,
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

    pub(crate) const fn is_character_of_attribute(&self) -> bool {
        matches!(
            self.returns,
            Some(State::AttributeValueDoubleQuoted)
                | Some(State::AttributeValueSingleQuoted)
                | Some(State::AttributeValueUnquoted)
        )
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl<C> HTMLTokenizerProcessInterface for HTMLTokenizer<C> where
    C: Iterator<Item = CodePoint>
{
}

impl HTMLTokenizerProcessInterface for HTMLTokenizerState {}

impl<C> Iterator for HTMLTokenizer<C>
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
                | State::RCDATALessThanSign =>
                    self.handle_rcdata_less_than_sign_state(),
                | State::RCDATAEndTagOpen =>
                    self.handle_rcdata_end_tag_open_state(),
                | State::RCDATAEndTagName =>
                    self.handle_rcdata_end_tag_name_state(),
                | State::RAWTEXTLessThanSign =>
                    self.handle_rawtext_less_than_sign_state(),
                | State::RAWTEXTEndTagOpen =>
                    self.handle_rawtext_end_tag_open_state(),
                | State::RAWTEXTEndTagName =>
                    self.handle_rawtext_end_tag_name_state(),
                | State::ScriptDataLessThanSign =>
                    self.handle_script_data_less_than_sign_state(),
                | State::ScriptDataEndTagOpen =>
                    self.handle_script_data_end_tag_open_state(),
                | State::ScriptDataEndTagName =>
                    self.handle_script_data_end_tag_name_state(),
                | State::ScriptDataEscapeStart =>
                    self.handle_script_data_escape_start_state(),
                | State::ScriptDataEscapeStartDash =>
                    self.handle_script_data_escape_start_dash_state(),
                | State::ScriptDataEscaped =>
                    self.handle_script_data_escaped_state(),
                | State::ScriptDataEscapedDash =>
                    self.handle_script_data_escaped_dash_state(),
                | State::ScriptDataEscapedDashDash =>
                    self.handle_script_data_escaped_dash_dash_state(),
                | State::ScriptDataEscapedLessThanSign =>
                    self.handle_script_data_escaped_less_than_sign_state(),
                | State::ScriptDataEscapedEndTagOpen =>
                    self.handle_script_data_escaped_end_tag_open_state(),
                | State::ScriptDataEscapedEndTagName =>
                    self.handle_script_data_escaped_end_tag_name_state(),
                | State::ScriptDataDoubleEscapeStart =>
                    self.handle_script_data_double_escape_start_state(),
                | State::ScriptDataDoubleEscapedState =>
                    self.handle_script_data_double_escaped_state(),
                | State::ScriptDataDoubleEscapedDash =>
                    self.handle_script_data_double_escaped_dash_state(),
                | State::ScriptDataDoubleEscapedDashDash =>
                    self.handle_script_data_double_escaped_dash_dash_state(),
                | State::ScriptDataDoubleEscapedLessThanSign =>
                    self.handle_script_data_double_escaped_less_than_sign_state(),
                | State::ScriptDataDoubleEscapeEnd =>
                    self.handle_script_data_double_escape_end_state(),
                | State::BeforeAttributeName =>
                    self.handle_before_attribute_name_state(),
                | State::AttributeName => self.handle_attribute_name_state(),
                | State::AfterAttributeName =>
                    self.handle_after_attribute_name_state(),
                | State::BeforeAttributeValue =>
                    self.handle_before_attribute_value_state(),
                | State::AttributeValueDoubleQuoted =>
                    self.handle_attribute_value_quoted_state('"'),
                | State::AttributeValueSingleQuoted =>
                    self.handle_attribute_value_quoted_state('\''),
                | State::AttributeValueUnquoted =>
                    self.handle_attribute_value_unquoted_state(),
                | State::AfterAttributeValueQuoted =>
                    self.handle_after_attribute_value_quoted_state(),
                | State::SelfClosingStartTag =>
                    self.handle_self_closing_start_tag_state(),
                | State::MarkupDeclarationOpen =>
                    self.handle_markup_declaration_open_state(),
                | State::BogusComment => self.handle_bogus_comment_state(),
                | State::CommentStart => self.handle_comment_start_state(),
                | State::CommentLessThanSign =>
                    self.handle_comment_less_than_sign_state(),
                | State::CommentLessThanSignBang =>
                    self.handle_comment_less_than_sign_bang_state(),
                | State::CommentLessThanSignBangDash =>
                    self.handle_comment_less_than_sign_bang_dash_state(),
                | State::CommentLessThanSignBangDashDash =>
                    self.handle_comment_less_than_sign_bang_dash_dash_state(),
                | State::CommentStartDash =>
                    self.handle_comment_start_dash_state(),
                | State::Comment => self.handle_comment_state(),
                | State::CommentEndDash => self.handle_comment_end_dash_state(),
                | State::CommentEnd => self.handle_comment_end_state(),
                | State::CommentEndBang => self.handle_comment_end_bang_state(),
                | State::DOCTYPE => self.handle_doctype_state(),
                | State::BeforeDOCTYPEName =>
                    self.handle_before_doctype_name_state(),
                | State::DOCTYPEName => self.handle_doctype_name_state(),
                | State::AfterDOCTYPEName =>
                    self.handle_after_doctype_name_state(),
                | State::AfterDOCTYPEPublicKeyword =>
                    self.handle_after_doctype_public_keyword_state(),
                | State::BeforeDOCTYPEPublicIdentifier =>
                    self.handle_before_doctype_public_identifier_state(),
                | State::DOCTYPEPublicIdentifierDoubleQuoted =>
                    self.handle_doctype_public_identifier_quoted('"'),
                | State::DOCTYPEPublicIdentifierSingleQuoted =>
                    self.handle_doctype_public_identifier_quoted('\''),
                | State::AfterDOCTYPEPublicIdentifier =>
                    self.handle_after_doctype_public_identifier_state(),
                | State::BetweenDOCTYPEPublicAndSystemIdentifiers =>
                    self.handle_between_doctype_public_and_system_identifiers_state(),
                | State::AfterDOCTYPESystemKeyword =>
                    self.handle_after_doctype_system_keyword_state(),
                | State::BeforeDOCTYPESystemIdentifier =>
                    self.handle_before_doctype_system_identifier_state(),
                | State::DOCTYPESystemIdentifierDoubleQuoted =>
                    self.handle_doctype_system_identifier_quoted_state('"'),
                | State::DOCTYPESystemIdentifierSingleQuoted =>
                    self.handle_doctype_system_identifier_quoted_state('\''),
                | State::AfterDOCTYPESystemIdentifier =>
                    self.handle_after_doctype_system_identifier_state(),
                | State::BogusDOCTYPE => self.handle_bogus_doctype_state(),
                | State::CDATASection => self.handle_cdata_section_state(),
                | State::CDATASectionBracket =>
                    self.handle_cdata_section_bracket_state(),
                | State::CDATASectionEnd =>
                    self.handle_cdata_section_end_state(),
                | State::CharacterReference =>
                    self.handle_character_reference_state(),
                | State::NamedCharacterReference =>
                    self.handle_named_character_reference_state(),
                | State::AmbiguousAmpersand =>
                    self.handle_ambiguous_ampersand_state(),
                | State::NumericCharacterReference =>
                    self.handle_numeric_character_reference_state(),
                | State::HexadecimalCharacterReferenceStart =>
                    self.handle_hexadecimal_character_reference_start_state(),
                | State::DecimalCharacterReferenceStart =>
                    self.handle_decimal_character_reference_start_state(),
                | State::HexadecimalCharacterReference =>
                    self.handle_hexadecimal_character_reference_state(),
                | State::DecimalCharacterReference =>
                    self.handle_decimal_character_reference_state(),
                | State::NumericCharacterReferenceEnd =>
                    self.handle_numeric_character_reference_end_state(),
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
        HTMLTokenizer::new(DocumentNode::default(), stream)
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
            Some(
                HTMLToken::new_start_tag()
                    .with_name("a")
                    .with_attributes([(attr_name, attr_value)])
            ),
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
            Some(
                HTMLToken::new_start_tag()
                    .with_name("div")
                    .with_attributes([("id", "foo")])
            ),
        );

        // Hello World</div> ...
        token.nth(12);

        assert_eq!(
            token.next_token(),
            Some(
                HTMLToken::new_start_tag()
                    .with_name("input")
                    .with_attributes([("value", "Hello World")])
                    .with_self_closing_flag()
            )
        );
    }
}
