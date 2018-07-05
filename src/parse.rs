use ::std::collections::HashMap;


#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Object<'s> {
    /// literal may be empty
    StringLiteral(&'s str),
    Compound(Compound<'s>)
}

/// only the result of parsing. does not do any smart stuff. only holds string results.
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Compound<'s> {
    pub prototype: Reference<'s>, /// may be an empty string
    pub overrides: NamedObjects<'s>,
}

/// parse result. supports looking up variables, e.g. prototypes by name
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct NamedObjects<'s> {
    pub objects: Vec<Object<'s>>, // separated from hashmap, to perserve declaration order

    /// indices into self.objects
    pub identifiers: HashMap<Identifier<'s>, usize>,
}

/// the local, simple name of an object
#[derive(Eq, PartialEq, Debug, Hash, Clone)]
pub struct Identifier<'s> {
    pub name: &'s str,
}

/// the absolute, qualified name for a prototype
#[derive(Eq, PartialEq, Debug, Hash, Clone)]
pub struct Reference<'s> {
    /// before parsing, these entries would be separated by dots
    pub identifiers: Vec<Identifier<'s>>,
}


pub type Source<'s> = &'s str;

pub type ParseResult<'s, T> = ::std::result::Result<T, ParseError<'s>>;

#[derive(Eq, PartialEq, Debug, Hash, Clone, Copy)]
pub enum ParseError<'s> {
    UnexpectedSymbol { expected: Option<char>, found: Source<'s> },
    UnexpectedEndOfInput { expected: Option<char> },
}

pub type ResolveResult<'s, T> = ::std::result::Result<T, ResolveError<'s>>;

#[derive(Eq, PartialEq, Debug, Hash, Clone)]
pub enum ResolveError<'s> {
    ReferenceNotFound(Identifier<'s>),
    StringLiteralHasNoProperties,
}


impl<'s> Reference<'s> {
    /// false if this reference is an empty string
    pub fn has_target(&self) -> bool {
        !self.identifiers.is_empty()
            && self.identifiers.iter().any(|id| !id.name.is_empty())
    }
}

impl<'s> NamedObjects<'s> {
    pub fn resolve_reference<'o>(&'o self, reference: &'o Reference<'o>) -> ResolveResult<&'o Object<'o>> {
        self.resolve_reference_names(&reference.identifiers)
    }

    fn resolve_reference_names<'o>(&'o self, identifiers: &'o [Identifier<'o>]) -> ResolveResult<&'o Object<'o>> {
        let (first, sub_identifiers) = identifiers.split_first()
            .expect("resolve_reference_names: identifiers must not be empty");

        let index = self.identifiers.get(first)
            .ok_or_else(|| ResolveError::ReferenceNotFound(first.clone()))?;

        let identified = self.objects.get(*index)
            .expect("Invalid NamedObject::names Index");

        if sub_identifiers.is_empty() {
            Ok(identified)

        } else {
            match *identified {
                Object::Compound(ref compound) => {
                    compound.overrides.resolve_reference_names(sub_identifiers)
                },

                Object::StringLiteral(_) => Err(ResolveError::StringLiteralHasNoProperties),
            }
        }
    }
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
        |symbol| !symbol.is_whitespace() && !(".:{}").contains(symbol)
    );

    (Identifier { name }, source)
}

// TODO test these, and test lookup
/// parse a series of identifiers, separated by dots, e.g. 'label.dimensions.x'
fn parse_reference(source: Source) -> (Reference, Source) {
    let mut identifiers = Vec::new();

    let (first_identifier, mut source) = parse_identifier(source);
    if !first_identifier.name.is_empty() {
        identifiers.push(first_identifier);
        let mut remaining = source;

        while let Some(new_source) = skip(remaining, '.') {
            let (identifier, new_source) = parse_identifier(new_source);
            if !identifier.name.is_empty() {
                identifiers.push(identifier);
            } /* else {
                TODO
                return Err(ParseError::UnexpectedSymbol {
                    expected: None, // TODO expected("")
                    found: source,
                })
            }*/

            remaining = new_source;
        }

        source = remaining;
    }

    (Reference { identifiers }, source)

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

    Ok((NamedObjects { identifiers: names, objects }, source))
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

    Ok((NamedObjects { identifiers: names, objects }, source))
}


/// skips leading whitespace, parses either a string literal or a compound overriden object
fn parse_object(source: Source) -> ParseResult<(Object, Source)> {
    if let (Some(string_literal), source) = parse_string_literal(source)? {
        Ok((
            Object::StringLiteral(string_literal),
            source
        ))

    } else {
        let (prototype, source) = parse_reference(source);
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




#[cfg(test)]
mod test_parsing {
    use super::*;

    // Object is not designed to be instantiated, but only to be parsed,
    // thus this is not a constructor but a test-helper
    fn compound_with_prototype_and_overrides<'s>(
        prototype: Vec<&'s str>,
        overrides: Vec<(&'s str, Object<'s>)>
    ) -> Object<'s> {
        Object::Compound(Compound {
            prototype: Reference {
                identifiers: prototype.iter()
                    .map(|id| Identifier { name: id }).collect()
            },
            overrides: NamedObjects {
                identifiers: overrides.iter().enumerate()
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

    fn compound_with_prototype(prototype: Vec<&str>) -> Object {
        compound_with_prototype_and_overrides(prototype, vec![])
    }

    fn empty_compound() -> Object<'static> {
        compound_with_prototype(vec![])
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
    fn test_parse_identifier() {
        assert_eq!(parse_identifier("x$%&?/(|="), (Identifier { name: "x$%&?/(|=" }, ""));
        assert_eq!(parse_identifier("xy "), (Identifier { name: "xy" }, " "));
        assert_eq!(parse_identifier(" xy "), (Identifier { name: "xy" }, " "));
        assert_eq!(parse_identifier(" xy9 "), (Identifier { name: "xy9" }, " "));
        assert_eq!(parse_identifier(" 9 "), (Identifier { name: "9" }, " "));
        assert_eq!(parse_identifier("x§"), (Identifier { name: "x§" }, ""));
    }

    #[test]
    fn test_parse_reference() {
        assert_eq!(parse_reference("x"), (Reference { identifiers: vec![Identifier { name: "x" }]}, ""));
        assert_eq!(parse_reference("x.y"), (Reference {
            identifiers: vec![
                Identifier { name: "x" },
                Identifier { name: "y" }
            ] }, "")
        );

        assert_eq!(parse_reference("x.y.$"), (Reference {
            identifiers: vec![
                Identifier { name: "x" },
                Identifier { name: "y" },
                Identifier { name: "$" },
            ] }, "")
        );

        assert_eq!(parse_reference(" x . y . $ "), (Reference {
            identifiers: vec![
                Identifier { name: "x" },
                Identifier { name: "y" },
                Identifier { name: "$" },
            ] }, " ")
        );

        assert_eq!(parse_reference(" "), (Reference { identifiers: vec![] }, ""));
    }


    #[test]
    fn test_parse_flat_value(){
        assert_eq!(parse_object("'xyz'"), Ok((Object::StringLiteral("xyz"), "")));
        assert_eq!(parse_object(" 'xyz' "), Ok((Object::StringLiteral("xyz"), " ")));
        assert_eq!(parse_object("div"), Ok((compound_with_prototype(vec!["div"]), "")));
        assert_eq!(parse_object(" div!"), Ok((compound_with_prototype(vec!["div!"]), "")));
        assert_eq!(parse_object("div{}"), Ok((compound_with_prototype(vec!["div"]), "")));
        assert_eq!(parse_object(" div { } "), Ok((compound_with_prototype(vec!["div"]), " ")));
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
                compound_with_prototype(vec!["div"]),
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
                compound_with_prototype_and_overrides(vec!["div"], vec![
                    ("text", Object::StringLiteral("xy z")),
                ]),
                " "
            ))
        );

        assert_eq!(
            parse_named_object(" my_div: div { text: 'xy z' content: default {} } "),
            Ok((
                Identifier { name: "my_div" },
                compound_with_prototype_and_overrides(vec!["div"], vec![
                    ("text", Object::StringLiteral("xy z")),
                    ("content", compound_with_prototype(vec!["default"])),
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
