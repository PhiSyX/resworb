/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops;

use dom::node::Node;
use html_elements::tag_names;
use infra::structure::tree::TreeNode;

// --------- //
// Structure //
// --------- //

#[derive(Default)]
pub(crate) struct ListOfActiveFormattingElements {
    entries: Vec<Entry>,
}

// ----------- //
// Énumération //
// ----------- //

#[derive(PartialEq)]
pub(crate) enum Entry {
    Marker,
    Element(TreeNode<Node>),
}

// -------------- //
// Implémentation //
// -------------- //

impl ListOfActiveFormattingElements {
    pub(crate) fn clear_up_to_the_last_marker(&mut self) {
        while let Some(entry) = self.entries.pop() {
            if entry.is_marker() {
                break;
            }
        }
    }

    pub(crate) fn contains_element(
        &self,
        element: &TreeNode<Node>,
    ) -> bool {
        self.entries
            .iter()
            .any(|entry| Entry::Element(element.to_owned()).eq(entry))
    }

    pub(crate) fn insert_marker_at_end(&mut self) {
        self.entries.push(Entry::Marker);
    }

    pub(crate) fn last_element_before_marker(
        &self,
        tag_name: tag_names,
    ) -> Option<(usize, TreeNode<Node>)> {
        self.entries
            .iter()
            .enumerate()
            .rfind(|(_, entry)| {
                if let Entry::Element(element) = entry {
                    tag_name == element.element_ref().local_name()
                } else {
                    false
                }
            })
            .and_then(|(idx, entry)| {
                entry.element().map(|node| (idx, node.to_owned()))
            })
    }

    pub(crate) fn remove_element(&mut self, element: &TreeNode<Node>) {
        if let Some(idx) = self.entries.iter().position(|entry| {
            if let Entry::Element(element_node) = entry {
                element_node == element
            } else {
                false
            }
        }) {
            self.entries.remove(idx);
        }
    }

    pub(crate) fn position_of(
        &self,
        element: &TreeNode<Node>,
    ) -> Option<usize> {
        self.entries.iter().enumerate().rposition(|(_, entry)| {
            if let Entry::Element(element_ref) = entry {
                element_ref == element
            } else {
                false
            }
        })
    }
}

impl Entry {
    pub(crate) const fn is_marker(&self) -> bool {
        matches!(self, Self::Marker)
    }

    pub(crate) const fn element(&self) -> Option<&TreeNode<Node>> {
        match self {
            | Entry::Marker => None,
            | Entry::Element(node) => Some(node),
        }
    }

    pub(crate) const fn element_unchecked(&self) -> &TreeNode<Node> {
        match self {
            | Entry::Marker => {
                panic!("N'est pas une entrée de type Entry::Element.")
            }
            | Entry::Element(node) => node,
        }
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl ops::Deref for ListOfActiveFormattingElements {
    type Target = Vec<Entry>;

    fn deref(&self) -> &Self::Target {
        self.entries.as_ref()
    }
}

impl ops::DerefMut for ListOfActiveFormattingElements {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.entries.as_mut()
    }
}
