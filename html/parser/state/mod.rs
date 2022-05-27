/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/// 13.2.4.1 The insertion mode
mod insertion_mode;

/// 13.2.4.2 The stack of open elements
mod stack_of_open_elements;

/// 13.2.4.3 The list of active formatting elements
mod list_of_active_formatting_elements;

/// 13.2.4.4 The element pointers
mod element_pointers;

/// 13.2.4.5 Other parsing state flags
mod flags;

pub(crate) use self::{
    element_pointers::{FormElementPointer, HeadElementPointer},
    flags::{FramesetOkFlag, ScriptingFlag},
    insertion_mode::InsertionMode,
    list_of_active_formatting_elements::{
        Entry, ListOfActiveFormattingElements,
    },
    stack_of_open_elements::StackOfOpenElements,
};
