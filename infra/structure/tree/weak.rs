/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use core::ops;
use std::sync::{Arc, Weak};

use super::{node::Node, TreeNode};

// ---- //
// Type //
// ---- //

type WeakNode<T> = Weak<Node<T>>;

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
pub struct TreeNodeWeak<T> {
    node_weak: WeakNode<T>,
}

// -------------- //
// Implémentation //
// -------------- //

impl<T> TreeNodeWeak<T> {
    pub(super) fn new(node_weak: WeakNode<T>) -> Self {
        Self { node_weak }
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl<T> Clone for TreeNodeWeak<T> {
    fn clone(&self) -> Self {
        Self::new(self.node_weak.clone())
    }
}

impl<T> ops::Deref for TreeNodeWeak<T> {
    type Target = WeakNode<T>;

    fn deref(&self) -> &Self::Target {
        &self.node_weak
    }
}

impl<T> From<&TreeNode<T>> for TreeNodeWeak<T> {
    fn from(arc: &TreeNode<T>) -> Self {
        Self::new(Arc::downgrade(arc))
    }
}

impl<T> From<TreeNode<T>> for TreeNodeWeak<T> {
    fn from(arc: TreeNode<T>) -> Self {
        Self::new(Arc::downgrade(&arc))
    }
}
