use ::std::collections::HashMap;

pub type ParseResult<'s, T> = ::std::result::Result<T, ParseError<'s>>;
pub type Source<'s> = &'s str;

#[derive(Eq, PartialEq, Debug, Hash, Clone, Copy)]
pub enum ParseError<'s> {
    UnexpectedSymbol { expected: Option<char>, found: Source<'s> },
    UnexpectedEndOfInput { expected: Option<char> },
}


/// returns Some(remaining_source) if the next character is the specified symbol
fn skip_char(source: Source, symbol: char) -> Option<Source> {
    if source.starts_with(symbol) {
        Some(&source[symbol.len_utf8() .. ])

    } else {
        None
    }
}

/// returns Ok(remaining_source) if the first character is the specified symbol
fn expect_char(source: Source, expected_symbol: char) -> ParseResult<Source> {
    skip_char(source, expected_symbol).ok_or(ParseError::UnexpectedSymbol {
        found: source, expected: Some(expected_symbol),
    })
}

fn parse_chars_while<F: Fn(char) -> bool>(source: Source, predicate: F) -> (&str, Source) {
    source.split_at(
        source.char_indices()
            .skip_while(|&(_byte_index, character)| predicate(character))
            .map(|(byte_index, character)| byte_index)
            .next().unwrap_or(source.len()) // if the end was reached, split after the last char
    )
}

/// returns (parsed, remaining), both strings may be empty, discards the delimiter, result strings may start with whitespace
fn parse_over_delimiter_char(source: Source, delimiter: char) -> ParseResult<(&str, Source)> {
    let (parsed, source) = parse_chars_while(source, |character| character != delimiter);
    expect_char(source, delimiter)
        .map_err(|e| ParseError::UnexpectedEndOfInput { expected: Some(delimiter) })
        .map(|source_without_delimiter| (parsed, source_without_delimiter))
}



/// skips whitespace, returns Some(remaining_source) if the first character is the specified symbol
// TODO perf: on None return, discards trimming, and must be trimmed again..!
fn skip(source: Source, symbol: char) -> Option<Source> {
    skip_char(source.trim_left(), symbol)
}

/// skips whitespace, returns Ok(remaining_source) if the first character is the specified symbol
fn expect(source: Source, expected_symbol: char) -> ParseResult<Source> {
    expect_char(source.trim_left(), expected_symbol)
}

/// skips white, returns (parsed, remaining), both strings may be empty, discards the delimiter, result strings may start with whitespace
fn parse_over_delimiter(source: Source, delimiter: char) -> ParseResult<(&str, Source)> {
    parse_over_delimiter_char(source.trim_left(), delimiter)
}

/// skips leading whitespace, returns (parsed, remaining), both strings may be empty
fn parse_while<F: Fn(char) -> bool>(source: Source, predicate: F) -> (&str, Source) {
    parse_chars_while(source.trim_left(), predicate)
}

/// skips leading whitespace, returns Ok(none) if there is no string literal, and an error if there was a string literal detected but it was malformed
fn parse_string_literal(source: Source) -> ParseResult<(Option<&str>, Source)> {
    if let Some(source) = skip(source, '\'') {
        parse_over_delimiter_char(source, '\'')
            .map(|(literal, source)| (Some(literal), source))

    } else {
        Ok((None, source))
    }
}

/// skips leading whitespace, may return an empty identifier
fn parse_identifier(source: Source) -> (Identifier, Source) {
    let (name, source) = parse_while(
        source.trim_left(),
        |char| char.is_alphanumeric() || char == '_'
    );

    (Identifier { name }, source)
}

/// skips leading whitespace, parses until a '}' is found, throws error on file end without '}'
fn parse_delimited_named_objects(mut source: Source) -> ParseResult<(NamedObjects, Source)> {
    let mut names = HashMap::new();
    let mut objects = Vec::new();

    if let Some(mut remaining_source) = skip(source, '{') {
        loop {
            let remaining_objects = remaining_source.trim_left();

            if remaining_objects.is_empty() { // source is over, without finding delimiter
                return Err(ParseError::UnexpectedEndOfInput {
                    expected: Some('}')
                });

            } else { // more text remaining, probably containing a delimiter

                // end on delimiter found
                if let Some(skipped_source) = skip(remaining_objects, '}') {
                    remaining_source = skipped_source;
                    break;

                } else { // more overridden properties to parse
                    let (name, object, new_source) = parse_named_object(remaining_objects)?;
                    names.insert(name, objects.len());
                    objects.push(object);

                    remaining_source = new_source;
                }
            }
        }

        source = remaining_source;
    }

    Ok((NamedObjects { names, objects }, source))
}

/// skips leading whitespace, parses until file end, throws error on unexpected '}'
fn parse_remaining_named_objects(mut source: Source) -> ParseResult<(NamedObjects, Source)> {
    let mut names = HashMap::new();
    let mut objects = Vec::new();

    loop {
        let remaining_objects = source.trim_left();

        // no more properties to parse
        if remaining_objects.is_empty() {
            source = remaining_objects;
            break;

        } else { // text remaining, probably an object
            let (name, object, new_source) = parse_named_object(remaining_objects)?;
            names.insert(name, objects.len());
            objects.push(object);
            source = new_source;
        }
    }

    Ok((NamedObjects { names, objects }, source))
}


/// skips leading whitespace, parses either a string literal or a compound overriden object
fn parse_object(source: Source) -> ParseResult<(Object, Source)> {
    if let (Some(string_literal), source) = parse_string_literal(source)? {
        Ok((
            Object::StringLiteral(string_literal),
            source
        ))

    } else {
        let (prototype, source) = parse_identifier(source);
        let (overrides, source) = parse_delimited_named_objects(source)?;

        Ok((
            Object::Compound(Compound {
                prototype,
                overrides,
            }),
            source
        ))
    }
}


/// skips leading whitespace
fn parse_named_object(source: Source) -> ParseResult<(Identifier, Object, Source)> {
    let (name, source) = parse_identifier(source);
    let source = expect(source, ':')?;

    let (object, source) = parse_object(source)?;

    Ok((name, object, source))
}


/// parses objects from a string
pub fn parse(source: Source) -> ParseResult<NamedObjects> {
    parse_remaining_named_objects(source)
        .map(|(objects, _rest_src)| objects)
}




#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Object<'s> {
    /// literal may be empty
    StringLiteral(&'s str),
    Compound(Compound<'s>)
}

/// only the result of parsing. does not do any smart stuff. only holds string results.
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Compound<'s> {
    /// may be an empty string
    pub prototype: Identifier<'s>, // TODO later change to IdentifierChain
    pub overrides: NamedObjects<'s>,
}

/// parse result. supports looking up variables, e.g. prototypes by name
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct NamedObjects<'s> {
    pub objects: Vec<Object<'s>>, // separated from hashmap to perserve declaration order
    pub names: HashMap<Identifier<'s>, usize>, // indices into self.objects
}

#[derive(Eq, PartialEq, Debug, Hash, Clone)]
pub struct Identifier<'s> {
    pub name: &'s str, // currently only global variables supported
}


impl<'s> NamedObjects<'s> {
    pub fn find_by_identifier(&self, identifier: &Identifier) -> Option<&Object> {
        self.names.get(identifier).map(|index| &self.objects[*index])
    }
}


#[cfg(test)]
mod test_parsing {
    use super::*;

    // Object is not designed to be instantiated, but only to be parsed,
    // thus this is not a constructor but a test-helper
    fn compound_with_prototype_and_overrides<'s>(
        prototype: &'s str,
        overrides: Vec<(&'s str, Object<'s>)>
    ) -> Object<'s> {
        Object::Compound(Compound {
            prototype: Identifier { name: prototype },
            overrides: NamedObjects {
                names: overrides.iter().enumerate()
                    .map(|(index, &(ref name, _))| {
                        (Identifier { name }, index)
                    })
                    .collect(),

                objects: overrides.into_iter()
                    .map(|(_, object)| object)
                    .collect(),
            },
        })
    }

    fn compound_with_prototype(prototype: &str) -> Object {
        compound_with_prototype_and_overrides(prototype, vec![])
    }

    fn empty_compound() -> Object<'static> {
        compound_with_prototype("")
    }


    #[test]
    fn test_skip_symbol(){
        assert_eq!(skip("{}", '{'), Some("}"));
        assert_eq!(skip("{}", 'x'), None);

        assert_eq!(skip(" {}", '{'), Some("}"));
        assert_eq!(skip(" \n\t {}", '{'), Some("}"));

        assert_eq!(skip("", '{'), None);
        assert_eq!(skip("", 'x'), None);

        assert_eq!(skip(" ", 'x'), None);
        assert_eq!(skip("x", 'x'), Some(""));
        assert_eq!(skip(" \nx", 'x'), Some(""));
        assert_eq!(skip(" \nx ", 'x'), Some(" "));
        assert_eq!(skip_char(" \nx ", 'x'), None);
    }

    #[test]
    fn test_expect_symbol(){
        assert_eq!(expect("{}", '{'), Ok("}"));

        assert_eq!(
            expect("{}", 'x'),
            Err(ParseError::UnexpectedSymbol {
                expected: Some('x'),
                found: "{}"
            })
        );

        assert_eq!(expect(" {}", '{'), Ok("}"));
        assert_eq!(expect(" \n\t {}", '{'), Ok("}"));

        // TODO assert_eq!(expect_symbol("", '{'), Err(ParseError::UnexpectedEndOfInput { expected: Some('{') }));
        // TODO assert_eq!(expect_symbol(" \n", 'x'), Err(ParseError::UnexpectedEndOfInput { expected: Some('x') }));

        assert_eq!(expect("x", 'x'), Ok(""));
        assert_eq!(expect(" \nx", 'x'), Ok(""));

        assert_eq!(expect_char(" \nx", 'x'), Err(ParseError::UnexpectedSymbol {
            expected: Some('x'),
            found: " \nx",
        }));

        assert_eq!(expect(" \nx ", 'x'), Ok(" "));
    }

    #[test]
    fn test_parse_over_delimiter(){
        assert_eq!(parse_over_delimiter("x|z", '|'), Ok(("x", "z")));
        assert_eq!(parse_over_delimiter("|", '|'), Ok(("", "")));
        assert_eq!(parse_over_delimiter("xx|zz", '|'), Ok(("xx", "zz")));
        assert_eq!(parse_over_delimiter("xx||z", '|'), Ok(("xx", "|z")));
        assert_eq!(parse_over_delimiter("|||", '|'), Ok(("", "||")));

        assert_eq!(parse_over_delimiter(" | ", '|'), Ok(("", " ")));
        assert_eq!(parse_over_delimiter_char(" | ", '|'), Ok((" ", " ")));

        assert_eq!(parse_over_delimiter("xxzz", '|'), Err(ParseError::UnexpectedEndOfInput { expected: Some('|') }));
        assert_eq!(parse_over_delimiter("", '|'), Err(ParseError::UnexpectedEndOfInput { expected: Some('|') }));
        assert_eq!(parse_over_delimiter("   ", '|'), Err(ParseError::UnexpectedEndOfInput { expected: Some('|') }));
        assert_eq!(parse_over_delimiter_char("   ", '|'), Err(ParseError::UnexpectedEndOfInput { expected: Some('|') }));
    }

    #[test]
    fn test_parse_while(){
        assert_eq!(parse_while("xy", |c| c != 'y'), ("x", "y"));
        assert_eq!(parse_while("\n xy", |c| c != 'y'), ("x", "y"));
        assert_eq!(parse_while("xyz", |c| true), ("xyz", ""));
        assert_eq!(parse_while("xyz", |c| false), ("", "xyz"));
        assert_eq!(parse_while("", |c| true), ("", ""));
        assert_eq!(parse_while("", |c| false), ("", ""));

        assert_eq!(parse_while("9b", |c| c.is_numeric()), ("9", "b"));
        assert_eq!(parse_while(" 9 b", |c| c.is_numeric()), ("9", " b"));
        assert_eq!(parse_while(" x", |c| c.is_numeric()), ("", "x"));
    }

    #[test]
    fn test_parse_string_literal(){
        assert_eq!(parse_string_literal("xy"), Ok((None, "xy")));
        assert_eq!(parse_string_literal(" \n xy "), Ok((None, " \n xy ")));
        assert_eq!(parse_string_literal("' \n xy '"), Ok((Some(" \n xy "), "")));
        assert_eq!(parse_string_literal(" \n 'xy' "), Ok((Some("xy"), " ")));

        assert_eq!(parse_string_literal("'pls nooooo"), Err(ParseError::UnexpectedEndOfInput { expected: Some('\'') }));
        assert_eq!(parse_string_literal(" \n'\"pls nooooo\""), Err(ParseError::UnexpectedEndOfInput { expected: Some('\'') }));
    }

    #[test]
    fn test_parse_identifier(){
        assert_eq!(parse_identifier("?"), (Identifier{name:""}, "?"));
        assert_eq!(parse_identifier("-"), (Identifier{name:""}, "-"));
        assert_eq!(parse_identifier("$"), (Identifier{name:""}, "$"));

        assert_eq!(parse_identifier("x"), (Identifier{name:"x"}, ""));
        assert_eq!(parse_identifier("xy"), (Identifier{name:"xy"}, ""));
        assert_eq!(parse_identifier("xy "), (Identifier{name:"xy"}, " "));
        assert_eq!(parse_identifier(" xy "), (Identifier{name:"xy"}, " "));
        assert_eq!(parse_identifier(" xy9 "), (Identifier{name:"xy9"}, " "));

        assert_eq!(parse_identifier(" 9 "), (Identifier{name:"9"}, " "));
        assert_eq!(parse_identifier("x§"), (Identifier{name:"x"}, "§"));
    }


    #[test]
    fn test_parse_flat_value(){
        assert_eq!(parse_object("'xyz'"), Ok((Object::StringLiteral("xyz"), "")));
        assert_eq!(parse_object(" 'xyz' "), Ok((Object::StringLiteral("xyz"), " ")));
        assert_eq!(parse_object("div"), Ok((compound_with_prototype("div"), "")));
        assert_eq!(parse_object(" div!"), Ok((compound_with_prototype("div"), "!")));
        assert_eq!(parse_object("div{}"), Ok((compound_with_prototype("div"), "")));
        assert_eq!(parse_object(" div { } "), Ok((compound_with_prototype("div"), " ")));
        assert_eq!(parse_object(""), Ok((empty_compound(), "")));


        /* TODO
        assert_eq!(
            parse_value("?"),
            Err(ParseError::UnexpectedSymbol {
                expected: None,
                found: "?",
            })
        );*/
    }

    #[test]
    fn test_parse_flat_named_object(){
        assert_eq!(
            parse_named_object(" text: 'xyz' "),
            Ok((Identifier { name: "text" }, Object::StringLiteral("xyz"), " "))
        );

        assert_eq!(
            parse_named_object(" text: div { } "),
            Ok((
                Identifier { name: "text" },
                compound_with_prototype("div"),
                " "
            ))
        );
    }

    #[test]
    fn test_parse_nested_object(){
        assert_eq!(
            parse_named_object(" my_div: div { text: 'xy z' } "),
            Ok((
                Identifier { name: "my_div", },
                compound_with_prototype_and_overrides("div", vec![
                    ("text", Object::StringLiteral("xy z")),
                ]),
                " "
            ))
        );

        assert_eq!(
            parse_named_object(" my_div: div { text: 'xy z' content: default {} } "),
            Ok((
                Identifier { name: "my_div" },
                compound_with_prototype_and_overrides("div", vec![
                    ("text", Object::StringLiteral("xy z")),
                    ("content", compound_with_prototype("default")),
                ]),
                " "
            ))
        );

        assert_eq!(
            parse_named_object(" my_div: div { text: 'xy z' "),
            Err(ParseError::UnexpectedEndOfInput { expected: Some('}') } )
        );
    }
}
