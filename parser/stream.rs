/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use infra::primitive::codepoint::CodePoint;

use crate::{
    interface::{StreamInputInterface, StreamIteratorInterface},
    preprocessor::InputStreamPreprocessor,
};

// ---- //
// Type //
// ---- //

pub type InputStream<T, I> = InputStreamPreprocessor<T, I>;

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
pub struct TokenStream<Item> {
    list: Vec<Item>,
    current_input: Option<Item>,
    is_replayed: bool,
}

// -------------- //
// Implémentation //
// -------------- //

impl<I> TokenStream<I>
where
    I: StreamInputInterface,
{
    pub fn new<O>(mut stream: O) -> Self
    where
        O: StreamIteratorInterface<Input = I>,
    {
        let mut list = Vec::new();

        loop {
            match stream.consume_next_input() {
                | Some(token) if !token.is_eof() => list.push(token),
                | _ => break,
            }
        }

        Self {
            list,
            current_input: None,
            is_replayed: false,
        }
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl<Input> StreamIteratorInterface for TokenStream<Input>
where
    Input: StreamInputInterface,
{
    type Input = Input;

    fn advance_as_long_as_possible<
        'a,
        Predicate: Fn(&Self::Input) -> bool,
        Limit: infra::algorithms::Parameter<'a, usize>,
    >(
        &mut self,
        predicate: Predicate,
        with_limit: Limit,
    ) -> Vec<Self::Input> {
        let with_limit = unsafe { with_limit.param().value() };
        let mut limit = with_limit.map(|n| n + 1).unwrap_or(0);
        let mut result = vec![];

        while let Some(token) = self.consume_next_input() {
            if predicate(&token) && (limit > 0 || with_limit.is_none()) {
                result.push(token);
                if with_limit.is_some() {
                    limit -= 1;
                }
            } else {
                self.reconsume_current_input();
                break;
            }
        }

        result
    }

    fn consume_next_input(&mut self) -> Option<Self::Input> {
        if self.is_replayed {
            self.is_replayed = false;
            return self.current_input.clone();
        }

        self.current_input = self.next_input();
        self.current_input.clone()
    }

    fn current_input(&self) -> Option<&Self::Input> {
        self.current_input.as_ref()
    }

    fn next_input(&mut self) -> Option<Self::Input> {
        if self.list.is_empty() {
            return Some(Self::Input::eof());
        }
        Some(self.list.remove(0))
    }

    fn reconsume_current_input(&mut self) {
        self.is_replayed = true;
    }
}

impl StreamInputInterface for CodePoint {
    fn eof() -> Self {
        '\0'
    }
}
