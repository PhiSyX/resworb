/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{
    mem, ptr,
    sync::atomic::{AtomicBool, Ordering},
};

use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Input::KeyboardAndMouse::{VIRTUAL_KEY, VK_ESCAPE},
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DispatchMessageA,
                GetWindowLongPtrW, PeekMessageW, RegisterClassW,
                SetWindowLongPtrW, TranslateMessage, CS_HREDRAW,
                CS_VREDRAW, CW_USEDEFAULT, GWLP_USERDATA, MSG, PM_REMOVE,
                WINDOW_EX_STYLE, WINDOW_STYLE, WM_ACTIVATEAPP, WM_CLOSE,
                WM_DESTROY, WM_KEYDOWN, WM_KEYUP, WM_QUIT, WM_SIZE,
                WM_SYSKEYDOWN, WM_SYSKEYUP, WNDCLASSW, WNDCLASS_STYLES,
                WS_MINIMIZEBOX, WS_OVERLAPPED, WS_OVERLAPPEDWINDOW,
                WS_SYSMENU, WS_VISIBLE,
            },
        },
    },
};

use crate::api::{WindowAPI, WindowOptions};

// ----- //
// Macro //
// ----- //

macro_rules! pcwstr {
    ($str:expr) => {{
        use std::os::windows::ffi::OsStrExt;
        PCWSTR(
            (std::ffi::OsStr::new($str)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect::<Vec<_>>())
            .as_ptr(),
        )
    }};
}

// --------- //
// Interface //
// --------- //

pub trait WindowProcInterface {
    unsafe fn window_proc(
        &self,
        handle_window: HWND,
        message: u32,
        word_param: WPARAM,
        long_param: LPARAM,
    ) -> LRESULT;
}

// --------- //
// Structure //
// --------- //

pub struct Window {
    handle: HWND,
    proc: WindowProc,
}

#[derive(Debug)]
pub struct WindowProc {
    pub running: AtomicBool,
}

// -------------- //
// Implémentation //
// -------------- //

impl Window {
    /// Crée une fenêtre.
    fn create_window(options: WindowOptions) -> HWND {
        let hwnd_instance = unsafe { GetModuleHandleW(None) }
            .expect("L'instance de la fenêtre du module");

        let class_name = pcwstr!(&options.cname);
        let window_name = pcwstr!(&options.title);

        let window_class =
            Self::build_window_class(hwnd_instance, class_name);

        let atom = unsafe { RegisterClassW(&window_class) };
        debug_assert!(
            atom != 0,
            "Impossible d'enregistrer la classe WNDCLASSW."
        );

        // TODO(phisyx): générer le style de la fenêtre en fonction des
        //               options.
        let ex_style = WINDOW_EX_STYLE::default();
        let mut style = WINDOW_STYLE::default();

        style |= WS_VISIBLE;

        if options.style.resizable {
            style |= WS_OVERLAPPEDWINDOW;
        } else {
            style |= WS_OVERLAPPED;
            style |= WS_SYSMENU;
            style |= WS_MINIMIZEBOX;
        }

        let width = if options.width < 500 {
            CW_USEDEFAULT
        } else {
            options.width
        };

        let height = if options.height < 500 {
            CW_USEDEFAULT
        } else {
            options.height
        };

        unsafe {
            CreateWindowExW(
                ex_style,
                class_name,
                window_name,
                style,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                width,
                height,
                None,
                None,
                hwnd_instance,
                ptr::null_mut(),
            )
        }
    }

    /// Enregistre une classe de fenêtre pour une utilisation ultérieure
    /// dans les appels à la fonction `CreateWindowEx?`.
    fn build_window_class(
        handle_instance: HINSTANCE,
        class_name: PCWSTR,
    ) -> WNDCLASSW {
        let mut style = WNDCLASS_STYLES::default();
        style |= CS_HREDRAW;
        style |= CS_VREDRAW;

        WNDCLASSW {
            style,
            lpfnWndProc: Some(Self::window_proc_callback),
            hInstance: handle_instance,
            lpszClassName: class_name,
            ..Default::default()
        }
    }
}

impl Window {
    /// Fonction de retour, qui traite les messages envoyés à la fenêtre.
    unsafe extern "system" fn window_proc_callback(
        handle_window: HWND,
        message: u32,
        word_param: WPARAM,
        long_param: LPARAM,
    ) -> LRESULT {
        let win_proc = GetWindowLongPtrW(handle_window, GWLP_USERDATA);
        let window_proc_ptr = win_proc as *const Self;

        if window_proc_ptr.is_null() {
            DefWindowProcW(handle_window, message, word_param, long_param)
        } else {
            let window = &*window_proc_ptr;
            window.proc.window_proc(
                handle_window,
                message,
                word_param,
                long_param,
            )
        }
    }
}

// &Window
impl Window {
    /// Est-ce que la fenêtre est ouverte ?
    #[inline]
    pub fn is_running(&self) -> bool {
        self.proc.running.load(Ordering::Relaxed)
    }

    /// Distribue les messages entrants non mis en file d'attente par l'API
    /// Windows.
    unsafe fn peek_message(&self) {
        let mut peek_msg = MSG::default();

        while PeekMessageW(&mut peek_msg, self.handle, 0, 0, PM_REMOVE)
            .into()
        {
            if peek_msg.message == WM_QUIT {
                self.proc.running.swap(false, Ordering::Relaxed);
            }

            TranslateMessage(&peek_msg);
            DispatchMessageA(&peek_msg);
        }
    }
}

// -------------- //
// Implémentation // -> API
// -------------- //

impl WindowAPI for Window {
    type Window = Self;

    /// Crée une nouvelle fenêtre pour la plateforme Windows.
    fn new(options: WindowOptions) -> Self {
        let handle_window = Self::create_window(options);
        let window_proc = WindowProc::default();
        Self {
            handle: handle_window,
            proc: window_proc,
        }
    }

    fn spawn_forever(&self) {
        while self.is_running() {
            self.tick()
        }
    }

    fn tick(&self) {
        unsafe {
            self.peek_message();

            SetWindowLongPtrW(
                self.handle,
                GWLP_USERDATA,
                mem::transmute(self),
            );
        }
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl Default for WindowProc {
    fn default() -> Self {
        Self {
            running: AtomicBool::new(true),
        }
    }
}

impl WindowProcInterface for WindowProc {
    unsafe fn window_proc(
        &self,
        handle_window: HWND,
        message: u32,
        word_param: WPARAM,
        long_param: LPARAM,
    ) -> LRESULT {
        // let window_ptr = GetWindowLongPtrW(handle_window, GWLP_USERDATA)
        //     as *const Window;
        // let window = &*window_ptr;

        match message {
            | WM_DESTROY | WM_CLOSE => {
                self.running.swap(false, Ordering::Relaxed);
            }

            | WM_SIZE => {
                println!("WM_SIZE");
            }

            | WM_ACTIVATEAPP => {
                println!("WM_ACTIVATEAPP");
            }

            | WM_SYSKEYDOWN | WM_KEYDOWN => {
                let data = word_param.0;
                println!("KEYDOWN event codepoint: '{data}'");
                if VK_ESCAPE == VIRTUAL_KEY(data as u16) {
                    println!(" -> ESCAPE == {data} : close window");
                    self.running.swap(false, Ordering::Relaxed);
                }
            }

            | WM_SYSKEYUP | WM_KEYUP => {
                let data = word_param.0;
                println!("KEYUP event codepoint: {}", data);
            }

            | _ => {}
        }

        DefWindowProcW(handle_window, message, word_param, long_param)
    }
}
