/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom::node::Element;

// ----------- //
// Énumération //
// ----------- //

/// 13.2.4.1 The insertion mode
///
/// Le mode d'insertion est une variable d'état qui contrôle l'opération
/// primaire de l'étape de construction de l'arbre.  Le mode d'insertion
/// affecte la manière dont les tokens sont traités et si les sections
/// CDATA sont supportées.
#[derive(Debug)]
#[derive(PartialEq)]
pub enum InsertionMode {
    Initial,
    BeforeHTML,
    BeforeHead,
    InHead,
    InHeadNoscript,
    AfterHead,
    InBody,
    Text,
    InTable,
    InTableText,
    InCaption,
    InColumnGroup,
    InTableBody,
    InRow,
    InCell,
    InSelect,
    InSelectInTable,
    InTemplate,
    AfterBody,
    InFrameset,
    AfterFrameset,
    AfterAfterBody,
    AfterAfterFrameset,
}

/// 13.2.4.2 The stack of open elements
#[derive(Default)]
pub struct StackOfOpenElements {
    pub(crate) elements: Vec<Element>,
}

/// 13.2.4.3 The list of active formatting elements
#[derive(Default)]
pub struct ListOfActiveFormattingElements {
    entries: Vec<Entry>,
}

#[derive(Default)]
struct Entry {
    element: Element,
}

// -------------- //
// Implémentation //
// -------------- //

impl StackOfOpenElements {
    pub fn current_node(&self) -> Option<&Element> {
        self.elements.last()
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl Default for InsertionMode {
    /// Initialement, le mode d'insertion est "initial".
    fn default() -> Self {
        Self::Initial
    }
}
