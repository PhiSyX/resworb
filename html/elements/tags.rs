/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/// Espaces de noms pour les balises HTML.
///! Liste des attributes de balises HTML.
mod attributes;
///! Liste des noms de balises HTML.
mod names;

pub use attributes::tag_attributes;
pub use names::tag_names;
