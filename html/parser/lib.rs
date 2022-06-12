/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![feature(
    explicit_generic_args_with_impl_trait,
    type_name_of_val,
    option_result_contains
)]

mod codepoint;
mod error;
mod state;
mod tokenization;
mod tree_construction;

use std::{borrow::BorrowMut, ops::ControlFlow};

use dom::node::DocumentNode;
use infra::{self, primitive::codepoint::CodePoint};
use state::{FramesetOkFlag, InsertionMode};
use tokenization::HTMLToken;
use tree_construction::HTMLTreeConstruction;

use self::tokenization::HTMLTokenizer;

// --------- //
// Structure //
// --------- //

pub struct HTMLParser<C> {
    tokenizer: HTMLTokenizer<C>,
}

pub enum HTMLParserFlag {
    Pause,
    Stop,
}

pub enum HTMLParserState {
    Ignore,
    Continue,
    SwitchTo(String),
    ProcessNextTokenInLF,
    ProcessNextTokenExceptLF,
    CustomRcdata,
}

// -------------- //
// Implémentation //
// -------------- //

impl<C> HTMLParser<C> {
    pub fn new(document: DocumentNode, input: C) -> Self {
        let tokenizer = HTMLTokenizer::new(document, input);
        Self { tokenizer }
    }
}

impl<C> HTMLParser<C>
where
    C: Iterator<Item = CodePoint>,
{
    pub fn run(&mut self) {
        loop {
            let token = self.tokenizer.next_token();

            // TODO(phisyx): à améliorer ASAP.
            match self.tokenizer.tree_construction.dispatcher(token) {
                | ControlFlow::Continue(HTMLParserState::SwitchTo(
                    state,
                )) => {
                    self.tokenizer.switch_state_to(state);
                    continue;
                }
                | ControlFlow::Continue(
                    HTMLParserState::ProcessNextTokenExceptLF,
                ) => {
                    let next = self.tokenizer.next_token();
                    match next {
                        | Some(HTMLToken::Character('\n')) => continue,
                        | None => continue,
                        | _ => {
                            self.tokenizer
                                .tree_construction
                                .dispatcher(next);
                            continue;
                        }
                    }
                }

                // TODO(phisyx): à améliorer ASAP.
                | ControlFlow::Continue(HTMLParserState::CustomRcdata) => {
                    if let Some(HTMLToken::Character('\n')) =
                        self.tokenizer.next_token()
                    {
                        self.tokenizer.next();
                    }

                    self.tokenizer.switch_state_to("rcdata");
                    self.tokenizer
                        .tree_construction
                        .original_insertion_mode
                        .switch_to(
                            self.tokenizer
                                .tree_construction
                                .insertion_mode,
                        );
                    self.tokenizer.tree_construction.frameset_ok_flag =
                        FramesetOkFlag::NotOk;
                    self.tokenizer
                        .tree_construction
                        .insertion_mode
                        .switch_to(InsertionMode::Text);
                    continue;
                }

                | ControlFlow::Continue(_) => {
                    continue;
                }

                | ControlFlow::Break(HTMLParserFlag::Pause) => break, /* Voir TODO ci-haut */
                | ControlFlow::Break(HTMLParserFlag::Stop) => break, /* Voir TODO ci-haut */
            }
        }
    }

    pub fn tree_construction(&mut self) -> &mut HTMLTreeConstruction {
        self.tokenizer.tree_construction.borrow_mut()
    }
}
