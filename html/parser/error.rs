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
    /// Cette erreur se produit si l'analyseur syntaxique rencontre la fin
    /// du flux d'entrée où un nom de balise est attendu. Dans ce cas,
    /// l'analyseur syntaxique traite le début d'une balise de début
    /// (c.-à-d. <) ou d'une balise de fin (c.-à-d. </) comme du contenu
    /// textuel.
    EofBeforeTagName,

    /// Cette erreur se produit si l'analyseur syntaxique rencontre la fin
    /// du flux d'entrée dans une balise de début ou une balise de fin
    /// (par exemple, <div id=). Une telle balise est ignorée.
    EofInTag,

    /// Cette erreur se produit si l'analyseur rencontre un point de code
    /// qui n'est pas un alpha ASCII où le premier point de code d'une
    /// balise de début ou d'une balise de fin est attendu. Si une balise
    /// de début était attendue, ce point de code et un U+003C (<) qui le
    /// précède sont traités comme du contenu texte, et tout le contenu
    /// qui suit est traité comme du balisage. En revanche, si une balise
    /// de fin était attendue, ce point de code et tout le contenu qui
    /// suit jusqu'à un point de code U+003E (>) (s'il est présent) ou
    /// jusqu'à la fin du flux d'entrée est traité comme un commentaire.
    ///
    /// Example:
    ///   <42></42>
    ///
    /// Parsed into:
    ///   |- html
    ///      |- head
    ///      |- body
    ///         |- #text: <42>
    ///         |- #comment: 42
    ///
    /// Note: alors que le premier point de code d'un nom de balise est
    /// limité à un alpha ASCII, un large éventail de points de code (y
    /// compris des chiffres ASCII) est autorisé dans les positions
    /// suivantes.
    InvalidFirstCharacterOfTagName,

    /// Cette erreur se produit si l'analyseur rencontre un point de code
    /// U+003E (>) là où un nom de balise de fin est attendu, c'est-à-dire
    /// </>. L'analyseur syntaxique ignore l'ensemble de la séquence de
    /// points de code "</>".
    MissingEndTagName,

    /// Cette erreur se produit si l'analyseur syntaxique rencontre un
    /// point de code U+0000 NULL dans le flux d'entrée à certaines
    /// positions. En général, ces points de code sont soit ignorés, soit,
    /// pour des raisons de sécurité, remplacés par un CHARACTER DE
    /// REMPLACEMENT U+FFFD.
    UnexpectedNullCharacter,

    /// Cette erreur se produit si l'analyseur rencontre un point de code
    /// U+003F (?) alors que le premier point de code d'un nom de balise
    /// de début est attendu. Le point de code U+003F (?) et tout le
    /// contenu qui suit jusqu'à un point de code U+003E (>) (s'il est
    /// présent) ou jusqu'à la fin du flux d'entrée est traité comme un
    /// commentaire.
    ///
    /// Example:
    ///   <?xml-stylesheet type="text/css" href="style.css"?>
    ///
    /// Parsed into:
    ///   |- #comment: ?xml-stylesheet type="text/css" href="style.css"?
    ///   |- html
    ///      | - head
    ///      | - body
    ///
    /// Note: la raison courante de cette erreur est une instruction de
    /// traitement XML (par exemple, `<?xml-stylesheet type="text/css"
    /// href="style.css"?>`) ou une déclaration XML (par exemple, `<?xml
    /// version="1.0" encoding="UTF-8"?>`) utilisée dans HTML.
    UnexpectedQuestionMarkInsteadOfTagName,
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
                | Self::EofBeforeTagName => "eof-before-tag-name",
                | Self::EofInTag => "eof-in-tag",
                | Self::InvalidFirstCharacterOfTagName =>
                    "invalid-first-character-of-tag-name",
                | Self::MissingEndTagName => "missing-end-tag-name",
                | Self::UnexpectedNullCharacter =>
                    "unexpected-null-character",
                | Self::UnexpectedQuestionMarkInsteadOfTagName =>
                    "unexpected-question-mark-instead-of-tag-name",
            }
        )
    }
}
