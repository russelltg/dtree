extern crate dtree;

use dtree::parser::parse_dtree;

use std::env;
use std::fs::File;
use std::io::prelude::{Read, Write};
use std::io::{stdin, stdout};

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
    let dtree = match parse_dtree(&dtree_text) {
        Ok(o) => o,
        Err(e) => return eprintln!("Failed to parse: {:?}", e),
    };

    // run the dtree

    // find the root node
    let mut current_section = match dtree.sections.get("start") {
        Some(n) => n,
        None => return eprintln!("No start section")
    };


    loop {
        println!("{}", current_section.description);

        // print the options
        for mapping in &current_section.mappings {
            println!("({:?}) {}", mapping.triggers, mapping.description.replace("\n",
                &(String::from("\n") + &String::from(" ".repeat(mapping.triggers.len() + 3))) ));
        }
        print!("> ");
        stdout().flush().expect("Could not flush stdout");

        // get input
        let mut input = String::new();
        stdin().read_line(&mut input).expect("bad input");

        // read_line gets the newline, remove that
        input.pop();

        match current_section.mapping(&input) {
            Some(to) => {
                current_section = match dtree.sections.get(to) {
                    Some(n) => n,
                    None => panic!("Internal error: invalid to reference")
                };
            },
            None => {}
        }
    }

}
