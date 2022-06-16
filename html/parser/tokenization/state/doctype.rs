/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use infra::{
    primitive::codepoint::{CodePoint, CodePointIterator},
    structure::lists::peekable::PeekableInterface,
};
use parser::StreamIteratorInterface;

use crate::{
    codepoint::HTMLCodePoint,
    tokenization::{
        token::ForceQuirksFlag,
        tokenizer::{
            HTMLTokenizerProcessInterface, HTMLTokenizerProcessResult,
        },
        HTMLToken, HTMLTokenizer,
    },
};

impl<C> HTMLTokenizer<C>
where
    C: CodePointIterator,
{
    pub(crate) fn handle_doctype_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Passer à l'état `before-doctype-name`.
            | Some(ch) if ch.is_html_whitespace() => {
                self.switch_state_to("before-doctype-name").and_continue()
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
            // jeton `end-of-file`.
            | None => self
                .emit_token(HTMLToken::new_doctype().with_quirks_mode())
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

    pub(crate) fn handle_before_doctype_name_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
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
            // (ajouter 0x0020 au point de code du caractère). Passer à
            // l'état `doctype-name`.
            | Some(ch) if ch.is_ascii_uppercase() => self
                .set_token(
                    HTMLToken::new_doctype()
                        .with_name(ch.to_ascii_lowercase()),
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
                        .with_name(char::REPLACEMENT_CHARACTER),
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
                .set_token(HTMLToken::new_doctype().with_quirks_mode())
                .switch_state_to("data")
                .and_emit_with_error("missing-doctype-name"),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-doctype`.
            // Créer un nouveau jeton `doctype`. Mettre son drapeau
            // force-quirks à vrai. Émettre le jeton actuel. Émettre un
            // jeton de `end-of-file`.
            | None => self
                .emit_token(HTMLToken::new_doctype().with_quirks_mode())
                .set_token(HTMLToken::EOF)
                .and_emit_with_error("eof-in-doctype"),

            // Anything else
            //
            // Créer un nouveau jeton `doctype`. Définir le nom du jeton
            // sur le caractère actuel. Passer à l'état `doctype-name`.
            | Some(ch) => self
                .set_token(HTMLToken::new_doctype().with_name(ch))
                .switch_state_to("doctype-name")
                .and_continue(),
        }
    }

    pub(crate) fn handle_doctype_name_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Passer à l'état `after-doctype-name`.
            | Some(ch) if ch.is_html_whitespace() => {
                self.switch_state_to("after-doctype-name").and_continue()
            }

            // U+003E GREATER-THAN SIGN (>)
            //
            // Passer à l'état `data`. Émettre le jeton `doctype` actuel.
            | Some('>') => self.switch_state_to("data").and_emit(),

            // ASCII upper alpha
            //
            // Ajouter la version en minuscules du caractère actuel
            // (ajouter 0x0020 au point de code du caractère) au nom
            // du jeton `doctype` actuel.
            | Some(ch) if ch.is_ascii_uppercase() => self
                .change_current_token(|token| {
                    token.append_character(ch.to_ascii_lowercase());
                })
                .and_continue(),

            // U+0000 NULL
            //
            // Il s'agit d'une erreur d'analyse de type
            // `unexpected-null-character`. Ajouter un caractère U+FFFD
            // REPLACEMENT CHARACTER au nom du jeton `doctype`
            // actuel.
            | Some('\0') => self
                .change_current_token(|token| {
                    token.append_character(char::REPLACEMENT_CHARACTER);
                })
                .and_continue_with_error("unexpected-null-character"),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-doctype`.
            // Définir le drapeau force-quirks du jeton `doctype` actuel
            // sur "on". Émettre le jeton `doctype` actuel. Émettre d'un
            // jeton de `end-of-file`.
            | None => self
                .change_current_token(|token| {
                    token.set_force_quirks_flag(ForceQuirksFlag::On);
                })
                .and_emit_current_token()
                .set_token(HTMLToken::EOF)
                .and_emit_with_error("eof-in-doctype"),

            // Anything else
            //
            // Ajouter le caractère actuel au nom du jeton `doctype`
            // actuel.
            | Some(ch) => self
                .change_current_token(|token| {
                    token.append_character(ch);
                })
                .and_continue(),
        }
    }

    pub(crate) fn handle_after_doctype_name_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
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
            | Some('>') => self.switch_state_to("data").and_emit(),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-doctype`.
            // Définir le drapeau force-quirks du jeton `doctype` actuel
            // sur "on". Émettre le jeton `doctype` actuel. Émettre un
            // jeton `end-of-file`.
            | None => self
                .change_current_token(|token| {
                    token.set_force_quirks_flag(ForceQuirksFlag::On);
                })
                .and_emit_current_token()
                .set_token(HTMLToken::EOF)
                .and_emit_with_error("eof-in-doctype"),

            // Anything else
            //
            // Si les six caractères à partir du caractère actuel sont une
            // correspondance ASCII insensible à la casse pour le
            // mot "PUBLIC", consommer ces caractères et passer à l'état
            // `after-doctype-public-keyword`.
            //
            // Sinon, si les six caractères à partir du caractère d'entrée
            // actuel sont une correspondance ASCII insensible à la casse
            // pour le mot "SYSTEM", consommer ces caractères et passer à
            // l'état `after-doctype-system-keyword`.
            //
            // Sinon, il s'agit d'une erreur d'analyse de type
            // `invalid-character-sequence-after-doctype-name`. Mettre
            // le drapeau force-quirks du jeton actuel à vrai. Reprendre
            // dans l'état `bogus-doctype`.
            | Some(ch) => {
                let mut f = false;

                if let Some(word) = self.input.peek_until::<String>(5) {
                    f = false;

                    let word =
                        format!("{ch}{}", word.to_ascii_uppercase());

                    if word == "PUBLIC" {
                        f = true;

                        self.switch_state_to(
                            "after-doctype-public-keyword",
                        );
                        self.input.advance(6);
                    } else if word == "SYSTEM" {
                        f = true;

                        self.switch_state_to(
                            "after-doctype-system-keyword",
                        );
                        self.input.advance(6);
                    }
                }

                if !f {
                    self.change_current_token(|token| {
                        token.set_force_quirks_flag(ForceQuirksFlag::On);
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

    pub(crate) fn handle_after_doctype_public_keyword_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Passer à l'état `before-doctype-public-identifier`.
            | Some(ch) if ch.is_html_whitespace() => self
                .switch_state_to("before-doctype-public-identifier")
                .and_continue(),

            // U+0022 QUOTATION MARK (")
            //
            // Il s'agit d'une erreur d'analyse de type
            // `missing-whitespace-after-doctype-public-keyword`. Donner à
            // l'identifiant public du jeton `doctype` actuel la valeur de
            // la chaîne de caractères vide, passer à l'état
            // `doctype-public-identifier-double-quoted`.
            | Some('"') => self
                .change_current_token(|token| {
                    token.set_public_identifier(String::new());
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
                .change_current_token(|token| {
                    token.set_public_identifier(String::new());
                })
                .switch_state_to("doctype-public-identifier-single-quoted")
                .and_continue_with_error(
                    "missing-whitespace-after-doctype-public-keyword",
                ),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Il s'agit d'une erreur d'analyse de type
            // `missing-doctype-public-identifier`. Définir le drapeau
            // force-quirks du jeton `doctype` actuel sur "on". Passer à
            // l'état `data`. Émettre le jeton `doctype` actuel.
            | Some('>') => self
                .change_current_token(|token| {
                    token.set_force_quirks_flag(ForceQuirksFlag::On);
                })
                .switch_state_to("data")
                .and_emit_with_error("missing-doctype-public-identifier"),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-doctype`.
            // Définir le drapeau force-quirks du jeton `doctype` actuel
            // sur "on". Émettre le jeton `doctype` actuel. Émettre d'un
            // jeton `end-of-file`.
            | None => self
                .change_current_token(|token| {
                    token.set_force_quirks_flag(ForceQuirksFlag::On);
                })
                .and_emit_current_token()
                .set_token(HTMLToken::EOF)
                .and_emit_with_error("eof-in-doctype"),

            // Anything else
            //
            // Il s'agit d'une erreur d'analyse de type
            // `missing-quote-before-doctype-public-identifier`. Définir le
            // drapeau force-quirks du jeton `doctype` actuel sur "on".
            // Reprendre dans l'état `bogus-doctype`.
            | Some(_) => self
                .change_current_token(|token| {
                    token.set_force_quirks_flag(ForceQuirksFlag::On);
                })
                .reconsume("bogus-doctype")
                .and_continue_with_error(
                    "missing-quote-before-doctype-public-identifier",
                ),
        }
    }

    pub(crate) fn handle_before_doctype_public_identifier_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
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
                .set_token(HTMLToken::new_doctype().with_name('\0'))
                .switch_state_to("doctype-public-identifier-double-quoted")
                .and_continue(),

            // U+0027 APOSTROPHE (')
            //
            // Définir l'identifiant public du jeton `doctype` actuel à une
            // chaîne de caractères vide, passer à l'état
            // `doctype-public-identifier-single-quoted`.
            | Some('\'') => self
                .set_token(HTMLToken::new_doctype().with_name('\0'))
                .switch_state_to("doctype-public-identifier-single-quoted")
                .and_continue(),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Il s'agit d'une erreur d'analyse de type
            // `missing-doctype-public-identifier`. Définir le drapeau
            // force-quirks du jeton `doctype` actuel sur "on". Passer à
            // l'état `data`. Émettre le jeton `doctype` actuel.
            | Some('>') => self
                .change_current_token(|token| {
                    token.set_force_quirks_flag(ForceQuirksFlag::On);
                })
                .switch_state_to("data")
                .and_emit_with_error("missing-doctype-public-identifier"),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-doctype`.
            // Définir le drapeau force-quirks du jeton `doctype` actuel
            // sur "on". Émettre le jeton `doctype` actuel. Émettre un
            // jeton `end-of-file`.
            | None => self
                .change_current_token(|token| {
                    token.set_force_quirks_flag(ForceQuirksFlag::On);
                })
                .and_emit_current_token()
                .set_token(HTMLToken::EOF)
                .and_emit_with_error("eof-in-doctype"),

            // Anything else
            //
            // Il s'agit d'une erreur d'analyse de type
            // `missing-quote-before-doctype-public-identifier`. Définir le
            // drapeau force-quirks du jeton `doctype` actuel sur "on".
            // Reprendre à l'état `bogus-doctype`.
            | Some(_) => self
                .change_current_token(|token| {
                    token.set_force_quirks_flag(ForceQuirksFlag::On);
                })
                .reconsume("bogus-doctype")
                .and_continue_with_error(
                    "missing-quote-before-doctype-public-identifier",
                ),
        }
    }

    pub(crate) fn handle_doctype_public_identifier_quoted(
        &mut self,
        quote: CodePoint,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
            // U+0022 QUOTATION MARK (")
            //
            // Passer à l'état `after-doctype-public-identifier`.
            | Some('"') if quote == '"' => self
                .switch_state_to("after-doctype-public-identifier")
                .and_continue(),

            // U+0027 APOSTROPHE (')
            //
            // Passer à l'état `after-doctype-public-identifier`.
            | Some('\'') if quote == '\'' => self
                .switch_state_to("after-doctype-public-identifier")
                .and_continue(),

            // U+0000 NULL
            //
            // Il s'agit d'une erreur d'analyse de type
            // `unexpected-null-character`. Ajouter un caractère U+FFFD
            // `REPLACEMENT_CHARACTER` à l'identifiant public du jeton
            // `doctype` actuel.
            | Some('\0') => self
                .change_current_token(|token| {
                    token.append_character_to_public_identifier(
                        char::REPLACEMENT_CHARACTER,
                    );
                })
                .and_continue_with_error("unexpected-null-character"),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Il s'agit d'une erreur d'analyse de type
            // `abrupt-doctype-public-identifier`. Définir le drapeau
            // force-quirks du jeton `doctype` actuel sur "on". Passer à
            // l'état `data`. Émettre le jeton `doctype` actuel.
            | Some('>') => self
                .change_current_token(|token| {
                    token.set_force_quirks_flag(ForceQuirksFlag::On);
                })
                .switch_state_to("data")
                .and_emit_with_error("abrupt-doctype-public-identifier"),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-doctype`.
            // Définir le drapeau force-quirks du jeton `doctype` actuel
            // sur "on". Émettre le jeton `doctype` actuel. Émettre un
            // jeton `end-of-file`.
            | None => self
                .change_current_token(|token| {
                    token.set_force_quirks_flag(ForceQuirksFlag::On);
                })
                .and_emit_current_token()
                .set_token(HTMLToken::EOF)
                .and_emit_with_error("eof-in-doctype"),

            // Anything else
            //
            // Ajouter le caractère actuel à l'identifiant public du jeton
            // `doctype` actuel.
            | Some(ch) => self
                .change_current_token(|token| {
                    token.append_character_to_public_identifier(ch);
                })
                .and_continue(),
        }
    }

    pub(crate) fn handle_after_doctype_public_identifier_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Passer à l'état
            // `between-doctype-public-and-system-identifiers`.
            | Some(ch) if ch.is_html_whitespace() => self
                .switch_state_to(
                    "between-doctype-public-and-system-identifiers",
                )
                .and_continue(),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Passer à l'état `data`. Émettre le jeton `DOCTYPE` actuel.
            | Some('>') => self.switch_state_to("data").and_emit(),

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
                self.change_current_token(|token| {
                    token.set_system_identifier(String::new());
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
            // sur "on". Émettre le jeton `doctype` actuel. Émettre d'un
            // jeton `end-of-file`.
            | None => self
                .change_current_token(|token| {
                    token.set_force_quirks_flag(ForceQuirksFlag::On);
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

    pub(crate) fn handle_between_doctype_public_and_system_identifiers_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
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
            | Some('>') => self.switch_state_to("data").and_emit(),

            // U+0022 QUOTATION MARK (")
            // U+0027 APOSTROPHE (')
            //
            // Définir l'identifiant système du jeton `doctype` actuel à
            // la chaîne de caractères vide, puis passer à l'état
            // `doctype-system-identifier-double-quoted` ou
            // `doctype-system-identifier-single-quoted`.
            | Some(ch @ ('"' | '\'')) => self
                .change_current_token(|token| {
                    token.set_system_identifier(String::new());
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
            // sur "on". Émettre le jeton `doctype` actuel. Émettre un
            // jeton `end-of-file`.
            | None => self
                .change_current_token(|token| {
                    token.set_force_quirks_flag(ForceQuirksFlag::On);
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
                .change_current_token(|token| {
                    token.set_force_quirks_flag(ForceQuirksFlag::On);
                })
                .reconsume("bogus-doctype")
                .and_continue_with_error(
                    "missing-quote-before-doctype-system-identifier",
                ),
        }
    }

    pub(crate) fn handle_after_doctype_system_keyword_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Passer à l'état `before-doctype-system-identifier`.
            | Some(ch) if ch.is_html_whitespace() => self
                .switch_state_to("before-doctype-system-identifier")
                .and_continue(),

            // U+0022 QUOTATION MARK (")
            //
            // Il s'agit d'une erreur d'analyse de type
            // `missing-whitespace-after-doctype-system-keyword`.
            // Définir l'identifiant système du jeton `doctype` actuel
            // à une chaîne de caractères vide, passer à l'état
            // `doctype-system-identifier-double-quoted`.
            | Some('"') => self
                .change_current_token(|token| {
                    token.set_system_identifier(String::new());
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
                .change_current_token(|token| {
                    token.set_system_identifier(String::new());
                })
                .switch_state_to("doctype-system-identifier-single-quoted")
                .and_continue_with_error(
                    "missing-whitespace-after-doctype-system-keyword",
                ),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Il s'agit d'une erreur d'analyse de type
            // `missing-doctype-system-identifier`. Définir le drapeau
            // force-quirks du jeton `doctype` actuel sur "on". Passer à
            // l'état `data`. Émettre le jeton `doctype` actuel.
            | Some('>') => self
                .change_current_token(|token| {
                    token.set_force_quirks_flag(ForceQuirksFlag::On);
                })
                .switch_state_to("data")
                .and_emit_with_error("missing-doctype-system-identifier"),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse type `eof-in-doctype`.
            // Définir le drapeau force-quirks du jeton `doctype` actuel
            // sur "on". Émettre le jeton `doctype` actuel. Émettre un
            // jeton `end-of-file`.
            | None => self
                .change_current_token(|token| {
                    token.set_force_quirks_flag(ForceQuirksFlag::On);
                })
                .and_emit_current_token()
                .set_token(HTMLToken::EOF)
                .and_emit_with_error("eof-in-doctype"),

            // Anything else
            //
            // Il s'agit d'une erreur d'analyse de type
            // `missing-quote-before-doctype-system-identifier`. Définir le
            // drapeau force-quirks du jeton `doctype` actuel sur "on".
            // Reprendre dans l'état `bogus-doctype`.
            | Some(_) => self
                .change_current_token(|token| {
                    token.set_force_quirks_flag(ForceQuirksFlag::On);
                })
                .set_token(HTMLToken::EOF)
                .reconsume("bogus-doctype")
                .and_continue_with_error(
                    "missing-quote-before-doctype-system-identifier",
                ),
        }
    }

    pub(crate) fn handle_before_doctype_system_identifier_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
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
                .change_current_token(|token| {
                    token.set_system_identifier(String::new());
                })
                .switch_state_to("doctype-system-identifier-double-quoted")
                .and_continue(),

            // U+0027 APOSTROPHE (')
            //
            // Définir l'identifiant système du jeton `doctype` actuel à la
            // chaîne de caractères vide, passer à  l'état
            // `doctype-system-identifier-single-quoted`
            | Some('\'') => self
                .change_current_token(|token| {
                    token.set_system_identifier(String::new());
                })
                .switch_state_to("doctype-system-identifier-single-quoted")
                .and_continue(),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Il s'agit d'une erreur d'analyse de type
            // `missing-doctype-system-identifier`. Définir le drapeau
            // force-quirks du jeton `doctype` actuel sur "on". Passer à
            // l'état `data`. Émettre le jeton `doctype` actuel.
            | Some('>') => self
                .change_current_token(|token| {
                    token.set_force_quirks_flag(ForceQuirksFlag::On);
                })
                .switch_state_to("data")
                .and_emit_with_error("missing-doctype-system-identifier"),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse type `eof-in-doctype`.
            // Définir le drapeau force-quirks du jeton `doctype` actuel
            // sur "on". Émettre le jeton `doctype` actuel. Émettre un
            // jeton `end-of-file`.
            | None => self
                .change_current_token(|token| {
                    token.set_force_quirks_flag(ForceQuirksFlag::On);
                })
                .and_emit_current_token()
                .set_token(HTMLToken::EOF)
                .and_emit_with_error("eof-in-doctype"),

            // Anything else
            //
            // Il s'agit d'une erreur d'analyse de type
            // `missing-quote-before-doctype-system-identifier`. Définir
            // le drapeau force-quirks du jeton `doctype` actuel sur "on".
            // Reprendre dans l'état `bogus-doctype`.
            | Some(_) => self
                .change_current_token(|token| {
                    token.set_force_quirks_flag(ForceQuirksFlag::On);
                })
                .reconsume("bogus-doctype")
                .and_continue_with_error(
                    "missing-quote-before-doctype-system-identifier",
                ),
        }
    }

    pub(crate) fn handle_doctype_system_identifier_quoted_state(
        &mut self,
        quote: CodePoint,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
            // U+0022 QUOTATION MARK (")
            // U+0027 APOSTROPHE (')
            //
            // Passer à l'état `after-doctype-system-identifier`.
            | Some(ch) if ch == quote => self
                .switch_state_to("after-doctype-system-identifier")
                .and_continue(),

            // U+0000 NULL
            //
            // Il s'agit d'une erreur d'analyse de type
            // unexpected-null-character. Ajouter un caractère U+FFFD
            // `REPLACEMENT_CHARACTER` à l'identifiant système du jeton
            // DOCTYPE actuel.
            | Some('\0') => self
                .change_current_token(|token| {
                    token.append_character_to_system_identifier(
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
                .change_current_token(|token| {
                    token.set_force_quirks_flag(ForceQuirksFlag::On);
                })
                .switch_state_to("data")
                .and_emit_with_error("abrupt-doctype-system-identifier"),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse type `eof-in-doctype`.
            // Définir le drapeau force-quirks du jeton
            // `doctype` actuel sur "on". Émettre le jeton
            // `doctype` actuel. Émission d'un jeton de
            // fin de fichier.
            | None => self
                .change_current_token(|token| {
                    token.set_force_quirks_flag(ForceQuirksFlag::On);
                })
                .and_emit_current_token()
                .set_token(HTMLToken::EOF)
                .and_emit_with_error("eof-in-doctype"),

            // Anything else
            //
            // Ajouter le caractère actuel à l'identifiant système du jeton
            // DOCTYPE actuel.
            | Some(ch) => self
                .change_current_token(|token| {
                    token.append_character_to_system_identifier(ch);
                })
                .and_continue(),
        }
    }

    pub(crate) fn handle_after_doctype_system_identifier_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
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
            | Some('>') => self.switch_state_to("data").and_emit(),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse type `eof-in-doctype`.
            // Définir le drapeau force-quirks du jeton `doctype` actuel
            // sur "on". Émettre le jeton `doctype` actuel. Émettre un
            // jeton `end-of-file`.
            | None => self
                .change_current_token(|token| {
                    token.set_force_quirks_flag(ForceQuirksFlag::On);
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

    pub(crate) fn handle_bogus_doctype_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
            // U+003E GREATER-THAN SIGN (>)
            //
            // Passer à l'état `data`. Émettre le jeton `doctype`.
            | Some('>') => self.switch_state_to("data").and_emit(),

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
}
