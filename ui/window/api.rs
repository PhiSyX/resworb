/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use core::marker::PhantomData;

// --------- //
// Interface //
// --------- //

pub trait WindowAPI: Sized {
    type Window: WindowAPI;

    /// Crée une fenêtre à partir d'[options](WindowOptions).
    fn new(options: WindowOptions) -> Self;

    /// Constructeur de fenêtre.
    fn builder() -> WindowBuilder<Self> {
        WindowBuilder::new()
    }

    /// Ouvre la fenêtre, dans un nouveau thread (^1), jusqu'à ce qu'un
    /// événement d'arrêt soit reçu.
    ///
    /// TODO(^1): construire un système de thread pour pouvoir ouvrir
    ///           plusieurs fenêtres en même temps.
    fn spawn_forever(&self);

    fn tick(&self);
}

// --------- //
// Structure //
// --------- //

/// Options générales d'une fenêtre.
#[non_exhaustive]
pub struct WindowOptions {
    pub cname: String,
    pub title: String,
    pub width: i32,
    pub height: i32,
    // TODO(phisyx): ajouter des options de style.
    pub style: WindowOptionsStyle,
}

/// Options de style d'une fenêtre.
#[non_exhaustive]
pub struct WindowOptionsStyle {
    pub resizable: bool,
}

#[derive(Default)]
pub struct WindowBuilder<W> {
    cname: String,
    title: String,
    width: i32,
    height: i32,
    style: WindowOptionsStyle,
    _marker: PhantomData<W>,
}

// -------------- //
// Implémentation //
// -------------- //

impl<Window: WindowAPI> WindowBuilder<Window> {
    pub fn new() -> Self {
        Self {
            cname: Default::default(),
            title: Default::default(),
            width: Default::default(),
            height: Default::default(),
            style: Default::default(),
            _marker: Default::default(),
        }
    }

    pub fn with_title(mut self, title: impl ToString) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn define_width(mut self, width: i32) -> Self {
        self.width = width;
        self
    }

    pub fn define_height(mut self, height: i32) -> Self {
        self.height = height;
        self
    }

    pub fn define_size(mut self, width: i32, height: i32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn define_cname(mut self, cname: impl ToString) -> Self {
        self.cname = cname.to_string();
        self
    }

    pub fn resizable(mut self) -> Self {
        self.style.resizable = true;
        self
    }

    pub fn build(self) -> Window {
        let style = WindowOptionsStyle {
            resizable: self.style.resizable,
        };

        let options = WindowOptions {
            cname: self.cname,
            title: self.title,
            width: self.width,
            height: self.height,
            style,
        };
        Window::new(options)
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl Default for WindowOptionsStyle {
    fn default() -> Self {
        Self { resizable: false }
    }
}
