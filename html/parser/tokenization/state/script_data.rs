/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use infra::primitive::codepoint::CodePointIterator;

use crate::{
    codepoint::HTMLCodePoint,
    tokenization::{
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
    pub(crate) fn handle_script_data_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
            // U+003C LESS-THAN SIGN (<)
            //
            // Passer à l'état `script-data-less-than-sign`.
            | Some('<') => self
                .switch_state_to("script-data-less-than-sign")
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
            // Émettre un jeton `end-of-file`
            | None => self.set_token(HTMLToken::EOF).and_emit(),

            // Anything else
            //
            // Émettre le caractère actuel comme un jeton `character`.
            | Some(ch) => {
                self.set_token(HTMLToken::Character(ch)).and_emit()
            }
        }
    }

    pub(crate) fn handle_script_data_less_than_sign_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
            // U+002F SOLIDUS (/)
            //
            // Définir le tampon temporaire comme une chaîne de caractères
            // vide. Passer à l'état `script-data-end-tag-open`.
            | Some('/') => self
                .set_temporary_buffer(String::new())
                .switch_state_to("script-data-end-tag-open")
                .and_continue(),

            // U+0021 EXCLAMATION MARK (!)
            //
            // Passer à l'état `script-data-escape-start`. Émettre un
            // jeton `character` U+003C LESS-THAN SIGN et un jeton
            // `character` U+0021 EXCLAMATION MARK.
            | Some('!') => self
                .switch_state_to("script-data-escape-start")
                .emit_token(HTMLToken::Character('<'))
                .emit_token(HTMLToken::Character('!'))
                .and_continue(),

            // Anything else
            //
            // Émettre un jeton `character` U+003C LESS-THAN SIGN.
            // Reprendre dans l'état de données du script.
            | _ => self
                .emit_token(HTMLToken::Character('<'))
                .reconsume("script-data")
                .and_continue(),
        }
    }

    pub(crate) fn handle_script_data_end_tag_open_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
            // ASCII alpha
            //
            // Créer un nouveau jeton `end-tag`, définir son nom de balise
            // en une chaîne de caractères vide. Reprendre dans l'état
            // `script-data-end-tag-name`.
            | Some(ch) if ch.is_ascii_alphabetic() => self
                .set_token(HTMLToken::new_end_tag())
                .reconsume("script-data-end-tag-name")
                .and_continue(),

            // Émettre un jeton `character` U+003C LESS-THAN SIGN et un
            // jeton `character` U+002F SOLIDUS. Reprendre dans l'état
            // `script-data`.µ
            | _ => self
                .emit_token(HTMLToken::Character('<'))
                .emit_token(HTMLToken::Character('/'))
                .reconsume("script-data")
                .and_continue(),
        }
    }

    pub(crate) fn handle_script_data_end_tag_name_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Si le jeton `end-tag` actuel est un jeton `end-tag`
            // approprié, il faut passer à l'état `before-attribute-name`.
            // Sinon, le traiter comme indiqué dans l'entrée `Anything
            // else` ci-dessous.
            | Some(ch)
                if ch.is_html_whitespace()
                    && self.is_appropriate_end_tag() =>
            {
                self.switch_state_to("before-attribute-name")
                    .and_continue()
            }

            // U+002F SOLIDUS (/)
            //
            // Si le jeton `end-tag` actuel est un jeton `end-tag`
            // approprié, il faut passer à l'état `self-closing-start-tag`.
            // Sinon, le traiter comme indiqué dans l'entrée `Anything
            // else` ci-dessous.
            | Some('/') if self.is_appropriate_end_tag() => self
                .switch_state_to("self-closing-start-tag")
                .and_continue(),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Si le jeton `end-tag` actuel est un jeton `end-tag`
            // approprié, il faut passer à l'état `data`.
            // Sinon, le traiter comme indiqué dans l'entrée `Anything
            // else` ci-dessous.
            | Some('>') if self.is_appropriate_end_tag() => {
                self.switch_state_to("data").and_continue()
            }

            // ASCII upper alpha
            //
            // Ajouter la version en minuscules du caractère d'entrée
            // actuel (ajouter 0x0020 au point de code du caractère) au nom
            // de la balise du jeton `*-tag` actuel. Ajouter le caractère
            // actuel au tampon temporaire.
            | Some(ch) if ch.is_ascii_uppercase() => self
                .change_current_token(|token| {
                    token.append_character(ch.to_ascii_lowercase())
                })
                .append_character_to_temporary_buffer(ch)
                .and_continue(),

            // ASCII lower alpha
            //
            // Ajouter le caractère actuel au nom de la balise du jeton
            // `*-tag` actuel. Ajouter le caractère actuel au tampon
            // temporaire.
            | Some(ch) if ch.is_ascii_lowercase() => self
                .change_current_token(|token| token.append_character(ch))
                .append_character_to_temporary_buffer(ch)
                .and_continue(),

            // Anything else
            //
            // Émettre un jeton `character` U+003C LESS-THAN SIGN, un
            // jeton `character` U+002F SOLIDUS et un jeton `character`
            // pour chacun des caractères du tampon temporaire (dans
            // l'ordre où ils ont été ajoutés au tampon). Reprendre dans
            // l'état `script-data`.
            | _ => self
                .emit_token(HTMLToken::Character('<'))
                .emit_token(HTMLToken::Character('/'))
                .emit_each_characters_of_temporary_buffer()
                .reconsume("script-data")
                .and_continue(),
        }
    }

    pub(crate) fn handle_script_data_escape_start_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
            // U+002D HYPHEN-MINUS (-)
            //
            // Passer à l'état `script-data-escape-start-dash`. Émettre un
            // jeton `character` U+002D HYPHEN-MINUS.
            | Some(ch @ '-') => self
                .switch_state_to("script-data-escape-start-dash")
                .set_token(HTMLToken::Character(ch))
                .and_emit(),

            // Anything else
            //
            // Reprendre dans l'état `script-data`.
            | _ => self.reconsume("script-data").and_continue(),
        }
    }

    pub(crate) fn handle_script_data_escape_start_dash_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
            // U+002D HYPHEN-MINUS (-)
            //
            // Passer à l'état `script-data-escaped-dash-dash`.
            // Émettre un jeton `character` U+002D HYPHEN-MINUS.
            | Some(ch @ '-') => self
                .switch_state_to("script-data-escaped-dash-dash")
                .set_token(HTMLToken::Character(ch))
                .and_emit(),

            // Anything else
            //
            // Reprendre dans l'état `script-data`.
            | _ => self.reconsume("script-data").and_continue(),
        }
    }

    pub(crate) fn handle_script_data_escaped_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
            // U+002D HYPHEN-MINUS (-)
            //
            // Passer à l'état `script-data-escaped-dash`. Émettre un jeton
            // `character` U+002D HYPHEN-MINUS.
            | Some(ch @ '-') => self
                .switch_state_to("script-data-escaped-dash")
                .set_token(HTMLToken::Character(ch))
                .and_emit(),

            // U+003C LESS-THAN SIGN (<)
            //
            // Passer à l'état `script-data-escaped-less-than-sign`.
            | Some('<') => self
                .switch_state_to("script-data-escaped-less-than-sign")
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
            // Il s'agit d'une erreur d'analyse de type
            // `eof-in-script-html-comment-like-text`. Émettre un jeton
            // `end-of-file`.
            | None => self.set_token(HTMLToken::EOF).and_emit_with_error(
                "eof-in-script-html-comment-like-text",
            ),

            // Anything else
            //
            // Émettre le caractère actuel comme un jeton `character`.
            | Some(ch) => {
                self.set_token(HTMLToken::Character(ch)).and_emit()
            }
        }
    }

    pub(crate) fn handle_script_data_escaped_dash_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
            // U+002D HYPHEN-MINUS (-)
            //
            // Passer à l'état `script-data-escaped-dash-dash`.
            // Émettre un jeton `character` U+002D HYPHEN-MINUS.
            | Some(ch @ '-') => self
                .switch_state_to("script-data-escaped-dash-dash")
                .set_token(HTMLToken::Character(ch))
                .and_emit(),

            // U+003C LESS-THAN SIGN (<)
            //
            // Passer à l'état `script-data-escaped-less-than-sign`.
            | Some('<') => self
                .switch_state_to("script-data-escaped-less-than-sign")
                .and_continue(),

            // U+0000 NULL
            //
            // Il s'agit d'une erreur d'analyse de type
            // `unexpected-null-character`. Passer à l'état
            // `script-data-escaped`. Émettre un jeton `character` U+FFFD
            // REPLACEMENT CHARACTER.
            | Some('\0') => self
                .switch_state_to("script-data-escaped")
                .set_token(HTMLToken::Character(
                    char::REPLACEMENT_CHARACTER,
                ))
                .and_emit(),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type
            // `eof-in-script-html-comment-like-text`. Émettre un jeton
            // `end-of-file`.
            | None => self.set_token(HTMLToken::EOF).and_emit_with_error(
                "eof-in-script-html-comment-like-text",
            ),

            // Anything else
            //
            // Passer à l'état `script-data-escaped`. Émettre le caractère
            // actuel comme un jeton `character`.
            | Some(ch) => self
                .switch_state_to("script-data-escaped")
                .set_token(HTMLToken::Character(ch))
                .and_emit(),
        }
    }

    pub(crate) fn handle_script_data_escaped_dash_dash_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
            // U+002D HYPHEN-MINUS (-)
            //
            // Émettre un jeton `character` U+002D HYPHEN-MINUS.
            | Some(ch @ '-') => {
                self.set_token(HTMLToken::Character(ch)).and_emit()
            }

            // U+003C LESS-THAN SIGN (<)
            //
            // Passer à l'état `script-data-escaped-less-than-sign`.
            | Some('<') => self
                .switch_state_to("script-data-escaped-less-than-sign")
                .and_continue(),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Passer à l'état `script-data`. Émettre un jeton `character`
            // U+003E GREATER-THAN SIGN.
            | Some(ch @ '>') => self
                .switch_state_to("script-data")
                .set_token(HTMLToken::Character(ch))
                .and_emit(),

            // U+0000 NULL
            //
            // Il s'agit d'une erreur d'analyse de type
            // `unexpected-null-character`. Passer à l'état
            // `script-data-escaped`. Émettre un jeton `character` U+FFFD
            // REPLACEMENT CHARACTER.
            | Some('\0') => self
                .switch_state_to("script-data-escaped")
                .set_token(HTMLToken::Character(
                    char::REPLACEMENT_CHARACTER,
                ))
                .and_emit_with_error("unexpected-null-character"),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type
            // `eof-in-script-html-comment-like-text`. Émettre un jeton
            // `end-of-file`.
            | None => self.set_token(HTMLToken::EOF).and_emit_with_error(
                "eof-in-script-html-comment-like-text",
            ),

            // Anything else
            //
            // Passer à l'état `script-data-escaped`. Émettre le
            // caractère actuel comme un jeton `character`.
            | Some(ch) => self
                .switch_state_to("script-data-escaped")
                .set_token(HTMLToken::Character(ch))
                .and_emit(),
        }
    }

    pub(crate) fn handle_script_data_escaped_less_than_sign_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
            // U+002F SOLIDUS (/)
            //
            // Définir le tampon temporaire à une chaîne de caractères
            // vide. Passer à l'état `script-data-escaped-end-tag-open`.
            | Some('/') => self
                .set_temporary_buffer(String::new())
                .switch_state_to("script-data-escaped-end-tag-open")
                .and_continue(),

            // ASCII alpha
            //
            // Définir le tampon temporaire à une chaîne de caractères
            // vide. Émettre un jeton `character` U+003C LESS-THAN SIGN.
            // Reprendre dans l'état `script-data-double-escape-start`.
            | Some(ch) if ch.is_ascii_alphabetic() => self
                .set_temporary_buffer(String::new())
                .emit_token(HTMLToken::Character('<'))
                .reconsume("script-data-double-escape-start")
                .and_continue(),

            // Anything else
            //
            // Émettre un jeton `character` U+003C LESS-THAN SIGN.
            // Reprendre dans l'état `script-data-escaped`.
            | _ => self
                .set_token(HTMLToken::Character('<'))
                .reconsume("script-data-escaped")
                .and_continue(),
        }
    }

    pub(crate) fn handle_script_data_escaped_end_tag_open_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
            // ASCII alpha
            //
            // Créer un nouveau jeton `end-tag`, définir son nom de balise
            // en une chaîne de caractères vide. Reprendre dans l'état
            // `script-data-escaped-end-tag-name`.
            | Some(ch) if ch.is_ascii_alphabetic() => self
                .set_token(HTMLToken::new_end_tag())
                .reconsume("script-data-escaped-end-tag-name")
                .and_continue(),

            // Anything else
            //
            // Émettre un jeton `character` U+003C LESS-THAN SIGN et un
            // jeton `character` U+002F SOLIDUS. Reprendre dans l'état
            // `script-data-escaped`.
            | _ => self
                .emit_token(HTMLToken::Character('<'))
                .emit_token(HTMLToken::Character('/'))
                .reconsume("script-data-escaped")
                .and_continue(),
        }
    }

    pub(crate) fn handle_script_data_escaped_end_tag_name_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            //
            // Si le jeton `end-tag` actuel est un jeton `end-tag`
            // approprié, passer à l'état `before-attribute-name`. Sinon,
            // le traiter comme indiqué dans l'entrée "anything else"
            // ci-dessous.
            | Some(ch)
                if ch.is_html_whitespace()
                    && self.is_appropriate_end_tag() =>
            {
                self.switch_state_to("before-attribute-name")
                    .and_continue()
            }

            // U+002F SOLIDUS (/)
            //
            // Si le jeton `end-tag` actuel est un jeton `end-tag`
            // approprié, passer à l'état `self-closing-start-tag`. Sinon,
            // le traiter comme indiqué dans l'entrée "anything else"
            // ci-dessous.
            | Some('/') if self.is_appropriate_end_tag() => self
                .switch_state_to("self-closing-start-tag")
                .and_continue(),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Si le jeton `end-tag` actuel est un jeton `end-tag`
            // approprié, passer à l'état `data`. Sinon, le traiter comme
            // indiqué dans l'entrée "anything else" ci-dessous.
            | Some('>') if self.is_appropriate_end_tag() => self
                .switch_state_to("self-closing-start-tag")
                .and_continue(),

            // ASCII upper alpha
            //
            // Ajouter la version en minuscules du caractère actuel
            // (ajouter 0x0020 au point de code du caractère) au nom
            // de balise du jeton `*-tag` actuel. Ajouter le caractère
            // actuel au tampon temporaire.
            | Some(ch) if ch.is_ascii_uppercase() => self
                .change_current_token(|token| {
                    token.append_character(ch.to_ascii_lowercase());
                })
                .append_character_to_temporary_buffer(ch)
                .and_continue(),

            // ASCII lower alpha
            //
            // Ajouter le caractère actuel au nom de balise du jeton
            // `*-tag` actuel. Ajouter le caractère actuel au tampon
            // temporaire.
            | Some(ch) if ch.is_ascii_lowercase() => self
                .change_current_token(|token| {
                    token.append_character(ch);
                })
                .append_character_to_temporary_buffer(ch)
                .and_continue(),

            // Anything else
            //
            // Émettre un jeton `character` U+003C LESS-THAN SIGN, un jeton
            // `character` U+002F SOLIDUS et un jeton `character` pour
            // chacun des caractères du tampon temporaire (dans l'ordre où
            // ils ont été ajoutés au tampon). Reprendre dans l'état
            // `script-data-escaped`.
            | _ => self
                .emit_token(HTMLToken::Character('<'))
                .emit_token(HTMLToken::Character('/'))
                .emit_each_characters_of_temporary_buffer()
                .reconsume("script-data-escaped")
                .and_continue(),
        }
    }

    pub(crate) fn handle_script_data_double_escape_start_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            // U+002F SOLIDUS (/)
            // U+003E GREATER-THAN SIGN (>)
            //
            // Si le tampon temporaire est la chaîne de caractères
            // "script", nous devons passer à l'état
            // `script-data-double-escaped`. Sinon, passer à l'état
            // `script-data-escaped`. Émettre le caractère actuel comme un
            // jeton `character`.
            | Some(ch)
                if (ch.is_html_whitespace()
                    || matches!(ch, '/' | '>')) =>
            {
                self.switch_state_to(
                    if self.temporary_buffer == "script" {
                        "script-data-double-escaped"
                    } else {
                        "script-data-escaped"
                    },
                )
                .set_token(HTMLToken::Character(ch))
                .and_emit()
            }

            // ASCII upper alpha
            //
            // Ajouter la version en minuscules du caractère d'entrée
            // actuel (ajouter 0x0020 au point de code du caractère) au
            // tampon temporaire. Émet le caractère actuel comme un jeton
            // `character`.
            | Some(ch) if ch.is_ascii_uppercase() => self
                .append_character_to_temporary_buffer(
                    ch.to_ascii_lowercase(),
                )
                .set_token(HTMLToken::Character(ch))
                .and_emit(),

            // ASCII lower alpha
            //
            // Ajouter le caractère actuel au tampon temporaire.
            // Émettre le caractère actuel comme un jeton `character`.
            | Some(ch) if ch.is_ascii_lowercase() => self
                .append_character_to_temporary_buffer(ch)
                .set_token(HTMLToken::Character(ch))
                .and_emit(),

            // Anything else
            //
            // Reprendre dans l'état `script-data-escaped`.
            | _ => self.reconsume("script-data-escaped").and_continue(),
        }
    }

    pub(crate) fn handle_script_data_double_escaped_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
            // U+002D HYPHEN-MINUS (-)
            //
            // Passer à l'état `script-data-double-escaped-dash`. Émettre
            // un jeton `character` U+002D HYPHEN-MINUS.
            | Some(ch @ '-') => self
                .switch_state_to("script-data-double-escaped-dash")
                .set_token(HTMLToken::Character(ch))
                .and_emit(),

            // U+003C LESS-THAN SIGN (<)
            //
            // Passer à l'état `script-data-double-escaped-less-than-sign`.
            // Émettre un jeton `character` U+003C LESS-THAN SIGN.
            | Some(ch @ '<') => self
                .switch_state_to(
                    "script-data-double-escaped-less-than-sign",
                )
                .set_token(HTMLToken::Character(ch))
                .and_emit(),

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
            // Il s'agit d'une erreur d'analyse de type
            // `eof-in-script-html-comment-like-text`. Émettre un jeton
            // `end-of-file`.
            | None => self.set_token(HTMLToken::EOF).and_emit_with_error(
                "eof-in-script-html-comment-like-text",
            ),

            // Anything else
            //
            // Émettre le caractère actuel comme un jeton `character`.
            | Some(ch) => {
                self.set_token(HTMLToken::Character(ch)).and_emit()
            }
        }
    }

    pub(crate) fn handle_script_data_double_escaped_dash_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
            // U+002D HYPHEN-MINUS (-)
            //
            // Passer à l'état `script-data-double-escaped-dash-dash`.
            // Émettre un jeton `character` U+002D HYPHEN-MINUS.
            | Some(ch @ '-') => self
                .switch_state_to("script-data-double-escaped-dash-dash")
                .set_token(HTMLToken::Character(ch))
                .and_emit(),

            // U+003C LESS-THAN SIGN (<)
            //
            // Passer à l'état `script-data-double-escaped-less-than-sign`.
            // Émettre un jeton `character` U+003C LESS-THAN SIGN.
            | Some(ch @ '<') => self
                .switch_state_to(
                    "script-data-double-escaped-less-than-sign",
                )
                .set_token(HTMLToken::Character(ch))
                .and_emit(),

            // U+0000 NULL
            //
            // Il s'agit d'une erreur d'analyse de type
            // `unexpected-null-character`. Passer à l'état
            // `script-data-double-escaped`. Émettre un jeton `character`
            // U+FFFD REPLACEMENT CHARACTER.
            | Some('\0') => self
                .switch_state_to("script-data-double-escaped")
                .set_token(HTMLToken::Character(
                    char::REPLACEMENT_CHARACTER,
                ))
                .and_emit_with_error("unexpected-null-character"),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type
            // `eof-in-script-html-comment-like-text`. Émettre un jeton
            // `end-of-file`.
            | None => self.set_token(HTMLToken::EOF).and_emit_with_error(
                "eof-in-script-html-comment-like-text",
            ),

            // Anything else
            //
            // Passer à l'état `script-data-double-escaped`. Émettre le
            // caractère actuel comme un jeton `character`.
            | Some(ch) => self
                .switch_state_to("script-data-double-escaped")
                .set_token(HTMLToken::Character(ch))
                .and_emit(),
        }
    }

    pub(crate) fn handle_script_data_double_escaped_dash_dash_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
            // U+002D HYPHEN-MINUS (-)
            //
            // Émettre un jeton `character` U+002D HYPHEN-MINUS.
            | Some(ch @ '-') => {
                self.set_token(HTMLToken::Character(ch)).and_emit()
            }

            // U+003C LESS-THAN SIGN (<)
            //
            // Passer à l'état `script-data-double-escaped-less-than-sign`.
            // Émettre un jeton `character` U+003C LESS-THAN SIGN.
            | Some(ch @ '<') => self
                .switch_state_to(
                    "script-data-double-escaped-less-than-sign",
                )
                .set_token(HTMLToken::Character(ch))
                .and_emit(),

            // U+003E GREATER-THAN SIGN (>)
            //
            // Passer à l'état `script-data`. Émettre un jeton `character`
            // U+003E GREATER-THAN SIGN.
            | Some(ch @ '>') => self
                .switch_state_to("script-data")
                .set_token(HTMLToken::Character(ch))
                .and_emit(),

            // U+0000 NULL
            //
            // Il s'agit d'une erreur d'analyse de type
            // `unexpected-null-character`. Passer à l'état
            // `script-data-double-escaped`. Émettre un jeton `character`
            // U+FFFD REPLACEMENT CHARACTER.
            | Some('\0') => self
                .switch_state_to("script-data-double-escaped")
                .set_token(HTMLToken::Character(
                    char::REPLACEMENT_CHARACTER,
                ))
                .and_emit_with_error("unexpected-null-character"),

            // EOF
            //
            // Il s'agit d'une erreur d'analyse de type
            // `eof-in-script-html-comment-like-text`. Émettre un jeton
            // `end-of-file`.
            | None => self.set_token(HTMLToken::EOF).and_emit_with_error(
                "eof-in-script-html-comment-like-text",
            ),

            // Anything else
            //
            // Passer à l'état `script-data-double-escaped`. Émettre le
            // caractère actuel comme un jeton `character`.
            | Some(ch) => self
                .switch_state_to("script-data-double-escaped")
                .set_token(HTMLToken::Character(ch))
                .and_emit(),
        }
    }

    pub(crate) fn handle_script_data_double_escaped_less_than_sign_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
            // U+002F SOLIDUS (/)
            //
            // Définir le tampon temporaire à une chaîne de caractères
            // vide. Passer à l'état `script-data-double-escape-end`.
            // Émettre un jeton `character` U+002F SOLIDUS.
            | Some(ch @ '/') => self
                .set_temporary_buffer(String::new())
                .switch_state_to("script-data-double-escape-end")
                .set_token(HTMLToken::Character(ch))
                .and_emit(),

            // Anything else
            //
            // Reprendre dans l'état `script-data-double-escaped`.
            | _ => {
                self.reconsume("script-data-double-escaped").and_continue()
            }
        }
    }

    pub(crate) fn handle_script_data_double_escape_end_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.input.consume_next_input_character() {
            // U+0009 CHARACTER TABULATION (tab)
            // U+000A LINE FEED (LF)
            // U+000C FORM FEED (FF)
            // U+0020 SPACE
            // U+002F SOLIDUS (/)
            // U+003E GREATER-THAN SIGN (>)
            //
            // Si le tampon temporaire est la chaîne de caractères
            // "script", nous devons passer à l'état `script-data-escaped`.
            // Sinon, passer à l'état `script-data-double-escaped`. Émettre
            // le caractère actuel comme un jeton `character`.
            | Some(ch)
                if ch.is_html_whitespace() && matches!(ch, '/' | '>') =>
            {
                self.switch_state_to(
                    if self.temporary_buffer == "script" {
                        "script-data-escaped"
                    } else {
                        "script-data-double-escaped"
                    },
                )
                .set_token(HTMLToken::Character(ch))
                .and_emit()
            }

            // ASCII upper alpha
            //
            // Ajouter la version en minuscules du caractère actuel
            // (ajouter 0x0020 au point de code du caractère) au
            // tampon temporaire. Émettre le caractère actuel comme
            // un jeton `character`.
            | Some(ch) if ch.is_ascii_uppercase() => self
                .append_character_to_temporary_buffer(
                    ch.to_ascii_lowercase(),
                )
                .set_token(HTMLToken::Character(ch))
                .and_emit(),

            // ASCII lower alpha
            //
            // Ajouter le caractère actuel au tampon temporaire. Émettre le
            // caractère actuel comme un jeton `character`.
            | Some(ch) if ch.is_ascii_lowercase() => self
                .append_character_to_temporary_buffer(ch)
                .set_token(HTMLToken::Character(ch))
                .and_emit(),

            // Anything else
            //
            // Reprendre dans l'état `script-data-double-escaped`.
            | _ => {
                self.reconsume("script-data-double-escaped").and_continue()
            }
        }
    }
}
