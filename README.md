# PROTO-TEMPLATES

Proto-Templates aims to be a visually minimalist but feature-rich markup language.
It could be used in places where HTML, JSON, or similar languages are used.
In contrast to HTML, Proto-Templates aims to 
reduce visual complexity and duplication of information.
To reduce redundancy, prototypal inheritance and variables are introduced.

This repository holds the Specification of Proto-Templates, 
and also implements it using Rust.


### Features
- prototypal inheritance
- polymorphic overriding of inherited properties
- variables (by inheriting without overriding)
- prototype instantiation with parameters

See the [specification](https://github.com/johannesvollmer/proto-templates/blob/master/SPECIFICATION.md)
for a more detailed description of the language.


### Examples

```
persons: {
    peter: {
        real_name: "Peter Parker"
        title: "Spiderman"
    }
    
    clark: {
        real_name: "Clark Kent"
        title: "Superman"
    }
}

Comic: {
    title: "Untitled",
    published: "true",
    author: {
        name: "unknown",
        age: "unknown"
    }
}

comics: {
    spiderman: Comic {
        title: "The Amazing Spiderman" // override the title
        protagonist: persons.peter // add a new property
        // inherits published: "true" and author-values
    }
    
    superman: Comic {
        title: "The Amazing Spiderman" // override the title
        protagonist: persons.clark // add a new property
        // inherits published: "true" and author-values
    }
}

```

See the [examples directory](https://github.com/johannesvollmer/proto-templates/blob/master/assets) 
for more examples.