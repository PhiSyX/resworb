/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::Range;

use super::peekable::PeekableInterface;

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
pub struct ListQueue<T, I> {
    iter: T,
    queue: Vec<Option<I>>,
    offset: usize,
}

// -------------- //
// Implémentation //
// -------------- //

impl<T, I> ListQueue<T, I> {
    pub fn new(iter: T) -> Self {
        Self {
            iter,
            queue: Vec::default(),
            offset: 0,
        }
    }
}

impl<T, I> ListQueue<T, I>
where
    T: Iterator<Item = I>,
{
    fn fill_queue_max(&mut self) {
        let stored_elements = self.queue.len();
        (0..=stored_elements).for_each(|_| self.enqueue());
    }

    fn fill_queue(&mut self, required_elements: usize) {
        let stored_elements = self.queue.len();
        if stored_elements <= required_elements {
            (stored_elements..=required_elements)
                .for_each(|_| self.enqueue());
        }
    }

    fn increment(&mut self) {
        if self.offset < usize::MAX {
            self.offset += 1;
        }
    }

    pub fn enqueue(&mut self) {
        self.queue.push(self.iter.next());
    }

    fn decrement(&mut self) {
        if self.offset > usize::MIN {
            self.offset -= 1;
        }
    }

    pub fn dequeue(&mut self) -> Option<T::Item> {
        self.queue.remove(0)
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl<T, I> PeekableInterface<T, I> for ListQueue<T, I>
where
    T: Iterator<Item = I>,
    I: Clone,
{
    fn peek(&mut self) -> Option<&T::Item> {
        self.fill_queue(self.offset);
        self.queue.get(self.offset).and_then(|v| v.as_ref())
    }

    fn peek_until<R: FromIterator<T::Item>>(
        &mut self,
        lookahead_offset: usize,
    ) -> Option<R> {
        Option::from(
            self.peek_range(0..lookahead_offset)
                .iter()
                .filter_map(|mch| mch.to_owned())
                .collect::<R>(),
        )
    }

    fn peek_until_end<R: FromIterator<T::Item>>(&mut self) -> R {
        self.fill_queue_max();
        self.queue.as_slice()[0..]
            .iter()
            .filter_map(|mch| mch.to_owned())
            .collect::<R>()
    }

    fn peek_range(&mut self, range: Range<usize>) -> &[Option<T::Item>] {
        if range.end > self.queue.len() {
            self.fill_queue(range.end);
        }
        &self.queue.as_slice()[range]
    }
}

impl<T, I> Iterator for ListQueue<T, I>
where
    T: Iterator<Item = I>,
{
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let consumed_item = if self.queue.is_empty() {
            self.iter.next()
        } else {
            self.dequeue()
        };

        self.decrement();

        consumed_item
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peek() {
        let mut stream = ListQueue::new("Hello World !".chars());

        assert_eq!(stream.next(), Some('H')); // -> 'H'ello Word !

        // On se rend au 5ème caractère sans avancer dans l'itération
        assert_eq!(stream.nth(4), Some(' ')); // -> ello' 'World !

        assert_eq!(stream.peek(), Some(&'W')); // -> 'W'orld !
        assert_eq!(stream.peek(), Some(&'W')); // -> 'W'orld !
        assert_eq!(stream.peek(), Some(&'W')); // -> 'W'orld !

        assert_eq!(stream.collect::<String>(), "World !".to_string());
    }

    #[test]
    fn test_peek_until() {
        let mut stream = ListQueue::new("Hello World !".chars());
        assert_eq!(
            stream.peek_until::<String>(5),
            Some(String::from("Hello"))
        );
        assert_eq!(stream.next(), Some('H'));
    }
}
