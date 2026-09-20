# Port satay-rs ast-grep test-hygiene lints as Dylint lints

## Context

satay-rs enforces two test-hygiene rules via ast-grep (`satay-rs/.config/ast-grep/rules/no-inline-modules.yml`, `no-tests-outside-test-files.yml`). The user wants the same rules enforced from this Dylint library (`style-rust-aku`) instead, so they fire inside `cargo dylint` / rust-analyzer check on any consuming workspace. Scope: add the two lints here only; satay-rs cutover is a separate follow-up (explicitly out of this plan). Per user decision: **generic port, no satay-rs-specific `ignores`** — drop `crates/satay-oas3/**`; keep the generic file-pattern allowlist (`tests.rs`, `*_tests.rs`, `tests/**`).

## Approach

Both rules are purely syntactic, so both become `EarlyLintPass`es registered **pre-expansion** (mirrors `tracing_macro_imports` at `src/lib.rs:40` — pre-expansion passes see the raw AST including `#[cfg(...)]` attributes before cfg-stripping/macro expansion, which is exactly ast-grep's semantics). Shared matcher helper lives in the first module and is `pub(crate)`-reused by the second. No new dependencies.

### Step 0 — Feasibility probe (do first)

Write a throwaway early lint pass in a scratch branch (or the real module) and a UI fixture containing `#[cfg(test)] mod tests { fn helper() {} }`. Run the UI test. This proves pre-expansion passes see cfg-gated modules in a normal (non-`--test`) build.

**If the module is cfg-stripped before the pass runs** (fixture produces no warning): stop and report — the cfg-gated branch cannot be ported faithfully at the rustc level; the fallback is implementing the lint on the ungated branch only (`fn` with test attribute inside an inline module, which is cfg-independent) and documenting the divergence in the README. Do not silently proceed.

### Step 1 — `src/no_inline_modules.rs`

Declare `NO_INLINE_MODULES` (`Warn`, description: `"Inline test module bodies are not allowed"`) and `declare_lint_pass!(NoInlineModules => [NO_INLINE_MODULES])`. Copy the `declare_lint!`/pass-structure convention from `src/prefer_vec_macro.rs` (doc comment "### What it does", `Warn` level, `declare_lint_pass!`).

Implement `impl EarlyLintPass for NoInlineModules { fn check_item(...) }`:

1. Skip unless `item.kind` is `ItemKind::Mod(_, ModKind::Loaded(..))`. (`ModKind::Loaded(ThinVec<P<Item>>, Inline, Span)` — third field is the braces span; `ModKind::Unloaded` = `mod x;`, allowed.) Read the exact `ItemKind`/`ModKind` shape from the pinned rustc source before writing (nightly-2026-07-09; check `rustc_ast::ast` in the toolchain's rustc-dev rdocs or `rustup doc` — signatures at that rev are the authority).
2. Skip if `item.span.from_expansion() || item.span.in_external_macro(cx.sess().source_map())` — same generated-code guard convention as `src/tracing_macro_imports.rs:20-23`. (`is_from_proc_macro` is LateContext-only; do not use it here.)
3. Condition A — cfg-test-gated: any of `item.attrs` where the attribute path's last segment is `cfg` (r#-stripped) and its `meta_item_list()` contains a positive `test` predicate:
   ```rust
   fn positive_test_ident(list: &[NestedMetaItem]) -> bool {
       list.iter().any(|nmi| match nmi.meta_item() {
           Some(mi) => {
               let last = mi.path.segments.last().unwrap().ident;
               let name = last.name.as_str().strip_prefix("r#").unwrap_or(last.name.as_str());
               if name == "test" && !mi.list_args().is_some() && !mi.name_args().is_some() { true }
               else if name == "not" { false }              // negated subtree: skip entirely
               else { mi.meta_item_list().is_some_and(positive_test_ident) }
           }
           None => false,
       })
   }
   ```
   This mirrors the yaml's `test-cfg-attribute` + `positive-test-identifier` utils: bare `test` positive; `all(...)`/`any(...)` recurse; `not(...)` subtrees excluded; string values like `feature = "test-support"` are `MetaItem` named args, never bare `test` idents. `cfg_attr(...)` does NOT match (path last segment is `cfg_attr`, not `cfg`) — matches the yaml, which matches only `cfg`.
4. Condition B — ungated inline module whose **direct** items include a test-attribute fn: `items.iter().any(|child| child.attrs.iter().any(is_test_attr))` (direct children only, matching the yaml's `stopBy: neighbor` on the declaration_list `has`).
5. Fire when A || B: `clippy_utils::diagnostics::span_lint_and_sugg(cx, NO_INLINE_MODULES, braces_span, "inline test module bodies are not allowed; move the body to the standalone module file and preserve this module's attributes and visibility", "replace the module body with", ";", Applicability::MaybeIncorrect)` where `braces_span` is the `ModKind::Loaded` third field. `MaybeIncorrect` is required — the suggestion deletes the body, only correct after the body has been moved by hand.

`is_test_attr` (pub(crate), shared with step 2): `attr.path.segments.last()` ident name r#-stripped ∈ {test, rstest, test_case, parameterized, test_with, wasm_bindgen_test, quickcheck, proptest, test_matrix, test_each_path} — the last-segment rule handles scoped forms (`#[tokio::test]` → last segment `test`) exactly as the yaml's scoped_identifier branch; companion-only attrs (`#[serial]`, `#[fixture]`, `#[should_panic]`) do not match.

### Step 2 — `src/no_tests_outside_test_files.rs`

Declare `NO_TESTS_OUTSIDE_TEST_FILES` (`Warn`), same pass structure. `EarlyLintPass` (pre-expansion), implement:

- `check_item` for `ItemKind::Fn(..)` and `check_impl_item` / `check_trait_item` for `AssocItemKind::Fn(..)` — these callbacks expose the item's `attrs` directly at all three syntactic positions (free fn, inherent impl method, trait method). Do not use `check_fn`/`FnKind` — its attribute access is version-dependent; the three item callbacks are deterministic. Functions nested inside fn bodies are intentionally out of scope (ast-grep equally matches none inside macro token trees; nested-fn tests are pathological).
- Skip if `from_expansion() || in_external_macro(...)` as in step 1.
- Fire if any attr matches `is_test_attr` AND the fn's file is not on the allowlist:
  ```rust
  fn is_test_file(filename: &FileName) -> bool {
      let FileName::Real(real) = filename else { return false };
      let Some(path) = real.local_path() else { return false };
      let name = path.file_name().and_then(|n| n.to_str());
      (name == Some("tests.rs") || name.is_some_and(|n| n.ends_with("_tests.rs")))
          || path.ancestors().skip(1).any(|a| a.file_name().is_some_and(|n| n == "tests"))
  }
  ```
  Get the filename: `cx.sess().source_map().span_to_filename(item.span)`. Virtual/remapped paths (no `local_path()`) → not a test file → **skip the lint** (no local path means we cannot apply the rule faithfully; never guess). Ancestor check via `Path::ancestors().skip(1)` covers `**/tests/**/*.rs` (any dir component named `tests`, anywhere in the tree) — the `skip(1)` drops the file's own name so a file literally named `tests` in place of the component isn't confused with the dir.
- Emit: `clippy_utils::diagnostics::span_lint_and_help(cx, NO_TESTS_OUTSIDE_TEST_FILES, item.span, "Rust test functions belong in `tests.rs`, `*_tests.rs`, or `tests/**/*.rs`; move this test to one of those standalone test files", None, None)`. ast-grep offers no fix for this rule; help-only is the faithful port.

### Step 3 — Register in `src/lib.rs`

- Add `pub mod no_inline_modules;` and `pub mod no_tests_outside_test_files;` next to the existing module declarations (lib.rs:23-27).
- Add both `::NO_INLINE_MODULES` and `::NO_TESTS_OUTSIDE_TEST_FILES` to the `register_lints` list (lib.rs:33-39).
- Add `lint_store.register_pre_expansion_lint_pass(Box::new(|| Box::new(no_inline_modules::NoInlineModules)));` and the same for the second pass, next to lib.rs:40 (existing pre-expansion registration style).

### Step 4 — UI fixtures (`ui/`)

`dylint_testing::ui_test` compiles each `ui/*.rs` standalone — fixtures must not depend on external crates (an unexpandable attribute macro like `#[rstest]` is a compile error there), so use plain `#[test]`/`#[cfg(...)]` only.

- `ui/no_inline_modules.rs`:
  - invalid: `#[cfg(test)] mod tests { fn helper() {} }`; `#[cfg(all(test, feature = "json"))] mod tests { fn helper() {} }` (the `feature = "json"` token must be inert here — it's a value, fine); ungated `mod checks { #[test] fn catches() {} }` (this fires both this lint and NO_TESTS_OUTSIDE_TEST_FILES on the inner fn — expected, stderr shows both).
  - valid (no warnings, must be in the same file so the single .stderr proves their silence): `pub mod serde_string { pub fn serialize() {} }`, `#[cfg(test)] mod tests;`, `#[cfg(not(test))] mod production { fn helper() {} }`, `#[cfg(feature = "test-support")] mod support { fn helper() {} }`.
- `ui/no_tests_outside_test_files.rs`:
  - invalid: top-level `#[test] fn t() {}`; `#[should_panic] #[test] fn t2() {}` (companion attr must not double-fire); impl method `struct S; impl S { #[test] fn m(&self) {} }` — this fixture validates that the assoc-item callbacks fire pre-expansion (see Step 0 fallback note: if the impl case is missing from the blessed stderr, the pre-expansion driver isn't visiting assoc items — then register the assoc-item checks additionally via `register_early_lint_pass` with a dedup guard, i.e. skip items whose span `from_expansion()` and skip files already covered... simplest concrete fallback: keep the single pre-expansion pass and document in README that only module-level fns are covered; do not double-register without confirming non-overlap).
- Generate/refresh both `.stderr` files with `DYLINT_BLESS=1 cargo test` (env var documented in README:95-105).

### Step 5 — README

Add one row per lint to the table in `README.md:5-13` (descriptions mirroring the yaml messages; Auto-fixable: Yes for `no_inline_modules` — the `;` sugg; No for `no_tests_outside_test_files`) and a one-line behavior note about pre-expansion semantics: both lints inspect raw source pre-cfg-strip, so gated modules are flagged even in non-test builds, and attribute-macro tests (`#[tokio::test]` etc.) are matched on their written attribute, not their expansion.

## Critical files & anchors

- `src/lib.rs:23-45` — module declarations + `register_lints` + pre-expansion registration; the tracing pass registration at :40 is the exact pattern to copy.
- `src/prefer_vec_macro.rs:10-18` — `declare_lint!` structure convention.
- `src/tracing_macro_imports.rs:18-25` — `EarlyLintPass` + span guard convention.
- `satay-rs/.config/ast-grep/rules/no-inline-modules.yml` — reference semantics (utils, valid/invalid cases) for the port; read before coding step 1.
- `satay-rs/.config/ast-grep/rule-tests/no-inline-modules-test.yml` — the fixture matrix (valid/invalid pairs) mirrored into `ui/no_inline_modules.rs`.

## Verification

1. Library builds under the pinned nightly: `cargo build` in `style-rust-aku` (rust-toolchain pins nightly-2026-07-09; dylint-link linker configured via `.cargo/config.toml`).
2. UI tests: `DYLINT_BLESS=1 cargo test` then plain `cargo test` — both new `.stderr` files green; the no_inline_modules fixture's valid cases (e.g. `#[cfg(not(test))] mod production`) appear with zero warnings, the invalid cases with the expected `;` suggestion.
3. New-behavior check (the gate): a scratch crate (create under `/tmp`, not this repo) with `#[cfg(test)] mod tests { #[test] fn t() {} }` in `src/lib.rs` and `cargo dylint --path /home/zeon256/Documents/work/style-rust-aku` run from it must emit both new lints (pre-expansion visibility + cross-lint overlap proven end-to-end).
4. Cross-repo parity check, from `satay-rs`: `cargo dylint --path ../style-rust-aku` — expected: zero new warnings for every site currently passing the ast-grep rules **except** satay-oas3, which the generic port no longer exempts (intended divergence per user decision). Any warning elsewhere that ast-grep does not flag is a false positive to fix before delivery.

## Assumptions & contingencies

- Default lint level `Warn` (matches the repo's five existing lints). Escalate per-target with `-D warnings`; not configurable per-lint since the user declined dylint.toml plumbing.
- If pre-expansion passes prove not to see cfg-gated modules (Step 0 probe fails): ship the ungated branch only and record the divergence — do not attempt to recover stripped nodes.
- If pre-expansion drivers don't visit assoc items: ship module-level-fn coverage and document the limitation (Step 4 explicit fallback).
