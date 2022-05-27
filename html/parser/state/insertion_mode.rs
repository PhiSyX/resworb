/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// ----------- //
// Énumération //
// ----------- //

/// Le mode d'insertion est une variable d'état qui contrôle l'opération
/// primaire de l'étape de construction de l'arbre.  Le mode d'insertion
/// affecte la manière dont les tokens sont traités et si les sections
/// CDATA sont supportées.
#[derive(Debug)]
#[derive(Copy, Clone)]
#[derive(PartialEq)]
pub(crate) enum InsertionMode {
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

// -------------- //
// Implémentation //
// -------------- //

impl InsertionMode {
    pub(crate) fn switch_to(&mut self, mode: Self) {
        *self = mode;
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl Default for InsertionMode {
    /// Initialement, le mode d'insertion est "initial".
    fn default() -> Self {
        Self::Initial
    }
}
