/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use crate::interface::HTMLElementInterface;

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
#[derive(Default)]
#[derive(PartialEq)]
pub struct HTMLTemplateElement<DocumentFragmentNode> {
    pub content: RefCell<DocumentFragmentNode>,
}

// -------------- //
// Implémentation //
// -------------- //

impl<DocumentFragment> HTMLTemplateElement<DocumentFragment> {
    pub const NAME: &'static str = "template";

    pub fn new(content: DocumentFragment) -> Self {
        Self {
            content: RefCell::new(content),
        }
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl<DocumentFragment> HTMLElementInterface
    for HTMLTemplateElement<DocumentFragment>
{
    fn tag_name(&self) -> &'static str {
        Self::NAME
    }
}
