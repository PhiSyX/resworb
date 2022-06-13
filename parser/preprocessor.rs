/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use core::ops;
use std::borrow::Cow;

use infra::{
    algorithms::Parameter,
    primitive::codepoint::CodePointIterator,
    structure::lists::{peekable::PeekableInterface, queue::ListQueue},
};

use crate::{StreamInputInterface, StreamIteratorInterface};

// ---- //
// Type //
// ---- //

// NOTE(phisyx): ces types peuvent être amélioré.
type InputStreamCurrentInput<I> = Option<I>;
type InputStreamFilteredInput<I> = Option<I>;
type InputStreamPreScanFn<I> =
    fn(InputStreamCurrentInput<I>) -> InputStreamFilteredInput<I>;

// --------- //
// Structure //
// --------- //

/// Le flux d'entrée est constitué de caractères qui y sont insérés lors
/// du décodage du flux d'octets d'entrée ou par les diverses API qui
/// manipulent directement le flux d'entrée.
#[derive(Debug)]
pub struct InputStreamPreprocessor<Stream, Input> {
    queue: ListQueue<Stream, Input>,
    pub current_input: InputStreamCurrentInput<Input>,
    pre_scan: Option<InputStreamPreScanFn<Input>>,
}

// -------------- //
// Implémentation //
// -------------- //

impl<S, I> InputStreamPreprocessor<S, I> {
    /// Crée un nouveau flux d'entrée.
    pub fn new(stream: S) -> Self {
        Self {
            queue: ListQueue::new(stream),
            current_input: None,
            // NOTE(phisyx): par défaut, nous n'avons pas besoin
            // d'effectuer de filtre particulier.
            pre_scan: None,
        }
    }

    pub fn with_pre_scan(
        mut self,
        scan_fn: InputStreamPreScanFn<I>,
    ) -> Self {
        self.pre_scan.replace(scan_fn);
        self
    }
}

impl<Chars> InputStreamPreprocessor<Chars, Chars::Item>
where
    Chars: CodePointIterator,
    Chars::Item: StreamInputInterface,
{
    /// Alias de [StreamIteratorInterface::consume_next_input].
    //
    // NOTE(phisyx): Nomenclature de la spécification HTML.
    pub fn consume_next_input_character(&mut self) -> Option<Chars::Item> {
        self.consume_next_input()
    }

    /// Alias de [StreamIteratorInterface::next_input].
    //
    // NOTE(phisyx): Nomenclature de la spécification HTML.
    pub fn next_input_character(&mut self) -> Option<Chars::Item> {
        self.next_input()
    }

    /// Alias de [StreamIteratorInterface::next_n_input], mais renvoie une
    /// chaîne de caractères au lieu d'un tableau.
    //
    // NOTE(phisyx): Nomenclature de la spécification HTML.
    pub fn next_n_input_character(&mut self, n: usize) -> Cow<str> {
        self.next_n_input(n).into_iter().collect()
    }

    /// Consomme les prochains caractères du flux d'entrée qui sont
    /// identiques à l'argument `codepoints`.
    pub fn consume_next_input_character_if_are<
        const SIZE_OF_CODE_POINTS: usize,
    >(
        &mut self,
        codepoints: [Chars::Item; SIZE_OF_CODE_POINTS],
    ) -> bool {
        if self.next_n_input(SIZE_OF_CODE_POINTS) == codepoints {
            self.advance(SIZE_OF_CODE_POINTS);
            true
        } else {
            false
        }
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl<T, I> StreamIteratorInterface for InputStreamPreprocessor<T, I>
where
    T: Iterator<Item = I>,
    I: Clone,
    I: StreamInputInterface,
{
    type Input = I;

    fn advance(&mut self, mut n: usize) -> Option<Self::Input> {
        if n >= 1 {
            n -= 1;
        }
        self.nth(n)
    }

    fn advance_as_long_as_possible<
        'a,
        Predicate: Fn(&Self::Input) -> bool,
        Limit: Parameter<'a, usize>,
    >(
        &mut self,
        predicate: Predicate,
        with_limit: Limit,
    ) -> Vec<Self::Input> {
        let with_limit = unsafe { with_limit.param().value() };
        let mut limit = with_limit.map(|n| n + 1).unwrap_or(0);
        let mut result = vec![];

        while (self.peek().is_some() && predicate(self.peek().unwrap()))
            && (limit > 0 || with_limit.is_none())
        {
            result.push(self.advance(1).unwrap());
            if with_limit.is_some() {
                limit -= 1;
            }
        }
        result
    }

    fn consume_next_input(&mut self) -> Option<Self::Input> {
        if let Some(pre_scan) = &self.pre_scan {
            (pre_scan)(self.queue.next())
        } else {
            self.queue.next()
        }
        .and_then(|item| {
            let some_item = Some(item);
            self.current_input = some_item.to_owned();
            some_item
        })
    }

    fn current_input(&self) -> Option<&Self::Input> {
        self.current_input.as_ref()
    }

    fn next_input(&mut self) -> Option<Self::Input> {
        let item = self.peek().cloned();
        if let Some(pre_scan) = &self.pre_scan {
            (pre_scan)(item)
        } else {
            item
        }
    }

    fn next_n_input(&mut self, n: usize) -> Vec<Self::Input> {
        self.queue.peek_until(n).unwrap_or_default()
    }

    fn reconsume_current_input(&mut self) {
        let cloned_current_input = self.current_input.clone();
        self.reconsume(cloned_current_input);
    }
}

impl<T, I> ops::Deref for InputStreamPreprocessor<T, I> {
    type Target = ListQueue<T, I>;

    fn deref(&self) -> &Self::Target {
        &self.queue
    }
}

impl<T, I> ops::DerefMut for InputStreamPreprocessor<T, I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.queue
    }
}

// ---- //
// Test //
// ---- //

#[cfg(test)]
mod tests {
    use infra::primitive::codepoint::CodePoint;

    use super::*;

    fn get_input_stream(
        input: &'static str,
    ) -> InputStreamPreprocessor<impl CodePointIterator, CodePoint> {
        InputStreamPreprocessor::new(input.chars())
    }

    #[test]
    fn test_next_n_input_character() {
        let mut stream = get_input_stream("Hello World");
        assert_eq!(
            stream.next_n_input_character(5),
            Cow::Borrowed("Hello")
        );
        assert_eq!(stream.consume_next_input(), Some('H'));
    }

    #[test]
    fn test_reconsume() {
        let mut stream = get_input_stream("Hello World !");
        stream.consume_next_input(); // H
        stream.consume_next_input(); // e
        stream.reconsume_current_input(); // H
    }
}
