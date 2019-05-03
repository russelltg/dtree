use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use std::str;

use nom::{
    call, do_parse, multispace, named, opt, separated_list, space, tag, Context, Err, ErrorKind,
    IResult,
};

use crate::dtree::{Mapping, Section, Tree};

#[derive(Debug, PartialEq)]
struct PartialDTreeOption {
    parent: String,
    description: String,
    triggers: Vec<String>,
    dest: String,
}

#[derive(Debug, PartialEq)]
struct PartialDTree {
    name: String,
    description: String,
}

fn identifier(input_u8: &[u8]) -> IResult<&[u8], &str> {
    let input = match str::from_utf8(input_u8) {
        Err(_) => return Result::Err(Err::Failure(Context::Code(input_u8, ErrorKind::Custom(13)))),
        Ok(t) => t,
    };

    let mut bytes_i: usize = 0;
    let mut last_size: usize = 0;

    for c in input.chars() {
        if !c.is_alphabetic() && !c.is_numeric() && c != '_' && c != '-' {
            break;
        }
        bytes_i += c.len_utf8();
        last_size = c.len_utf8();
    }

    // bytes_i - last_size + 1 because not all chars are 1 long
    Ok((&input_u8[bytes_i..], &input[0..=bytes_i - last_size]))
}

named!(desc_text<String>, call!(escaped_until, '\n'));

named!(mapping_name<String>, call!(escaped_until, ')'));

named!(
    section_desc<PartialDTree>,
    do_parse!(
        opt!(multispace)
            >> tag!("[")
            >> opt!(space)
            >> n: identifier
            >> opt!(space)
            >> tag!("]")
            >> opt!(space)
            >> m: desc_text
            >> (PartialDTree {
                name: String::from(n),
                description: m
            })
    )
);

/// Parse any utf-8 character until `until` is reached. Allows escaping of `until` with \
fn escaped_until(input_u8: &[u8], until: char) -> IResult<&[u8], String> {
    let input = match str::from_utf8(input_u8) {
        Ok(t) => t,
        Err(_) => return Result::Err(Err::Failure(Context::Code(input_u8, ErrorKind::Custom(31)))),
    };

    let mut s = String::new();

    let mut iter = input.chars().peekable();

    let mut bytes_i: usize = 0;

    while let Some(c) = iter.next() {
        match iter.peek() {
            Some(&a) if c == '\\' && a == until => {
                s.push(until);
                iter.next();

                bytes_i += 1 + until.len_utf8(); // \ is a 1-byte char

                continue;
            }
            _ => {}
        }

        // if we've reacehd a end character, stop
        if c == until {
            return Ok((&input_u8[bytes_i..], s));
        }

        bytes_i += c.len_utf8();
        s.push(c);
    }

    Ok((&b""[..], s))
}

/// Parses something in the form
named!(
    trigger_from<Vec<String>>,
    separated_list!(
        tag!("|"),
        do_parse!(opt!(space) >> tag!("(") >> n: mapping_name >> tag!(")") >> opt!(space) >> (n))
    )
);

named!(
    trigger_to<&str>,
    do_parse!(tag!("->") >> opt!(space) >> to: identifier >> (to))
);

named!(
    mapping<PartialDTreeOption>,
    do_parse!(
        opt!(multispace)
            >> tag!("[")
            >> opt!(space)
            >> n: identifier
            >> opt!(space)
            >> from: trigger_from
            >> opt!(space)
            >> to: trigger_to
            >> opt!(space)
            >> tag!("]")
            >> opt!(space)
            >> m: desc_text
            >> (PartialDTreeOption {
                parent: String::from(n),
                description: m,
                triggers: from,
                dest: String::from(to)
            })
    )
);

fn dtree_parse_impl(input: &[u8]) -> IResult<&[u8], Tree, String> {
    let mut in_mut = input;

    let mut sects_partial: Vec<PartialDTree> = Vec::new();
    let mut maps_partial: Vec<PartialDTreeOption> = Vec::new();

    loop {
        // see if it parses as a mapping
        match mapping(in_mut) {
            Ok((i, o)) => {
                in_mut = i;
                maps_partial.push(o);
                continue;
            }
            Result::Err(Err::Incomplete(_)) => break,
            _ => {}
        }

        // see if it's a section
        match section_desc(in_mut) {
            Ok((i, o)) => {
                in_mut = i;
                sects_partial.push(o);
                continue;
            }
            Err(Err::Incomplete(_)) => {
                break;
            }
            _ => {
                return Err(Err::Failure(Context::Code(input, ErrorKind::Tag)));
            }
        }
    }

    // Link
    let mut sections = HashMap::<String, Section>::new();

    for s in sects_partial {
        // make sure it doesn't exist yet
        match sections.entry(s.name.clone()) {
            Occupied(_) => {
                return Err(Err::Failure(Context::Code(
                    input,
                    ErrorKind::Custom(format!("Section '{}' already has a description", s.name)),
                )))
            }
            Vacant(e) => e.insert(Section {
                name: s.name,
                description: s.description,
                mappings: Vec::new(),
            }),
        };
    }

    // Make all mappings
    for m in maps_partial {
        // make sure the destination exists
        if !sections.contains_key(&m.dest) {
            return Err(Err::Failure(Context::Code(
                input,
                ErrorKind::Custom(format!(
                    "Destinaton {} for mapping ({:?})->{} in section {} does not exist",
                    m.dest, m.triggers, m.dest, m.parent
                )),
            )));
        }

        match sections.entry(m.parent.clone()) {
            Vacant(_) => {
                return Err(Err::Failure(Context::Code(
                    input,
                    ErrorKind::Custom(format!(
                        "Section {} does not exist, and a mapping ({:?})->{} was created for it",
                        m.parent, m.triggers, m.dest
                    )),
                )))
            }

            Occupied(e) => e.into_mut().mappings.push(Mapping {
                triggers: m.triggers,
                description: m.description,
                to: m.dest,
            }),
        };
    }

    Ok((in_mut, Tree { sections }))
}

/// Parse a dtree file
/// if `input` complies to the specification (defined in Spec.md)
/// Then it will pase correctly
pub fn parse_dtree(input: &str) -> Result<Tree, String> {
    match dtree_parse_impl(input.as_bytes()) {
        Ok((_, t)) => Ok(t),
        Err(Err::Error(e)) | Err(Err::Failure(e)) => Err(format!("{:?}", e)),
        Err(Err::Incomplete(e)) => Err(format!("{:?}", e)),
    }
}

#[test]
fn parse_identifier_test() {
    assert_eq!(identifier(b"asdf"), Ok((&b""[..], "asdf")));
    assert_eq!(identifier(b"asdf123"), Ok((&b""[..], "asdf123")));
    assert_eq!(identifier(b"asd 12f"), Ok((&b" 12f"[..], "asd")));
    assert_eq!(identifier(b"asd^f"), Ok((&b"^f"[..], "asd")));
    assert_eq!(identifier(b"a_d^f"), Ok((&b"^f"[..], "a_d")));
    assert_eq!(identifier("a京d^f".as_bytes()), Ok((&b"^f"[..], "a京d")));
}

#[test]
fn parse_desc_test() {
    assert_eq!(
        desc_text("💝hiboi\\\na".as_bytes()),
        Ok((&b""[..], String::from("💝hiboi\na")))
    );
    assert_eq!(
        desc_text(b"hiboi\na"),
        Ok((&b"\na"[..], String::from("hiboi")))
    );
    assert_eq!(
        desc_text(b"\\hiboi\\\na"),
        Ok((&b""[..], String::from("\\hiboi\na")))
    );
    assert_eq!(
        desc_text(b"hello \nasdf"),
        Ok((&b"\nasdf"[..], String::from("hello ")))
    );
}

#[test]
fn section_test() {
    assert_eq!(
        section_desc(b"[ a ] hello \nasdf"),
        Ok((
            &b"\nasdf"[..],
            PartialDTree {
                name: String::from("a"),
                description: String::from("hello ")
            }
        ))
    );
    assert_eq!(
        section_desc(b"[a] hello\\\naaaa"),
        Ok((
            &b""[..],
            PartialDTree {
                name: String::from("a"),
                description: String::from("hello\naaaa")
            }
        ))
    );
}

#[test]
fn mapping_test() {
    assert_eq!(
        mapping(b" [ a (b) -> c ] adf"),
        Ok((
            &b""[..],
            PartialDTreeOption {
                parent: String::from("a"),
                description: String::from("adf"),
                triggers: vec!(String::from("b")),
                dest: String::from("c")
            }
        ))
    );

    assert_eq!(
        mapping(b"[ a123 (b\\)a\\d)->c] adf \\\nhello\na"),
        Ok((
            &b"\na"[..],
            PartialDTreeOption {
                parent: String::from("a123"),
                description: String::from("adf \nhello"),
                triggers: vec!(String::from("b)a\\d")),
                dest: String::from("c")
            }
        ))
    );

    assert_eq!(
        mapping(b"[ a123 (b) | (e)->c] adf"),
        Ok((
            &b""[..],
            PartialDTreeOption {
                parent: String::from("a123"),
                description: String::from("adf"),
                triggers: vec!(String::from("b"), String::from("e")),
                dest: String::from("c")
            }
        ))
    );
}
