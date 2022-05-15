/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use core::{fmt, str};

use crate::interface::IsOneOfTagsInterface;

// ------ //
// Macros //
// ------ //

macro_rules! enumerate_html_tag_names {
    ($($name:ident)*) => {
        #[allow(non_camel_case_types)]
        #[derive(Debug)]
        #[derive(Copy, Clone)]
        #[derive(PartialEq)]
        pub enum tag_names {
        $(
            #[allow(non_upper_case_globals)]
            #[doc = "Nom de la balise :"]
            #[doc = stringify!($name)]
            $name
        ),*
        }

        impl str::FromStr for tag_names {
            type Err = &'static str;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(match s {
                    $(| stringify!($name) => Self::$name),*,
                    | _ => return Err("Élément inconnu")
                })
            }
        }

        impl fmt::Display for tag_names {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", match self {
                    $(Self::$name => stringify!($name)),*
                })
            }
        }
    };
}

// -------------- //
// Implémentation //
// -------------- //

impl tag_names {
    /*
    NameStartChar ::= ":" | [A-Z]     | "_" | [a-z]     | [#xC0-#xD6]
                    | [#xD8-#xF6]     | [#xF8-#x2FF]    | [#x370-#x37D]
                    | [#x37F-#x1FFF]  | [#x200C-#x200D] | [#x2070-#x218F]
                    | [#x2C00-#x2FEF] | [#x3001-#xD7FF] | [#xF900-#xFDCF]
                    | [#xFDF0-#xFFFD] | [#x10000-#xEFFFF]
    */
    fn name_start_char(ch: char) -> bool {
        ch.is_ascii_alphabetic()
            || matches!(ch, | ':' | '_'
             | '\u{00C0}'..='\u{00D6}' | '\u{00D8}'..='\u{00F6}'
             | '\u{00F8}'..='\u{02FF}' | '\u{0370}'..='\u{037D}'
             | '\u{037F}'..='\u{1FFF}' | '\u{200C}'..='\u{200D}'
             | '\u{2070}'..='\u{218F}' | '\u{2C00}'..='\u{2FEF}'
             | '\u{3001}'..='\u{D7FF}' | '\u{F901}'..='\u{FDCF}'
             | '\u{FDF0}'..='\u{FFFD}' | '\u{10000}'..='\u{EFFFF}'
            )
    }

    /*
    NameChar :: = NameStartChar   | "-" | "." | [0-9] | #xB7
                | [#x0300-#x036F] | [#x203F-#x2040]
    */
    fn name_char(ch: char) -> bool {
        Self::name_start_char(ch)
            || ch.is_ascii_alphanumeric()
            || matches!(ch, '-' | '.'
             | '\u{00B7}'
             | '\u{0300}'..='\u{036F}'
             | '\u{203F}'..='\u{2040}'
            )
    }

    pub fn is_valid_name(name: impl AsRef<str>) -> bool {
        let name = name.as_ref();

        if name.is_empty() {
            return false;
        }

        let mut chars = name.chars();

        let next_ch = chars.next().unwrap();
        if !Self::name_start_char(next_ch) {
            return false;
        }

        chars.any(Self::name_char)
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl IsOneOfTagsInterface for tag_names {
    fn is_one_of(self, arr: impl IntoIterator<Item = Self>) -> bool {
        arr.into_iter().any(|tag_names| self == tag_names)
    }
}

impl<S> IsOneOfTagsInterface for S
where
    S: AsRef<str>,
    S: Copy,
{
    fn is_one_of(self, arr: impl IntoIterator<Item = tag_names>) -> bool {
        arr.into_iter().any(|tag_names| tag_names == self)
    }
}

impl<S> PartialEq<S> for tag_names
where
    S: AsRef<str>,
{
    fn eq(&self, other: &S) -> bool {
        self.to_string().eq(other.as_ref())
    }
}

// ----------------------- //
// Application de la macro //
// ----------------------- //

enumerate_html_tag_names! {
    a
    abbr
    acronym
    address
    applet
    area
    article
    aside
    audio
    b
    base
    basefont
    bdi
    bdo
    bgsound
    big
    blink
    blockquote
    body
    br
    button
    canvas
    caption
    center
    cite
    code
    col
    colgroup
    data
    datalist
    dd
    del
    details
    dfn
    dialog
    dir
    div
    dl
    dt
    em
    embed
    fieldset
    figcaption
    figure
    font
    footer
    form
    frame
    frameset
    h1
    h2
    h3
    h4
    h5
    h6
    head
    header
    hgroup
    hr
    html
    i
    iframe
    image
    img
    input
    ins
    kbd
    keygen
    label
    legend
    li
    link
    listing
    main
    map
    mark
    marquee
    math
    menu
    meta
    meter
    nav
    nobr
    noembed
    noframes
    noscript
    object
    ol
    optgroup
    option
    output
    p
    param
    picture
    path
    plaintext
    pre
    progress
    q
    ruby
    rb
    rp
    rt
    rtc
    s
    samp
    script
    section
    select
    slot
    small
    source
    span
    strike
    strong
    style
    sub
    sup
    summary
    svg
    table
    tbody
    td
    template
    textarea
    tfoot
    th
    thead
    time
    title
    tr
    track
    tt
    u
    ul
    var
    video
    wbr
    xmp
    name
}
