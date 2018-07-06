# PROTO-TEMPLATES: TODO

## Specification
- [x] More Examples in the Specification

## Implementation
- [ ] Change implementation to quotes 
      instead of apostrophes for string literals!
- [x] Implement FlatObjects 
- [ ] Implement ReferenceObjects 
- [ ] Streaming and Zero-Copy parsing variants
- [ ] FlatObjects and ReferenceObjects should implement an interface
      which also allows for more complex queries like 
      `document.find_where(|obj| obj.get("name") == "peter")`