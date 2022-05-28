/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use core::{fmt, str};

pub const NAMESPACES: [(&str, &str); 6] = [
    ("HTML", "http://www.w3.org/1999/xhtml"),
    ("MathML", "http://www.w3.org/1998/Math/MathML"),
    ("SVG", "http://www.w3.org/2000/svg"),
    ("XLink", "http://www.w3.org/1999/xlink"),
    ("XML", "http://www.w3.org/XML/1998/namespace"),
    ("XMLNS", "http://www.w3.org/2000/xmlns/"),
];

// ----------- //
// Énumération //
// ----------- //

/// Les différents espaces de noms.
#[derive(Debug)]
#[derive(Copy, Clone)]
#[derive(PartialEq, Eq)]
pub enum Namespace {
    HTML,
    MathML,
    SVG,
    XLink,
    XML,
    XMLNS,
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl fmt::Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                | Self::HTML => "http://www.w3.org/1999/xhtml",
                | Self::MathML => "http://www.w3.org/1998/Math/MathML",
                | Self::SVG => "http://www.w3.org/2000/svg",
                | Self::XLink => "http://www.w3.org/1999/xlink",
                | Self::XML => "http://www.w3.org/XML/1998/namespace",
                | Self::XMLNS => "http://www.w3.org/2000/xmlns/",
            }
        )
    }
}

impl str::FromStr for Namespace {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_ref() {
            | "html" => Self::HTML,
            | "mathml" => Self::MathML,
            | "svg" => Self::SVG,
            | "xlink" => Self::XLink,
            | "xml" => Self::XML,
            | "xmlns" => Self::XMLNS,
            | _ => {
                return Err("\
                    Espace de nom inconnu. \
                    Valeur attendu: \
                    'html' | 'mathml' | 'svg' | 'xlink' | 'xml' | 'xmlns'\
                ")
            }
        })
    }
}
