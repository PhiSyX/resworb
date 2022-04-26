/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// ----- //
// Macro //
// ----- //

#[macro_export]
macro_rules! emit_html_error {
    ($err:expr) => {
        log::error!("[HTMLParserError]: {}", $err);
    };
}

macro_rules! define_errors {
    (
    $(
        $(#[$attr:meta])*
        $enum:ident = $str:literal
    ),*
    ) => {
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
    $( $(#[$attr])* $enum ),*
}

impl core::str::FromStr for HTMLParserError {
    type Err = &'static str;

    #[allow(unreachable_patterns)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            $( | $str => Self::$enum, )*
            | _ => return Err("Nom d'erreur de l'analyseur HTML inconnu."),
        })
    }
}

impl core::fmt::Display for HTMLParserError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!( f, "{}", match self { $( | Self::$enum => $str, )* } )
    }
}
    };
}

// ----------- //
// Énumération //
// ----------- //

define_errors! {
    /// Cette erreur se produit si l'analyseur rencontre un commentaire
    /// vide qui est brusquement fermé par un point de code U+003E (>)
    /// (c'est-à-dire `<!-->` ou `<!--->`). L'analyseur se comporte comme
    /// si le commentaire était fermé correctement.
    AbruptClosingOfEmptyComment = "abrupt-closing-of-empty-comment",

    /// Cette erreur se produit si l'analyseur syntaxique rencontre un
    /// point de code U+003E (>) dans l'identifiant public DOCTYPE (par
    /// exemple, `<!DOCTYPE html PUBLIC "foo>`). Dans un tel cas, si le
    /// DOCTYPE est correctement placé comme préambule du document,
    /// l'analyseur syntaxique place le document en mode quirks.
    AbruptDOCTYPEPublicIdentifier = "abrupt-doctype-public-identifier",

    /// Cette erreur se produit si l'analyseur syntaxique rencontre un
    /// point de code U+003E (>) dans l'identifiant système DOCTYPE (par
    /// exemple, `<!DOCTYPE html PUBLIC "-//W3C//DTD HTML 4.01//EN"
    /// "foo>`). Dans ce cas, si le DOCTYPE est correctement placé
    /// comme préambule du document, l'analyseur syntaxique place le
    /// document en mode quirks.
    AbruptDOCTYPESystemIdentifier = "abrupt-doctype-system-identifier",

    /// Cette erreur se produit si l'analyseur rencontre une section CDATA
    /// en dehors d'un contenu étranger (SVG ou MathML). L'analyseur
    /// syntaxique traite ces sections CDATA (y compris les chaînes de
    /// tête "[CDATA[" et de fin "]]") comme des commentaires.
    CDATAInHtmlContent = "cdata-in-html-content",

    /// Cette erreur se produit si l'analyseur syntaxique rencontre la fin
    /// du flux d'entrée où un nom de balise est attendu. Dans ce cas,
    /// l'analyseur syntaxique traite le début d'une balise de début
    /// (c.-à-d. <) ou d'une balise de fin (c.-à-d. </) comme du contenu
    /// textuel.
    EofBeforeTagName = "cdata-in-html-content",

    /// Cette erreur se produit si l'analyseur syntaxique rencontre la fin
    /// du flux d'entrée dans un commentaire. L'analyseur traite de tels
    /// commentaires comme s'ils étaient fermés immédiatement avant la fin
    /// du flux d'entrée.
    EofInComment = "eof-in-comment",

    /// Cette erreur se produit si l'analyseur syntaxique rencontre la fin
    /// du flux d'entrée dans un DOCTYPE. Dans un tel cas, si le DOCTYPE
    /// est correctement placé comme préambule du document, l'analyseur
    /// syntaxique place le document en mode quirks.
    EofInDOCTYPE = "eof-in-doctype",

    /// Cette erreur se produit si l'analyseur syntaxique rencontre la fin
    /// du flux d'entrée dans une balise de début ou une balise de fin
    /// (par exemple, `<div id=`). Une telle balise est ignorée.
    EofInTag = "eof-in-tag",

    /// Cette erreur se produit si l'analyseur rencontre la séquence de
    /// points de code "< !" qui n'est pas immédiatement suivie de deux
    /// points de code U+002D (-) et qui n'est pas le début d'un DOCTYPE
    /// ou d'une section CDATA. Tout le contenu qui suit la séquence de
    /// points de code "< !" jusqu'à un point de code U+003E (>) (si
    /// présent) ou jusqu'à la fin du flux d'entrée est traité comme un
    /// commentaire.
    ///
    /// Note: une cause possible de cette erreur est l'utilisation d'une
    /// déclaration de balisage XML (par exemple, `<!ELEMENT br EMPTY>`)
    /// dans l'HTML.
    IncorrectlyOpenedComment = "incorrectly-opened-comment",

    /// Cette erreur se produit si l'analyseur syntaxique rencontre un
    /// commentaire qui est fermé par la séquence de points de code "--!>".
    /// L'analyseur traite de tels commentaires comme s'ils étaient
    /// correctement fermés par la séquence de points de code "-->".
    IncorrectlyClosedComment = "incorrectly-closed-comment",

    /// Cette erreur se produit si l'analyseur syntaxique rencontre une
    /// séquence de points de code autre que les mots-clés "PUBLIC" et
    /// "SYSTEM" après un nom de DOCTYPE. Dans ce cas, l'analyseur ignore
    /// tout identificateur public ou système suivant, et si le DOCTYPE
    /// est correctement placé en tant que préambule du document, et si
    /// l'analyseur ne peut pas changer le drapeau de mode est faux, il
    /// place le document en mode quirks.
    InvalidCharacterSequenceAfterDOCTYPEName = "invalid-character-sequence-after-doctype-name",

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
    /// Example: `<42></42>`
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
    InvalidFirstCharacterOfTagName = "invalid-first-character-of-tag-name",

    /// Cette erreur se produit si l'analyseur rencontre un point de code
    /// U+003E (>) là où une valeur d'attribut est attendue (par exemple,
    /// `<div id=>`). L'analyseur syntaxique traite l'attribut comme ayant
    /// une valeur vide.
    MissingAttributeValue = "missing-attribute-value",

    /// Cette erreur se produit si l'analyseur syntaxique rencontre un
    /// DOCTYPE auquel il manque un nom (par exemple, `<!DOCTYPE>`). Dans
    /// un tel cas, si le DOCTYPE est correctement placé comme préambule
    /// du document, l'analyseur syntaxique place le document en mode
    /// quirks.
    MissingDOCTYPEName = "missing-doctype-name",

    /// Cette erreur se produit si l'analyseur syntaxique rencontre un
    /// point de code U+003E (>) où le début de l'identifiant public
    /// DOCTYPE est attendu (par exemple, `<!DOCTYPE html PUBLIC >`). Dans
    /// un tel cas, si le DOCTYPE est correctement placé comme préambule
    /// du document, l'analyseur syntaxique place le document en mode
    /// quirks.
    MissingDOCTYPEPublicIdentifier = "missing-doctype-public-identifier",

    /// Cette erreur se produit si l'analyseur rencontre un point de code
    /// U+003E (>) où le début de l'identificateur de système DOCTYPE est
    /// attendu (par exemple, `<!DOCTYPE html SYSTEM >`). Dans un tel cas,
    /// si le DOCTYPE est correctement placé comme préambule du document,
    /// l'analyseur syntaxique place le document en mode quirks.
    MissingDOCTYPESystemIdentifier = "missing-doctype-system-identifier",

    /// Cette erreur se produit si l'analyseur rencontre un point de code
    /// U+003E (>) là où un nom de balise de fin est attendu, c'est-à-dire
    /// </>. L'analyseur syntaxique ignore l'ensemble de la séquence de
    /// points de code "</>".
    MissingEndTagName = "missing-end-tag-name",

    /// Cette erreur se produit si l'analyseur syntaxique rencontre
    /// l'identificateur public DOCTYPE qui n'est pas précédé d'une
    /// citation (par exemple, `<!DOCTYPE html PUBLIC -//W3C//DTD HTML
    /// 4.01//EN">`). Dans un tel cas, l'analyseur syntaxique ignore
    /// l'identificateur public et, si le DOCTYPE est correctement placé
    /// en tant que préambule du document, place le document en mode
    /// quirks.
    MissingQuoteBeforeDOCTYPEPublicIdentifier = "missing-quote-before-doctype-public-identifier",

    /// Cette erreur se produit si l'analyseur syntaxique rencontre
    /// l'identifiant système DOCTYPE qui n'est pas précédé d'un guillemet
    /// (par exemple, `<!DOCTYPE html SYSTEM http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd">`).
    /// Dans un tel cas, l'analyseur syntaxique ignore l'identificateur de
    /// système et, si le DOCTYPE est correctement placé en tant que
    /// préambule du document, place le document en mode quirks.
    MissingQuoteBeforeDOCTYPESystemIdentifier = "missing-quote-before-doctype-system-identifier",

    /// Cette erreur se produit si l'analyseur syntaxique rencontre un
    /// DOCTYPE dont le mot clé "PUBLIC" et l'identifiant public ne sont
    /// pas séparés par un espace ASCII. Dans ce cas, l'analyseur se
    /// comporte comme si un espace ASCII était présent.
    MissingWhitespaceAfterDOCTYPEPublicKeyword = "missing-whitespace-after-doctype-public-keyword",

    /// Cette erreur se produit si l'analyseur syntaxique rencontre un
    /// DOCTYPE dont le mot-clé "SYSTEM" et l'identificateur de système ne
    /// sont pas séparés par un espace ASCII. Dans ce cas, l'analyseur se
    /// comporte comme si un espace ASCII était présent.
    MissingWhitespaceAfterDOCTYPESystemKeyword = "missing-whitespace-after-doctype-system-keyword",

    /// Cette erreur se produit si l'analyseur syntaxique rencontre un
    /// DOCTYPE dont le mot clé "DOCTYPE" et le nom ne sont pas séparés
    /// par un espace ASCII. Dans ce cas, l'analyseur se comporte comme si
    /// un espace ASCII était présent.
    MissingWhitespaceBeforeDOCTYPEName = "missing-whitespace-before-doctype-name",

    /// Cette erreur se produit si l'analyseur rencontre des attributs qui
    /// ne sont pas séparés par des espaces blancs ASCII (par exemple,
    /// `<div id="foo"class="bar">`). Dans ce cas, l'analyseur se comporte
    /// comme si un espace blanc ASCII était présent.
    MissingWhitespaceBetweenAttributes = "missing-whitespace-between-attributes",

    /// Cette erreur se produit si l'analyseur syntaxique rencontre un
    /// DOCTYPE dont les identifiants public et système ne sont pas
    /// séparés par un espace ASCII. Dans ce cas, l'analyseur se comporte
    /// comme si un espace ASCII était présent.
    MissingWhitespaceBetweenDOCTYPEPublicAndSystemIdentifiers = "missing-whitespace-between-doctype-public-and-system-identifiers",

    /// Cette erreur se produit si l'analyseur rencontre un commentaire
    /// imbriqué (par exemple, `<!-- <!-- imbriqué --> -->`). Un tel
    /// commentaire sera fermé par la première séquence de points de
    /// code "-->" et tout ce qui suit sera traité comme du balisage.
    NestedComment = "nested-comment",

    /// Cette erreur se produit si l'analyseur syntaxique rencontre des
    /// points de code autres que des espaces blancs ASCII ou la fermeture
    /// U+003E (>) après l'identificateur de système DOCTYPE. L'analyseur
    /// syntaxique ignore ces points de code.
    UnexpectedCharacterAfterDoctypeSystemIdentifier = "unexpected-character-after-doctype-system-identifier",

    /// Cette erreur se produit si l'analyseur syntaxique rencontre un
    /// point de code U+0022 ("), U+0027 (') ou U+003C (<) dans un nom
    /// d'attribut. L'analyseur syntaxique inclut ces points de code dans
    /// le nom de l'attribut.
    ///
    /// Note: les points de code qui déclenchent cette erreur font
    /// généralement partie d'une autre construction syntaxique et peuvent
    /// être le signe d'une faute de frappe autour du nom de l'attribut.
    ///
    /// Example: `<div foo<div>`
    /// En raison d'un point de code U+003E (>) oublié après foo,
    /// l'analyseur syntaxique traite ce balisage comme un seul élément
    /// div avec un attribut "foo<div".
    ///
    /// Example: `<div id'bar'>`
    /// En raison d'un point de code U+003D (=) oublié entre un nom
    /// d'attribut et une valeur, l'analyseur syntaxique traite ce
    /// balisage comme un élément div dont l'attribut "id'bar'" a une
    /// valeur vide.
    UnexpectedCharacterInAttributeName = "unexpected-character-in-attribute-name",

    /// Cette erreur se produit si l'analyseur syntaxique rencontre un
    /// point de code U+0022 ("), U+0027 ('), U+003C (<), U+003D (=) ou
    /// U+0060 (`) dans une valeur d'attribut (unquoted). L'analyseur
    /// syntaxique inclut ces points de code dans la valeur de l'attribut.
    ///
    /// Note 1: les points de code qui déclenchent cette erreur font
    /// généralement partie d'une autre construction syntaxique et peuvent
    /// être le signe d'une faute de frappe autour de la valeur de
    /// l'attribut.
    ///
    /// Note 2: U+0060 (`) figure dans la liste des points de code qui
    /// déclenchent cette erreur parce que certains agents utilisateurs
    /// anciens le traitent comme un guillemet.
    ///
    /// Exemple: `<div foo=b'ar'>`
    /// En raison d'un point de code U+0027 (') mal placé, l'analyseur
    /// définit la valeur de l'attribut "foo" à "b'ar'".
    UnexpectedCharacterInUnquotedAttributeValue = "unexpected-character-in-unquoted-attribute-value",

    /// Cette erreur se produit si l'analyseur syntaxique rencontre un
    /// point de code U+003D (=) avant un nom d'attribut. Dans ce cas,
    /// l'analyseur syntaxique traite U+003D (=) comme le premier point de
    /// code du nom de l'attribut.
    ///
    /// Note: la raison courante de cette erreur est un nom d'attribut
    /// oublié.
    ///
    /// Example: `<div foo="bar" ="baz">`
    /// En raison d'un nom d'attribut oublié, l'analyseur syntaxique
    /// traite ce balisage comme un élément div avec deux attributs : un
    /// attribut "foo" avec une valeur "bar" et un attribut "="baz"" avec
    /// une valeur vide.
    UnexpectedEqualsSignBeforeAttributeName = "unexpected-equals-sign-before-attribute-name",

    /// Cette erreur se produit si l'analyseur syntaxique rencontre un
    /// point de code U+0000 NULL dans le flux d'entrée à certaines
    /// positions. En général, ces points de code sont soit ignorés, soit,
    /// pour des raisons de sécurité, remplacés par un CHARACTER DE
    /// REMPLACEMENT U+FFFD.
    UnexpectedNullCharacter = "unexpected-null-character",

    /// Cette erreur se produit si l'analyseur rencontre un point de code
    /// U+003F (?) alors que le premier point de code d'un nom de balise
    /// de début est attendu. Le point de code U+003F (?) et tout le
    /// contenu qui suit jusqu'à un point de code U+003E (>) (s'il est
    /// présent) ou jusqu'à la fin du flux d'entrée est traité comme un
    /// commentaire.
    ///
    /// Example:
    ///   `<?xml-stylesheet type="text/css" href="style.css"?>`
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
    UnexpectedQuestionMarkInsteadOfTagName = "unexpected-question-mark-instead-of-tag-name",

    /// Cette erreur se produit si l'analyseur rencontre un point de code
    /// U+002F (/) qui ne fait pas partie d'une valeur d'attribut citée et
    /// qui n'est pas immédiatement suivi d'un point de code U+003E (>)
    /// dans une balise (par exemple, `<div / id="foo">`). Dans ce cas,
    /// l'analyseur se comporte comme s'il rencontrait un espace blanc
    /// ASCII.
    UnexpectedSolidusInTag = "unexpected-solidus-in-tag",

    /// Cette erreur se produit si l'analyseur syntaxique rencontre une
    /// esperluette ambiguë. Dans ce cas, l'analyseur syntaxique ne résout
    /// pas la référence du caractère.
    UnknownNamedCharacterReference = "unknown-named-character-reference"
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

#[cfg(test)]
mod tests {
    use parser::preprocessor::InputStreamPreprocessor;

    use crate::parser::{token::HTMLToken, tokenizer::HTMLTokenizer};

    fn get_tokenizer_html(
        input: &'static str,
    ) -> HTMLTokenizer<impl Iterator<Item = char>> {
        let stream = InputStreamPreprocessor::new(input.chars());
        HTMLTokenizer::new(stream)
    }

    #[test]
    fn test_error_abrupt_closing_of_empty_comment() {
        let mut html_tok = get_tokenizer_html(include_str!(
            "crashtests/comment/abrupt_closing_of_empty_comment.html"
        ));

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::Comment("".into()))
        );

        html_tok.next_token();

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::Comment("".into()))
        );
    }

    #[test]
    fn test_error_abrupt_doctype_identifier() {
        let mut html_tok = get_tokenizer_html(include_str!(
            "crashtests/doctype/abrupt_doctype_identifier.html"
        ));

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::DOCTYPE {
                name: Some("html".into()),
                public_identifier: Some("foo".into()),
                system_identifier: None,
                force_quirks_flag: true,
            })
        );

        html_tok.next_token();

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::DOCTYPE {
                name: Some("html".into()),
                public_identifier: Some(
                    "-//W3C//DTD HTML 4.01//EN".into()
                ),
                system_identifier: Some("foo".into()),
                force_quirks_flag: true,
            })
        );
    }

    #[test]
    fn test_error_eof_before_tag_name() {
        let mut html_tok = get_tokenizer_html(include_str!(
            "crashtests/tag/eof_before_tag_name.html"
        ));

        assert_eq!(html_tok.next_token(), Some(HTMLToken::EOF));
    }

    #[test]
    fn test_error_eof_in_comment() {
        let mut html_tok = get_tokenizer_html(include_str!(
            "crashtests/comment/eof_in_comment.html"
        ));

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::Comment("".into()))
        );
    }

    #[test]
    fn test_error_eof_in_doctype() {
        let mut html_tok = get_tokenizer_html(include_str!(
            "crashtests/doctype/eof_in_doctype.html"
        ));

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::DOCTYPE {
                name: None,
                public_identifier: None,
                system_identifier: None,
                force_quirks_flag: true
            })
        );
    }

    #[test]
    fn test_error_eof_in_tag() {
        let mut html_tok = get_tokenizer_html(include_str!(
            "crashtests/tag/eof_in_tag.html"
        ));

        assert_eq!(html_tok.next_token(), Some(HTMLToken::EOF));
    }

    #[test]
    fn test_error_incorrectly_closed_comment() {
        let mut html_tok = get_tokenizer_html(include_str!(
            "crashtests/comment/incorrectly_closed_comment.html"
        ));

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::Comment(
                " incorrectly closed comment ".into()
            ))
        );

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::Character('\n'))
        );
        assert_eq!(html_tok.next_token(), Some(HTMLToken::EOF));
    }

    #[test]
    fn test_error_incorrectly_opened_comment() {
        let mut html_tok = get_tokenizer_html(include_str!(
            "crashtests/comment/incorrectly_opened_comment.html"
        ));

        assert_eq!(html_tok.next_token(), Some(HTMLToken::Character('<')));
        assert_eq!(html_tok.next_token(), Some(HTMLToken::Character(' ')));
        assert_eq!(html_tok.next_token(), Some(HTMLToken::Character('!')));
        // DOCTYPE ...
        html_tok.nth(8);

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::Comment("ELEMENT br EMPTY".into()))
        );
    }

    #[test]
    fn test_error_invalid_character_sequence_after_doctype_name() {
        let mut html_tok = get_tokenizer_html(include_str!(
            "crashtests/doctype/invalid_character_sequence_after_doctype_name.html"
        ));

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::DOCTYPE {
                name: Some("html".into()),
                public_identifier: None,
                system_identifier: None,
                force_quirks_flag: true
            })
        );
    }

    #[test]
    fn test_error_invalid_first_character_of_tag_name() {
        let mut html_tok = get_tokenizer_html(include_str!(
            "crashtests/tag/invalid_first_character_of_tag_name.html"
        ));

        // |- #text: <42>
        assert_eq!(html_tok.next_token(), Some(HTMLToken::Character('<')));
        assert_eq!(html_tok.next_token(), Some(HTMLToken::Character('4')));
        assert_eq!(html_tok.next_token(), Some(HTMLToken::Character('2')));
        assert_eq!(html_tok.next_token(), Some(HTMLToken::Character('>')));

        // |- #comment: 42
        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::Comment("42".into()))
        );
    }

    #[test]
    fn test_error_missing_attribute_value() {
        let mut html_tok = get_tokenizer_html(include_str!(
            "crashtests/tag/missing_attribute_value.html"
        ));

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::StartTag {
                name: "div".into(),
                self_closing_flag: false,
                attributes: vec![("id".into(), "".into())]
            })
        );
    }

    #[test]
    fn test_error_missing_doctype_name() {
        let mut html_tok = get_tokenizer_html(include_str!(
            "crashtests/doctype/missing_doctype_name.html"
        ));

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::DOCTYPE {
                name: None,
                public_identifier: None,
                system_identifier: None,
                force_quirks_flag: true
            })
        );
    }

    #[test]
    fn test_error_missing_doctype_identifier() {
        let mut html_tok = get_tokenizer_html(include_str!(
            "crashtests/doctype/missing_doctype_identifier.html"
        ));

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::DOCTYPE {
                name: Some("html".into()),
                public_identifier: None,
                system_identifier: None,
                force_quirks_flag: true
            })
        );

        html_tok.next_token();

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::DOCTYPE {
                name: Some("html".into()),
                public_identifier: None,
                system_identifier: None,
                force_quirks_flag: true
            })
        );
    }

    #[test]
    fn test_error_missing_end_tag_name() {
        let mut html_tok = get_tokenizer_html(include_str!(
            "crashtests/tag/missing_end_tag_name.html"
        ));

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::new_start_tag("div".into()))
        );

        html_tok.next_token();

        assert_eq!(html_tok.next_token(), Some(HTMLToken::EOF));
    }

    #[test]
    #[ignore = "pas sûr du comportement."]
    fn test_error_missing_quote_before_doctype_identifier() {
        let mut html_tok = get_tokenizer_html(include_str!(
            "crashtests/doctype/missing_quote_before_doctype_identifier.html"
        ));

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::DOCTYPE {
                name: Some("html".into()),
                public_identifier: None,
                system_identifier: None,
                force_quirks_flag: true
            })
        );

        html_tok.next_token();

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::DOCTYPE {
                name: Some("html".into()),
                public_identifier: None,
                system_identifier: None,
                force_quirks_flag: true
            })
        );
    }

    #[test]
    fn test_error_missing_whitespace_after_doctype_keyword() {
        let mut html_tok = get_tokenizer_html(include_str!(
            "crashtests/doctype/missing_whitespace_after_doctype_keyword.html"
        ));

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::DOCTYPE {
                name: Some("html".into()),
                public_identifier: Some(
                    "-//W3C//DTD HTML 4.01//EN".into()
                ),
                system_identifier: None,
                force_quirks_flag: false
            })
        );

        html_tok.next_token();

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::DOCTYPE {
                name: Some("html".into()),
                public_identifier: None,
                system_identifier: Some("http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd".into()),
                force_quirks_flag: false
            })
        );
    }

    #[test]
    fn test_error_missing_whitespace_before_doctype_name() {
        let mut html_tok = get_tokenizer_html(include_str!(
            "crashtests/doctype/missing_whitespace_before_doctype_name.html"
        ));

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::DOCTYPE {
                name: Some("html".into()),
                public_identifier: Some(
                    "-//W3C//DTD HTML 4.01//EN".into()
                ),
                system_identifier: None,
                force_quirks_flag: false
            })
        );
    }

    #[test]
    fn test_error_missing_whitespace_between_attributes() {
        let mut html_tok = get_tokenizer_html(include_str!(
            "crashtests/tag/missing_whitespace_between_attributes.html"
        ));

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::StartTag {
                name: "div".into(),
                self_closing_flag: false,
                attributes: vec![
                    ("id".into(), "foo".into()),
                    ("class".into(), "bar".into())
                ]
            })
        );
    }

    #[test]
    #[ignore = "pas sûr du contenu HTML du test."]
    fn test_error_missing_whitespace_between_doctype_identifiers() {
        let mut html_tok = get_tokenizer_html(include_str!(
            "crashtests/doctype/missing_whitespace_between_doctype_identifiers.html"
        ));

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::DOCTYPE {
                name: Some("html".into()),
                public_identifier: Some(
                    "-//W3C//DTD HTML 4.01//EN".into()
                ),
                system_identifier: None,
                force_quirks_flag: false
            })
        );
    }

    #[test]
    fn test_error_nested_comment() {
        let mut html_tok = get_tokenizer_html(include_str!(
            "crashtests/comment/nested_comment.html"
        ));

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::Comment(" <!-- nested ".into()))
        );

        assert_eq!(html_tok.next_token(), Some(HTMLToken::Character(' ')));
        assert_eq!(html_tok.next_token(), Some(HTMLToken::Character('-')));
        assert_eq!(html_tok.next_token(), Some(HTMLToken::Character('-')));
        assert_eq!(html_tok.next_token(), Some(HTMLToken::Character('>')));
    }

    #[test]
    fn test_error_unexpected_character_in_attribute_name() {
        let mut html_tok = get_tokenizer_html(include_str!(
            "crashtests/tag/unexpected_character_in_attribute_name.html"
        ));

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::StartTag {
                name: "div".into(),
                self_closing_flag: false,
                attributes: vec![("foo<div".into(), "".into())]
            })
        );

        html_tok.next_token();
        html_tok.next_token();

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::StartTag {
                name: "div".into(),
                self_closing_flag: false,
                attributes: vec![("id'bar'".into(), "".into())]
            })
        );
    }

    #[test]
    fn test_error_unexpected_character_in_unquoted_attribute_value() {
        let mut html_tok = get_tokenizer_html(include_str!(
            "crashtests/tag/unexpected_character_in_unquoted_attribute_value.html"
        ));

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::StartTag {
                name: "div".into(),
                self_closing_flag: false,
                attributes: vec![("foo".into(), "b'ar'".into())]
            })
        );
    }

    #[test]
    #[ignore = "bug: n'a pas le comportement attendu."]
    fn test_error_unexpected_equals_sign_before_attribute_name() {
        let mut html_tok = get_tokenizer_html(include_str!(
            "crashtests/tag/unexpected_equals_sign_before_attribute_name.html"
        ));

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::StartTag {
                name: "div".into(),
                self_closing_flag: false,
                attributes: vec![
                    ("foo".into(), "bar".into()),
                    (r#"="baz""#.into(), "".into())
                ]
            })
        );
    }

    #[test]
    #[ignore = "bug: n'a pas le comportement attendu."]
    fn test_error_unexpected_question_mark_instead_of_tag_name() {
        let mut html_tok = get_tokenizer_html(include_str!(
            "crashtests/tag/unexpected_question_mark_instead_of_tag_name.html"
        ));

        // |- #comment: ?xml-stylesheet type="text/css" href="style.css"?
        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::Comment(
                r#"?xml-stylesheet type="text/css" href="style.css"?"#
                    .into()
            ))
        );
    }

    #[test]
    fn test_error_unexpected_solidus_in_tag() {
        let mut html_tok = get_tokenizer_html(include_str!(
            "crashtests/tag/unexpected_solidus_in_tag.html"
        ));

        assert_eq!(
            html_tok.next_token(),
            Some(HTMLToken::StartTag {
                name: "div".into(),
                self_closing_flag: false,
                attributes: vec![("id".into(), "foo".into())]
            })
        );
    }
}
