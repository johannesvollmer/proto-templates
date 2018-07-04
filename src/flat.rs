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
        for (override_identifier, override_index) in &objects.names {
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

        if !compound.prototype.name.is_empty() {
            // insert all inherited properties, if not already overridden
            if let Object::Compound(ref compound) = *world.find_by_identifier(&compound.prototype)
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
                // 'inline' the special case of 'variables' being a prototype without overrides
                if compound.overrides.objects.is_empty() && !compound.prototype.name.is_empty() {
                    let prototype = world.find_by_identifier(&compound.prototype)
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

    #[test]
    fn test_build_flat_object(){
        assert_eq!(
            FlatObject::parse(" color: ' #abc ' ").expect("Parsing Error"),
            FlatObject::Compound(
                vec![
                    (String::from("color"), FlatObject::StringLiteral(String::from(" #abc ")))
                ].into_iter().collect()
            )
        );

        assert_eq!(
            FlatObject::parse(r#"

                default_color: { r:'0' g:'0' b:'0' }
                red: default_color { r:'1' }

            "#).expect("Parsing Error"),

            FlatObject::Compound(
                vec![
                    (String::from("default_color"), FlatObject::Compound(
                        vec![
                            (String::from("r"), FlatObject::StringLiteral(String::from("0"))),
                            (String::from("g"), FlatObject::StringLiteral(String::from("0"))),
                            (String::from("b"), FlatObject::StringLiteral(String::from("0"))),
                        ].into_iter().collect()
                    )),

                    (String::from("red"), FlatObject::Compound(
                        vec![
                            (String::from("r"), FlatObject::StringLiteral(String::from("1"))),
                            (String::from("g"), FlatObject::StringLiteral(String::from("0"))),
                            (String::from("b"), FlatObject::StringLiteral(String::from("0"))),
                        ].into_iter().collect()
                    ))
                ].into_iter().collect()
            )
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

            FlatObject::Compound(
                vec![
                    (String::from("ok_text"), FlatObject::StringLiteral(String::from("Ok"))),

                    (String::from("Button"), FlatObject::Compound(
                        vec![
                            (String::from("visible"), FlatObject::StringLiteral(String::from("true"))),
                            (String::from("text"), FlatObject::StringLiteral(String::from("Click Here"))),
                        ].into_iter().collect()
                    )),

                    (String::from("ok_button"), FlatObject::Compound(
                        vec![
                            (String::from("visible"), FlatObject::StringLiteral(String::from("true"))),
                            (String::from("text"), FlatObject::StringLiteral(String::from("Ok"))),
                        ].into_iter().collect()
                    ))


                ].into_iter().collect()
            )
        );
    }
}