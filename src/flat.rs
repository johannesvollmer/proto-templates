use ::std::collections::HashMap;


enum FlatObject {
    StringLiteral(String),
    Compound(HashMap<String, FlatObject>),
}

impl FlatObject {
    /*pub fn parse(world: &::parse::Object) -> FlatObject {
        use ::parse::*;

        match world {
            Object::StringLiteral(literal) => {
                FlatObject::StringLiteral(literal.to_string())
            },

            Object::Compound(compound) => {
                let mut properties = HashMap::new();

                for property in &compound.overrides {
                    let name = property.name.to_string();
                    if !properties.contains(&name) {
                        properties.insert(name, FlatObject::parse(property.value)); // TODO handle references
                    }
                }

                // replace all properties that have not been defined by overrides yet
                FlatObject::parse_prototype_into(properties, lookup(compound.prototype_identifier));
                FlatObject::Compound(properties)
            }
        }
    }*/
}