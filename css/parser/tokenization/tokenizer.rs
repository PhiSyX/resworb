/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{
    borrow::Cow,
    ops::{AddAssign, MulAssign},
};

use infra::{
    primitive::codepoint::{
        CodePoint, CodePointInterface, CodePointIterator,
    },
    structure::lists::peekable::PeekableInterface,
};
use parser::{
    stream::{InputStream, TokenStream},
    StreamInputIterator, StreamIterator, StreamTokenIterator,
};

use super::{
    token::{DimensionUnit, NumberFlag},
    CSSToken, CSSTokenVariant,
};
use crate::{codepoint::CSSCodePoint, tokenization::token::HashFlag};

// ---- //
// Type //
// ---- //

type CSSInputStream<Iter> = InputStream<Iter, CodePoint>;

// --------- //
// Structure //
// --------- //

/// Pour tokeniser un flux de points de code en un flux de jetons CSS en
/// entrée, nous devons consommer de manière répétée un jeton en entrée
/// jusqu'à ce qu'un <EOF-token> soit atteint, en poussant chacun des
/// jetons retournés dans un flux.
#[derive(Debug)]
pub struct CSSTokenizer<Chars> {
    input: CSSInputStream<Chars>,
    current_token: Option<CSSTokenVariant>,
    is_replayed: bool,
}

// -------------- //
// Implémentation //
// -------------- //

impl<C> CSSTokenizer<C> {
    /// Crée un nouveau [tokenizer](CSSTokenizer) à partir d'un flux de
    /// points de code.
    pub(crate) fn new(chars: C) -> Self {
        // Remplacer tous les points de code
        //   - U+000D CARRIAGE RETURN (CR),
        //   - U+000C FORM FEED (FF)
        //   - U+000D CARRIAGE RETURN (CR) suivis de U+000A LINE FEED (LF)
        // par un seul point de code U+000A LINE FEED (LF).
        //
        // Remplacer tout point de code U+0000 NULL ou de substitution en
        // entrée par U+FFFD REPLACEMENT CHARACTER (�).
        let stream = CSSInputStream::new(chars).pre_scan(|ch| match ch {
            | Some('\r' | '\n' | '\x0C') => Some('\n'),
            | Some('\0') => Some(CodePoint::REPLACEMENT_CHARACTER),
            | Some(ch) if ch.is_surrogate() => {
                Some(CodePoint::REPLACEMENT_CHARACTER)
            }
            | n => n,
        });

        Self {
            input: stream,
            current_token: None,
            is_replayed: false,
        }
    }
}

impl<C> CSSTokenizer<C>
where
    C: CodePointIterator,
{
    pub(crate) fn stream(self) -> TokenStream<CSSTokenVariant> {
        TokenStream::new(self)
    }
}

impl<C> CSSTokenizer<C>
where
    C: CodePointIterator,
{
    /// Voir <https://www.w3.org/TR/css-syntax-3/#consume-comment>
    fn consume_comments(&mut self) {
        'f: loop {
            if !self.input.consume_next_input_character_if_are(['/', '*'])
            {
                break 'f;
            }

            's: loop {
                if self
                    .input
                    .consume_next_input_character_if_are(['*', '/'])
                {
                    break 's;
                } else if self.input.next_input().is_none() {
                    // TODO(css): gérer les erreurs.
                    break 'f;
                } else {
                    self.input.advance(1);
                }
            }
        }
    }

    /// Voir <https://www.w3.org/TR/css-syntax-3/#consume-an-ident-like-token>
    fn consume_ident_like_token(&mut self) -> CSSToken {
        let result = self.consume_ident_sequence();

        if result.eq_ignore_ascii_case("url") {
            if let Some('(') = self.input.next_input_character() {
                self.input.advance(1);

                self.input.advance_as_long_as_possible_with_limit(
                    |ch| ch.is_css_whitespace(),
                    2,
                );

                if let Some(v) = self.input.peek_until::<Vec<_>>(2) {
                    let cond0 = v[0] == '\'' || v[0] == '"';
                    let cond1 = v[1] == '\'' || v[1] == '"';
                    let cond2 = v[0] == ' ' && cond1;

                    if cond0 && cond1 || cond2 {
                        return CSSToken::Function(result);
                    }
                }

                return self.consume_url_token();
            }
        }

        if let Some('(') = self.input.next_input_character() {
            self.input.advance(1);
            return CSSToken::Function(result);
        }

        CSSToken::Ident(result)
    }

    /// Voir <https://www.w3.org/TR/css-syntax-3/#consume-number>
    //
    // NOTE(phisyx): `type` est un mot clé réservé de Rust.
    //               C'est pour cela qu'on ne respecte pas vraiment la
    //               nomenclature de la présente spécification dans le
    //               cas présent.
    fn consume_number(&mut self) -> Option<(f64, NumberFlag)> {
        let mut flag = NumberFlag::Integer;
        // ---- ^^^^ : voir la NOTE ci-haut.
        let mut repr = String::new();

        if let Some(ch @ ('+' | '-')) = self.input.next_input_character() {
            repr.push(ch);
            self.input.advance(1);
        }

        let digits = self
            .input
            .advance_as_long_as_possible(|next_ch| next_ch.is_css_digit());

        repr.extend(&digits);

        if let Some(v) = self.input.peek_until::<Vec<_>>(2) {
            if v[0] == '.' && v[1].is_css_digit() {
                self.input.advance(2);
                repr.extend(&v);
                flag = NumberFlag::Number;
                repr.extend(&self.input.advance_as_long_as_possible(
                    |next_ch| next_ch.is_css_digit(),
                ));
            }
        }

        if let Some(v) = self.input.peek_until::<Vec<_>>(3) {
            let is_minus_or_plus = v[1] == '-' || v[1] == '+';
            if v[0].to_ascii_lowercase() == 'e'
                && (is_minus_or_plus && v[2].is_css_digit())
                || v[1].is_css_digit()
            {
                let offset = if is_minus_or_plus { 3 } else { 2 };
                self.input.advance(offset);
                repr.extend(&v[..offset]);
                flag = NumberFlag::Number;
                repr.extend(&self.input.advance_as_long_as_possible(
                    |next_ch| next_ch.is_css_digit(),
                ));
            }
        }

        let value = convert_string_to_number(repr);
        value.map(|v| (v, flag))
    }

    /// Voir <https://www.w3.org/TR/css-syntax-3/#consume-numeric-token>
    fn consume_numeric_token(&mut self) -> CSSToken {
        let (number, number_flag) =
            self.consume_number().expect("Nombre attendu");

        if check_3_codepoints_would_start_an_ident_sequence(
            self.input.next_n_input_character(3),
        ) {
            return CSSToken::Dimension(
                number,
                number_flag,
                DimensionUnit(self.consume_ident_sequence()),
            );
        }

        if let Some('%') = self.input.next_input_character() {
            self.input.advance(1);
            return CSSToken::Percentage(number);
        }

        CSSToken::Number(number, number_flag)
    }

    fn consume_string_token(
        &mut self,
        ending_codepoint: CodePoint,
    ) -> CSSToken {
        let mut string = String::new();

        loop {
            match self.input.consume_next_input_character() {
                // ending code point
                //
                // Retourner un <string-token>.
                | Some(ch) if ch == ending_codepoint => break,

                // EOF
                //
                // Il s'agit d'une erreur d'analyse. Retourner un
                // <string-token>.
                | None => {
                    // TODO(phisyx): gérer l'erreur.
                    break;
                }

                // newline
                //
                // Il s'agit d'une erreur d'analyse. Re-consommer le
                // point de code d'entrée actuel, créer un
                // <bad-string-token> et le retourner.
                | Some(ch) if ch.is_newline() => {
                    // TODO(phisyx): gérer l'erreur.
                    self.input.reconsume_current_input();
                    return CSSToken::BadString;
                }

                // U+005C REVERSE SOLIDUS (\)
                //
                // Si le prochain point de code d'entrée est EOF, ne rien
                // faire.
                // Sinon, si le prochain point de code d'entrée est une
                // nouvelle ligne, nous devons le consommer.
                // Sinon, (le flux commence par un échappement valide)
                // consommer un point de code échappé et ajouter le point
                // de code renvoyé à la valeur de <string-token>.
                | Some('\\') => {
                    match self.input.next_input_character() {
                        | Some(ch) if ch.is_newline() => {
                            self.input.advance(1);
                        }
                        | _ => {}
                    };

                    if check_2_codepoints_are_a_valid_escape(
                        self.input.next_n_input_character(2),
                    ) {
                        self.input.advance(2);
                        string.push_str("\\\\");
                    }
                }

                // Anything else
                //
                // Ajouter le point de code d'entrée actuel à la valeur de
                // <string-token>.
                | _ => {
                    let current_ch = self
                        .input
                        .current_input()
                        .cloned()
                        .expect(
                        "Le caractère courant, qui a forcément déjà été \
                             assigné.",
                    );
                    string.push(current_ch);
                }
            }
        }

        CSSToken::String(string)
    }

    /// Voir <https://www.w3.org/TR/css-syntax-3/#consume-token>
    fn consume_token(&mut self) -> CSSToken {
        // Consume comments.
        self.consume_comments();

        fn delim(maybe_current_ch: Option<&CodePoint>) -> CSSToken {
            CSSToken::Delim(*maybe_current_ch.expect("Caractère courant"))
        }

        // Consume the next input code point.
        match self.input.consume_next_input_character() {
            // whitespace
            //
            // Consomme autant d'espace blanc que possible. Retourne un
            // <whitespace-token>.
            | Some(ch) if ch.is_css_whitespace() => {
                self.input.advance_as_long_as_possible(|ch| {
                    ch.is_css_whitespace()
                });
                CSSToken::Whitespace
            }

            // U+0022 QUOTATION MARK (")
            // U+0027 APOSTROPHE (')
            //
            // Consomme un jeton de chaîne et le renvoie.
            | Some(ch @ ('"' | '\'')) => self.consume_string_token(ch),

            // U+0023 NUMBER SIGN (#)
            //
            // Si le point de code d'entrée suivant est un point de code
            // ident ou si les deux points de code d'entrée suivants sont
            // un échappement valide, alors nous devons:
            //   1. Créer un <hash-token>.
            //   2. Si les 3 points de code d'entrée suivants commencent
            // une par une séquence ident, nous devons définir un drapeau
            // de type <hash-token> sur "id".
            //   3. Consommer une séquence ident, et définir la valeur du
            // <hash-token> à la chaîne retournée.
            //   4. Retourner le <hash-token>.
            //
            // Sinon, retourner un <delim-token> avec sa valeur définie sur
            // le point de code d'entrée actuel.
            | Some('#') => {
                fn then<C>(tokenizer: &mut CSSTokenizer<C>) -> CSSToken
                where
                    C: CodePointIterator,
                {
                    let mut flag = Default::default();
                    let mut hash = String::new();

                    if check_3_codepoints_would_start_an_ident_sequence(
                        tokenizer.input.next_n_input_character(3),
                    ) {
                        flag = HashFlag::ID;
                    }

                    hash.push_str(&tokenizer.consume_ident_sequence());

                    CSSToken::Hash(hash, flag)
                }

                if let Some(ch) = self.input.next_input_character() {
                    if ch.is_ident_codepoint() {
                        return then(self);
                    }
                }

                if check_2_codepoints_are_a_valid_escape(
                    self.input.next_n_input_character(2),
                ) {
                    return then(self);
                }

                delim(self.input.current_input())
            }

            // U+0028 LEFT PARENTHESIS (()
            //
            // Consomme un jeton de type <(-token>.
            | Some('(') => CSSToken::LeftParenthesis,

            // U+0029 RIGHT PARENTHESIS ())
            //
            // Retourne un <)-token>.
            | Some(')') => CSSToken::RightParenthesis,

            // U+002B PLUS SIGN (+)
            //
            // Si le flux d'entrée commence par un nombre, nous devons
            // re-consommer le point de code d'entrée actuel, consommer un
            // jeton numérique et le retourner.
            | Some('+')
                if check_3_codepoints_would_start_a_number(
                    self.input.next_n_input_character(3),
                ) =>
            {
                self.input.reconsume_current_input();
                self.consume_numeric_token()
            }

            // U+002B PLUS SIGN (+)
            //
            // Retourner un <delim-token> dont la valeur est fixée
            // au point de code d'entrée actuel.
            | Some('+') => delim(self.input.current_input()),

            // U+002C COMMA (,)
            //
            // Retourner un <comma-token>.
            | Some(',') => CSSToken::Comma,

            // U+002D HYPHEN-MINUS (-)
            //
            // Si le flux d'entrée commence par un nombre, nous devons
            // re-consommer le point de code d'entrée actuel, consommer un
            // jeton numérique et le retourner.
            | Some('-')
                if check_3_codepoints_would_start_a_number(
                    self.input.next_n_input_character(3),
                ) =>
            {
                self.input.reconsume_current_input();
                self.consume_numeric_token()
            }

            // U+002D HYPHEN-MINUS (-)
            //
            // Si les 2 prochains points de code d'entrée sont U+002D
            // HYPHEN-MINUS U+003E GREATER-THAN SIGN (->), les consommer et
            // retourner un <CDC-token>.
            | Some('-') if self.input.next_n_input_character(2) == "->" => {
                self.input.advance(2);
                CSSToken::CDC
            }

            // U+002D HYPHEN-MINUS (-)
            //
            // si le flux d'entrée commence par une séquence ident,
            // re-consommer le point de code d'entrée actuel, consommer un
            // jeton de type ident, et le retourner.
            | Some('-')
                if check_3_codepoints_would_start_an_ident_sequence(
                    self.input.next_n_input_character(3),
                ) =>
            {
                self.input.reconsume_current_input();
                self.consume_ident_like_token()
            }

            // U+002D HYPHEN-MINUS (-)
            //
            // Retourner un <delim-token> dont la valeur est fixée au
            // point de code d'entrée actuel.
            | Some('-') => delim(self.input.current_input()),

            // U+002E FULL STOP (.)
            //
            // Si le flux d'entrée commence par un nombre, nous devons
            // re-consommer le point de code d'entrée actuel, consommer un
            // jeton numérique et le retourner.
            | Some('.')
                if check_3_codepoints_would_start_a_number(
                    self.input.next_n_input_character(3),
                ) =>
            {
                self.input.reconsume_current_input();
                self.consume_numeric_token()
            }

            // U+002E FULL STOP (.)
            //
            // Retourner un <delim-token> dont la valeur est fixée au
            // point de code d'entrée actuel.
            | Some('.') => delim(self.input.current_input()),

            // U+003A COLON (:)
            //
            // Retourner un <colon-token>.
            | Some(':') => CSSToken::Colon,

            // U+003B SEMICOLON (;)
            //
            // Retourner un <semicolon-token>.
            | Some(';') => CSSToken::Semicolon,

            // U+003C LESS-THAN SIGN (<)
            //
            // Si les 3 points de code d'entrée suivants sont U+0021
            // EXCLAMATION MARK U+002D HYPHEN-MINUS U+002D HYPHEN-MINUS
            // (!--), les consommer et retourner un <CDO-token>.
            | Some('<') if self.input.next_n_input_character(3) == "!--" =>
            {
                self.input.advance(3);
                CSSToken::CDO
            }

            // U+003C LESS-THAN SIGN (<)
            //
            // Retourner un <delim-token> dont la valeur est fixée au
            // point de code d'entrée actuel.
            | Some('<') => delim(self.input.current_input()),

            // U+0040 COMMERCIAL AT (@)
            //
            // Si les 3 points de code d'entrée qui suivent démarrent une
            // séquence d'identification, consommer une séquence
            // d'identification, créer un <at-keyword-token> avec sa
            // valeur définie sur la valeur renvoyée, et le retourner.
            | Some('@')
                if check_3_codepoints_would_start_an_ident_sequence(
                    self.input.next_n_input_character(3),
                ) =>
            {
                CSSToken::AtKeyword(self.consume_ident_sequence())
            }

            // U+0040 COMMERCIAL AT (@)
            //
            // Retourner un <delim-token> dont la valeur est fixée au
            // point de code d'entrée actuel.
            | Some('@') => delim(self.input.current_input()),

            // U+005B LEFT SQUARE BRACKET ([)
            //
            // Retourner un <[-token>.
            | Some('[') => CSSToken::LeftSquareBracket,

            // U+005C REVERSE SOLIDUS (\)
            //
            // Si le flux d'entrée commence par un échappement valide, nous
            // devons re-consommer le point de code d'entrée actuel,
            // consommer un jeton de type ident-like, et le retourner.
            | Some('\\')
                if check_3_codepoints_would_start_an_ident_sequence(
                    self.input.next_n_input_character(3),
                ) =>
            {
                self.input.reconsume_current_input();
                self.consume_ident_like_token()
            }

            // U+005C REVERSE SOLIDUS (\)
            //
            // Retourner un <delim-token> dont la valeur est fixée au
            // point de code d'entrée actuel.
            | Some('\\') => delim(self.input.current_input()),

            // U+005D RIGHT SQUARE BRACKET (])
            //
            // Retourner un <]-token>.
            | Some(']') => CSSToken::RightSquareBracket,

            // U+007B LEFT CURLY BRACKET ({)
            //
            // Retourner un <{-token>.
            | Some('{') => CSSToken::LeftCurlyBracket,

            // U+007D RIGHT CURLY BRACKET (})
            //
            // Retourner un <}-token>.
            | Some('}') => CSSToken::RightCurlyBracket,

            // digit
            //
            // Re-consommer le point de code d'entrée actuel, consommer un
            // jeton numérique et le retourner.
            | Some(ch) if ch.is_css_digit() => {
                self.input.reconsume_current_input();
                self.consume_numeric_token()
            }

            // ident-start code point
            //
            // Re-consommer le point de code d'entrée actuel, consommer un
            // jeton de type ident-like, et le retourner.
            | Some(ch) if ch.is_ident_start_codepoint() => {
                self.input.reconsume_current_input();
                self.consume_ident_like_token()
            }

            // EOF
            //
            // Retourner un <EOF-token>.
            | None => CSSToken::EOF,

            // Anything else
            | _ => delim(self.input.current_input()),
        }
    }

    /// Voir <https://www.w3.org/TR/css-syntax-3/#consume-name>
    fn consume_ident_sequence(&mut self) -> String {
        let mut result = String::new();

        loop {
            if let Some(next_ch) = self.input.consume_next_input() {
                if next_ch.is_ident_codepoint() {
                    result.push(next_ch);
                    continue;
                }

                if let Some(next_peek_ch) =
                    self.input.next_input_character()
                {
                    if check_2_codepoints_are_a_valid_escape(format!(
                        "{next_ch}{next_peek_ch}"
                    )) {
                        result.push(self.consume_escaped_codepoint());
                        continue;
                    }
                }

                self.input.reconsume_current_input();
            }

            break;
        }

        result
    }

    /// Voir <https://www.w3.org/TR/css-syntax-3/#consume-escaped-code-point>
    fn consume_escaped_codepoint(&mut self) -> CodePoint {
        match self.input.consume_next_input_character() {
            // hex digit
            //
            // Consommer autant de chiffres hexadécimaux que possible, mais
            // pas plus de 5.
            // NOTE(css): cela signifie que 1 à 6 chiffres hexadécimaux ont
            // été consommés au total.
            // Si le prochain point de code d'entrée est un espace blanc,
            // nous devons le consommer. Interpréter les chiffres
            // hexadécimaux comme un nombre hexadécimal. Si ce nombre est
            // zéro, ou s'il s'agit d'un substitut, ou s'il est supérieur
            // au point de code maximum autorisé, retourner U+FFFD
            // REPLACEMENT CHARACTER (�). Sinon, retourner le point de code
            // avec cette valeur.
            | Some(ch) if ch.is_ascii_hexdigit() => {
                const HEXARADIX: u32 = 16;

                // NOTE(phisyx): nous pouvons utiliser .expect() ici, sans
                // que ce soit problématique, car la condition ci-dessus
                // vérifie que `ch` s' agit bien d'une valeur hexadécimale.
                let total_hexdigits = self
                    .input
                    .advance_as_long_as_possible_with_limit(
                        |ch| ch.is_ascii_digit(),
                        5,
                    )
                    .iter()
                    .fold(
                        ch.to_digit(HEXARADIX)
                            .expect("Voir la note ci-dessus"),
                        |mut total, ch| {
                            total.mul_assign(HEXARADIX);
                            total.add_assign(
                                ch.to_digit(HEXARADIX)
                                    .expect("Voir la note ci-dessus"),
                            );
                            total
                        },
                    );

                let next_peek_ch = self.input.next_input_character();
                if let Some('\n') = next_peek_ch {
                    self.input.advance(1);
                }

                let hexnumber = CodePoint::from_u32(total_hexdigits)
                    .unwrap_or(CodePoint::REPLACEMENT_CHARACTER);

                // NOTE(phisyx): n'a peut-être pas le comportement attendue
                // par la spécification ; à tester.
                if hexnumber == '\0'
                    || hexnumber.is_surrogate()
                    || hexnumber.is_gt_maximum_allowed_codepoint()
                {
                    CodePoint::REPLACEMENT_CHARACTER
                } else {
                    hexnumber
                }
            }

            // EOF
            //
            // Il s'agit d'une erreur d'analyse. Retourner U+FFFD
            // REPLACEMENT CHARACTER (�).
            // TODO(phisyx): gérer cette erreur d'analyse.
            | None => CodePoint::REPLACEMENT_CHARACTER,

            // Anything else
            //
            // Retourner le point de code d'entrée actuel.
            | _ => self.input.current_input().cloned().expect(
                "Le caractère courant, qui a forcément déjà été assigné.",
            ),
        }
    }

    /// Voir <https://www.w3.org/TR/css-syntax-3/#consume-remnants-of-bad-url>
    fn consume_remnants_of_bad_url(&mut self) {
        loop {
            match self.input.consume_next_input() {
                // U+0029 RIGHT PARENTHESIS ())
                // EOF
                //
                // Retourner.
                | Some(')') | None => break,

                // Anything else
                | Some(ch) => {
                    // the input stream starts with a valid escape
                    //
                    // Consommer un point de code échappé.
                    // NOTE(css): cela permet de rencontrer une parenthèse
                    // droite échappée ("\)") sans terminer le
                    // <bad-url-token>. Cette clause est par ailleurs
                    // identique à la clause "anything else".
                    if let Some(next_peek_ch) =
                        self.input.next_input_character()
                    {
                        if check_2_codepoints_are_a_valid_escape(format!(
                            "{ch}{next_peek_ch}"
                        )) {
                            self.consume_escaped_codepoint();
                        }
                    }
                }
            }
        }
    }

    /// Voir <https://www.w3.org/TR/css-syntax-3/#consume-url-token>
    fn consume_url_token(&mut self) -> CSSToken {
        self.input.advance_as_long_as_possible(|next_ch| {
            next_ch.is_css_whitespace()
        });

        let mut url_token = CSSToken::Url(String::default());

        loop {
            match self.input.consume_next_input() {
                // U+0029 RIGHT PARENTHESIS ())
                //
                // Retourner un <url-token>.
                | Some(')') => break,

                // EOF
                | None => break,

                // whitespace
                //
                // Consommer autant d'espace blanc que possible. Si le
                // prochain point de code d'entrée est U+0029 RIGHT
                // PARENTHESIS ()) ou EOF, nous devons le consommer et
                // retourner le <url-token> (si EOF a été rencontré, c'est
                // une erreur d'analyse) ; sinon, nous devons consommer les
                // restes d'une mauvaise url, créer un <bad-url-token>, et
                // le retourner.
                | Some(ch) if ch.is_css_whitespace() => {
                    self.input.advance_as_long_as_possible(|next_ch| {
                        next_ch.is_css_whitespace()
                    });

                    if let next_peek_ch @ (Some(')') | None) =
                        self.input.next_input_character()
                    {
                        self.input.advance(1);

                        if next_peek_ch.is_none() { // eof
                             // TODO(phisyx): gérer les erreurs.
                        }

                        break;
                    }

                    self.consume_remnants_of_bad_url();
                    return CSSToken::BadUrl;
                }

                // U+0022 QUOTATION MARK (")
                // U+0027 APOSTROPHE (')
                // U+0028 LEFT PARENTHESIS (()
                //
                // Il s'agit d'une erreur d'analyse. Consommer les
                // restes d'une mauvaise url, créer un <bad-url-token>,
                // et le retourner.
                | Some('"' | '\'' | '(') => {
                    self.consume_remnants_of_bad_url();
                    return CSSToken::BadUrl;
                }
                // non-printable code point
                //
                // Il s'agit d'une erreur d'analyse. Consommer les
                // restes d'une mauvaise url, créer un <bad-url-token>,
                // et le retourner.
                | Some(ch) if ch.is_non_printable_codepoint() => {
                    self.consume_remnants_of_bad_url();
                    return CSSToken::BadUrl;
                }

                // U+005C REVERSE SOLIDUS (\)
                //
                // Si le flux commence par un échappement valide, nous
                // devons consommer un point de code échappé et ajoute le
                // point de code renvoyé à la valeur de <url-token>.
                //
                // Sinon, il s'agit d'une erreur d'analyse. Consomme les
                // restes d'une mauvaise url, crée un <bad-url-token>, et
                // le renvoie.
                | Some(ch @ '\\') => {
                    if let Some(next_peek_ch) =
                        self.input.next_input_character()
                    {
                        if check_2_codepoints_are_a_valid_escape(format!(
                            "{ch}{next_peek_ch}",
                        )) {
                            url_token.append_character(
                                self.consume_escaped_codepoint(),
                            );
                        } else {
                            // TODO(phisyx): gérer les erreurs.
                            self.consume_remnants_of_bad_url();
                            return CSSToken::BadUrl;
                        }
                    }
                }

                // Anything else
                //
                // Ajouter le point de code d'entrée actuel à la valeur de
                // <url-token>.
                | Some(ch) => url_token.append_character(ch),
            }
        }

        url_token
    }
}

/// Vérifie si trois points de code permettent de lancer une séquence
/// ident.
fn check_3_codepoints_would_start_an_ident_sequence(
    maybe_ident_sequence: Cow<str>,
) -> bool {
    let mut chars = maybe_ident_sequence.chars();
    let first_codepoint = chars.next();
    match first_codepoint {
        // U+002D HYPHEN-MINUS
        //
        // Si le deuxième point de code est un point de code de début
        // ident ou un HYPHEN-MINUS U+002D, ou si le deuxième et
        // troisième points de code sont des échappements valides,
        // nous devons alors retourner true. Sinon false.
        | Some('-') => {
            let second_codepoint = chars.next();
            match second_codepoint {
                | Some(ch) if ch.is_ident_start_codepoint() => true,
                | Some('-') => true,
                | Some('\\') => {
                    let third_codepoint = chars.next();
                    match third_codepoint {
                        | Some('\\') => true,
                        | _ => false,
                    }
                }
                | _ => false,
            }
        }

        // ident-start code point
        | Some(ch) if ch.is_ident_start_codepoint() => true,

        // U+005C REVERSE SOLIDUS (\)
        //
        // Si le premier et le second point de code sont des
        // échappements valides, nous devons retourner
        // true. Sinon false.
        | Some('\\') => {
            let second_codepoint = chars.next();
            match second_codepoint {
                | Some('\\') => true,
                | _ => false,
            }
        }

        // Anything else
        //
        // Retourner false.
        | _ => false,
    }
}

/// Vérifier si trois points de code permettent de commencer un numéro
fn check_3_codepoints_would_start_a_number(
    maybe_number: Cow<str>,
) -> bool {
    let mut chars = maybe_number.chars();
    let first_codepoint = chars.next();
    match first_codepoint {
        // U+002B PLUS SIGN (+)
        // U+002D HYPHEN-MINUS (-)
        //
        // Si le deuxième point de code est un chiffre, nous devons
        // retourner true.
        // Sinon, si le deuxième point de code est un U+002E FULL STOP (.)
        // et le troisième point de code est un chiffre, nous devons
        // retourner true.
        // Sinon false.
        | Some('+' | '-') => {
            let second_codepoint = chars.next();
            match second_codepoint {
                | Some(ch) if ch.is_css_digit() => true,
                | Some('.') => {
                    let third_codepoint = chars.next();
                    match third_codepoint {
                        | Some(ch) if ch.is_css_digit() => true,
                        | _ => false,
                    }
                }
                | _ => false,
            }
        }

        // U+002E FULL STOP (.)
        //
        // Si le deuxième point de code est un chiffre, nous devons
        // retourner true. Sinon false.
        | Some('.') => {
            let second_codepoint = chars.next();
            match second_codepoint {
                | Some(ch) if ch.is_css_digit() => true,
                | _ => false,
            }
        }

        // digit
        | Some(ch) if ch.is_css_digit() => true,

        // Anything else
        | _ => false,
    }
}

/// Vérifie si deux points de code constituent un échappement valide.
fn check_2_codepoints_are_a_valid_escape(
    maybe_valid_escape: impl AsRef<str>,
) -> bool {
    let mut chars = maybe_valid_escape.as_ref().chars();

    let first_codepoint = chars.next();
    if first_codepoint != Some('\\') {
        return false;
    }

    let second_codepoint = chars.next();
    if second_codepoint == Some('\n') {
        return false;
    }

    true
}

/// Voir <https://www.w3.org/TR/css-syntax-3/#convert-string-to-number>
/// Le langage Rust implémente déjà cela.
fn convert_string_to_number(s: String) -> Option<f64> {
    s.parse().ok()
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl<C> StreamTokenIterator for CSSTokenizer<C>
where
    C: CodePointIterator,
{
    type Token = CSSTokenVariant;

    fn consume_next_token(&mut self) -> Option<Self::Token> {
        if self.is_replayed {
            self.is_replayed = false;
            return self.current_token.clone();
        }

        self.current_token = self.next_token();
        self.current_token.clone()
    }

    fn current_token(&self) -> Option<&Self::Token> {
        self.current_token.as_ref()
    }

    fn next_token(&mut self) -> Option<Self::Token> {
        Some(self.consume_token().into())
    }

    fn reconsume_current_token(&mut self) {
        self.is_replayed = true;
    }
}

// ---- //
// Test //
// ---- //

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenization::token::HashFlag;

    macro_rules! load_fixture {
        ($filename:literal) => {{
            let css_file = include_str!($filename);
            CSSTokenizer::new(css_file.chars())
        }};
    }

    macro_rules! test_the_str {
        ($str:literal) => {{
            let s = $str;
            CSSTokenizer::new(s.chars())
        }};
    }

    #[test]
    fn test_consume_comments() {
        let mut tokenizer = test_the_str!(
            "/* comment 1 */\r\n#id { color: red }/* comment 2 */"
        );
        assert_eq!(tokenizer.consume_token(), CSSToken::Whitespace);
    }

    #[test]
    fn test_consume_token_quotation_mark() {
        let mut tokenizer = test_the_str!("'hello world'");
        assert_eq!(
            tokenizer.consume_token(),
            CSSToken::String("hello world".into())
        );

        let mut tokenizer = test_the_str!(r#""foo bar""#);
        assert_eq!(
            tokenizer.consume_token(),
            CSSToken::String("foo bar".into())
        );

        let mut tokenizer = test_the_str!(r#""foo'bar""#);
        assert_eq!(
            tokenizer.consume_token(),
            CSSToken::String("foo'bar".into())
        );

        let mut tokenizer = test_the_str!(r#""foo"bar""#);
        assert_eq!(
            tokenizer.consume_token(),
            CSSToken::String("foo".into())
        );

        let mut tokenizer = test_the_str!("\"bad\nstring\"");
        assert_eq!(tokenizer.consume_token(), CSSToken::BadString);
    }

    #[test]
    fn test_consume_token_number_sign() {
        let mut tokenizer = test_the_str!("#id { color: #000 }");

        assert_eq!(
            tokenizer.consume_token(),
            CSSToken::Hash("id".into(), HashFlag::ID)
        );

        let mut tokenizer = test_the_str!("#id\\:2 {color: red; }");

        assert_eq!(
            tokenizer.consume_token(),
            CSSToken::Hash("id:2".into(), HashFlag::ID)
        );

        // TODO(phisyx): tester les couleurs de ce test.
    }

    #[test]
    fn test_consume_ident_like() {
        let mut tokenizer =
            test_the_str!("#id { background: url(img.png); }");

        for _ in 0..7 {
            tokenizer.consume_token();
        }

        assert_eq!(
            tokenizer.consume_token(),
            CSSToken::Url("img.png".into())
        );

        let mut tokenizer =
            test_the_str!("#id { transform: translateX(0deg); }");

        for _ in 0..7 {
            tokenizer.consume_token();
        }

        assert_eq!(
            tokenizer.consume_token(),
            CSSToken::Function("translateX".into())
        );

        let mut tokenizer = test_the_str!("#id { color: red; }");

        for _ in 0..7 {
            tokenizer.consume_token();
        }

        assert_eq!(
            tokenizer.consume_token(),
            CSSToken::Ident("red".into())
        );
    }
}
