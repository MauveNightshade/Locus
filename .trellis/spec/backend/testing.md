# Testing

> Test organization, patterns, and conventions for the Rust backend.

---

## Test Location

Tests live **in the same file** as the code they test, under a `#[cfg(test)]` module:

```rust
// In some_module.rs

// ... implementation ...

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specific_behavior() {
        // ...
    }
}
```

Reference: `src-tauri/src/tool/builtins/mod.rs:202-228` (`should_skip_generated_root_entry` tests)

This is the standard Rust convention (unit tests colocated with source). Integration tests go in `src-tauri/tests/` or are marked as `selftest`.

---

## Selftest Modules

For tests that need more setup or test integration between multiple modules, the project uses `selftest` modules:

```
src-tauri/src/
├── unity_type_index_selftest.rs   ← tests for unity_type_index
├── unity_bridge/
│   ├── native_selftest.rs         ← tests for native bridge
│   └── state_probe/
│       └── selftest.rs            ← tests for state probe
├── unity_hotreload/
│   └── selftest.rs                ← tests for hot reload
└── unity_yaml/
    └── binding_tests.rs           ← tests for YAML bindings
```

**Naming convention:** Use `selftest` for integration-level tests that exercise cross-module behavior. Use `*_tests` suffix (e.g., `binding_tests.rs`) for focused module tests extracted to a separate file for size reasons.

These are declared as `pub mod` in `lib.rs` so they're compiled during `cargo test`:

```rust
pub mod unity_type_index_selftest;
```

Reference: `src-tauri/src/lib.rs:60`

---

## Dev Dependencies

```toml
[dev-dependencies]
tempfile = "3"
```

`tempfile` is the only dev-dependency. Use it for tests that need temporary directories/files. For tests that don't need the filesystem, prefer pure in-memory test data.

---

## Test Patterns

### Pure Unit Test (most common)

```rust
#[test]
fn generated_root_entry_detection_is_root_scoped() {
    let root = Path::new("C:/Project");
    assert!(should_skip_generated_root_entry(root, Path::new("C:/Project/Library/Artifacts")));
    assert!(!should_skip_generated_root_entry(root, Path::new("C:/Project/Assets/Scripts/BuildPipeline")));
}
```

Reference: `src-tauri/src/tool/builtins/mod.rs:207-227`

### Table-Driven Test

When testing multiple input/output pairs:
```rust
#[test]
fn test_multiple_cases() {
    let cases = vec![
        (input1, expected1),
        (input2, expected2),
    ];
    for (input, expected) in cases {
        assert_eq!(function_under_test(input), expected);
    }
}
```

---

## Running Tests

```bash
# All tests
cargo test

# Specific module
cargo test unity_csharp::tests

# Specific test
cargo test generated_root_entry_detection_is_root_scoped

# With output
cargo test -- --nocapture
```

---

## What to Test

| Scenario | Test type |
|----------|-----------|
| Parsing logic (tree-sitter, YAML) | Unit test in same file with sample input |
| Error handling paths | Unit test with malformed input |
| Serialization round-trip | Unit test: serialize → deserialize → assert equal |
| Cross-module integration (bridge, hot reload) | Selftest module |
| C# parser node layout changes | Test in `unity_csharp/tests.rs` — critical after tree-sitter bumps |

## Scenario: Windows native TLS transport regression

### 1. Scope / Trigger

- Trigger: a provider returns a parser or schema error for a request already
  proven valid before it reaches `reqwest`.
- Scope: Windows builds that resolve `reqwest` through `native-tls` and
  `schannel`; this is a dependency-resolution and transport investigation, not
  a prompt or JSON-shape change.

### 2. Signatures

- Request construction remains the existing provider request path (for example,
  `network::reqwest_client`); do not introduce a provider-specific transport
  wrapper or retry signature to mask a corrupted write.
- The locked dependency contract is `reqwest -> native-tls -> schannel`, with
  `schannel` at a version containing the required transport fix.

### 3. Contracts

- A captured reproduction must use the configured official endpoint and the
  original Locus Rust network path.
- Test artifacts may record body length and SHA-256, status, protocol, and
  resolved dependency versions. They must not contain API credentials or raw
  request bodies.

### 4. Validation & Error Matrix

| Condition | Required conclusion |
| --- | --- |
| Same valid bytes fail in Locus but succeed in an independent client | Investigate transport/dependency behavior before changing serialization. |
| Same Rust client and lockfile flips with only the TLS dependency version | Treat the lockfile dependency as the root-cause boundary. |
| Locked build resolves the corrected TLS dependency and formal in-app replay succeeds | Accept the dependency correction; no provider workaround is needed. |

### 5. Good/Base/Bad Cases

- Good: replay the production-shaped tool follow-up through Locus and an
  independent client, then run an A/B dependency check when outcomes differ.
- Base: run `cargo check --locked --bin locus` and `cargo tree -i schannel@<version>`.
- Bad: infer malformed JSON from an upstream parser error without comparing the
  exact emitted bytes and a non-Locus client.

### 6. Tests Required

- Compile the locked application dependency graph with `cargo check --locked --bin locus`.
- Inspect the resolved TLS dependency with `cargo tree -i schannel@<version>`.
- For a live provider regression, preserve a credential-free evidence note with
  the independent-client control and formal application outcome.

### 7. Wrong vs Correct

#### Wrong

Add a DeepSeek-only retry or alter a valid JSON request after seeing a 400 parser
response.

#### Correct

First compare the exact body through the original Rust path and an independent
client. When the result is isolated to the TLS dependency, update the lockfile
and validate the rebuilt application path.

---

## Anti-Patterns

- **Don't create `tests/` directories** inside `src/` — use `#[cfg(test)] mod tests` inline or a top-level `*_selftest.rs`.
- **Don't skip tests with `#[ignore]` without a comment explaining why** and when they should be re-enabled.
- **Don't use real Unity projects in unit tests** — use minimal YAML/C# snippets inline.
- **Don't commit `cargo test` failures** — the CI runs `cargo test` and failing tests block merge.
