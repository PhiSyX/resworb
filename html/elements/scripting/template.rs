/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::RwLock;

use crate::interface::HTMLElementInterface;

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
#[derive(Default)]
pub struct HTMLTemplateElement<DocumentFragmentNode> {
    pub content: RwLock<DocumentFragmentNode>,
}

// -------------- //
// Implémentation //
// -------------- //

impl<DocumentFragment> HTMLTemplateElement<DocumentFragment> {
    pub const NAME: &'static str = "template";

    pub fn new(content: DocumentFragment) -> Self {
        Self {
            content: RwLock::new(content),
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

impl<DocumentFragment> PartialEq for HTMLTemplateElement<DocumentFragment>
where
    DocumentFragment: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.tag_name() == other.tag_name()
            && *self.content.read().unwrap()
                == *other.content.read().unwrap()
    }
}

impl<DocumentFragment> Eq for HTMLTemplateElement<DocumentFragment> where
    DocumentFragment: Eq
{
}
