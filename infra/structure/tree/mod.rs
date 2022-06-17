/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod node;
mod weak;

use core::ops;
use std::rc::Rc;

use self::node::Node;
pub use self::weak::TreeNodeWeak;

// --------- //
// Structure //
// --------- //

/// Un arbre est une structure arborescente hiérarchique finie. L'ordre
/// d'un arbre est un pré-ordre, une traversée en profondeur d'un arbre.
#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub struct TreeNode<T> {
    node_ref: Rc<Node<T>>,
}

// -------------- //
// Implémentation //
// -------------- //

impl<T> TreeNode<T> {
    pub fn new(data: T) -> Self {
        Self {
            node_ref: Rc::new(Node::new(data)),
        }
    }

    fn new_node(rc: Rc<Node<T>>) -> Self {
        Self { node_ref: rc }
    }

    /// Pour ajouter un noeud à un parent, il faut pré-insérer le noeud
    /// dans le parent avant null.
    ///
    /// Pour pré-insérer un nœud dans un parent avant un enfant, il faut
    /// exécuter ces étapes :
    ///   - 1) Assurer la validité de la pré-insertion du nœud dans le
    ///     parent avant l'enfant.
    ///   - 2) Que referenceChild soit l'enfant.
    ///   - 3) Si referenceChild est un nœud, alors définir referenceChild
    ///     sur le prochain frère du nœud.
    ///   - 4) Insérer le nœud dans le parent avant la référenceChild.
    ///   - 5) Retourner le nœud.
    pub fn append_child(&self, node: impl Into<Self>) {
        let child: Self = node.into();

        assert!(child.parent.borrow().is_none());

        if let Some(last_node) = self.get_last_child().as_ref() {
            last_node.next_sibling.replace(child.to_owned().into());
            child
                .prev_sibling
                .replace(TreeNodeWeak::from(&child).into());
        }

        child.parent.replace(TreeNodeWeak::from(self).into());

        if self.get_first_child().is_none() {
            self.first_child.replace(child.to_owned().into());
        }

        self.last_child.replace(child.into());
    }

    pub fn detach_node(&self) {
        if let Some(prev_node) = self.prev_sibling() {
            prev_node.next_sibling.replace(self.next_sibling());
        }

        if let Some(next_node) = self.next_sibling() {
            next_node
                .prev_sibling
                .replace(self.prev_sibling.borrow().clone());
        }

        if let Some(parent) = self.parent_node() {
            let first_child =
                parent.get_first_child().expect("Le premier enfant");
            let last_child =
                parent.get_last_child().expect("Le dernier enfant");

            if Rc::ptr_eq(self, &first_child) {
                parent.first_child.replace(self.next_sibling());
            } else if Rc::ptr_eq(self, &last_child) {
                parent.last_child.replace(self.prev_sibling());
            }
        }

        self.parent.replace(Default::default());
        self.prev_sibling.replace(Default::default());
        self.next_sibling.replace(Default::default());
    }

    pub fn foreach_child<F>(&self, mut f: F)
    where
        F: FnMut(&Self),
    {
        let mut current_node = self.get_first_child();
        while let Some(node) = current_node {
            f(&node);
            current_node = node.next_sibling();
        }
    }

    /// Récupère le premier enfant de l'arbre.
    pub fn get_first_child(&self) -> Option<TreeNode<T>> {
        self.first_child.borrow().clone()
    }

    /// Récupère le dernier enfant de l'arbre.
    pub fn get_last_child(&self) -> Option<TreeNode<T>> {
        self.last_child.borrow().clone()
    }

    pub fn insert_before(&self, node: Self, maybe_child: Option<&Self>) {
        if maybe_child.is_none() {
            self.append_child(node);
            return;
        }

        assert!(node.parent.borrow().is_none());

        if let Some(child) = maybe_child {
            child.parent.replace(Some(self.into()));
            match child.prev_sibling() {
                | Some(prev_sibling) => {
                    prev_sibling
                        .next_sibling
                        .replace(child.to_owned().into());
                    child.prev_sibling.replace(Some(prev_sibling.into()));
                }
                | None => {
                    self.first_child.replace(child.to_owned().into());
                }
            }
        }
    }

    pub fn next_sibling(&self) -> Option<Self> {
        self.next_sibling.borrow().clone()
    }

    /// Un objet qui participe à un arbre a un parent, qui est soit null
    /// soit un objet.
    pub fn parent_node(&self) -> Option<Self> {
        self.parent.borrow().as_deref().and_then(|node_weak| {
            node_weak.upgrade().map(|node_ref| node_ref.into())
        })
    }

    /// Le frère précédent d'un objet est son premier frère précédent ou
    /// null s'il n'a pas de frère précédent.
    pub fn prev_sibling(&self) -> Option<Self> {
        self.prev_sibling.borrow().as_deref().and_then(|node_weak| {
            node_weak.upgrade().map(|node_ref| node_ref.into())
        })
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl<T> From<Rc<Node<T>>> for TreeNode<T> {
    fn from(rc: Rc<Node<T>>) -> Self {
        Self::new_node(rc)
    }
}

impl<T> Clone for TreeNode<T> {
    fn clone(&self) -> Self {
        Self::new_node(self.node_ref.clone())
    }
}

impl<T> ops::Deref for TreeNode<T> {
    type Target = Rc<Node<T>>;

    fn deref(&self) -> &Self::Target {
        &self.node_ref
    }
}
