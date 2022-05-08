/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use core::{borrow::Borrow, ops};
use std::{borrow::BorrowMut, cell::RefCell};

use super::{TreeNode, TreeNodeWeak};

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
pub struct Node<T> {
    data: T,

    pub(crate) parent: RefCell<Option<TreeNodeWeak<T>>>,

    pub(crate) first_child: RefCell<Option<TreeNode<T>>>,
    pub(crate) last_child: RefCell<Option<TreeNode<T>>>,

    pub(crate) prev_sibling: RefCell<Option<TreeNodeWeak<T>>>,
    pub(crate) next_sibling: RefCell<Option<TreeNode<T>>>,
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
