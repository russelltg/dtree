use std::str;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::fmt::Debug;

use nom::{space, multispace, alphanumeric, IResult, ErrorKind};

use dtree::{Tree, Section, Mapping};

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

fn identifier(input_u8: &[u8]) -> IResult<&[u8], &str> {

    let input = match str::from_utf8(input_u8) {
        Err(_) => return IResult::Error(ErrorKind::Custom(13)),
        Ok(t) => t
    };

    let mut bytes_i: usize = 0;
    let mut last_size: usize = 0;

    for c in input.chars() {
        if (!c.is_alphabetic() && !c.is_numeric() && c != '_' && c != '-') {
            break;
        }
        bytes_i += c.len_utf8();
        last_size = c.len_utf8();
    }

    // bytes_i - last_size + 1 because not all chars are 1 long
    return IResult::Done(&input_u8[bytes_i..], &input[0..bytes_i - last_size + 1])
}

named!(desc_text<String>,
    call!(escaped_until, '\n')
);

named!(mapping_name<String>,
    call!(escaped_until, ')')
);

named!(section_desc<PartialDTree>,
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

);

/// Parse any utf-8 character until `until` is reached. Allows escaping of `until` with \
fn escaped_until(input_u8: &[u8], until: char) -> IResult<&[u8], String> {

    let input = match str::from_utf8(input_u8) {
        Ok(t) => t,
        Err(e) => return IResult::Error(ErrorKind::Custom(31)),
    };

    let mut s = String::new();

    let mut iter = input.chars().peekable();

    let mut bytes_i: usize = 0;

    loop {

        let c = match iter.next() {
            Some(n) => n,
            None => break,
        };


        match iter.peek() {
            Some(&a) if c == '\\' && a == until  => {
                s.push(until);
                iter.next();

                bytes_i += 1 + until.len_utf8(); // \ is a 1-byte char

                continue;
            },
            _ => {}
        }

        // if we've reacehd a end character, stop
        if c == until {
            return IResult::Done(&input_u8[bytes_i..], s);
        }

        bytes_i += c.len_utf8();
        s.push(c);
    }

    return IResult::Done(&b""[..], s);
}


named!(mapping<PartialDTreeOption>,
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

);


fn dtree_parse_impl(input: &[u8]) -> IResult<&[u8], Tree, String> {

    let mut in_mut = input;

    let mut sects_partial: Vec<PartialDTree> = Vec::new();
    let mut maps_partial: Vec<PartialDTreeOption> = Vec::new();

    loop {
        // see if it parses as a mapping
        match mapping(in_mut) {
            IResult::Done(i, o) => {
                in_mut = i;
                maps_partial.push(o);
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
                sects_partial.push(o);
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
    let mut sections = HashMap::<String, Section>::new();

    for s in sects_partial {
        // make sure it doesn't exist yet
        match sections.entry(s.name.clone()) {
            Occupied(_) => {
                return IResult::Error(ErrorKind::Custom(
                    format!("Section '{}' already has a description", s.name),
                ))
            }
            Vacant(e) => {
                e.insert(Section {
                    name: s.name,
                    description: s.description,
                    mappings: HashMap::new(),
                })
            }
        };
    }

    // Make all mappings
    for m in maps_partial {
        // make sure the destination exists
        if !sections.contains_key(&m.dest) {
            return IResult::Error(ErrorKind::Custom(
                format!("Destinaton '{}' for mapping ({})->{} in section '{}' does not exist",
                m.dest, m.opt_name, m.dest, m.parent),
            ));
        }

        match sections.entry(m.parent.clone()) {
            Vacant(_) => {
                return IResult::Error(ErrorKind::Custom(
                    format!("Section '{}' does not exist, and a mapping ({})->{} was created for it",
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

    return IResult::Done(in_mut, Tree { sections });
}

/// Parse a dtree file 
/// if `input` complies to the specification (defined in Spec.md)
/// Then it will pase correctly
pub fn parse_dtree (input: &str) -> Result<Tree, Box<Debug>> {
    match dtree_parse_impl(input.as_bytes()) {
        IResult::Done(_, t) => return Result::Ok(t),
        IResult::Error(e) => return Result::Err(Box::new(e)),
        IResult::Incomplete(i) => return Result::Err(Box::new(i))
    }
}


#[test]
fn parse_identifier_test() {

    assert_eq!(identifier(b"asdf"), IResult::Done(&b""[..], "asdf"));
    assert_eq!(identifier(b"asdf123"), IResult::Done(&b""[..], "asdf123"));
    assert_eq!(identifier(b"asd 12f"), IResult::Done(&b" 12f"[..], "asd"));
    assert_eq!(identifier(b"asd^f"), IResult::Done(&b"^f"[..], "asd"));
    assert_eq!(identifier(b"a_d^f"), IResult::Done(&b"^f"[..], "a_d"));
    assert_eq!(identifier("a京d^f".as_bytes()), IResult::Done(&b"^f"[..], "a京d"));
}

#[test]
fn parse_desc_test() {

    assert_eq!(desc_text("💝hiboi\\\na".as_bytes()), IResult::Done(&b""[..], String::from("💝hiboi\na")));
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

    assert_eq!(mapping(b"[ a123 (b\\)a\\d)->c] adf \\\nhello\na"),
        IResult::Done(&b"\na"[..], PartialDTreeOption{parent: String::from("a123"),
        description: String::from("adf \nhello"), opt_name: String::from("b)a\\d"),
        dest: String::from("c")}));

}
