/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use core::ops;
use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
};

use super::{TreeNode, TreeNodeWeak};

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
pub struct Node<T> {
    data: T,

    pub(super) parent: RefCell<Option<TreeNodeWeak<T>>>,

    pub(super) first_child: RefCell<Option<TreeNode<T>>>,
    pub(super) last_child: RefCell<Option<TreeNode<T>>>,

    pub(super) prev_sibling: RefCell<Option<TreeNodeWeak<T>>>,
    pub(super) next_sibling: RefCell<Option<TreeNode<T>>>,
}

// ----------- //
// Énumération //
// ----------- //

// -------------- //
// Implémentation //
// -------------- //

impl<T> Node<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,

            parent: Default::default(),

            first_child: Default::default(),
            last_child: Default::default(),

            prev_sibling: Default::default(),
            next_sibling: Default::default(),
        }
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl<T> ops::Deref for Node<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data.borrow()
    }
}

impl<T> ops::DerefMut for Node<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data.borrow_mut()
    }
}

impl<T: PartialEq> PartialEq for Node<T> {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<T: Eq> Eq for Node<T> {}
