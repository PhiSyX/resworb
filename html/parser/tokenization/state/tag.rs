/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use infra::primitive::codepoint::CodePoint;

use crate::{
    codepoint::HTMLCodePoint,
    tokenization::{
        tokenizer::{
            HTMLTokenizerProcessInterface, HTMLTokenizerProcessResult,
        },
        HTMLTagAttribute, HTMLToken, HTMLTokenizer,
    },
};

impl<C> HTMLTokenizer<C>
where
    C: Iterator<Item = CodePoint>,
{
    pub(crate) fn handle_tag_open_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.stream.next_input_char() {
            // U+0021 EXCLAMATION MARK (!)
            //
            // Passer à l'état `markup-declaration-open`.
            | Some('!') => self
                .switch_state_to("markup-declaration-open")
                .and_continue(),

            // U+002F SOLIDUS (/)
            //
            // Passer à l'état `end-tag-open`.
            | Some('/') => {
                self.switch_state_to("end-tag-open").and_continue()
            }

            // ASCII alpha
            //
            // Créer un nouveau jeton `start tag`, et définir son nom
            // en une chaîne de caractères vide. Reprendre dans `tag-name`.
            | Some(ch) if ch.is_ascii_alphabetic() => self
                .set_token(HTMLToken::new_start_tag())
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
            // `end-of-file`.
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

    pub(crate) fn handle_end_tag_open_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.stream.next_input_char() {
            // ASCII alpha
            //
            // Créer un nouveau jeton `end tag`, et lui définir son nom
            // comme une chaîne de caractères vide. Reprendre l'état
            // `tag-name`.
            | Some(ch) if ch.is_ascii_alphabetic() => self
                .set_token(HTMLToken::new_end_tag())
                .reconsume("tag-name")
                .and_continue(),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Il s'agit d'une erreur d'analyse de type
            // `missing-end-tag-name`. Passer à l'état `data`.
            | Some('>') => self
                .switch_state_to("data")
                .and_continue_with_error("missing-end-tag-name"),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type
            // `eof-before-tag-name`. Émettre un jeton `character`
            // U+003C LESS-THAN SIGN, un jeton de `character` U+002F
            // SOLIDUS et un jeton `end-of-file`.
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

    pub(crate) fn handle_tag_name_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.stream.next_input_char() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Si le jeton `end tag` actuel est un jeton `end tag`
            // approprié, passer à l'état `before-attribute-name`. Sinon,
            // le traiter comme indiqué dans l'entrée "Anything
            // else" ci-dessous.
            | Some(ch) if ch.is_html_whitespace() => self
                .switch_state_to("before-attribute-name")
                .and_continue(),

            // U+002F SOLIDUS (/)
            //
            // Si le jeton `end tag` actuel est un jeton `end tag`
            // approprié, passer l'état de balise de
            // `self-closing-start-tag`. Sinon, le traiter comme dans
            // l'entrée "Anything else" ci-dessous.
            | Some('/') => self
                .switch_state_to("self-closing-start-tag")
                .and_continue(),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Passer à l'état `data`. Émettre le jeton `tag` actuel.
            | Some('>') => self.switch_state_to("data").and_emit(),

            // ASCII upper alpha
            //
            // Ajouter la version en minuscules du caractère actuel
            // (ajouter 0x0020 au point de code du caractère) au nom du
            // jeton `tag` actuel.
            | Some(ch) if ch.is_ascii_uppercase() => self
                .change_current_token(|token| {
                    token.append_character(ch.to_ascii_lowercase());
                })
                .and_continue(),

            // U+0000 NULL
            //
            // Il s'agit d'une erreur d'analyse de type
            // `unexpected-null-character`. Ajouter un caractère U+FFFD
            // `REPLACEMENT_CHARACTER` au nom du jeton `tag` actuel.
            | Some('\0') => self
                .change_current_token(|token| {
                    token.append_character(char::REPLACEMENT_CHARACTER);
                })
                .and_continue_with_error("unexpected-null-character"),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type ` eof-in-tag`.
            // Émettre un jeton `end-of-file`.
            | None => self
                .set_token(HTMLToken::EOF)
                .and_emit_with_error("eof-in-tag"),

            // Anything else
            //
            // Ajouter le caractère actuel au nom du jeton `tag` actuel.
            | Some(ch) => self
                .change_current_token(|token| {
                    token.append_character(ch);
                })
                .and_continue(),
        }
    }

    pub(crate) fn handle_before_attribute_name_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
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
                .change_current_token(|token| {
                    let attribute = (ch.to_string(), "");
                    token.as_tag_mut().append_tag_attributes(attribute);
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
                .change_current_token(|token| {
                    let attribute = HTMLTagAttribute::default();
                    token.as_tag_mut().append_tag_attributes(attribute);
                })
                .reconsume("attribute-name")
                .and_continue(),
        }
    }

    pub(crate) fn handle_attribute_name_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
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
                .switch_state_to("before-attribute-value")
                .and_continue(),

            // ASCII upper alpha
            //
            // Ajouter la version en minuscules du caractère actuel
            // (ajouter 0x0020 au point de code du caractère) au nom de
            // l'attribut actuel.
            | Some(ch) if ch.is_ascii_uppercase() => self
                .change_current_token(|token| {
                    token.as_tag_mut().append_character_to_attribute_name(
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
                self.change_current_token(|token| {
                    token
                        .as_tag_mut()
                        .append_character_to_attribute_name(ch);
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

    pub(crate) fn handle_after_attribute_name_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
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
                .switch_state_to("self-closing-start-tag")
                .and_continue(),

            // U+003D EQUALS SIGN (=)
            //
            // Passer à l'état `before-attribute-value`.
            | Some('=') => self
                .switch_state_to("before-attribute-value")
                .and_continue(),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Passer à l'état `data`. Émettre le jeton actuel.
            | Some('>') => self.switch_state_to("data").and_emit(),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-tag`.
            // Émettre un jeton `end-of-file`.
            | None => self
                .set_token(HTMLToken::EOF)
                .and_emit_with_error("eof-in-tag"),

            // Anything else
            //
            // Commencer un nouvel attribut dans le jeton `tag` actuel.
            // Définir le nom et la valeur de cet attribut à une chaîne de
            // caractères vide. Reprendre l'état `attribute-name`.
            | Some(_) => self
                .change_current_token(|token| {
                    let attribute = HTMLTagAttribute::default();
                    token.as_tag_mut().append_tag_attributes(attribute);
                })
                .reconsume("attribute-name")
                .and_continue(),
        }
    }

    pub(crate) fn handle_before_attribute_value_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
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
                .switch_state_to("attribute-value-double-quoted")
                .and_continue(),

            // U+0027 APOSTROPHE (')
            //
            // Passer à l'état `attribute-value-single-quoted`.
            | Some('\'') => self
                .switch_state_to("attribute-value-single-quoted")
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

    pub(crate) fn handle_attribute_value_quoted_state(
        &mut self,
        quote: CodePoint,
    ) -> HTMLTokenizerProcessResult {
        match self.stream.next_input_char() {
            // U+0022 QUOTATION MARK (")
            //
            // Passer à l'état `after-attribute-value-quoted`.
            | Some('"') if quote == '"' => self
                .switch_state_to("after-attribute-value-quoted")
                .and_continue(),

            // U+0027 APOSTROPHE (')
            //
            // Passer à l'état `after-attribute-value-quoted`.
            | Some('\'') if quote == '\'' => self
                .switch_state_to("after-attribute-value-quoted")
                .and_continue(),

            // U+0026 AMPERSAND (&)
            //
            // Définir l'état de retour à l'état
            // `attribute-value-double-quoted`. Passer à l'état
            // `character-reference`.
            | Some('&') => self
                .set_return_state_to(if quote == '"' {
                    "attribute-value-double-quoted"
                } else {
                    "attribute-value-single-quoted"
                })
                .switch_state_to("character-reference")
                .and_continue(),

            // U+0000 NULL
            //
            // Il s'agit d'une erreur d'analyse de type
            // `unexpected-null-character`. Ajouter un caractère U+FFFD
            // `REPLACEMENT_CHARACTER` à la valeur de l'attribut actuel.
            | Some('\0') => self
                .change_current_token(|token| {
                    token
                        .as_tag_mut()
                        .append_character_to_attribute_value(
                            char::REPLACEMENT_CHARACTER,
                        );
                })
                .and_continue_with_error("unexpected-null-character"),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-tag`.
            // Émettre un jeton `end-of-file`.
            | None => self
                .set_token(HTMLToken::EOF)
                .and_emit_with_error("eof-in-tag"),

            // Anything else
            //
            // Ajouter le caractère actuel à la valeur de l'attribut
            // actuel.
            | Some(ch) => self
                .change_current_token(|token| {
                    token
                        .as_tag_mut()
                        .append_character_to_attribute_value(ch);
                })
                .and_continue(),
        }
    }

    pub(crate) fn handle_attribute_value_unquoted_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.stream.next_input_char() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Passer à l'état `before-attribute-name`.
            | Some(ch) if ch.is_html_whitespace() => self
                .switch_state_to("before-attribute-name")
                .and_continue(),

            // U+0026 AMPERSAND (&)
            //
            // Définir l'état de retour à `attribute-value-unquoted`.
            // Passer à l'état `character-reference`.
            | Some('&') => self
                .set_return_state_to("attribute-value-unquoted")
                .switch_state_to("character-reference")
                .and_continue(),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Passer à l'état `data`. Émettre le jeton `tag` actuel.
            | Some('>') => self.switch_state_to("data").and_emit(),

            // U+0000 NULL
            //
            // Il s'agit d'une erreur d'analyse de type
            // `unexpected-null-character`. Ajouter un caractère
            // `REPLACEMENT_CHARACTER` U+FFFD à la valeur de l'attribut
            // actuel.
            | Some('\0') => self
                .change_current_token(|token| {
                    token
                        .as_tag_mut()
                        .append_character_to_attribute_value(
                            char::REPLACEMENT_CHARACTER,
                        );
                })
                .and_continue_with_error("unexpected-null-character"),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-tag`.
            // Émettre un jeton `end-of-file`.
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
                self.change_current_token(|token| {
                    token
                        .as_tag_mut()
                        .append_character_to_attribute_value(ch);
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

    pub(crate) fn handle_after_attribute_value_quoted_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.stream.next_input_char() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Passer à l'état `before-attribute-name`.
            | Some(ch) if ch.is_html_whitespace() => self
                .switch_state_to("before-attribute-name")
                .and_continue(),

            // U+002F SOLIDUS (/)
            //
            // Passer à l'état `self-closing-start-tag`.
            | Some('/') => self
                .switch_state_to("self-closing-start-tag")
                .and_continue(),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Passer à l'état `data`. Émettre le jeton `tag` actuel.
            | Some('>') => self.switch_state_to("data").and_emit(),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-tag`.
            // Émettre un jeton `end-of-file`.
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

    pub(crate) fn handle_self_closing_start_tag_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.stream.next_input_char() {
            // U+003E GREATER-THAN SIGN (>)
            //
            // Définir le drapeau `self-closing` au jeton `tag` actuel sur
            // vrai. Passer à l'état `data`. Émettre le jeton actuel.
            | Some('>') => self
                .change_current_token(|token| {
                    token.as_tag_mut().set_self_closing_tag(true);
                })
                .switch_state_to("data")
                .and_emit(),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type `eof-in-tag`.
            // Émettre un jeton `end-of-file`.
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
}
