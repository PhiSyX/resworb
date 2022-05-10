/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom::node::{Element, Node};
use infra::structure::tree::TreeNode;

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
#[derive(Copy, Clone)]
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
pub struct StackOfOpenElements {
    elements: Vec<TreeNode<Node>>,
}

/// 13.2.4.3 The list of active formatting elements
pub struct ListOfActiveFormattingElements {
    entries: Vec<Entry>,
}

struct Entry {
    element: Element,
}

// -------------- //
// Implémentation //
// -------------- //

impl InsertionMode {
    pub fn switch_to(&mut self, mode: Self) {
        *self = mode;
    }
}

impl StackOfOpenElements {
    /// Taille du vecteur de noeuds d'éléments.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Le nœud actuel est le nœud le plus bas de cette pile d'éléments
    /// ouverts.
    pub fn current_node(&self) -> Option<&TreeNode<Node>> {
        self.elements.last()
    }

    /// Vérifie si le vecteur de noeuds d'éléments est vide.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Premier élément du vecteur de noeuds d'éléments.
    pub fn first(&self) -> Option<&TreeNode<Node>> {
        self.elements.first()
    }

    /// Dernier élément (élément HTML) du vecteur de noeuds d'éléments,
    /// qui a le même nom que celui passé en argument.
    pub fn get_last_element_with_tag_name(
        &self,
        tag_name: &str,
    ) -> Option<(usize, &TreeNode<Node>)> {
        self.elements.iter().enumerate().rfind(|(_, element)| {
            element.element_ref().local_name() == tag_name
        })
    }

    pub fn element_immediately_above(
        &self,
        node_index: usize,
    ) -> Option<&TreeNode<Node>> {
        self.elements.get(node_index - 1)
    }

    /// Ajoute un nouvel arbre de noeud dans le vecteur.
    pub fn put(&mut self, element: TreeNode<Node>) {
        self.elements.push(element);
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

#[allow(clippy::derivable_impls)]
impl Default for StackOfOpenElements {
    fn default() -> Self {
        Self {
            elements: Default::default(),
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for ListOfActiveFormattingElements {
    fn default() -> Self {
        Self {
            entries: Default::default(),
        }
    }
}

impl Default for InsertionMode {
    /// Initialement, le mode d'insertion est "initial".
    fn default() -> Self {
        Self::Initial
    }
}
