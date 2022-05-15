/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{collections::HashMap, env::args, fs::File};

use dom::node::DocumentNode;
use html::parser::HTMLParser;
use parser::decoder::ByteStream;

fn main() {
    let args: HashMap<_, _> = args()
        .map(|s| {
            let mut splitted = s.split_ascii_whitespace();
            let arg = splitted.next().expect("Un argument.");
            let val = splitted.next().unwrap_or_default();
            (arg.to_owned(), val.to_owned())
        })
        .collect();

    let stream: ByteStream = if let Some(filename) = args.get("-f") {
        File::open(filename)
            .try_into()
            .expect("La lecture du fichier HTML.")
    } else {
        ByteStream::from(include_str!(
            "../html/parser/crashtests/site.html.local"
        ))
    };

    let document_node = DocumentNode::new();
    let mut parser = HTMLParser::new(document_node, stream.chars());
    parser.run();
}
