/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use core::{fmt, str};
use std::{cell::RefCell, collections::HashMap};

use html_elements::{
    interface::{HTMLElementInterface, IsOneOfTagsInterface},
    tag_names, HTMLScriptElement, HTMLTemplateElement,
};
use infra::{namespace::Namespace, primitive::string::DOMString};

use super::{document_fragment::DocumentFragmentNode, DocumentNode};

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub struct Element {
    inner: HTMLElement,
    // TODO(phisyx): changer le type de cet attribut en NamedNodeMap (cf. https://dom.spec.whatwg.org/#namednodemap)
    pub attributes: RefCell<HashMap<String, String>>,
    pub id: RefCell<Option<DOMString>>,
    pub is: RefCell<Option<DOMString>>,
    pub namespace_uri: RefCell<Namespace>,
}

// ----------- //
// Énumération //
// ----------- //

/// 4 The elements of HTML
#[derive(Debug)]
#[derive(PartialEq, Eq)]
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
    MetadataMeta(
        /// 4.2.5 The meta element
        html_elements::HTMLMetaElement,
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
    GroupingContentBlockquote(
        /// 4.4.4 The blockquote element
        html_elements::HTMLQuoteElement,
    ),
    GroupingContentOl(
        /// 4.4.5 The ol element
        html_elements::HTMLOListElement,
    ),
    GroupingContentUl(
        /// 4.4.6 The ul element
        html_elements::HTMLUListElement,
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

    // 4.8 Embedded content
    EmbeddedContentImg(
        /// 4.8.3 The img element
        html_elements::HTMLImageElement,
    ),

    // 4.9 Tabular data
    TabularDataTable(
        /// 4.9.1 The table element
        html_elements::HTMLTableElement,
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

    Unknown(HTMLUnknownElement),
}

#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub struct HTMLUnknownElement {
    name: String,
}

impl HTMLUnknownElement {
    pub fn tag_name(&self) -> &str {
        &self.name
    }
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
            is: RefCell::new(is.map(DOMString::from)),
            namespace_uri: RefCell::new(namespace_uri),
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
    ) -> Option<&HTMLTemplateElement<DocumentFragmentNode>> {
        assert!(matches!(self.inner, HTMLElement::ScriptingTemplate(_)));

        if let HTMLElement::ScriptingTemplate(el) = &self.inner {
            return Some(el);
        }

        None
    }

    pub fn namespace(&self) -> Option<Namespace> {
        (*self.namespace_uri.borrow()).into()
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
            let attrs = self.attributes.borrow().clone();
            let encoding_str = "encoding";
            let maybe_encoding = attrs.get(&encoding_str.to_owned());
            if let Some(encoding) = maybe_encoding {
                let encoding_str = "text/html";
                if encoding_str.eq_ignore_ascii_case(encoding) {
                    return true;
                }

                let encoding_str = "application/xhtml+xml";
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
        (*self.attributes.borrow()).contains_key(&name.to_owned())
    }

    pub fn set_attribute(&self, name: &str, value: &str) {
        if name == "id" {
            self.id.borrow_mut().replace(value.to_owned().into());
            return;
        }

        (self.attributes.borrow_mut())
            .insert(name.to_owned(), value.to_owned());
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl str::FromStr for HTMLElement {
    type Err = &'static str;

    fn from_str(local_name: &str) -> Result<Self, Self::Err> {
        let tag_name = local_name
            .to_ascii_lowercase()
            .parse::<tag_names>()
            .unwrap_or(tag_names::customElement);

        Ok(match tag_name {
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
            | tag_names::meta => Self::MetadataMeta(
                html_elements::HTMLMetaElement::default(),
            ),

            | tag_names::template => Self::ScriptingTemplate(
                html_elements::HTMLTemplateElement::default(),
            ),
            | tag_names::script => Self::ScriptingScript(
                html_elements::HTMLScriptElement::default(),
            ),

            | tag_names::hr => Self::GroupingContentHr(
                html_elements::HTMLHRElement::default(),
            ),
            | tag_names::pre => Self::GroupingContentPre(
                html_elements::HTMLPreElement::default(),
            ),
            | tag_names::blockquote => Self::GroupingContentBlockquote(
                html_elements::HTMLQuoteElement::default(),
            ),
            | tag_names::ol => Self::GroupingContentOl(
                html_elements::HTMLOListElement::default(),
            ),
            | tag_names::ul => Self::GroupingContentUl(
                html_elements::HTMLUListElement::default(),
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

            | tag_names::img => Self::EmbeddedContentImg(
                html_elements::HTMLImageElement::default(),
            ),

            | tag_names::table => Self::TabularDataTable(
                html_elements::HTMLTableElement::default(),
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
            | _ => Self::Unknown(HTMLUnknownElement {
                name: local_name.to_owned(),
            }),
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
                | Self::MetadataMeta(el) => el.tag_name(),
                | Self::GroupingContentHr(el) => el.tag_name(),
                | Self::GroupingContentPre(el) => el.tag_name(),
                | Self::GroupingContentBlockquote(el) => el.tag_name(),
                | Self::GroupingContentOl(el) => el.tag_name(),
                | Self::GroupingContentUl(el) => el.tag_name(),
                | Self::GroupingContentLi(el) => el.tag_name(),
                | Self::GroupingContentDl(el) => el.tag_name(),
                | Self::GroupingContentDiv(el) => el.tag_name(),
                | Self::TextLevelSpan(el) => el.tag_name(),
                | Self::EmbeddedContentImg(el) => el.tag_name(),
                | Self::TabularDataTable(el) => el.tag_name(),
                | Self::FormButton(el) => el.tag_name(),
                | Self::SectionBody(el) => el.tag_name(),
                | Self::SectionHeading(el) => el.tag_name(),
                | Self::ScriptingScript(el) => el.tag_name(),
                | Self::ScriptingTemplate(el) => el.tag_name(),
                | Self::Unknown(el) => el.tag_name(),
            }
        )
    }
}
