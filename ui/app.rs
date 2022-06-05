/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use gfx::window::{api::WindowAPI, Window};

// --------- //
// Structure //
// --------- //

pub struct App {
    window: Window,
}

// -------------- //
// Implémentation //
// -------------- //

impl App {
    const WINDOW_CNAME: &'static str = "resworb";
    const WINDOW_TITLE_BAR: &'static str = "RESWORB";

    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let window = Window::builder()
            .define_cname(Self::WINDOW_CNAME)
            .with_title(Self::WINDOW_TITLE_BAR)
            .build();
        Self { window }
    }

    pub fn launch(&self) {
        self.window.spawn_forever();
    }
}
