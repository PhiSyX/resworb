/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;

use infra::{
    primitive::codepoint::CodePoint,
    structure::lists::{peekable::PeekableInterface, queue::ListQueue},
};

// ---- //
// Type //
// ---- //

pub type InputStream<T, I> = InputStreamPreprocessor<T, I>;

// NOTE(phisyx): améliorer ce type.
type PreScanFn<I> = fn(Option<I>) -> Option<I>;

// --------- //
// Structure //
// --------- //

/// Le flux d'entrée est constitué de caractères qui y sont insérés lors
/// du décodage du flux d'octets d'entrée ou par les diverses API qui
/// manipulent directement le flux d'entrée.
pub struct InputStreamPreprocessor<T, I> {
    iter: ListQueue<T, I>,
    is_replayed: bool,
    pub current: Option<I>,
    last_consumed_item: Option<I>,
    pre_scan: Option<PreScanFn<I>>,
}

// -------------- //
// Implémentation //
// -------------- //

impl<T, I> InputStreamPreprocessor<T, I>
where
    T: Iterator<Item = I>,
{
    /// Crée un nouveau flux d'entrée.
    pub fn new(iter: T) -> Self {
        let queue = ListQueue::new(iter);
        Self {
            iter: queue,
            is_replayed: false,
            current: None,
            last_consumed_item: None,
            pre_scan: None,
        }
    }

    pub fn with_pre_scan(mut self, scan_fn: PreScanFn<I>) -> Self {
        self.pre_scan.replace(scan_fn);
        self
    }
}

impl<T, I> InputStreamPreprocessor<T, I>
where
    T: Iterator<Item = I>,
    I: Clone,
{
    /// Alias de [Iterator::nth] : sauf que l'on fait n - 1 si n est
    /// supérieur ou égal à 1. Ce qui veut dire que : (n = n - 1)
    ///   - 1 = 0
    ///   - 2 = 1
    ///   - 3 = 2
    /// etc...
    pub fn advance(&mut self, mut n: usize) -> Option<I> {
        if n >= 1 {
            n -= 1;
        }
        self.nth(n)
    }

    // NOTE(phisyx): utiliser un générique pour le second paramètre?
    pub fn advance_as_long_as(
        &mut self,
        predicate: impl Fn(&I) -> bool,
        with_limit: Option<usize>,
    ) -> Vec<I> {
        let mut limit = with_limit.map(|n| n + 1).unwrap_or(0);
        let mut result = vec![];

        // NOTE(phisyx): code qui peut être amélioré.
        while (self.meanwhile().peek().is_some()
            && predicate(self.meanwhile().peek().unwrap()))
            && (limit > 0 || with_limit.is_none())
        {
            result.push(self.advance(1).unwrap());
            if with_limit.is_some() {
                limit -= 1;
            }
        }
        result
    }

    /// Permet de revenir en arrière dans le flux.
    pub fn rollback(&mut self) {
        self.is_replayed = true;
    }

    pub fn meanwhile(&mut self) -> &mut impl PeekableInterface<T, I> {
        &mut self.iter
    }

    /// Consomme le prochain élément du flux.
    pub fn consume_next_input(&mut self) -> Option<I> {
        if let Some(pre_scan) = &self.pre_scan {
            (pre_scan)(self.next())
        } else {
            self.next()
        }
        .and_then(|item| {
            let some_item = Some(item);
            self.current = some_item.to_owned();
            some_item
        })
    }

    pub fn next_input(&mut self) -> Option<I> {
        let item = self.meanwhile().peek().cloned();
        if let Some(pre_scan) = &self.pre_scan {
            (pre_scan)(item)
        } else {
            item
        }
    }
}

impl<Chars> InputStreamPreprocessor<Chars, Chars::Item>
where
    Chars: Iterator<Item = CodePoint>,
{
    /// Alias de [InputStreamPreprocessor::next_input]
    ///
    /// Consomme le prochain caractère du flux.
    pub fn consume_next_input_character(&mut self) -> Option<Chars::Item> {
        self.consume_next_input()
    }

    pub fn next_input_codepoint(&mut self) -> Option<u8> {
        self.next_input().map(|item| item as u8)
    }

    pub fn next_input_character(&mut self) -> Option<Chars::Item> {
        self.next_input()
    }

    pub fn next_n_input_character(&mut self, offset: usize) -> Cow<str> {
        self.slice_until(offset)
    }

    pub fn next_n_input_codepoint(&mut self, offset: usize) -> Vec<u8> {
        self.iter
            .peek_until::<String>(offset)
            .unwrap_or_default()
            .into()
    }

    /// Récupère les prochains caractères du flux jusqu'à une certaine
    /// position dans l'itération, sans avancer dans l'itération, et le
    /// transforme en [Cow<str>].
    pub fn slice_until(&mut self, lookahead_offset: usize) -> Cow<str> {
        self.iter
            .peek_until::<Cow<str>>(lookahead_offset)
            .unwrap_or_default()
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl<T, I> Iterator for InputStreamPreprocessor<T, I>
where
    T: Iterator<Item = I>,
    I: Clone,
{
    type Item = I;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_replayed {
            self.is_replayed = false;
            return self.last_consumed_item.to_owned();
        };

        self.last_consumed_item = self.iter.next();
        self.last_consumed_item.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_input_stream(
        input: &'static str,
    ) -> InputStreamPreprocessor<impl Iterator<Item = CodePoint>, CodePoint>
    {
        InputStreamPreprocessor::new(input.chars())
    }

    #[test]
    fn test_slice_until() {
        let mut stream = get_input_stream("Hello World");
        assert_eq!(stream.slice_until(5), Cow::Borrowed("Hello"));
        assert_eq!(stream.next(), Some('H'));
    }

    #[test]
    fn test_rollback() {
        let mut stream = get_input_stream("Hello World !");

        stream.next(); // H
        stream.next(); // e
        stream.rollback(); // H

        assert_eq!(stream.collect::<String>(), "ello World !".to_string());
    }
}
