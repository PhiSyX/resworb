/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![feature(type_name_of_val)]

#[macro_export]
macro_rules! dd {
    ($expr: expr) => {{
        if cfg!(debug_assertions) {
            println!(
                "{} = {}::{:?}",
                stringify!($expr),
                std::any::type_name_of_val($expr)
                    .replace("core::option::", "")
                    .replace("char", "CodePoint")
                    .replace("resworb_html_parser::", ""),
                $expr
            );
        }
        $expr
    }};
}
