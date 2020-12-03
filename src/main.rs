#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;

mod consts;
mod env;
mod parser;
mod registry;
mod status;
mod token;
mod utils;

fn main() {
    let matches = clap_app!(arsm =>
        (version: "0.1")
        (author: "Joshua B. <zippymagician1@gmail.com>")
        (about: "A shitty assembly flavor written in rust")
        (@arg INPUT: +required "The input file to be run")
        (@arg STDIN: "The program's input")
        (@arg debug: -d --debug "Use this flag to enable some debug features")
    )
    .get_matches();

    let file = matches.value_of("INPUT").unwrap();
    let data = utils::read_file(file);
    if let Ok(program) = data {
        parser::parse(parser::construct_tree(parser::lex(&*program)), matches);
    } else {
        panic!("File not found: {}", file);
    }
}
