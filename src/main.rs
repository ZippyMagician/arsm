#![deny(rust_2018_idioms, clippy::all)]
#![deny(mutable_borrow_reservation_conflict)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::similar_names,
    // A bit too pedantic for me
    clippy::if_not_else,
    clippy::module_name_repetitions,
    clippy::single_match_else,
    clippy::match_same_arms,
    clippy::too_many_lines,
    // Clojures don't have side effects, no point
    clippy::option_if_let_else,
    clippy::map_err_ignore,
    clippy::pub_enum_variant_names,
    // I'm not writing out a bunch of imports when it isn't necessary
    clippy::wildcard_imports
)]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;

mod env;
mod parser;
mod registry;
mod utils;

fn main() {
    let matches = clap_app!(arsm =>
        (version: "0.2")
        (author: "Joshua B. <zippymagician1@gmail.com>")
        (about: "A toy assembly flavor written in rust")
        (@arg INPUT: +required "The input file to be run")
        (@arg STDIN: "The program's input")
        (@arg stdin_file: --stdin +takes_value "The program's input, through a file")
        (@arg debug: -d --debug "Use this flag to enable some debug features")
    )
    .get_matches();

    // You can call `unwrap` here as INPUT is required
    let file = matches.value_of("INPUT").unwrap();
    if let Ok(program) = std::fs::read_to_string(file) {
        let lexed = parser::lex(&*program);
        let tree = parser::construct_tree(&lexed);
        parser::parse(&tree, &matches);
    } else {
        panic!("File not found: {}", file);
    }
}
