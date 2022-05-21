/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod node;
mod weak;

use core::ops;
use std::sync::{Arc, RwLockReadGuard};

use self::node::Node;
pub use self::weak::TreeNodeWeak;

// --------- //
// Structure //
// --------- //

/// Un arbre est une structure arborescente hiérarchique finie. L'ordre
/// d'un arbre est un pré-ordre, une traversée en profondeur d'un arbre.
#[derive(Debug)]
#[derive(PartialEq)]
pub struct TreeNode<T> {
    node_ref: Arc<Node<T>>,
}

// -------------- //
// Implémentation //
// -------------- //

impl<T> TreeNode<T> {
    pub fn new(data: T) -> Self {
        Self {
            node_ref: Arc::new(Node::new(data)),
        }
    }

    fn new_node(arc: Arc<Node<T>>) -> Self {
        Self { node_ref: arc }
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

        assert!(child.parent.read().unwrap().is_none());

        if let Some(last_node) = self.get_last_child().as_ref() {
            last_node
                .next_sibling
                .write()
                .unwrap()
                .replace(child.to_owned());

            child
                .prev_sibling
                .write()
                .unwrap()
                .replace(TreeNodeWeak::from(&child));
        }

        child
            .parent
            .write()
            .unwrap()
            .replace(TreeNodeWeak::from(self));

        if self.get_first_child().is_none() {
            self.first_child.write().unwrap().replace(child.to_owned());
        }

        self.last_child.write().unwrap().replace(child);
    }

    pub fn detach_node(&self) {
        if let Some(prev_node) = self.previous_sibling() {
            prev_node
                .next_sibling
                .write()
                .unwrap()
                .replace(self.next_sibling().unwrap());
        }

        if let Some(next_node) = self.next_sibling() {
            *next_node.prev_sibling.write().unwrap() =
                self.prev_sibling.write().unwrap().to_owned();
        }

        if let Some(parent) = self.parent_node() {
            let first_child = parent
                .get_first_child()
                .to_owned()
                .expect("Le premier enfant");
            let last_child = parent
                .get_last_child()
                .to_owned()
                .expect("Le dernier enfant");

            if Arc::ptr_eq(self, &first_child) {
                parent
                    .first_child
                    .write()
                    .unwrap()
                    .replace(self.next_sibling().unwrap());
            } else if Arc::ptr_eq(self, &last_child) {
                parent
                    .last_child
                    .write()
                    .unwrap()
                    .replace(self.previous_sibling().unwrap());
            }
        }

        *self.parent.write().unwrap() = None;
        *self.prev_sibling.write().unwrap() = None;
        *self.next_sibling.write().unwrap() = None;
    }

    pub fn foreach_child<F>(&self, mut f: F)
    where
        F: FnMut(&Self),
    {
        let mut current_node = self.get_first_child().to_owned();
        while let Some(node) = current_node {
            f(&node);
            current_node = node.next_sibling().to_owned();
        }
    }

    /// Récupère le premier enfant de l'arbre.
    pub fn get_first_child(&self) -> RwLockReadGuard<Option<TreeNode<T>>> {
        self.first_child.read().unwrap()
    }

    /// Récupère le dernier enfant de l'arbre.
    pub fn get_last_child(&self) -> RwLockReadGuard<Option<TreeNode<T>>> {
        self.last_child.read().unwrap()
    }

    pub fn insert_before(&self, node: Self, maybe_child: Option<&Self>) {
        if maybe_child.is_none() {
            self.append_child(node);
            return;
        }

        assert!(node.parent.read().unwrap().is_none());

        if let Some(child) = maybe_child {
            child.parent.write().unwrap().replace(self.into());
            match child.previous_sibling() {
                | Some(prev_sibling) => {
                    prev_sibling
                        .next_sibling
                        .write()
                        .unwrap()
                        .replace(child.to_owned());
                    child
                        .prev_sibling
                        .write()
                        .unwrap()
                        .replace(prev_sibling.into());
                }
                | None => {
                    self.first_child
                        .write()
                        .unwrap()
                        .replace(child.to_owned());
                }
            }
        }
    }

    pub fn next_sibling(&self) -> Option<Self> {
        self.next_sibling.read().unwrap().to_owned()
    }

    /// Un objet qui participe à un arbre a un parent, qui est soit null
    /// soit un objet.
    pub fn parent_node(&self) -> Option<Self> {
        self.parent
            .read()
            .unwrap()
            .as_deref()
            .and_then(|node_weak| {
                node_weak.upgrade().map(|node_ref| node_ref.into())
            })
    }

    /// Le frère précédent d'un objet est son premier frère précédent ou
    /// null s'il n'a pas de frère précédent.
    pub fn previous_sibling(&self) -> Option<Self> {
        self.prev_sibling.read().unwrap().as_deref().and_then(
            |node_weak| {
                node_weak.upgrade().map(|node_ref| node_ref.into())
            },
        )
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl<T> From<Arc<Node<T>>> for TreeNode<T> {
    fn from(rc: Arc<Node<T>>) -> Self {
        Self::new_node(rc)
    }
}

impl<T> Clone for TreeNode<T> {
    fn clone(&self) -> Self {
        Self::new_node(self.node_ref.clone())
    }
}

impl<T> ops::Deref for TreeNode<T> {
    type Target = Arc<Node<T>>;

    fn deref(&self) -> &Self::Target {
        &self.node_ref
    }
}
