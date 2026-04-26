# style-rust-aku

A collection of custom, opinionated Rust style lints built using [Dylint](https://github.com/trailofbits/dylint).

## Available Lints

| Lint | Description | Auto-fixable |
|------|-------------|:---:|
| **`prefer_collect_turbofish`** | Enforces the use of turbofish syntax for `Iterator::collect()` instead of explicit `let` type annotations.<br><br>**Bad:** `let x: Vec<u32> = iter.collect();`<br>**Good:** `let x = iter.collect::<Vec<u32>>();` | Yes |
| **`literal_suffix`** | Enforces the use of suffixed numeric literals over explicit type annotations.<br><br>**Bad:** `let x: f32 = 0.0;`<br>**Good:** `let x = 0.0f32;` | Yes |
| **`minimal_imports`** | Prevents deeply nested, fully-qualified inline paths (>= 3 segments) and suggests bringing them into scope with `use`.<br><br>**Bad:** `let x: std::io::Result<()> = ...`<br>**Good:** `use std::io; let x: io::Result<()> = ...` | No |

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

To apply auto-fixes for `prefer_collect_turbofish` and `literal_suffix`:

```sh
cargo dylint --all --fix
```

## Usage in Other Projects

To enforce these lints in your own Rust projects, you need to configure your workspace to pull in this Dylint library.

Add the following to your project's root `Cargo.toml` (or `dylint.toml`):

```toml
[workspace.metadata.dylint]
libraries = [
    # Use a relative/absolute path if working locally:
    # { path = "../style-rust-aku" }
    
    # Or pull directly from git:
    { git = "https://github.com/YOUR_GITHUB_USERNAME/style-rust-aku", branch = "main" }
]
```

Then, run `cargo dylint --all` in your project's directory.

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
