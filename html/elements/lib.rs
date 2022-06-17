/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod document;
mod metadata;
mod scripting;
mod text_level;

mod embedded_content;
mod forms;
mod grouping_content;
pub mod interface;
mod sections;
mod tables;
mod tags;

use core::{fmt, ops};
use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
};

use infra::{namespace::Namespace, primitive::string::DOMString};
use interface::{HTMLElementInterface, IsOneOfTagsInterface};

pub use self::{
    document::*,
    embedded_content::*,
    forms::*,
    grouping_content::*,
    metadata::*,
    scripting::*,
    sections::*,
    tables::*,
    tags::{tag_attributes, tag_names},
    text_level::*,
};

// ----- //
// Macro //
// ----- //

#[macro_export]
macro_rules! html_element {
    (
        struct $name:ident $(< $generic:ident >)? ( ? ) {
            $($field:ident: $type:ty),*
        }
    ) => {
        #[derive(Debug)]
        #[derive(PartialEq, Eq)]
        pub struct $name$(< $generic >)? {
            extend: $crate::HTMLElement,
            $(pub $field: $type),*
        }

        impl$(< $generic >)? $name $(< $generic >)? {
            pub fn new(
                el: $crate::HTMLElement,
                $($field: $type),*
            ) -> Self {
                Self { extend: el, $($field),* }
            }
        }

        impl$(< $generic >)? ::core::ops::Deref for $name $(< $generic >)? {
            type Target = $crate::HTMLElement;

            fn deref(&self) -> &Self::Target {
                &self.extend
            }
        }
    };

    (
        struct $name:ident $(< $generic:ident >)? ( $tag_name:ident ) {
            $($field:ident: $type:ty),*
        }
    ) => {
        #[derive(Debug)]
        #[derive(PartialEq, Eq)]
        pub struct $name$(< $generic >)? {
            extend: $crate::HTMLElement,
            $(pub $field: $type),*
        }

        impl$(< $generic >)? $name$(< $generic >)? {
            pub const NAME: &'static str = stringify!($tag_name);

            pub fn new(
                el: $crate::HTMLElement,
                $($field: $type),*
            ) -> Self {
                Self { extend: el, $($field),* }
            }
        }

        impl$(< $generic >)? $crate::interface::HTMLElementInterface for $name$(< $generic >)? {
            fn tag_name(&self) -> String {
                Self::NAME.to_owned()
            }
        }

        impl$(< $generic >)? ::core::ops::Deref for $name$(< $generic >)? {
            type Target = $crate::HTMLElement;

            fn deref(&self) -> &Self::Target {
                &self.extend
            }
        }
    };
}

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub struct Element {
    name: DOMString,

    // TODO(phisyx): changer le type de cet attribut en NamedNodeMap (cf. https://dom.spec.whatwg.org/#namednodemap)
    pub attributes: RefCell<HashMap<String, String>>,
    pub id: RefCell<Option<DOMString>>,
    pub is: RefCell<Option<DOMString>>,
    pub namespace_uri: RefCell<Namespace>,
}

#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub struct HTMLElement {
    extend: Element,
}

html_element! {
    struct HTMLUnknownElement(?) {}
}

// ----------- //
// Énumération //
// ----------- //

/// 4 The elements of HTML
#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub enum HTMLElementVariant<Document, Fragment> {
    // 4.1 The document element
    DocumentHtml(
        /// 4.1.1 The html element
        HTMLHtmlElement,
    ),

    // 4.2 Document metadata
    MetadataHead(
        /// 4.2.1 The head element
        HTMLHeadElement,
    ),
    MetadataTitle(
        /// 4.2.2 The title element
        HTMLTitleElement,
    ),
    MetadataMeta(
        /// 4.2.5 The meta element
        HTMLMetaElement,
    ),

    // 4.3 Sections
    SectionBody(
        /// 4.3.1 The body element
        HTMLBodyElement,
    ),
    SectionHeading(
        /// 4.3.6 The h1, h2, h3, h4, h5, and h6 elements
        HTMLHeadingElement,
    ),

    // 4.4 Grouping content
    GroupingContentHr(
        /// 4.4.2 The hr element
        HTMLHRElement,
    ),
    GroupingContentPre(
        /// 4.4.3 The pre element
        HTMLPreElement,
    ),
    GroupingContentBlockquote(
        /// 4.4.4 The blockquote element
        HTMLQuoteElement,
    ),
    GroupingContentOl(
        /// 4.4.5 The ol element
        HTMLOListElement,
    ),
    GroupingContentUl(
        /// 4.4.6 The ul element
        HTMLUListElement,
    ),
    GroupingContentLi(
        /// 4.4.8 The li element
        HTMLLIElement,
    ),
    GroupingContentDl(
        /// 4.4.9 The dl element
        HTMLDListElement,
    ),
    GroupingContentDiv(
        /// 4.4.15 The div element
        HTMLDivElement,
    ),

    // 4.5 Text-level semantics
    TextLevelSpan(
        /// 4.5.26 The span element
        HTMLSpanElement,
    ),

    // 4.8 Embedded content
    EmbeddedContentImg(
        /// 4.8.3 The img element
        HTMLImageElement,
    ),

    // 4.9 Tabular data
    TabularDataTable(
        /// 4.9.1 The table element
        HTMLTableElement,
    ),

    // 4.10 Forms
    FormButton(
        /// 4.10.6 The button element
        HTMLButtonElement,
    ),

    // 4.12 Scripting
    ScriptingScript(HTMLScriptElement<Document>),
    ScriptingTemplate(
        /// 4.12.3 The template element
        HTMLTemplateElement<Fragment>,
    ),
    Unknown(HTMLUnknownElement),
}

// -------------- //
// Implementation //
// -------------- //

// Self
impl Element {
    pub fn new(
        name: DOMString,
        is: Option<String>,
        namespace_uri: Namespace,
    ) -> Self {
        Self {
            name,
            attributes: Default::default(),
            id: Default::default(),
            is: RefCell::new(is.map(DOMString::from)),
            namespace_uri: RefCell::new(namespace_uri),
        }
    }
}

// &Self
impl Element {
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

    pub fn local_name(&self) -> String {
        self.name.borrow().to_owned()
    }

    pub fn tag_name(&self) -> tag_names {
        self.name
            .borrow()
            .parse()
            .expect("Devrait être un nom de balise valide.")
    }

    pub fn has_attribute(&self, name: &str) -> bool {
        (*self.attributes.borrow()).contains_key(&name.to_owned())
    }
}

// &mut Self
impl Element {
    pub fn set_attribute(&self, name: &str, value: &str) {
        if name == "id" {
            self.id.borrow_mut().replace(value.to_owned().into());
            return;
        }

        (self.attributes.borrow_mut())
            .insert(name.to_owned(), value.to_owned());
    }
}

// Self
impl HTMLElement {
    pub fn new(element: Element) -> Self {
        Self { extend: element }
    }
}

// &Self
impl HTMLElement {}

impl HTMLUnknownElement {
    fn tag_name(&self) -> String {
        unsafe { &*self.name.as_ptr() }.to_owned()
    }
}

// &Self

impl<D, F> HTMLElementVariant<D, F> {
    pub fn content(&self) -> Option<Ref<'_, F>> {
        assert!(matches!(self, Self::ScriptingTemplate(_)));

        if let Self::ScriptingTemplate(el) = &self {
            return Some(el.content.borrow());
        }

        None
    }

    pub fn html(&self) -> &HTMLElement {
        match self {
            | Self::DocumentHtml(el) => el,
            | Self::MetadataHead(el) => el,
            | Self::MetadataTitle(el) => el,
            | Self::MetadataMeta(el) => el,
            | Self::SectionBody(el) => el,
            | Self::SectionHeading(el) => el,
            | Self::GroupingContentHr(el) => el,
            | Self::GroupingContentPre(el) => el,
            | Self::GroupingContentBlockquote(el) => el,
            | Self::GroupingContentOl(el) => el,
            | Self::GroupingContentUl(el) => el,
            | Self::GroupingContentLi(el) => el,
            | Self::GroupingContentDl(el) => el,
            | Self::GroupingContentDiv(el) => el,
            | Self::TextLevelSpan(el) => el,
            | Self::EmbeddedContentImg(el) => el,
            | Self::TabularDataTable(el) => el,
            | Self::FormButton(el) => el,
            | Self::ScriptingScript(el) => el,
            | Self::ScriptingTemplate(el) => el,
            | Self::Unknown(el) => el,
        }
    }

    pub fn script(&self) -> &HTMLScriptElement<D> {
        match self {
            | Self::ScriptingScript(script) => script,
            | _ => panic!("N'est pas un élément HTMLScriptElement."),
        }
    }
}

// -------------- //
// Implementation // -> Interface
// -------------- //

impl<D, F> From<HTMLElement> for HTMLElementVariant<D, F>
where
    D: Default,
    F: Default,
{
    fn from(el: HTMLElement) -> Self {
        let tag_name = el
            .local_name()
            .to_ascii_lowercase()
            .parse::<tag_names>()
            .unwrap_or(tag_names::customElement);

        match tag_name {
            | tag_names::html => {
                Self::DocumentHtml(HTMLHtmlElement::new(el))
            }

            | tag_names::head => {
                Self::MetadataHead(HTMLHeadElement::new(el))
            }
            | tag_names::body => {
                Self::SectionBody(HTMLBodyElement::new(el))
            }
            | tag_names::title => {
                Self::MetadataTitle(HTMLTitleElement::new(el))
            }
            | tag_names::meta => {
                Self::MetadataMeta(HTMLMetaElement::new(el))
            }

            | tag_names::template => Self::ScriptingTemplate(
                HTMLTemplateElement::new(el, Default::default()),
            ),
            | tag_names::script => {
                Self::ScriptingScript(HTMLScriptElement::new(
                    el,
                    Default::default(),
                    Default::default(),
                    Default::default(),
                ))
            }

            | tag_names::hr => {
                Self::GroupingContentHr(HTMLHRElement::new(el))
            }
            | tag_names::pre => {
                Self::GroupingContentPre(HTMLPreElement::new(el))
            }
            | tag_names::blockquote => {
                Self::GroupingContentBlockquote(HTMLQuoteElement::new(el))
            }
            | tag_names::ol => {
                Self::GroupingContentOl(HTMLOListElement::new(el))
            }

            | tag_names::ul => {
                Self::GroupingContentUl(HTMLUListElement::new(el))
            }
            | tag_names::li => {
                Self::GroupingContentLi(HTMLLIElement::new(el))
            }
            | tag_names::dl => {
                Self::GroupingContentDl(HTMLDListElement::new(el))
            }
            | tag_names::div => {
                Self::GroupingContentDiv(HTMLDivElement::new(el))
            }
            | tag_names::span => {
                Self::TextLevelSpan(HTMLSpanElement::new(el))
            }

            | tag_names::img => {
                Self::EmbeddedContentImg(HTMLImageElement::new(el))
            }

            | tag_names::table => {
                Self::TabularDataTable(HTMLTableElement::new(el))
            }

            | tag_names::button => {
                Self::FormButton(HTMLButtonElement::new(el))
            }

            | heading @ (tag_names::h1
            | tag_names::h2
            | tag_names::h3
            | tag_names::h4
            | tag_names::h5
            | tag_names::h6) => {
                let level = match heading {
                    | tag_names::h1 => 1,
                    | tag_names::h2 => 2,
                    | tag_names::h3 => 3,
                    | tag_names::h4 => 4,
                    | tag_names::h5 => 5,
                    | tag_names::h6 => 6,
                    | _ => unreachable!(),
                };
                Self::SectionHeading(HTMLHeadingElement::new(el, level))
            }

            | _ => Self::Unknown(HTMLUnknownElement::new(el)),
        }

        /*

            | _ => Self::Unknown(HTMLUnknownElement::new(local_name)),
        }) */
    }
}

impl<D, F> fmt::Display for HTMLElementVariant<D, F> {
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

impl ops::Deref for HTMLElement {
    type Target = Element;

    fn deref(&self) -> &Self::Target {
        &self.extend
    }
}

impl<D, F> ops::Deref for HTMLElementVariant<D, F> {
    type Target = Element;

    fn deref(&self) -> &Self::Target {
        self.html()
    }
}
