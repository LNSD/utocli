# utocli-derive

Procedural macros for the `utocli` library, providing derive macros for automatic OpenCLI schema generation.

This crate is the equivalent of `utoipa-gen` but for OpenCLI specifications instead of OpenAPI.

## Features

- **`#[derive(ToSchema)]`**: Automatically generate OpenCLI schema definitions from Rust types
- **`#[derive(ToParameter)]`**: Generate CLI parameter definitions (WIP)
- **`#[derive(ToResponse)]`**: Generate CLI response definitions (WIP)

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
utocli = "0.0.0"
```

Then derive schemas from your types:

```rust
/// A user in the system
#[derive(utocli::ToSchema)]
struct User {
    /// The user's unique identifier
    id: u64,
    /// The user's name
    name: String,
    /// Optional email address
    email: Option<String>,
}

/// User status
#[derive(utocli::ToSchema)]
enum Status {
    Active,
    Inactive,
    Pending,
}
```

### Schema Attributes

#### Container attributes (`#[schema(...)]`)

- `description = "..."` - Override the description from doc comments
- `title = "..."` - Set a custom title for the schema
- `rename_all = "..."` - Rename all fields (e.g., "camelCase", "snake_case")

#### Field attributes (`#[schema(...)]`)

- `description = "..."` - Override field description
- `rename = "..."` - Rename this specific field
- `format = "..."` - Specify the schema format
- `skip` - Skip this field in the generated schema

### Serde Compatibility

The derive macros respect common serde attributes:

- `#[serde(rename = "...")]` - Rename fields
- `#[serde(skip)]` - Skip fields
- `#[serde(rename_all = "...")]` - Rename all fields in a container

## Roadmap

This crate is based on the architecture of `utoipa-gen` and aims to provide similar functionality for OpenCLI specifications.

### Implemented

- [x] Basic `ToSchema` derive macro
- [x] Doc comment parsing
- [x] Field-level attributes
- [x] Enum support
- [x] Serde compatibility

### Planned

- [ ] Advanced type inference for field schemas
- [ ] `ToParameter` derive macro
- [ ] `ToResponse` derive macro
- [ ] Generic type support
- [ ] Nested schema references
- [ ] More validation attributes (min, max, pattern, etc.)

## License

Apache-2.0
