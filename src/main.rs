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
        (version: "0.1")
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
        parser::parse(parser::construct_tree(parser::lex(&*program)), matches);
    } else {
        panic!("File not found: {}", file);
    }
}
