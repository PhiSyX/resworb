/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// ----------- //
// Énumération //
// ----------- //

#[derive(Debug)]
#[derive(Default)]
#[derive(PartialEq, Eq)]
pub(crate) enum ScriptingFlag {
    #[default]
    Enabled = 1,
    Disabled = 0,
}

#[derive(Debug)]
#[derive(Default)]
#[derive(PartialEq, Eq)]
pub(crate) enum FramesetOkFlag {
    #[default]
    Ok = 1,
    NotOk = 0,
}
