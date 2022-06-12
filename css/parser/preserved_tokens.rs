/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops;

use crate::tokenization::CSSToken;

// ---- //
// Type //
// ---- //

/// Tout token produit par le
/// [tokenizer](crate::tokenization::CSSTokenizer) à l'exception des
/// <[fonction-token](crate::tokenization::CSSToken::Function)>s,
/// <[{-token](crate::tokenization::CSSToken::LeftCurlyBracket)>s,
/// <[(-token](crate::tokenization::CSSToken::LeftParenthesis)>s, et
/// <[\[-token](crate::tokenization::CSSToken::LeftSquareBracket)>s.
///
/// NOTE(css): les jetons non conservés listés ci-dessus sont toujours
/// consommés dans des objets de plus haut niveau, soit des fonctions ou
/// des blocs simples, et n'apparaissent donc jamais dans la sortie de
/// l'analyseur.
///
/// NOTE(css): les jetons
/// <[}-token](crate::tokenization::CSSToken::RightCurlyBracket)>s,
/// <[(-token](crate::tokenization::CSSToken::RightParenthesis)>s,
/// <[\[-token](crate::tokenization::CSSToken::RightSquareBracket)>s,
/// <[bad-string-token](crate::tokenization::CSSToken::BadString)>, et
/// <[bad-url-token](crate::tokenization::CSSToken::BadUrl)> sont toujours
/// des erreurs d'analyse, mais ils sont préservés dans le flux de tokens
/// par cette spécification pour permettre à d'autres spécifications,
/// telles que Media Queries, de définir une gestion des erreurs plus fine
/// que le simple abandon d'une déclaration ou d'un bloc entier.
#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub(crate) struct CSSPreservedToken(pub(crate) CSSToken);

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl ops::Deref for CSSPreservedToken {
    type Target = CSSToken;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<CSSToken> for CSSPreservedToken {
    fn from(token: CSSToken) -> Self {
        match token {
            | CSSToken::Function(_)
            | CSSToken::LeftCurlyBracket
            | CSSToken::LeftParenthesis
            | CSSToken::LeftSquareBracket => {
                panic!("CSSPreservedToken::try_from: {:?}", token)
            }
            | _ => Self(token),
        }
    }
}
