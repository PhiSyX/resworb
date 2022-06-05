/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use core::mem;

// --------- //
// Interface //
// --------- //

pub trait ParameterType {
    type Target;
}

pub trait Parameter<'a, T: ParameterType> {
    fn param(self) -> CowParameter<'a, T>;
}

// ----------- //
// Énumération //
// ----------- //

pub enum CowParameter<'a, T> {
    Borrowed(&'a T),
    Owned(T),
    None,
}

// -------------- //
// Implémentation //
// -------------- //

impl<'a, T: ParameterType> CowParameter<'a, T> {
    /// # Safety
    ///
    /// Retourne une `Option<T>` qui peut être `None` si la valeur
    /// `CowParameter` est `None`.
    pub unsafe fn value(&self) -> Option<T> {
        match self {
            | Self::Borrowed(value) => Some(mem::transmute_copy(value)),
            | Self::Owned(value) => Some(mem::transmute_copy(value)),
            | Self::None => None,
        }
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl<'a, T: ParameterType> Parameter<'a, T> for T {
    fn param(self) -> CowParameter<'a, T> {
        CowParameter::Owned(self)
    }
}

impl<'a, T: ParameterType> Parameter<'a, T> for &'a T {
    fn param(self) -> CowParameter<'a, T> {
        CowParameter::Borrowed(self)
    }
}

impl<'a, T: ParameterType> Parameter<'a, T> for Option<T> {
    fn param(self) -> CowParameter<'a, T> {
        match self {
            | Some(value) => CowParameter::Owned(value),
            | None => CowParameter::None,
        }
    }
}

impl<'a, T: ParameterType> Parameter<'a, T> for &'a Option<T> {
    fn param(self) -> CowParameter<'a, T> {
        match self {
            | Some(value) => CowParameter::Borrowed(value),
            | None => CowParameter::None,
        }
    }
}

impl<T> ParameterType for *mut T {
    type Target = Self;
}

impl<T> ParameterType for *const T {
    type Target = Self;
}

// todo: ajouter une implémentation pour des types génériques.
// impl<T: GenericInterface> ParameterType for T {
//     type Target = Self;
// }

#[cfg(test)]
mod tests {
    use super::*;

    struct MyStructSize {
        height: u32,
        width: u32,
    }

    impl ParameterType for MyStructSize {
        type Target = Self;
    }

    fn my_fn<'a, Size: Parameter<'a, MyStructSize>>(
        size: Size,
    ) -> (u32, u32) {
        let size: Option<MyStructSize> = unsafe { size.param().value() };
        match size {
            | Some(size) => (size.height, size.width),
            | None => (0, 0),
        }
    }

    #[test]
    fn example() {
        let struct_size = MyStructSize {
            height: 10,
            width: 20,
        };
        let size = my_fn(struct_size);
        assert_eq!(size, (10, 20));

        let size = my_fn(None);
        assert_eq!(size, (0, 0));
    }
}
