use ::std::collections::HashMap;
use ::parse::*;

pub type FlatCompound = HashMap<String, FlatObject>;

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum FlatObject {
    StringLiteral(String),
    Compound(FlatCompound),
}


impl FlatObject {
    pub fn parse(source: ::parse::Source) -> ::parse::ParseResult<FlatObject> {
        ::parse::parse(source).map(|parsed| Self::build_from_parsed(&parsed))
    }

    pub fn build_from_parsed(parsed: &NamedObjects) -> FlatObject {
        FlatObject::Compound(Self::build_from_parsed_named_objects(parsed, parsed))
    }

    fn build_from_parsed_named_objects(objects: &NamedObjects, world: &NamedObjects) -> FlatCompound {
        let mut properties = HashMap::new();
        Self::fill_named_objects(objects, world, &mut properties);
        properties
    }


    /// not recursive, will not add children
    fn fill_named_objects(
        objects: &NamedObjects,
        world: &NamedObjects,
        properties: &mut FlatCompound
    ){
        for (override_identifier, override_index) in &objects.identifiers {
            let name_string = override_identifier.name.to_owned();
            properties.entry(name_string).or_insert_with(||{
                FlatObject::build_from_parsed_unnamed_object(
                    &objects.objects[*override_index],
                    world
                )
            });
        }
    }

    fn deep_fill_parsed_compound(
        compound: &Compound,
        world: &NamedObjects,
        properties: &mut FlatCompound
    ) {
        Self::fill_named_objects(&compound.overrides, world, properties);

        if compound.prototype.has_target() {
            // insert all inherited properties, if not already overridden
            if let Object::Compound(ref compound) = *world.resolve_reference(&compound.prototype)
                .expect(&format!("variable not found: '{:?}'", compound.prototype)) // TODO use runtime error handling
                {
                    Self::deep_fill_parsed_compound(compound, world, properties);
                }
        }
    }

    fn build_from_parsed_unnamed_object(parsed: &Object, world: &NamedObjects) -> FlatObject {
        match *parsed {
            Object::StringLiteral(ref literal) => {
                FlatObject::StringLiteral(literal.to_string())
            },

            Object::Compound(ref compound) => {
                // inlining of variables,
                // needed for the special case where the prototype is a string literal
                if compound.overrides.objects.is_empty() && compound.prototype.has_target() {
                    let prototype = world.resolve_reference(&compound.prototype)
                        // TODO use runtime error handling
                        .expect(&format!("variable not found: '{:?}'", compound.prototype));

                    Self::build_from_parsed_unnamed_object(prototype, world)

                } else { // plain object with some overrides, or empty
                    FlatObject::Compound({
                        let mut properties = HashMap::new();
                        Self::deep_fill_parsed_compound(compound, world, &mut properties);
                        properties
                    })
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn literal(text: &str) -> FlatObject {
        FlatObject::StringLiteral(String::from(text))
    }

    fn compound(properties: Vec<(&str, FlatObject)>) -> FlatObject {
        FlatObject::Compound({
            properties.into_iter()
                .map(|(name, obj)| (String::from(name), obj))
                .collect()
        })
    }


    #[test]
    fn test_build_flat_object(){
        assert_eq!(
            FlatObject::parse(" color: ' #abc ' ").expect("Parsing Error"),
            compound(vec![
                ("color", literal(" #abc "))
            ])
        );

        assert_eq!(
            FlatObject::parse(r#"

                default_color: { r:'0' g:'0' b:'0' }
                red: default_color { r:'1' }

            "#).expect("Parsing Error"),

            compound(vec![
                ("default_color", compound(vec![
                    ("r", literal("0")),
                    ("g", literal("0")),
                    ("b", literal("0")),
                ])),

                ("red", compound(vec![
                    ("r", literal("1")),
                    ("g", literal("0")),
                    ("b", literal("0")),
                ]))
            ])
        );

        assert_eq!(
            FlatObject::parse(r#"
                ok_text: 'Ok'

                Button: {
                    visible: 'true'
                    text: 'Click Here'
                }

                ok_button: Button { text: ok_text }

            "#).expect("Parsing Error"),

            compound(vec![
                ("ok_text", literal("Ok")),

                ("Button", compound(vec![
                    ("visible", literal("true")),
                    ("text", literal("Click Here")),
                ])),

                ("ok_button", compound(vec![
                    ("visible", literal("true")),
                    ("text", literal("Ok")),
                ]))
            ])
        );


        assert_eq!(
            FlatObject::parse(r#"
                text: {
                    cancel: {
                        german: 'Abbrechen'
                    }
                }

                cancel_button: {
                    text: text.cancel.german
                }

            "#).expect("Parsing Error"),

            compound(vec![
                (("text"), compound(vec![
                    (("cancel"), compound(vec![
                        (("german"), literal("Abbrechen")),
                    ])),
                ])),

                (("cancel_button"), compound(vec![
                    (("text"), literal("Abbrechen")),
                ])),
            ])
        );
    }
}