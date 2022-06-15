/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use infra::{
    primitive::codepoint::{
        CodePoint, CodePointInterface, CodePointIterator,
    },
    structure::lists::peekable::PeekableInterface,
};
use parser::StreamIteratorInterface;

use crate::tokenization::{
    tokenizer::{
        HTMLTokenizerProcessInterface, HTMLTokenizerProcessResult,
    },
    HTMLToken, HTMLTokenizer,
};

impl<C> HTMLTokenizer<C>
where
    C: CodePointIterator,
{
    pub(crate) fn handle_character_reference_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        self.set_temporary_buffer(String::new())
            .append_character_to_temporary_buffer('&');

        match self.stream.consume_next_input_character() {
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
    pub(crate) fn handle_named_character_reference_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        let ch = self.stream.current_input.expect("le caractère actuel");
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
                    self.stream.consume_next_input();
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

    pub(crate) fn handle_ambiguous_ampersand_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.stream.consume_next_input_character() {
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

    pub(crate) fn handle_numeric_character_reference_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        // Définir le code de référence du caractère à zéro (0).
        self.character_reference_code = 0;

        match self.stream.consume_next_input_character() {
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

    pub(crate) fn handle_hexadecimal_character_reference_start_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.stream.consume_next_input_character() {
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

    pub(crate) fn handle_decimal_character_reference_start_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.stream.consume_next_input_character() {
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

    pub(crate) fn handle_hexadecimal_character_reference_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.stream.consume_next_input_character() {
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

    pub(crate) fn handle_decimal_character_reference_state(
        &mut self,
    ) -> HTMLTokenizerProcessResult {
        match self.stream.consume_next_input_character() {
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

    pub(crate) fn handle_numeric_character_reference_end_state(
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
