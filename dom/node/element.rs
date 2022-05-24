/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use core::{fmt, str};
use std::{
    collections::HashMap,
    sync::{RwLock, RwLockReadGuard},
};

use html_elements::{
    interface::{HTMLElementInterface, IsOneOfTagsInterface},
    tag_names, HTMLScriptElement,
};
use infra::{namespace::Namespace, primitive::string::DOMString};

use super::{document_fragment::DocumentFragmentNode, DocumentNode};

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
pub struct Element {
    inner: HTMLElement,
    // todo: changer en NamedNodeMap (cf. https://dom.spec.whatwg.org/#namednodemap)
    pub attributes: RwLock<HashMap<DOMString, DOMString>>,
    pub id: RwLock<Option<DOMString>>,
    pub is: RwLock<Option<DOMString>>,
    pub namespace_uri: RwLock<Namespace>,
}

// ----------- //
// Énumération //
// ----------- //

/// 4 The elements of HTML
#[derive(Debug)]
#[derive(PartialEq)]
pub enum HTMLElement {
    // 4.1 The document element
    DocumentHtml(
        /// 4.1.1 The html element
        html_elements::HTMLHtmlElement,
    ),

    // 4.2 Document metadata
    MetadataHead(
        /// 4.2.1 The head element
        html_elements::HTMLHeadElement,
    ),
    MetadataTitle(
        /// 4.2.2 The title element
        html_elements::HTMLTitleElement,
    ),

    // 4.3 Sections
    SectionBody(
        /// 4.3.1 The body element
        html_elements::HTMLBodyElement,
    ),
    SectionHeading(
        /// 4.3.6 The h1, h2, h3, h4, h5, and h6 elements
        html_elements::HTMLHeadingElement,
    ),

    // 4.4 Grouping content
    GroupingContentHr(
        /// 4.4.2 The hr element
        html_elements::HTMLHRElement,
    ),
    GroupingContentPre(
        /// 4.4.3 The pre element
        html_elements::HTMLPreElement,
    ),
    GroupingContentOl(
        /// 4.4.5 The ol element
        html_elements::HTMLOListElement,
    ),
    GroupingContentLi(
        /// 4.4.8 The li element
        html_elements::HTMLLIElement,
    ),
    GroupingContentDl(
        /// 4.4.9 The dl element
        html_elements::HTMLDListElement,
    ),
    GroupingContentDiv(
        /// 4.4.15 The div element
        html_elements::HTMLDivElement,
    ),

    // 4.5 Text-level semantics
    TextLevelSpan(
        /// 4.5.26 The span element
        html_elements::HTMLSpanElement,
    ),

    // 4.10 Forms
    FormButton(
        /// 4.10.6 The button element
        html_elements::HTMLButtonElement,
    ),

    // 4.12 Scripting
    ScriptingScript(html_elements::HTMLScriptElement<DocumentNode>),
    ScriptingTemplate(
        /// 4.12.3 The template element
        html_elements::HTMLTemplateElement<DocumentFragmentNode>,
    ),
}

// -------------- //
// Implémentation //
// -------------- //

impl Element {
    pub fn new(
        data: HTMLElement,
        is: Option<String>,
        namespace_uri: Namespace,
    ) -> Self {
        Self {
            inner: data,
            attributes: Default::default(),
            id: Default::default(),
            is: RwLock::new(is.map(DOMString::from)),
            namespace_uri: RwLock::new(namespace_uri),
        }
    }
}

impl Element {
    pub fn local_name(&self) -> String {
        self.inner.to_string()
    }

    pub fn tag_name(&self) -> tag_names {
        self.local_name()
            .parse()
            .expect("Devrait être un nom de balise valide")
    }

    pub fn content(
        &self,
    ) -> Option<RwLockReadGuard<DocumentFragmentNode>> {
        assert!(matches!(self.inner, HTMLElement::ScriptingTemplate(_)));

        if let HTMLElement::ScriptingTemplate(el) = &self.inner {
            return el.content.read().ok();
        }

        None
    }

    pub fn namespace(&self) -> Option<Namespace> {
        self.namespace_uri.read().ok().map(|ns| *ns)
    }

    pub fn isin_html_namespace(&self) -> bool {
        self.namespace() == Some(Namespace::HTML)
    }

    pub fn isin_svg_namespace(&self) -> bool {
        self.namespace() == Some(Namespace::SVG)
    }

    pub fn is_mathml_text_integration_point(&self) -> bool {
        self.tag_name().is_one_of([
            tag_names::mi,
            tag_names::mo,
            tag_names::mn,
            tag_names::ms,
            tag_names::mtext,
        ])
    }

    pub fn is_html_text_integration_point(&self) -> bool {
        if self.tag_name() == tag_names::annotationXml {
            let attrs = self.attributes.read().unwrap();
            let encoding_str: DOMString = "encoding".into();
            let maybe_encoding = attrs.get(&encoding_str);
            if let Some(encoding) = maybe_encoding {
                let encoding_str: DOMString = "text/html".into();
                if encoding_str.eq_ignore_ascii_case(encoding) {
                    return true;
                }

                let encoding_str: DOMString =
                    "application/xhtml+xml".into();
                if encoding_str.eq_ignore_ascii_case(encoding) {
                    return true;
                }
            }
            return false;
        }

        self.tag_name().is_one_of([
            tag_names::foreignObject,
            tag_names::desc,
            tag_names::title,
        ])
    }

    pub fn script(&self) -> &HTMLScriptElement<DocumentNode> {
        match &self.inner {
            | HTMLElement::ScriptingScript(script) => script,
            | _ => panic!("N'est pas un élément HTMLScriptElement."),
        }
    }

    pub fn has_attribute(&self, name: &str) -> bool {
        let dom_string = DOMString::new(name);
        self.attributes.read().unwrap().contains_key(&dom_string)
    }

    pub fn set_attribute(&self, name: &str, value: &str) {
        if name == "id" {
            *self.id.write().unwrap() = Some(DOMString::from(value));
            return;
        }

        self.attributes
            .write()
            .unwrap()
            .insert(DOMString::new(name), DOMString::new(value));
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl PartialEq for Element {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
            && *self.attributes.read().unwrap()
                == *other.attributes.read().unwrap()
            && *self.id.read().unwrap() == *other.id.read().unwrap()
    }
}

impl str::FromStr for HTMLElement {
    type Err = &'static str;

    fn from_str(local_name: &str) -> Result<Self, Self::Err> {
        let local_name =
            local_name.to_ascii_lowercase().parse::<tag_names>()?;

        Ok(match local_name {
            | tag_names::html => Self::DocumentHtml(
                html_elements::HTMLHtmlElement::default(),
            ),
            | tag_names::head => Self::MetadataHead(
                html_elements::HTMLHeadElement::default(),
            ),
            | tag_names::body => {
                Self::SectionBody(html_elements::HTMLBodyElement::default())
            }
            | tag_names::title => Self::MetadataTitle(
                html_elements::HTMLTitleElement::default(),
            ),
            | tag_names::template => Self::ScriptingTemplate(
                html_elements::HTMLTemplateElement::default(),
            ),
            | tag_names::script => Self::ScriptingScript(
                html_elements::HTMLScriptElement::default(),
            ),
            | _ => {
                return Err("Element non pris en charge pour le moment.")
            | tag_names::hr => Self::GroupingContentHr(
                html_elements::HTMLHRElement::default(),
            ),
            | tag_names::pre => Self::GroupingContentPre(
                html_elements::HTMLPreElement::default(),
            ),
            | tag_names::ol => Self::GroupingContentOl(
                html_elements::HTMLOListElement::default(),
            ),
            | tag_names::li => Self::GroupingContentLi(
                html_elements::HTMLLIElement::default(),
            ),
            | tag_names::dl => Self::GroupingContentDl(
                html_elements::HTMLDListElement::default(),
            ),
            | tag_names::div => Self::GroupingContentDiv(
                html_elements::HTMLDivElement::default(),
            ),
            | tag_names::span => Self::TextLevelSpan(
                html_elements::HTMLSpanElement::default(),
            ),
            | tag_names::button => Self::FormButton(
                html_elements::HTMLButtonElement::default(),
            ),

            | heading @ (tag_names::h1
            | tag_names::h2
            | tag_names::h3
            | tag_names::h4
            | tag_names::h5
            | tag_names::h6) => Self::SectionHeading(
                html_elements::HTMLHeadingElement::new(heading),
            ),
            }
        })
    }
}

impl fmt::Display for HTMLElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                | Self::DocumentHtml(el) => el.tag_name(),
                | Self::MetadataHead(el) => el.tag_name(),
                | Self::MetadataTitle(el) => el.tag_name(),
                | Self::GroupingContentHr(el) => el.tag_name(),
                | Self::GroupingContentPre(el) => el.tag_name(),
                | Self::GroupingContentOl(el) => el.tag_name(),
                | Self::GroupingContentLi(el) => el.tag_name(),
                | Self::GroupingContentDl(el) => el.tag_name(),
                | Self::GroupingContentDiv(el) => el.tag_name(),
                | Self::TextLevelSpan(el) => el.tag_name(),
                | Self::FormButton(el) => el.tag_name(),
                | Self::SectionBody(el) => el.tag_name(),
                | Self::SectionHeading(el) => el.tag_name(),
                | Self::ScriptingScript(el) => el.tag_name(),
                | Self::ScriptingTemplate(el) => el.tag_name(),
            }
        )
    }
}
