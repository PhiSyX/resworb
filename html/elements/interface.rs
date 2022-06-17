/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{tag_attributes, tag_names};

// --------- //
// Interface //
// --------- //

pub trait HTMLElementInterface {
    fn tag_name(&self) -> String;
}

pub trait IsOneOfTagsInterface: Copy {
    fn is_one_of(self, arr: impl IntoIterator<Item = tag_names>) -> bool;
}

pub trait IsOneOfAttributesInterface: Copy {
    fn is_one_of(
        self,
        arr: impl IntoIterator<Item = tag_attributes>,
    ) -> bool;
}
