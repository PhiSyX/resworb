/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// ----------- //
// Énumération //
// ----------- //

#[derive(Debug)]
#[derive(PartialEq)]
pub(crate) enum ScriptingFlag {
    Enabled = 1,
    Disabled = 0,
}

#[derive(Debug)]
#[derive(PartialEq)]
pub(crate) enum FramesetOkFlag {
    Ok = 1,
    NotOk = 0,
}

// -------------- //
// Implémentation //
// -------------- //

impl Default for ScriptingFlag {
    fn default() -> Self {
        ScriptingFlag::Enabled
    }
}

impl Default for FramesetOkFlag {
    fn default() -> Self {
        FramesetOkFlag::Ok
    }
}
