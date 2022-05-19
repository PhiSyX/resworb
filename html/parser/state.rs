/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use core::ops;

use dom::node::Node;
use html_elements::tag_names;
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
    pub(crate) elements: Vec<TreeNode<Node>>,
}

/// 13.2.4.3 The list of active formatting elements
pub struct ListOfActiveFormattingElements {
    entries: Vec<Entry>,
}

// ----------- //
// Énumération //
// ----------- //

#[derive(PartialEq)]
pub enum Entry {
    Marker,
    Element(TreeNode<Node>),
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
    ///   - applet
    ///   - caption
    ///   - html
    ///   - table
    ///   - td
    ///   - th
    ///   - marquee
    ///   - object
    ///   - template
    ///   - MathML mi
    ///   - MathML mo
    ///   - MathML mn
    ///   - MathML ms
    ///   - MathML mtext
    ///   - MathML annotation-xml
    ///   - SVG foreignObject
    ///   - SVG desc
    ///   - SVG title
    pub const SCOPE_ELEMENTS: [tag_names; 9] = [
        tag_names::applet,
        tag_names::caption,
        tag_names::html,
        tag_names::table,
        tag_names::td,
        tag_names::th,
        tag_names::marquee,
        tag_names::object,
        tag_names::template,
        // todo: ajouter les éléments manquants MathML & SVG
    ];

    /// Le nœud actuel est le nœud le plus bas de cette pile d'éléments
    /// ouverts.
    pub fn current_node(&self) -> Option<&TreeNode<Node>> {
        self.elements.last()
    }

    /// Dernier élément (élément HTML) du vecteur de noeuds d'éléments,
    /// qui a le même nom que celui passé en argument.
    pub fn get_last_element_with_tag_name(
        &self,
        tag_name: tag_names,
    ) -> Option<(usize, &TreeNode<Node>)> {
        self.elements.iter().enumerate().rfind(|(_, element)| {
            tag_name == element.element_ref().local_name()
        })
    }

    /// On dit que la pile d'éléments ouverts a un élément particulier dans
    /// son champ d'application lorsqu'elle a cet élément dans le champ
    /// d'application spécifique composé des types d'éléments suivants :
    /// Voir la constante: [Self::SCOPE_ELEMENTS].
    pub fn has_element_in_scope<const N: usize>(
        &self,
        tag_name: tag_names,
        list: [tag_names; N],
    ) -> bool {
        let tag_name: tag_names = tag_name
            .try_into()
            .ok()
            .expect("Devrait être un nom de balise valide.");

        self.elements.iter().rev().any(|node| {
            let element = node.element_ref();
            let name = element.local_name();
            tag_name == name
                || !list.into_iter().any(|tag_name| tag_name == name)
        })
    }

    pub fn has_element_with_tag_name(&self, tag_name: tag_names) -> bool {
        self.elements
            .iter()
            .any(|element| tag_name == element.element_ref().local_name())
    }

    pub fn element_immediately_above(
        &self,
        node_index: usize,
    ) -> Option<&TreeNode<Node>> {
        self.elements.get(node_index - 1)
    }

    pub fn pop_until_tag(&mut self, tag_name: tag_names) {
        while let Some(node) = self.current_node() {
            let element = node.element_ref();
            if tag_name == element.local_name() {
                self.elements.pop();
                break;
            }
            self.elements.pop();
        }
    }

    pub fn remove_first_tag_matching<P>(&mut self, predicate: P)
    where
        P: Fn(&TreeNode<Node>) -> bool,
    {
        let maybe_head_element = self
            .elements
            .iter()
            .rev()
            .enumerate()
            .find(|(_, node)| predicate(node));
        if let Some((idx, _)) = maybe_head_element {
            self.elements.remove(idx);
        }
    }

    /// Ajoute un nouvel arbre de noeud dans le vecteur.
    pub fn put(&mut self, element: TreeNode<Node>) {
        self.elements.push(element);
    }

    // <https://github.com/rust-lang/rust/issues/83701>
    pub fn scoped_elements_with<const N: usize>(
        list: impl IntoIterator<Item = tag_names>,
    ) -> [tag_names; N] {
        let mut elements: [tag_names; N] = [tag_names::var; N];

        Self::SCOPE_ELEMENTS
            .into_iter()
            .enumerate()
            .for_each(|(i, t)| {
                elements[i] = t;
            });

        list.into_iter().enumerate().for_each(|(i, t)| {
            elements[i] = t;
        });

        elements
    }
}

impl ListOfActiveFormattingElements {
    pub fn clear_up_to_the_last_marker(&mut self) {
        while let Some(entry) = self.entries.pop() {
            if entry.is_marker() {
                break;
            }
        }
    }

    pub fn insert_marker_at_end(&mut self) {
        self.entries.push(Entry::Marker);
    }
}

impl Entry {
    pub fn is_element(&self) -> bool {
        matches!(self, Self::Element(_))
    }

    pub fn is_marker(&self) -> bool {
        matches!(self, Self::Marker)
    }

    pub fn element(&self) -> Option<&TreeNode<Node>> {
        match self {
            | Entry::Marker => None,
            | Entry::Element(node) => Some(node),
        }
    }

    pub fn element_unchecked(&self) -> &TreeNode<Node> {
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

impl ops::Deref for StackOfOpenElements {
    type Target = Vec<TreeNode<Node>>;

    fn deref(&self) -> &Self::Target {
        self.elements.as_ref()
    }
}

impl ops::DerefMut for StackOfOpenElements {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.elements.as_mut()
    }
}

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
