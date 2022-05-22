/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use core::{fmt, str};

use crate::interface::IsOneOfAttributesInterface;

// ------ //
// Macros //
// ------ //

macro_rules! enumerate_html_tag_attributes {
    ($($name:ident)*) => {
        #[allow(non_camel_case_types)]
        #[derive(Debug)]
        #[derive(Copy, Clone)]
        #[derive(PartialEq)]
        pub enum tag_attributes {
        $(
            #[allow(non_upper_case_globals)]
            #[doc = "Nom de l'attribut :"]
            #[doc = stringify!($name)]
            $name
        ),*
        }

        impl str::FromStr for tag_attributes {
            type Err = &'static str;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(match s {
                    $(| stringify!($name) => Self::$name),*,
                    | _ => return Err("Attribut inconnu")
                })
            }
        }

        impl fmt::Display for tag_attributes {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", match self {
                    $(Self::$name => stringify!($name)),*
                })
            }
        }
    };
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl IsOneOfAttributesInterface for tag_attributes {
    fn is_one_of(
        self,
        arr: impl IntoIterator<Item = tag_attributes>,
    ) -> bool {
        arr.into_iter().any(|tag_attributes| tag_attributes == self)
    }
}

impl<S> IsOneOfAttributesInterface for S
where
    S: AsRef<str>,
    S: Copy,
{
    fn is_one_of(
        self,
        arr: impl IntoIterator<Item = tag_attributes>,
    ) -> bool {
        arr.into_iter().any(|tag_attributes| tag_attributes == self)
    }
}

impl<S> PartialEq<S> for tag_attributes
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

enumerate_html_tag_attributes! {
    abbr
    accept
    accept_charset
    action
    align
    alink
    allow
    allowfullscreen
    alt
    archive
    r#async
    autoplay
    axis
    background
    behavior
    bgcolor
    border
    cellpadding
    cellspacing
    r#char
    charoff
    charset
    checked
    cite
    class_
    clear
    code
    codetype
    color
    cols
    colspan
    compact
    content
    contenteditable
    controls
    coords
    data
    datetime
    declare
    r#default
    defer
    direction
    dirname
    disabled
    download
    event
    face
    r#for
    form
    formnovalidate
    formtarget
    frame
    frameborder
    headers
    height
    hidden
    href
    hreflang
    hspace
    http_equiv
    id
    imagesizes
    imagesrcset
    integrity
    ismap
    label
    lang
    language
    link
    longdesc
    r#loop
    marginheight
    marginwidth
    max
    media
    method
    min
    multiple
    name
    nohref
    nomodule
    noshade
    novalidate
    nowrap
    onabort
    onauxclick
    onblur
    oncancel
    oncanplay
    oncanplaythrough
    onchange
    onclick
    onclose
    oncontextmenu
    oncuechange
    ondblclick
    ondrag
    ondragend
    ondragenter
    ondragleave
    ondragover
    ondragstart
    ondrop
    ondurationchange
    onemptied
    onended
    onerror
    onfocus
    onformdata
    oninput
    oninvalid
    onkeydown
    onkeypress
    onkeyup
    onload
    onloadeddata
    onloadedmetadata
    onloadstart
    onmousedown
    onmouseenter
    onmouseleave
    onmousemove
    onmouseout
    onmouseover
    onmouseup
    onpause
    onplay
    onplaying
    onprogress
    onratechange
    onreset
    onresize
    onscroll
    onsecuritypolicyviolation
    onseeked
    onseeking
    onselect
    onslotchange
    onstalled
    onsubmit
    onsuspend
    ontimeupdate
    ontoggle
    onvolumechange
    onwaiting
    onwebkitanimationend
    onwebkitanimationiteration
    onwebkitanimationstart
    onwebkittransitionend
    onwheel
    open
    pattern
    ping
    placeholder
    playsinline
    poster
    preload
    readonly
    rel
    required
    rev
    reversed
    rows
    rowspan
    rules
    scheme
    scrolling
    selected
    shape
    size
    sizes
    src
    srcdoc
    srclang
    srcset
    standby
    step
    style
    summary
    target
    text
    title
    r#type
    usemap
    valign
    value
    valuetype
    version
    vlink
    vspace
    width
    wrap
}
