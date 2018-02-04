# DTree file specification

## Notation

This document uses [Extended Backus-Naur Form](https://en.wikipedia.org/wiki/Extended_Backus%E2%80%93Naur_form).

## Basic definitions

```EBNF
unicode_letter = (* A unicode code point classified as "Letter" *)
unicode_digit = (* A unicode code point classified as "Number, decimal digit" *)
unicode_char = (* Any unicode point *)

letter = unicode_letter | "_"
space = " " | (* tab character *)
newline = (* newline character *)
```

## Identifiers

An identifier is used to name a section. It has the form:

```EBNF
identifier = ( unicode_letter | unicode_digit ) { unicode_letter | unicode_digit }
```

Examples:
`asdfaba`
`23193aadf`
`blaher`

Non-Examples
`hello bob`

## Message for a section

Defining a message has the form:

This must take place at the beginning of a line--the `[` must be the first character of the line

```EBNF
section_message = "[", { space }, identifier { space } "]" [ " " { unicode_char } ]
```

Examples


`[asdf] Welcome to party`

`[1913]`

```a
[aaad] Hello \
This is a super dooper thing \
Even with [ characters and all \
```

Non-examples:

```a
[adfa] Hello \
This is pretty cool boi
[
```

Reason: Cannot start a line with `[`

## Mapping for a section

This must take place at the beginning of a line--the `[` must be the first character of the line

```EBNF
mapping = "[" { space } identifier { space } "(" {unicode_char - ")\n"} ")" {space} "->" {space} identifier {space} "]" [ space { unicode_char }]
```

Examples:

```a
[dd (a)->b]
```
