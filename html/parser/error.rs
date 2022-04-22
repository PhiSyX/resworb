/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use core::fmt;

// ----- //
// Macro //
// ----- //

#[macro_export]
macro_rules! emit_html_error {
    ($err:expr) => {
        log::error!("[HTMLParserError]: {}", $err);
    };
}

// ----------- //
// Énumération //
// ----------- //

/// ------------------------------------------------------------------- //
///                     HTMLParserError                                 //
/// ------------------------------------------------------------------- //
///
/// Cette énumération définit les règles d'analyse syntaxique des
/// documents HTML, qu'ils soient syntaxiquement corrects ou non. Certains
/// points de l'algorithme d'analyse syntaxique sont considérés comme des
/// erreurs d'analyse. Le traitement des erreurs d'analyse syntaxique est
/// bien défini (ce sont les règles de traitement décrites dans la
/// spécification), mais les agents utilisateurs, lorsqu'ils analysent un
/// document HTML, peuvent interrompre l'analyseur à la première erreur
/// d'analyse syntaxique qu'ils rencontrent et pour laquelle ils ne
/// souhaitent pas appliquer les règles décrites dans la spécification.
///
/// Les vérificateurs de conformité doivent signaler au moins une condition
/// d'erreur de syntaxe à l'utilisateur si une ou plusieurs conditions
/// d'erreur de syntaxe existent dans le document et ne doivent pas
/// signaler de conditions d'erreur de syntaxe si aucune n'existe dans le
/// document. Les vérificateurs de conformité peuvent signaler plus d'une
/// condition d'erreur de syntaxe si plus d'une condition d'erreur de
/// syntaxe existe dans le document.
///
/// Les erreurs d'analyse syntaxique ne concernent que la syntaxe du
/// langage HTML. En plus de vérifier les erreurs d'analyse, les
/// vérificateurs de conformité vérifieront également que le document obéit
/// à toutes les autres exigences de conformité décrites dans cette
/// spécification.
///
/// Certaines erreurs d'analyse ont des codes spécifiques décrits dans le
/// tableau ci-dessous, qui doivent être utilisés par les vérificateurs de
/// conformité dans les rapports.
pub enum HTMLParserError {
    /// Cette erreur se produit si l'analyseur syntaxique rencontre un
    /// point de code U+0000 NULL dans le flux d'entrée à certaines
    /// positions. En général, ces points de code sont soit ignorés, soit,
    /// pour des raisons de sécurité, remplacés par un CHARACTER DE
    /// REMPLACEMENT U+FFFD.
    UnexpectedNullCharacter,
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl fmt::Display for HTMLParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                | HTMLParserError::UnexpectedNullCharacter => {
                    "unexpected-null-character"
                }
            }
        )
    }
}
