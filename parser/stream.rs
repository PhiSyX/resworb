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
            current_input: Default::default(),
            is_replayed: Default::default(),
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_iter<O>(iter: O) -> Self
    where
        O: Iterator<Item = I>,
    {
        let mut list = Vec::new();

        for item in iter {
            list.push(item);
        }

        Self {
            list,
            current_input: Default::default(),
            is_replayed: Default::default(),
        }
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl<I> StreamIteratorInterface for TokenStream<I>
where
    I: StreamInputInterface,
{
    type Input = I;

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

        while (self.next_input().is_some()
            && predicate(self.next_input().as_ref().unwrap()))
            && (limit > 0 || with_limit.is_none())
        {
            result.push(self.consume_next_input().unwrap());
            if with_limit.is_some() {
                limit -= 1;
            }
        }

        result
    }

    fn consume_next_input(&mut self) -> Option<Self::Input> {
        if self.is_replayed {
            self.is_replayed = false;
            return self.current_input.clone();
        }

        if self.list.is_empty() {
            self.current_input = Some(Self::Input::eof());
        } else {
            self.current_input = Some(self.list.remove(0));
        }

        self.current_input.clone()
    }

    fn current_input(&self) -> Option<&Self::Input> {
        self.current_input.as_ref()
    }

    fn next_input(&mut self) -> Option<Self::Input> {
        if self.list.is_empty() {
            return Some(Self::Input::eof());
        }
        self.list.iter().peekable().next().cloned()
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
