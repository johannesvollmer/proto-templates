# PROTO-TEMPLATES

## Specification

The largest unit in this language is an object. 
Objects are a name, followed by a semicolon, 
followed by either string literals, 
or a prototype with overridden properties.


Example of a string literal: 
```
name: "peter"
```

To inherit from a prototype, the colon is followed by the prototype's name, 
and optionally some curly braces. These braces can contain properties which
will override the properties with the same name of the prototype 
(or add a new value if the prototype does not have a property with that name).


Example of an object with a prototype and overridden properties:
```
page_header: Header {
    title: "Peter's Website"
    description: "This is my Website."
}
```


Example of an polymorphic data:
```
default_header: {
    title: "Peter's Website"
    description: "This is my Website."
}

page_1_header: default_header {
    title: "About Peter"
}
```