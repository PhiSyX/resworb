/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{borrow::Cow, ops::Range};

// --------- //
// Structure //
// --------- //

/// Le flux d'entrée est constitué de caractères qui y sont insérés lors
/// du décodage du flux d'octets d'entrée ou par les diverses API qui
/// manipulent directement le flux d'entrée.
pub struct InputStreamPreprocessor<T, I>
where
    T: Iterator<Item = I>,
    I: Clone,
{
    tokenizer: T,
    queue: Vec<Option<I>>,
    offset: usize,
    is_replayed: bool,
    current: Option<I>,
    last_consumed_item: Option<I>,
}

// -------------- //
// Implémentation //
// -------------- //

impl<T, I> InputStreamPreprocessor<T, I>
where
    T: Iterator<Item = I>,
    I: Clone,
{
    /// Crée un nouveau flux d'entrée.
    pub fn new(tokenizer: T) -> Self {
        Self {
            tokenizer,
            queue: vec![],
            offset: 0,
            is_replayed: false,
            current: None,
            last_consumed_item: None,
        }
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

    /// Récupère le prochain élément du flux sans avancer dans l'itération.
    pub fn peek(&mut self) -> Option<&I> {
        self.fill_queue(self.offset);
        self.queue.get(self.offset).and_then(|v| v.as_ref())
    }

    /// Récupère les prochains éléments du flux jusqu'à une certaine
    /// position dans l'itération, sans avancer dans l'itération.
    ///
    /// Le type générique est obligatoire.
    pub fn peek_until<R: FromIterator<I>>(
        &mut self,
        lookahead_offset: usize,
    ) -> Option<R> {
        Option::from(
            self.peek_range(0..lookahead_offset)
                .iter()
                .filter_map(|mch| mch.clone())
                .collect::<R>(),
        )
    }

    /// Permet de revenir en arrière dans le flux.
    pub fn rollback(&mut self) {
        self.is_replayed = true;
    }

    pub fn next_input(&mut self) -> Option<I> {
        self.tokenizer.next().and_then(|item| {
            let some_item = Some(item);
            self.current = some_item.clone();
            some_item
        })
    }

    fn decrement_offset(&mut self) {
        if self.offset > usize::MIN {
            self.offset -= 1;
        }
    }

    fn fill_queue(&mut self, required_elements: usize) {
        let stored_elements = self.queue.len();
        if stored_elements <= required_elements {
            (stored_elements..=required_elements)
                .for_each(|_| self.push_next_to_queue());
        }
    }

    fn peek_range(&mut self, range: Range<usize>) -> &[Option<I>] {
        if range.end > self.queue.len() {
            self.fill_queue(range.end);
        }
        &self.queue.as_slice()[range]
    }

    fn push_next_to_queue(&mut self) {
        let item = self.tokenizer.next();
        self.queue.push(item);
    }
}

impl<Chars> InputStreamPreprocessor<Chars, Chars::Item>
where
    Chars: Iterator<Item = char>,
{
    /// Alias de [InputStreamPreprocessor::next_input]
    pub fn next_input_char(&mut self) -> Option<Chars::Item> {
        self.next_input()
    }

    /// Récupère les prochains caractères du flux jusqu'à une certaine
    /// position dans l'itération, sans avancer dans l'itération, et le
    /// transforme en [Cow<str>].
    pub fn slice_until(&mut self, lookahead_offset: usize) -> Cow<str> {
        self.peek_until::<Cow<str>>(lookahead_offset)
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
            return self.last_consumed_item.clone();
        }

        let consumed_item = if self.queue.is_empty() {
            self.tokenizer.next()
        } else {
            self.queue.remove(0)
        };

        self.decrement_offset();

        self.last_consumed_item = consumed_item.clone();

        consumed_item
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_input_stream(
        input: &'static str,
    ) -> InputStreamPreprocessor<impl Iterator<Item = char>, char> {
        InputStreamPreprocessor::new(input.chars())
    }

    #[test]
    fn test_peek() {
        let mut stream = get_input_stream("Hello World !");

        assert_eq!(stream.next(), Some('H')); // -> 'H'ello Word !

        // On se rend au 5ème caractère sans avancer dans l'itération
        assert_eq!(stream.advance(5), Some(' ')); // -> ello' 'World !

        assert_eq!(stream.peek(), Some(&'W')); // -> 'W'orld !
        assert_eq!(stream.peek(), Some(&'W')); // -> 'W'orld !
        assert_eq!(stream.peek(), Some(&'W')); // -> 'W'orld !

        assert_eq!(stream.collect::<String>(), "World !".to_string());
    }

    #[test]
    fn test_peek_until() {
        let mut stream = get_input_stream("Hello World !");
        assert_eq!(
            stream.peek_until::<String>(5),
            Some(String::from("Hello"))
        );
        assert_eq!(stream.next(), Some('H'));
    }

    #[test]
    fn test_slice_until() {
        let mut stream = get_input_stream("Hello World !");
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
