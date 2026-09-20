use clippy_utils::diagnostics::span_lint;
use rustc_ast::{AssocItemKind, AttrVec, Item, ItemKind};
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use rustc_session::{declare_lint, declare_lint_pass};
use rustc_span::{FileName, Span};

use crate::no_inline_modules::attrs_have_test_attr;

declare_lint! {
    /// ### What it does
    /// Checks for functions carrying a test-registering attribute (`test`,
    /// `rstest`, `tokio::test`, ...) defined outside `tests.rs`, `*_tests.rs`,
    /// or any `tests/` directory.
    pub NO_TESTS_OUTSIDE_TEST_FILES,
    Warn,
    "Test functions belong in `tests.rs`, `*_tests.rs`, or `tests/**/*.rs`"
}

declare_lint_pass!(NoTestsOutsideTestFiles => [NO_TESTS_OUTSIDE_TEST_FILES]);

impl EarlyLintPass for NoTestsOutsideTestFiles {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        if let ItemKind::Fn(_) = item.kind {
            check(cx, item.span, &item.attrs);
        }
    }

    fn check_impl_item(&mut self, cx: &EarlyContext<'_>, item: &rustc_ast::AssocItem) {
        if let AssocItemKind::Fn(_) = item.kind {
            check(cx, item.span, &item.attrs);
        }
    }

    fn check_trait_item(&mut self, cx: &EarlyContext<'_>, item: &rustc_ast::AssocItem) {
        if let AssocItemKind::Fn(_) = item.kind {
            check(cx, item.span, &item.attrs);
        }
    }
}

fn check(cx: &EarlyContext<'_>, span: Span, attrs: &AttrVec) {
    if span.from_expansion() || span.in_external_macro(cx.sess().source_map()) {
        return;
    }
    if !attrs_have_test_attr(attrs) {
        return;
    }
    if is_test_file(&cx.sess().source_map().span_to_filename(span)) {
        return;
    }
    // ast-grep offers no fix for this rule, so no suggestion is attached.
    span_lint(
        cx,
        NO_TESTS_OUTSIDE_TEST_FILES,
        span,
        "Rust test functions belong in `tests.rs`, `*_tests.rs`, or `tests/**/*.rs`; move this \
         test to one of those standalone test files",
    );
}

/// Allowlist: files named `tests.rs` or `*_tests.rs`, or living anywhere under
/// a `tests/` directory. Virtual/remapped paths have no local path — we cannot
/// apply the rule faithfully, so treat them as allowed.
fn is_test_file(filename: &FileName) -> bool {
    let FileName::Real(real) = filename else { return false };
    let Some(path) = real.local_path() else { return false };
    let name = path.file_name().and_then(|n| n.to_str());
    (name == Some("tests.rs") || name.is_some_and(|n| n.ends_with("_tests.rs")))
        || path.ancestors().skip(1).any(|a| a.file_name().is_some_and(|n| n == "tests"))
}
