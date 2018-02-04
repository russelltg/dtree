use std::str;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};

use nom::{space, multispace, alphanumeric, IResult, ErrorKind};

use dtree::{Tree, Node, Mapping};

#[derive(Debug)]
struct PartialDTreeOption {
    parent: String,
    description: String,
    opt_name: String,
    dest: String,
}

impl PartialEq for PartialDTreeOption {
    fn eq(&self, other: &PartialDTreeOption) -> bool {
        return self.parent == other.parent && self.description == other.description &&
            self.opt_name == other.opt_name && self.dest == other.dest;
    }
}

#[derive(Debug)]
struct PartialDTree {
    name: String,
    description: String,
}

impl PartialEq for PartialDTree {
    fn eq(&self, other: &PartialDTree) -> bool {
        return self.description == other.description && self.name == other.name;
    }
}

named!(identifier<&str>,
    map_res!(
        alphanumeric,
        str::from_utf8
    )
);

fn desc_text(input: &[u8]) -> IResult<&[u8], String> {
    let mut s = String::new();
    let mut i = 0;

    while i < input.len() {
        if i < input.len() - 1 && input[i] == '\\' as u8 && input[i + 1] == '\n' as u8 {
            s.push('\n');

            i += 2;
            continue;
        }

        // if we've reacehd a newline, stop
        if input[i] == '\n' as u8 {
            return IResult::Done(&input[i..], s);
        }

        s.push(input[i] as char);
        i += 1;
    }

    return IResult::Done(&b""[..], s);
}

named!(section_desc<PartialDTree>,
    dbg!(
        do_parse!(
            opt!(multispace) >>
            tag!("[") >>
            opt!(space) >>
            n: identifier >>
            opt!(space) >>
            tag!("]") >>
            opt!(space) >>
            m: desc_text >>
            (PartialDTree{name: String::from(n), description: m})
        )
    )
);

fn mapping_name(input: &[u8]) -> IResult<&[u8], String> {
    let mut s = String::new();
    let mut i = 0;

    while i < input.len() {
        if i < input.len() - 1 && input[i] == '\\' as u8 && input[i + 1] == ')' as u8 {
            s.push(')');

            i += 2;
            continue;
        }

        // if we've reacehd a newline, stop
        if input[i] == ')' as u8 {
            return IResult::Done(&input[i..], s);
        }

        s.push(input[i] as char);
        i += 1;
    }

    return IResult::Done(&b""[..], s);
}


named!(mapping<PartialDTreeOption>,
    dbg_dmp!(
        do_parse!(
            opt!(multispace) >>
            tag!("[") >>
            opt!(space) >>
            n: identifier >>
            opt!(space) >>
            tag!("(") >>
            name: mapping_name >>
            tag!(")") >>
            opt!(space) >>
            tag!("->") >>
            opt!(space) >>
            to: identifier >>
            opt!(space) >>
            tag!("]") >>
            opt!(space) >>
            m: desc_text >>
            (PartialDTreeOption{parent: String::from(n), description: m,
                opt_name: name, dest: String::from(to)})
        )
    )
);

pub fn dtree_parse(input: &[u8]) -> IResult<&[u8], Tree, String> {

    let mut in_mut = input;

    let mut sects: Vec<PartialDTree> = Vec::new();
    let mut maps: Vec<PartialDTreeOption> = Vec::new();

    loop {
        // see if it parses as a mapping
        match mapping(in_mut) {
            IResult::Done(i, o) => {
                in_mut = i;
                maps.push(o);
                continue;
            }
            IResult::Error(_) => {}
            IResult::Incomplete(_) => {
                break;
            }

        }

        // see if it's a section
        match section_desc(in_mut) {
            IResult::Done(i, o) => {
                in_mut = i;
                sects.push(o);
                continue;
            }
            IResult::Error(_) => {
                return IResult::Error(ErrorKind::Tag);
            }
            IResult::Incomplete(_) => {
                break;
            }
        }
    }

    // Link
    let mut nodes: HashMap<String, Node> = HashMap::new();

    for s in sects {
        // make sure it doesn't exist yet
        match nodes.entry(s.name.clone()) {
            Occupied(_) => {
                return IResult::Error(ErrorKind::Custom(
                    format!("Node '{}' already has a description", s.name),
                ))
            }
            Vacant(e) => {
                e.insert(Node {
                    name: s.name,
                    description: s.description,
                    mappings: HashMap::new(),
                })
            }
        };
    }

    // Make all mappings
    for m in maps {
        // make sure the destination exists
        if !nodes.contains_key(&m.dest) {
            return IResult::Error(ErrorKind::Custom(
                format!("Destinaton '{}' for mapping ({})->{} in node '{}' does not exist",
                m.dest, m.opt_name, m.dest, m.parent),
            ));
        }

        match nodes.entry(m.parent.clone()) {
            Vacant(_) => {
                return IResult::Error(ErrorKind::Custom(
                    format!("Node '{}' does not exist, and a mapping ({})->{} was created for it",
                m.parent, m.opt_name, m.dest),
                ))
            }

            Occupied(e) => {
                e.into_mut().mappings.insert(
                    m.opt_name,
                    Mapping {
                        description: m.description,
                        to: m.dest,
                    },
                )
            }
        };
    }

    return IResult::Done(in_mut, Tree { nodes });
}


#[test]
fn parse_identifier_test() {

    assert_eq!(identifier(b"asdf"), IResult::Done(&b""[..], "asdf"));
    assert_eq!(identifier(b"asdf123"), IResult::Done(&b""[..], "asdf123"));
    assert_eq!(identifier(b"asd 12f"), IResult::Done(&b" 12f"[..], "asd"));
    assert_eq!(identifier(b"asd^f"), IResult::Done(&b"^f"[..], "asd"));
}

#[test]
fn parse_desc_test() {

    assert_eq!(desc_text(b"hiboi\\\na"), IResult::Done(&b""[..], String::from("hiboi\na")));
    assert_eq!(desc_text(b"hiboi\na"), IResult::Done(&b"\na"[..], String::from("hiboi")));
    assert_eq!(desc_text(b"\\hiboi\\\na"), IResult::Done(&b""[..], String::from("\\hiboi\na")));
    assert_eq!(desc_text(b"hello \nasdf"), IResult::Done(&b"\nasdf"[..], String::from("hello ")));
}

#[test]
fn section_test() {
    assert_eq!(section_desc(b"[ a ] hello \nasdf"), IResult::Done(&b"\nasdf"[..], PartialDTree{
        name: String::from("a"), description: String::from("hello ")}));
    assert_eq!(section_desc(b"[a] hello\\\naaaa"), IResult::Done(&b""[..], PartialDTree{
        name: String::from("a"), description: String::from("hello\naaaa")}));
}

#[test]
fn mapping_test() {
    assert_eq!(mapping(b" [ a (b) -> c ] adf"), IResult::Done(&b""[..], PartialDTreeOption{
        parent: String::from("a"), description: String::from("adf"),
        opt_name: String::from("b"), dest: String::from("c")}));

    assert_eq!(mapping(b"[ a123 (b\\))->c] adf \\\nhello\na"),
        IResult::Done(&b"\na"[..], PartialDTreeOption{parent: String::from("a123"),
        description: String::from("adf \nhello"), opt_name: String::from("b)"),
        dest: String::from("c")}));

}
