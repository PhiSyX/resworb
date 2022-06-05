/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub mod api;
#[cfg(target_os = "windows")]
mod win;

use core::ops;

#[cfg(target_os = "windows")]
use win as platform;

use self::api::{WindowAPI, WindowOptions};

// --------- //
// Structure //
// --------- //

pub struct Window(platform::Window);

// -------------- //
// Implémentation // -> API
// -------------- //

impl WindowAPI for Window {
    type Window = Self;

    fn new(options: WindowOptions) -> Self {
        Self(platform::Window::new(options))
    }

    fn spawn_forever(&self) {
        self.0.spawn_forever()
    }

    fn tick(&self) {
        self.0.tick()
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl ops::Deref for Window {
    type Target = platform::Window;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for Window {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
