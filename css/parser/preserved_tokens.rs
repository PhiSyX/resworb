/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops;

use crate::tokenization::CSSToken;

// --------- //
// Structure //
// --------- //

/// Tout token produit par le
/// [tokenizer](crate::tokenization::CSSTokenizer) à l'exception des
/// <[fonction-token](crate::tokenization::CSSToken::Function)>s,
/// <[{-token](LeftCurlyBracket)>s, <[(-token](LeftParenthesis)>s, et
/// <[\[-token](LeftSquareBracket)>s.
///
/// NOTE(css): les jetons non conservés listés ci-dessus sont toujours
/// consommés dans des objets de plus haut niveau, soit des fonctions ou
/// des blocs simples, et n'apparaissent donc jamais dans la sortie de
/// l'analyseur.
///
/// NOTE(css): les jetons
/// <[}-token](RightCurlyBracket)>s, <[(-token](RightParenthesis)>s,
/// <[\[-token](RightSquareBracket)>s,
/// <[bad-string-token](crate::tokenization::CSSToken::BadString)>, et
/// <[bad-url-token](crate::tokenization::CSSToken::BadUrl)> sont toujours
/// des erreurs d'analyse, mais ils sont préservés dans le flux de tokens
/// par cette spécification pour permettre à d'autres spécifications,
/// telles que Media Queries, de définir une gestion des erreurs plus fine
/// que le simple abandon d'une déclaration ou d'un bloc entier.
#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq, Eq)]
pub struct CSSPreservedToken(pub(super) CSSToken);

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq, Eq)]
pub enum CSSPreservedTokenError {
    ConsumedToken,
    SyntaxError,
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl ops::Deref for CSSPreservedToken {
    type Target = CSSToken;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<CSSToken> for CSSPreservedToken {
    type Error = CSSPreservedTokenError;

    fn try_from(token: CSSToken) -> Result<Self, Self::Error> {
        Ok(match token {
            | CSSToken::Function(_)
            | CSSToken::LeftCurlyBracket
            | CSSToken::LeftParenthesis
            | CSSToken::LeftSquareBracket => {
                return Err(Self::Error::ConsumedToken)
            }

            | CSSToken::BadString
            | CSSToken::BadUrl
            | CSSToken::RightCurlyBracket
            | CSSToken::RightParenthesis
            | CSSToken::RightSquareBracket => {
                return Err(Self::Error::SyntaxError)
            }

            | _ => Self(token),
        })
    }
}
