/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod token;
pub mod tokenizer;

mod state {
    mod data;
    mod rawtext;
    mod rcdata;
    mod script_data;
}

pub(crate) use self::{
    token::{
        HTMLDoctypeToken, HTMLTagAttribute, HTMLTagAttributeName,
        HTMLTagAttributeValue, HTMLTagToken, HTMLToken,
    },
    tokenizer::{
        HTMLInputStream, HTMLTokenizer, HTMLTokenizerState, State,
    },
};
