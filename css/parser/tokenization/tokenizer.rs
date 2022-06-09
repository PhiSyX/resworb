/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;

use infra::primitive::codepoint::CodePoint;
use parser::preprocessor::InputStream;

use super::CSSToken;
use crate::{codepoint::CSSCodePoint, tokenization::token::HashFlag};

// ---- //
// Type //
// ---- //

pub(crate) type CSSInputStream<Iter> = InputStream<Iter, CodePoint>;

// --------- //
// Structure //
// --------- //

/// Pour tokeniser un flux de points de code en un flux de jetons CSS en
/// entrée, nous devons consommer de manière répétée un jeton en entrée
/// jusqu'à ce qu'un <EOF-token> soit atteint, en poussant chacun des
/// jetons retournés dans un flux.
pub struct CSSTokenizer<Chars> {
    pub(crate) stream: CSSInputStream<Chars>,
}

// -------------- //
// Implémentation //
// -------------- //

impl<C> CSSTokenizer<C>
where
    C: Iterator<Item = CodePoint>,
{
    /// Crée un nouveau [tokenizer](CSSTokenizer) à partir d'un flux de
    /// points de code.
    pub fn new(iter: C) -> Self {
        // Remplacer tous les points de code
        //   - U+000D CARRIAGE RETURN (CR),
        //   - U+000C FORM FEED (FF)
        //   - U+000D CARRIAGE RETURN (CR) suivis de U+000A LINE FEED (LF)
        // par un seul point de code U+000A LINE FEED (LF).
        //
        // Remplacer tout point de code U+0000 NULL ou de substitution en
        // entrée par U+FFFD REPLACEMENT CHARACTER (�).
        let stream =
            CSSInputStream::new(iter).with_pre_scan(|ch| match ch {
                | Some('\r' | '\n' | '\x0C') => Some('\n'),
                | Some('\0') => Some(CodePoint::REPLACEMENT_CHARACTER),
                | n => n,
            });

        Self { stream }
    }
}

impl<C> CSSTokenizer<C>
where
    C: Iterator<Item = CodePoint>,
{
    fn consume_comments(&mut self) {
        'f: loop {
            let start = self.stream.next_n_input_character(2);

            if start != "/*" {
                break 'f;
            }

            self.stream.advance(2);

            's: loop {
                let last = self.stream.next_n_input_character(2);

                if last == "*/" {
                    self.stream.advance(2);
                    break 's;
                } else {
                    self.stream.advance(1);
                }
            }
        }
    }
    fn consume_token(&mut self) -> Option<CSSToken> {
        // Consume comments.
        self.consume_comments();

        // Consume the next input code point.
        match self.stream.consume_next_input_character() {
            // whitespace
            //
            // Consomme autant d'espace blanc que possible. Retourne un
            // <whitespace-token>.
            | Some(ch) if ch.is_css_whitespace() => {
                self.stream.advance_as_long_as(|next_ch| {
                    next_ch.is_css_whitespace()
                });
                Some(CSSToken::Whitespace)
            }
            // Anything else
            | _ => self.stream.current.map(CSSToken::Delim),
        }
    }
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

        // NOTE: tester si le premier caractère n'est pas '/'
        //       actuellement le script retourne None.
        assert_eq!(tokenizer.consume_token(), Some(CSSToken::Whitespace));
    }
}
