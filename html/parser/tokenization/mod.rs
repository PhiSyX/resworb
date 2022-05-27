/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod token;
pub mod tokenizer;

mod state {
    mod cdata;
    mod character_reference;
    mod comment;
    mod data;
    mod doctype;
    mod plaintext;
    mod rawtext;
    mod rcdata;
    mod script_data;
    mod tag;

    // ----- //
    // Macro //
    // ----- //

    macro_rules! define_state {
        (
        $(
            #[$attr:meta]
            $enum:ident = $str:literal
        ),*
        ) => {
    #[derive(Debug)]
    #[derive(Clone)]
    #[allow(clippy::upper_case_acronyms)]
    pub(crate) enum State {
        $( #[$attr] $enum ),*
    }

    impl core::str::FromStr for State {
        type Err = &'static str;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(match s {
                $( | $str => Self::$enum, )*
                | _ => return Err("Nom de l'état inconnu."),
            })
        }
    }

    impl core::fmt::Display for State {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!( f, "{}", match self { $( | Self::$enum => $str, )* } )
        }
    }
        };
    }

    define_state! {
        /// 13.2.5.1 Data state
        Data = "data",

        /// 13.2.5.2 RCDATA state
        RCDATA = "rcdata",

        /// 13.2.5.3 RAWTEXT state
        RAWTEXT = "rawtext",

        /// 13.2.5.4 Script data state
        ScriptData = "script-data",

        /// 13.2.5.5 PLAINTEXT state
        PLAINTEXT = "plaintext",

        /// 13.2.5.6 Tag open state
        TagOpen = "tag-open",

        /// 13.2.5.7 End tag open state
        EndTagOpen = "end-tag-open",

        /// 13.2.5.8 Tag name state
        TagName = "tag-name",

        /// 13.2.5.9 RCDATA less-than sign state
        RCDATALessThanSign = "rcdata-less-than-sign",

        /// 13.2.5.10 RCDATA end tag open state
        RCDATAEndTagOpen = "rcdata-end-tag-open",

        /// 13.2.5.11 RCDATA end tag name state
        RCDATAEndTagName = "rcdata-end-tag-name",

        /// 13.2.5.12 RAWTEXT less-than sign state
        RAWTEXTLessThanSign = "rawtext-less-than-sign",

        /// 13.2.5.13 RAWTEXT end tag open state
        RAWTEXTEndTagOpen = "rawtext-end-tag-open",

        /// 13.2.5.14 RAWTEXT end tag name state
        RAWTEXTEndTagName = "rawtext-end-tag-name",

        /// 13.2.5.15 Script data less-than sign state
        ScriptDataLessThanSign = "script-data-less-than-sign",

        /// 13.2.5.16 Script data end tag open state
        ScriptDataEndTagOpen = "script-data-end-tag-open",

        /// 13.2.5.17 Script data end tag name state
        ScriptDataEndTagName = "script-data-end-tag-name",

        /// 13.2.5.18 Script data escape start state
        ScriptDataEscapeStart = "script-data-escape-start",

        /// 13.2.5.19 Script data escape start dash state
        ScriptDataEscapeStartDash = "script-data-escape-start-dash",

        /// 13.2.5.20 Script data escaped state
        ScriptDataEscaped = "script-data-escaped",

        /// 13.2.5.21 Script data escaped dash state
        ScriptDataEscapedDash = "script-data-escaped-dash",

        /// 13.2.5.22 Script data escaped dash dash state
        ScriptDataEscapedDashDash = "script-data-escaped-dash-dash",

        /// 13.2.5.23 Script data escaped less-than sign state
        ScriptDataEscapedLessThanSign = "script-data-escaped-less-than-sign",

        /// 13.2.5.24 Script data escaped end tag open state
        ScriptDataEscapedEndTagOpen = "script-data-escaped-end-tag-open",

        /// 13.2.5.25 Script data escaped end tag name state
        ScriptDataEscapedEndTagName = "script-data-escaped-end-tag-name",

        /// 13.2.5.26 Script data double escape start state
        ScriptDataDoubleEscapeStart = "script-data-double-escape-start",

        /// 13.2.5.27 Script data double escaped state
        ScriptDataDoubleEscapedState = "script-data-double-escaped",

        /// 13.2.5.28 Script data double escaped dash state
        ScriptDataDoubleEscapedDash = "script-data-double-escaped-dash",

        /// 13.2.5.29 Script data double escaped dash dash state
        ScriptDataDoubleEscapedDashDash = "script-data-double-escaped-dash-dash",

        /// 13.2.5.30 Script data double escaped less-than sign state
        ScriptDataDoubleEscapedLessThanSign = "script-data-double-escaped-less-than-sign",

        /// 13.2.5.31 Script data double escape end state
        ScriptDataDoubleEscapeEnd = "script-data-double-escape-end",

        /// 13.2.5.32 Before attribute name state
        BeforeAttributeName = "before-attribute-name",

        /// 13.2.5.33 Attribute name state
        AttributeName = "attribute-name",

        /// 13.2.5.34 After attribute name state
        AfterAttributeName = "after-attribute-name",

        /// 13.2.5.35 Before attribute value state
        BeforeAttributeValue = "before-attribute-value",

        /// 13.2.5.36 Attribute value (double-quoted) state
        AttributeValueDoubleQuoted = "attribute-value-double-quoted",

        /// 13.2.5.37 Attribute value (single-quoted) state
        AttributeValueSingleQuoted = "attribute-value-single-quoted",

        /// 13.2.5.38 Attribute value (unquoted) state
        AttributeValueUnquoted = "attribute-value-unquoted",

        /// 13.2.5.39 After attribute value (quoted) state
        AfterAttributeValueQuoted = "after-attribute-value-quoted",

        /// 13.2.5.40 Self-closing start tag state
        SelfClosingStartTag = "self-closing-start-tag",

        /// 13.2.5.41 Bogus comment state
        BogusComment = "bogus-comment",

        /// 13.2.5.42 Markup declaration open state
        MarkupDeclarationOpen = "markup-declaration-open",

        /// 13.2.5.43 Comment start state
        CommentStart = "comment-start",

        /// 13.2.5.44 Comment start dash state
        CommentStartDash = "comment-start-dash",

        /// 13.2.5.45 Comment state
        Comment = "comment",

        /// 13.2.5.46 Comment less-than sign state
        CommentLessThanSign = "comment-less-than-sign",

        /// 13.2.5.47 Comment less-than sign bang state
        CommentLessThanSignBang = "comment-less-than-sign-bang",

        /// 13.2.5.48 Comment less-than sign bang dash state
        CommentLessThanSignBangDash = "comment-less-than-sign-bang-dash",

        /// 13.2.5.49 Comment less-than sign bang dash dash state
        CommentLessThanSignBangDashDash = "comment-less-than-sign-bang-dash-dash",

        /// 13.2.5.50 Comment end dash state
        CommentEndDash = "comment-end-dash",

        /// 13.2.5.51 Comment end state
        CommentEnd = "comment-end",

        /// 13.2.5.52 Comment end bang state
        CommentEndBang = "comment-end-bang",

        /// 13.2.5.53 DOCTYPE state
        DOCTYPE = "doctype",

        /// 13.2.5.54 Before DOCTYPE name state
        BeforeDOCTYPEName = "before-doctype-name",

        /// 13.2.5.55 DOCTYPE name state
        DOCTYPEName = "doctype-name",

        /// 13.2.5.56 After DOCTYPE name state
        AfterDOCTYPEName = "after-doctype-name",

        /// 13.2.5.57 After DOCTYPE public keyword state
        AfterDOCTYPEPublicKeyword = "after-doctype-public-keyword",

        /// 13.2.5.58 Before DOCTYPE public identifier state
        BeforeDOCTYPEPublicIdentifier = "before-doctype-public-identifier",

        /// 13.2.5.59 DOCTYPE public identifier (double-quoted) state
        DOCTYPEPublicIdentifierDoubleQuoted = "doctype-public-identifier-double-quoted",

        /// 13.2.5.60 DOCTYPE public identifier (single-quoted) state
        DOCTYPEPublicIdentifierSingleQuoted = "doctype-public-identifier-single-quoted",

        /// 13.2.5.61 After DOCTYPE public identifier state
        AfterDOCTYPEPublicIdentifier = "after-doctype-public-identifier",

        /// 13.2.5.62 Between DOCTYPE public and system identifiers state
        BetweenDOCTYPEPublicAndSystemIdentifiers = "between-doctype-public-and-system-identifiers",

        /// 13.2.5.63 After DOCTYPE system keyword state
        AfterDOCTYPESystemKeyword = "after-doctype-system-keyword",

        /// 13.2.5.64 Before DOCTYPE system identifier state
        BeforeDOCTYPESystemIdentifier = "before-doctype-system-identifier",

        /// 13.2.5.65 DOCTYPE system identifier (double-quoted) state
        DOCTYPESystemIdentifierDoubleQuoted = "doctype-system-identifier-double-quoted",

        /// 13.2.5.66 DOCTYPE system identifier (single-quoted) state
        DOCTYPESystemIdentifierSingleQuoted = "doctype-system-identifier-single-quoted",

        /// 13.2.5.67 After DOCTYPE system identifier state
        AfterDOCTYPESystemIdentifier = "after-doctype-system-identifier",

        /// 13.2.5.68 Bogus DOCTYPE state
        BogusDOCTYPE = "bogus-doctype",

        /// 13.2.5.69 CDATA section state
        CDATASection = "cdata-section",

        /// 13.2.5.70 CDATA section bracket state
        CDATASectionBracket = "cdata-section-bracket",

        /// 13.2.5.71 CDATA section end state
        CDATASectionEnd = "cdata-section-end",

        /// 13.2.5.72 Character reference state
        CharacterReference = "character-reference",

        /// 13.2.5.73 Named character reference state
        NamedCharacterReference = "named-character-reference",

        /// 13.2.5.74 Ambiguous ampersand state
        AmbiguousAmpersand = "ambiguous-ampersand",

        /// 13.2.5.75 Numeric character reference state
        NumericCharacterReference = "numeric-character-reference",

        /// 13.2.5.76 Hexadecimal character reference start state
        HexadecimalCharacterReferenceStart = "hexadecimal-character-reference-start",

        /// 13.2.5.77 Decimal character reference start state
        DecimalCharacterReferenceStart = "decimal-character-reference-start",

        /// 13.2.5.78 Hexadecimal character reference state
        HexadecimalCharacterReference = "hexadecimal-character-reference",

        /// 13.2.5.79 Decimal character reference state
        DecimalCharacterReference = "decimal-character-reference",

        /// 13.2.5.80 Numeric character reference end state
        NumericCharacterReferenceEnd = "numeric-character-reference-end"
    }
}

pub(crate) use self::{
    state::State as HTMLTokenizerState,
    token::{
        HTMLDoctypeToken, HTMLTagAttribute,
        HTMLTagToken, HTMLToken,
    },
    tokenizer::HTMLTokenizer,
};
