extern crate dtree;
extern crate nom;

use dtree::parser::dtree_parse;

use std::env;
use std::fs::File;
use std::io::prelude::{Read, Write};
use std::io::{stdin, stdout};

use nom::IResult::*;

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() <= 1 {
        eprintln!("Usage: dtree {{file.dtree}}");
        return;
    }

    let file = &args[1];

    let dtree_text = {
        let mut file = match File::open(file) {
            Ok(t) => t,
            Err(e) => return eprintln!("Failed to open file {}", e)
        };

        let mut dtree_text = String::new();
        match file.read_to_string(&mut dtree_text) {
            Ok(_) => {},
            Err(e) => return eprintln!("Failed to read file: {}", e)
        }

        dtree_text
    };

    // parse
    let dtree = match dtree_parse(dtree_text.as_bytes()) {
        Done(_, o) => o,
        Incomplete(_) => return eprintln!("Failed to parse--incomplete"),
        Error(e) => return eprintln!("Failed to parse {:?}", e)
    };

    // run the dtree

    // find the root node
    let mut current_node = match dtree.nodes.get("start") {
        Some(n) => n,
        None => return eprintln!("No start node")
    };


    loop {
        println!("{}", current_node.description);

        // print the options
        for (name, mapping) in &current_node.mappings {
            println!("({}) {}", name, mapping.description.replace("\n",
                &(String::from("\n") + &String::from(" ".repeat(name.len() + 3))) ));
        }
        print!("> ");
        stdout().flush().expect("Could not flush stdout");

        // get input
        let mut input = String::new();
        stdin().read_line(&mut input).expect("bad input");

        // read_line gets the newline, remove that
        input.pop();

        match current_node.mappings.get(&input) {
            Some(mapping) => {
                current_node = match dtree.nodes.get(&mapping.to) {
                    Some(n) => n,
                    None => panic!("Internal error: invalid to reference")
                };
            },
            None => {}
        }
    }

}
