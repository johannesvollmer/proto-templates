use ::std::collections::HashMap;

pub type ParseResult<'s, T> = ::std::result::Result<T, ParseError<'s>>;

#[derive(Eq, PartialEq, Debug, Hash, Clone, Copy)]
pub enum ParseError<'s> {
    UnexpectedSymbol { expected: Option<char>, found: &'s str },
    UnexpectedEndOfInput { expected: Option<char> },
}


/// returns Some(remaining_source) if the first character is the specified symbol
fn skip_char(source: &str, symbol: char) -> Option<&str> {
    if source.starts_with(symbol) {
        Some(&source[symbol.len_utf8() .. ])

    } else {
        None
    }
}

/// returns Ok(remaining_source) if the first character is the specified symbol
fn expect_char(source: &str, expected_symbol: char) -> ParseResult<&str> {
    skip_char(source, expected_symbol).ok_or(ParseError::UnexpectedSymbol {
        found: source, expected: Some(expected_symbol),
    })
}

/// returns (parsed, remaining), both strings may be empty, discards the delimiter, result strings may start with whitespace
fn parse_over_delimiter_char(source: &str, delimiter: char) -> ParseResult<(&str, &str)> {
    // TODO call parse_while & next()?
    source.char_indices()
        .find(|&(_, character)| character == delimiter)
        .ok_or(ParseError::UnexpectedEndOfInput{ expected: Some(delimiter) })
        .map(|(byte_index, _)| {
            (
                &source[..byte_index],
                &source[byte_index + delimiter.len_utf8()..],
            )
        })
}



/// skips whitespace, returns Some(remaining_source) if the first character is the specified symbol
// TODO perf: on None return, discards trimming, and must be trimmed again..!
fn skip(source: &str, symbol: char) -> Option<&str> {
    skip_char(source.trim_left(), symbol)
}

/// skips whitespace, returns Ok(remaining_source) if the first character is the specified symbol
fn expect(source: &str, expected_symbol: char) -> ParseResult<&str> {
    expect_char(source.trim_left(), expected_symbol)
}


/// returns (parsed, remaining), both strings may be empty, discards the delimiter, result strings may start with whitespace
fn parse_over_delimiter(source: &str, delimiter: char) -> ParseResult<(&str, &str)> {
    parse_over_delimiter_char(source.trim_left(), delimiter)
}

/// skips leading whitespace, returns (parsed, remaining), both strings may be empty
fn parse_while<F: Fn(char) -> bool>(source: &str, predicate: F) -> (&str, &str) {
    let source = source.trim_left();

    let byte_index = source.char_indices()
        .skip_while(|&(_byte_index, character)| predicate(character))
        .map(|(byte_index, character)| byte_index)
        .next().unwrap_or(source.len()); // if the end was reached, split after the last char

    source.split_at(byte_index)
}

/// skips leading whitespace, returns Ok(none) if there is no string literal, and an error if there was a string literal detected but it was malformed
fn parse_string_literal(source: &str) -> ParseResult<(Option<&str>, &str)> {
    if let Some(source) = skip(source, '\'') {
        parse_over_delimiter_char(source, '\'')
            .map(|(literal, source)| (Some(literal), source))

    } else {
        Ok((None, source))
    }
}

/// skips leading whitespace, may return an empty identifier
fn parse_identifier(source: &str) -> (&str, &str) {
    parse_while(source.trim_left(), |char| char.is_alphanumeric() || char == '_')
}

/// skips leading whitespace, parses until a '}' is found, throws error on file end without '}'
fn parse_delimited_named_objects(mut source: &str) -> ParseResult<(Vec<Named<Object>>, &str)> {
    let mut overrides = Vec::new();

    if let Some(mut remaining_source) = skip(source, '{') {
        loop {
            let remaining_objects = remaining_source.trim_left();

            if remaining_objects.is_empty() { // source is over without finding delimiter
                return Err(ParseError::UnexpectedEndOfInput {
                    expected: Some('}')
                });

            } else { // more text remaining, probably containing a delimiter

                // end on delimiter found
                if let Some(skipped_source) = skip(remaining_objects, '}') {
                    remaining_source = skipped_source;
                    break;

                } else { // more overridden properties to parse
                    let (object, new_source) = parse_named_object(remaining_objects)?;
                    overrides.push(object);
                    remaining_source = new_source;
                }
            }
        }

        source = remaining_source;
    }

    Ok((overrides, source))
}

/// skips leading whitespace, parses until file end, throws error on unexpected '}'
fn parse_remaining_named_objects(mut source: &str) -> ParseResult<(Vec<Named<Object>>, &str)> {
    let mut overrides = Vec::new();

    loop {
        let remaining_objects = source.trim_left();

        // no more properties to parse
        if remaining_objects.is_empty() {
            source = remaining_objects;
            break;

        } else { // text remaining, probably an object
            let (object, new_source) = parse_named_object(remaining_objects)?;
            overrides.push(object);
            source = new_source;
        }
    }

    Ok((overrides, source))
}


/// skips leading whitespace, parses either a string literal or a compound overriden object
fn parse_object(source: &str) -> ParseResult<(Object, &str)> {
    if let (Some(string_literal), source) = parse_string_literal(source)? {
        //literal_result.map(|(string_literal, source)|{
            Ok((
                Object::StringLiteral(string_literal),
                source
            ))
        //})

    } else {
        let (prototype_identifier, source) = parse_identifier(source);
        let (overrides, source) = parse_delimited_named_objects(source)?;

        Ok((
            Object::Compound(Compound {
                prototype_identifier,
                overrides,
            }),
            source
        ))
    }
}


/// skips leading whitespace
fn parse_named_object(source: &str) -> ParseResult<(Named<Object>, &str)> {
    let (name, source) = parse_identifier(source);
    let source = expect(source, ':')?;

    let (object, source) = parse_object(source)?;

    Ok((
        Named { name, value: object, },
        source
    ))
}


/// parses objects from a string
pub fn parse(source: &str) -> ParseResult<Vec<Named<Object>>> {
    parse_remaining_named_objects(source)
        .map(|(objects, _rest_src)| objects)
}


#[derive(Eq, PartialEq, Debug, Hash, Clone)]
pub enum Object<'s> {
    /// literal may be empty
    StringLiteral(&'s str),
    Compound(Compound<'s>)
}

/// only the result of parsing. does not do any smart stuff. only holds string results.
#[derive(Eq, PartialEq, Debug, Hash, Clone)]
pub struct Compound<'s> {
    /// may be an empty string
    pub prototype_identifier: &'s str,
    pub overrides: Vec<Named<'s, Object<'s>>>,
}

#[derive(Eq, PartialEq, Debug, Hash, Clone)]
pub struct Named<'s, T> {
    /// may be an empty string
    pub name: &'s str,
    pub value: T,
}


#[cfg(test)]
mod test_parsing {
    use super::*;

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
        assert_eq!(parse_identifier("?"), ("", "?"));
        assert_eq!(parse_identifier("-"), ("", "-"));
        assert_eq!(parse_identifier("$"), ("", "$"));

        assert_eq!(parse_identifier("x"), ("x", ""));
        assert_eq!(parse_identifier("xy"), ("xy", ""));
        assert_eq!(parse_identifier("xy "), ("xy", " "));
        assert_eq!(parse_identifier(" xy "), ("xy", " "));
        assert_eq!(parse_identifier(" xy9 "), ("xy9", " "));

        assert_eq!(parse_identifier(" 9 "), ("9", " "));
        assert_eq!(parse_identifier("x§"), ("x", "§"));
    }

    #[test]
    fn test_parse_flat_value(){
        assert_eq!(
            parse_object("'xyz'"),
            Ok((Object::StringLiteral("xyz"), ""))
        );

        assert_eq!(
            parse_object(" 'xyz' "),
            Ok((Object::StringLiteral("xyz"), " "))
        );

        assert_eq!(
            parse_object("div"),
            Ok((Object::Compound(Compound {
                prototype_identifier: "div",
                overrides: vec![],
            }), ""))
        );

        assert_eq!(
            parse_object(" div!"),
            Ok((Object::Compound(Compound {
                prototype_identifier: "div",
                overrides: vec![],
            }), "!"))
        );

        assert_eq!(
            parse_object("div{}"),
            Ok((Object::Compound(Compound {
                prototype_identifier: "div",
                overrides: vec![],
            }), ""))
        );

        assert_eq!(
            parse_object(" div { } "),
            Ok((Object::Compound(Compound {
                prototype_identifier: "div",
                overrides: vec![],
            }), " "))
        );

        assert_eq!(
            parse_object(""),
            Ok((Object::Compound(Compound {
                prototype_identifier: "",
                overrides: vec![],
            }), ""))
        );


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
            Ok((Named { name: "text", value: Object::StringLiteral("xyz")}, " "))
        );

        assert_eq!(
            parse_named_object(" text: div { } "),
            Ok((Named { name: "text", value: Object::Compound(Compound {
                prototype_identifier: "div",
                overrides: vec![],
            }) }, " "))
        );
    }

    #[test]
    fn test_parse_nested_object(){
        assert_eq!(
            parse_named_object(" my_div: div { text: 'xy z' } "),
            Ok((Named { name: "my_div", value: Object::Compound(Compound {
                prototype_identifier: "div",
                overrides: vec![
                    Named { name: "text", value: Object::StringLiteral("xy z") }
                ],
            }) }, " "))
        );

        assert_eq!(
            parse_named_object(" my_div: div { text: 'xy z' content: default {} } "),
            Ok((Named { name: "my_div", value: Object::Compound(Compound {
                prototype_identifier: "div",
                overrides: vec![
                    Named { name: "text", value: Object::StringLiteral("xy z") },
                    Named { name: "content", value: Object::Compound(Compound {
                        prototype_identifier: "default",
                        overrides: vec![],
                    }) },
                ],
            }) }, " "))
        );

        assert_eq!(
            parse_named_object(" my_div: div { text: 'xy z' "),
            Err(ParseError::UnexpectedEndOfInput { expected: Some('}') } )
        );
    }
}
