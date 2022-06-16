/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops;

use infra::primitive::codepoint::CodePoint;

use crate::{
    preprocessor::InputStreamPreprocessor, StreamInput, StreamIterator,
    StreamToken, StreamTokenIterator,
};

// ---- //
// Type //
// ---- //

pub type InputStream<T, I> = InputStreamPreprocessor<T, I>;
pub type OutputStream<T> = TokenStream<T>;

// --------- //
// Structure //
// --------- //

#[derive(Debug)]
pub struct TokenStream<Token> {
    list_of_tokens: Vec<Token>,
    token_currently_being_operated_on: Option<Token>,
    reconsume_now: bool,
}

// -------------- //
// Implémentation //
// -------------- //

impl<I> TokenStream<I> {
    pub fn empty() -> Self {
        Self {
            list_of_tokens: Default::default(),
            token_currently_being_operated_on: Default::default(),
            reconsume_now: Default::default(),
        }
    }
}

impl<I> TokenStream<I>
where
    I: StreamToken,
{
    pub fn new<O>(mut stream: O) -> Self
    where
        O: StreamTokenIterator<Token = I>,
    {
        let mut list = Vec::new();

        loop {
            match stream.consume_next_token() {
                | Some(token) if !token.is_eof() => list.push(token),
                | _ => break,
            }
        }

        Self {
            list_of_tokens: list,
            token_currently_being_operated_on: Default::default(),
            reconsume_now: Default::default(),
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_iter<O>(iter_list: O) -> Self
    where
        O: Iterator<Item = I>,
    {
        Self {
            list_of_tokens: iter_list.collect(),
            token_currently_being_operated_on: Default::default(),
            reconsume_now: Default::default(),
        }
    }
}

impl<I> TokenStream<I>
where
    I: StreamToken,
{
    pub fn append(&mut self, token: I) {
        self.list_of_tokens.push(token);
    }

    pub fn prepend(&mut self, token: I) {
        let mut temp = vec![token];
        self.list_of_tokens.splice(..0, temp.drain(..));
    }

    pub fn replace_current_token_with(&mut self, token: I) {
        self.token_currently_being_operated_on.replace(token);
    }
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl<I> StreamIterator for TokenStream<I>
where
    I: StreamToken,
{
    type Item = I;

    fn advance_as_long_as_possible<
        'a,
        Predicate: Fn(&Self::Item) -> bool,
        Limit: infra::algorithms::Parameter<'a, usize>,
    >(
        &mut self,
        predicate: Predicate,
        with_limit: Limit,
    ) -> Vec<Self::Item> {
        let with_limit = unsafe { with_limit.param().value() };
        let mut limit = with_limit.map(|n| n + 1).unwrap_or(0);
        let mut result = vec![];

        while (self.next_token().is_some()
            && predicate(self.next_token().as_ref().unwrap()))
            && (limit > 0 || with_limit.is_none())
        {
            result.push(self.consume_next_token().unwrap());
            if with_limit.is_some() {
                limit -= 1;
            }
        }

        result
    }
}

impl<I> StreamTokenIterator for TokenStream<I>
where
    I: StreamToken,
{
    type Token = I;

    fn consume_next_token(&mut self) -> Option<Self::Token> {
        if self.reconsume_now {
            self.reconsume_now = false;
            return self.token_currently_being_operated_on.clone();
        }

        if self.list_of_tokens.is_empty() {
            self.token_currently_being_operated_on =
                Some(Self::Token::eof());
        } else {
            self.token_currently_being_operated_on =
                Some(self.list_of_tokens.remove(0));
        }

        self.token_currently_being_operated_on.clone()
    }

    fn current_token(&self) -> Option<&Self::Token> {
        self.token_currently_being_operated_on.as_ref()
    }

    fn current_token_mut(&mut self) -> Option<&mut Self::Token> {
        self.token_currently_being_operated_on.as_mut()
    }

    fn current_token_mut_callback<F: FnOnce(&mut Self::Token)>(
        &mut self,
        callback: F,
    ) -> &mut Self {
        if let Some(token) = self.current_token_mut() {
            callback(token);
        }
        self
    }

    fn next_token(&mut self) -> Option<Self::Token> {
        if self.list_of_tokens.is_empty() {
            return Some(Self::Token::eof());
        }
        self.list_of_tokens.iter().peekable().next().cloned()
    }

    fn reconsume_current_token(&mut self) {
        self.reconsume_now = true;
    }
}

impl StreamInput for CodePoint {
    fn eof() -> Self {
        '\0'
    }
}

impl<I> ops::Deref for TokenStream<I>
where
    I: StreamToken,
{
    type Target = Vec<I>;

    fn deref(&self) -> &Self::Target {
        &self.list_of_tokens
    }
}
