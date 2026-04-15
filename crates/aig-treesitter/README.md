# aig-treesitter

Tree-sitter-based semantic diff engine for aig. Library only — no binary.

Parses source code into ASTs, extracts top-level symbol definitions, and computes semantic diffs (added/modified/removed symbols) between two versions of a file.

## Supported Languages (11)

| Language | Grammar Crate | Definition Types |
|----------|--------------|-----------------|
| TypeScript/JS | `tree-sitter-typescript` | functions, classes, interfaces, type aliases, methods |
| Python | `tree-sitter-python` | functions, classes |
| Rust | `tree-sitter-rust` | functions, structs, enums, impls, traits, types |
| Go | `tree-sitter-go` | functions, methods, types |
| Java | `tree-sitter-java` | classes, interfaces, methods, constructors, enums |
| C# | `tree-sitter-c-sharp` | classes, interfaces, methods, structs, enums, constructors |
| C++ | `tree-sitter-cpp` | functions, classes, structs, enums, templates |
| Ruby | `tree-sitter-ruby` | methods, classes, modules |
| PHP | `tree-sitter-php` | functions, methods, classes, interfaces, traits, enums |
| Kotlin | `tree-sitter-kotlin-ng` | functions, classes, objects |
| Swift | `tree-sitter-swift` | functions, classes, protocols, type aliases |

Unsupported languages fall back to line-based diffing in `aig-core` — no errors.

## Public API

```rust
// Detect language from file extension
let lang = aig_treesitter::detect_language("src/auth.py");

// Extract top-level definitions (functions, classes, etc.)
let defs = aig_treesitter::extract_definitions(source_code, lang)?;

// Compare two versions and get semantic changes
let changes = aig_treesitter::semantic_diff(old_source, new_source, lang)?;
// -> Vec<SemanticChange> with change_type, symbol_name, details
```

## Adding a new language

1. Add the `tree-sitter-<lang>` grammar crate to `Cargo.toml`
2. Add a variant to the `Language` enum
3. Add file extension mapping in `detect_language()`
4. Add parser wiring in `get_parser()`
5. Add definition kind mappings in `definition_node_kinds()`
6. Add an integration test in `crates/aig-core/tests/integration.rs`
