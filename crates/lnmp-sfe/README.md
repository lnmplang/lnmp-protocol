# lnmp-sfe

Semantic Fidelity Engine for LNMP (LLM Native Minimal Protocol).

This crate provides the semantic dictionary system that maps field IDs to human-readable names and provides equivalence mappings for semantic normalization.

## Features

- **Semantic Dictionary**: Maps field IDs to human-readable names
- **Equivalence Mappings**: Define synonym relationships for semantic normalization
- **YAML Support**: Load dictionaries from YAML files

## Usage

```rust
use lnmp_sfe::SemanticDictionary;

// Load dictionary from YAML file
let dict = SemanticDictionary::load_from_file("dictionary.yaml")?;

// Get field name
if let Some(name) = dict.get_field_name(12) {
    println!("Field 12 is: {}", name);
}

// Get equivalence mapping
if let Some(canonical) = dict.get_equivalence(7, "yes") {
    println!("'yes' maps to: {}", canonical);
}
```

## Dictionary Format

```yaml
fields:
  12:
    name: user_id
    type: integer
  7:
    name: is_active
    type: boolean
    equivalences:
      yes: "1"
      true: "1"
      no: "0"
      false: "0"
  23:
    name: roles
    type: string_array
    equivalences:
      admin: administrator
      dev: developer
```

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
