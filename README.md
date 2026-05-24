# style-rust-aku

A collection of custom, opinionated Rust style lints built using [Dylint](https://github.com/trailofbits/dylint).

## Available Lints

| Lint | Description | Rationale | Auto-fixable |
|------|-------------|-----------|:---:|
| **`prefer_collect_turbofish`** | Enforces the use of turbofish syntax for `Iterator::collect()` instead of explicit `let` type annotations.<br><br>**Bad:** `let x: Vec<u32> = iter.collect();`<br>**Good:** `let x = iter.collect::<Vec<u32>>();` | Keeps the destination type next to the operation that produces it, instead of splitting it across the `let` binding and initializer. | Yes |
| **`literal_suffix`** | Enforces the use of suffixed numeric literals over explicit type annotations.<br><br>**Bad:** `let x: f32 = 0.0;`<br>**Good:** `let x = 0.0f32;` | Keeps primitive numeric type information on the literal itself and removes redundant local annotations. | Yes |
| **`minimal_imports`** | Prevents deeply nested, fully-qualified inline paths (>= 3 segments) and suggests bringing them into scope with `use`.<br><br>**Bad:** `let x: std::io::Result<()> = ...`<br>**Good:** `use std::io; let x: io::Result<()> = ...` | Avoids noisy inline paths while still keeping enough context at the use site. | No |
| **`prefer_vec_macro`** | Enforces the use of `vec![]` over `Vec::new()` unless turbofish is used like `Vec::<u32>::new()`.<br><br>**Bad:** `let mut validators = Vec::new();`<br>**Good:** `let mut validators = vec![];` | Keeps empty vector construction visually aligned with non-empty `vec![...]` construction. This is a style lint, not an empty-vector performance lint. | Yes |
| **`tracing_macro_imports`** | Warns against calling `tracing` macros through a qualified path and suggests importing them instead.<br><br>**Bad:** `tracing::info!("hello");`<br>**Good:** `use tracing::info; info!("hello");` | Keeps `tracing` macro calls concise by importing macros into scope, matching the crate's intended usage pattern. | No |

## Behavior Notes

These lints are intentionally conservative around generated code:

- `minimal_imports` skips macro/proc-macro generated paths, root-marker paths, and underscore-prefixed helper paths such as `_serde::...`.
- `prefer_vec_macro` flags typed and mutable local initializers such as `let v: Vec<u32> = Vec::new();` and `let mut values = Vec::new();`, but still skips explicit turbofish calls such as `Vec::<u32>::new()`.
- `tracing_macro_imports` skips macro-generated and external-macro invocations to avoid false positives in generated code. It covers all standard `tracing` macros (`trace`, `debug`, `info`, `warn`, `error`, `event`, `span`, and their `*_span` variants, plus `enabled`).
- In Rust's standard library source, empty `vec![]` expands directly to `Vec::new()`. The special internal path using `write_box_via_move` applies to non-empty `vec![a, b, ...]` construction to improve stack usage for unoptimized programs constructing large vectors: <https://doc.rust-lang.org/src/alloc/macros.rs.html#42-61>.

## Prerequisites

Before running these lints, you must install the `dylint` cargo commands:

```sh
cargo install cargo-dylint dylint-link
```

## Running Locally (In this repo)

To test the lints against this workspace:

```sh
cargo dylint --all
```

To apply auto-fixes for `prefer_collect_turbofish`, `literal_suffix`, and `prefer_vec_macro`:

```sh
cargo dylint --all --fix
```

## Usage in Other Projects

To enforce these lints in your own Rust projects, you need to configure your workspace to pull in this Dylint library.

For local development, point the consuming project at this checkout:

```toml
[workspace.metadata.dylint]
libraries = [
    { path = "../style-rust-aku" }
]
```

Or pull the lints from Git:

```toml
[workspace.metadata.dylint]
libraries = [
    { git = "https://github.com/zeon256/style-rust-aku", branch = "main" }
]
```

Then, run `cargo dylint --all` in your project's directory.

To test a local checkout without changing the consuming project's `Cargo.toml`, run:

```sh
cargo dylint --path ../style-rust-aku --all
```

## VS Code Integration

To get real-time squiggly lines in your editor for these custom lints, configure `rust-analyzer` to use `dylint` instead of standard `cargo check`.

Add the following to your project's `.vscode/settings.json`:

```json
{
    "rust-analyzer.check.overrideCommand": [
        "cargo",
        "dylint",
        "--all",
        "--",
        "--all-targets",
        "--message-format=json"
    ]
}
```

## Writing New Lints

1. Create a new module in `src/` (e.g., `src/my_lint.rs`).
2. Implement the `LateLintPass` using `clippy_utils`.
3. Register the module and lint pass in `src/lib.rs` inside `register_lints`.
4. Create a UI test in the `ui/` folder.
5. Run `DYLINT_BLESS=1 cargo test` to generate/update the `.stderr` test reference files.
