/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod node;
mod weak;

use std::{cell::Ref, ops, rc::Rc};

pub use weak::TreeNodeWeak;

use self::node::Node;

// --------- //
// Structure //
// --------- //

/// Un arbre est une structure arborescente hiérarchique finie. L'ordre
/// d'un arbre est un pré-ordre, une traversée en profondeur d'un arbre.
#[derive(Debug)]
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
            last_node
                .next_sibling
                .borrow_mut()
                .replace(child.to_owned());

            child
                .prev_sibling
                .replace(TreeNodeWeak::from(&child).into());
        }

        child.parent.borrow_mut().replace(TreeNodeWeak::from(self));

        if self.get_first_child().is_none() {
            self.first_child.replace(child.to_owned().into());
        }

        self.last_child.replace(child.into());
    }

    pub fn get_first_child(&self) -> Ref<'_, Option<TreeNode<T>>> {
        self.first_child.borrow()
    }

    pub fn get_last_child(&self) -> Ref<'_, Option<TreeNode<T>>> {
        self.last_child.borrow()
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

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
