# PROTO-TEMPLATES

## Specification

Proto-Templates can be used to describe data. It supports 
computing values based on existing data, but will never modify existing data.

All information in this language is represented in atomic string literals,
which can be structured using composition. Unlike JSON,
it does not support various primitive types like numbers or booleans. 
Composition however can be used like JSON objects and arrays at the same time. 

Example of polymorphic data:
```

// possibly defined by the system which processes the data, like a prelude:
default_header: {
    title: "Peter's Website"
    description: "This is my Website."
}

// actual document content:
page_1_header: default_header {
    title: "About Peter"
    // inherits the description
}
```

---

The language is completely invariant to whitespace, 
except inside string literals of course.

__Let's look at the syntax of Proto-Templates using 
a custom mixture based on regular expressions and regular grammars:__

1.  The smallest bit of information is a string literal.

    `string_literal` → `" ([^"] | (\"))* "`
    
    String literals can contain any character 
    except their enclosing delimiters, namely `"`.
    If you need a `"` inside a string, prefix it with a back-slash: `\"`.
    
    Examples: `"Dr. Dogmeyer"`, `"He asked him: \"Who's a good boy?\" "`.

    
2.  String literals can be structured using objects and compositions.

    `object` → `name: (string_literal | composition)`
    
    An object has a name, always followed by a colon, 
    and is associated either a string literal or a composition.
    
    Examples: `color: "#beef24"`, `animal_type: "Kitten"`.

2.  Compositions are used to group multiple objects.
    
    `composition` → `prototype? ({  object*  })?  `
    
    They can have a prototype, which they will inherit all properties from.
    In curly braces however, they can override any property of that prototype,
    and define new properties. Compositions without overrides are basically a 
    copy of the prototype, so you can use it like variables.
    
    Examples of compositions: `{}`, `main_color`, `default_theme { primary_color: salmon }`.
    Examples of objects: `protagonist: { name: "Peter Parker" }`, `photographer_name: protagonist.name`.
    
2.  Compositions can have their prototype refer to any other object in the document, 
    or default objects defined by the system processing the information.
    
    `prototype` → `name (.name)*`
    
    Members of prototypes can be accessed using a `.`, like in many other languages.
    
    Examples: `main_color`, `main_color.r`, `post.author.name`.
        
2.  Objects need a name, in order to be looked up 
    by the system processing the information.
    
    `name` → `[^( {}:. )]`
    
    A name is any combination of characters, 
    except the ones which already have a special meaning, 
    namely `{`, `}`,`:`,  and `.`.
    
    This means that identifiers can be numbers, which corresponds to a JSON array. 
    
    A name can also be empty, which in the future may be used to avoid 
    manual indexing by implicitly assigning indices to unnamed objects.

    Note: As the specification changes, 
    additional symbols may be declared forbidden, such as `+`,`-`,`*`,`/`, or `&`.
        

2.  Finally, a document may contain any number of objects.

    `document` → `(object)*`




## Features to think about:
-   Import other documents
-   Simple calculations and collection operations,
    like appending to the prototypes collection. 
    Maybe ternary operators and comparison checks.
-   Parameters for more complex prototypes
-   Consider always having names in quotes, to enable more complex names.
