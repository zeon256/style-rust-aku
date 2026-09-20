use clippy_utils::diagnostics::span_lint_and_sugg;
use rustc_ast::{AttrKind, AttrVec, Attribute, Inline, Item, ItemKind, MetaItemInner, ModKind};
use rustc_errors::Applicability;
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use rustc_session::{declare_lint, declare_lint_pass};
use rustc_span::{BytePos, Span};

declare_lint! {
    /// ### What it does
    /// Checks for inline test modules: `#[cfg(test)] mod tests { ... }` (including
    /// compound `all`/`any` cfg predicates) and ungated inline modules whose body
    /// directly registers a test function.
    pub NO_INLINE_MODULES,
    Warn,
    "Inline test module bodies are not allowed"
}

declare_lint_pass!(NoInlineModules => [NO_INLINE_MODULES]);

impl EarlyLintPass for NoInlineModules {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        if item.span.from_expansion() || item.span.in_external_macro(cx.sess().source_map()) {
            return;
        }

        // `ModKind::Unloaded` is `mod foo;` (allowed). A file module loads as
        // `Loaded(.., Inline::No, ..)` — also allowed: the yaml only matches
        // inline `declaration_list` bodies.
        let ItemKind::Mod(_, _, ModKind::Loaded(items, Inline::Yes, mod_spans)) = &item.kind else {
            return;
        };

        let cfg_gated = item.attrs.iter().any(is_test_cfg_attr);
        let direct_test_item = items.iter().any(|child| attrs_have_test_attr(&child.attrs));
        if !(cfg_gated || direct_test_item) {
            return;
        }

        let Some(braces_span) = module_braces_span(item.span, mod_spans.inner_span, cx) else {
            return;
        };

        span_lint_and_sugg(
            cx,
            NO_INLINE_MODULES,
            braces_span,
            "inline test module bodies are not allowed; move the body to the standalone module \
             file and preserve this module's attributes and visibility",
            "replace the module body with",
            ";".to_string(),
            Applicability::MaybeIncorrect,
        );
    }
}

/// Computes the span covering the `{ ... }` braces of an inline module.
///
/// `inner_span` starts just past the opening brace, so locate both braces in
/// the item's own snippet: the first `{` is the module's opening brace, the
/// last `}` its closing one (the module body is brace-balanced). Sanity-checked
/// against `inner_span`, whose end may include the closing brace in this
/// nightly.
fn module_braces_span(item_span: Span, inner_span: Span, cx: &EarlyContext<'_>) -> Option<Span> {
    let sm = cx.sess().source_map();
    let snippet = sm.span_to_snippet(item_span).ok()?;
    let open = snippet.find('{')?;
    let close = snippet.rfind('}')?;
    let data = item_span.data();
    let lo = BytePos(data.lo.0 + open as u32);
    let hi = BytePos(data.lo.0 + close as u32 + 1);
    // Braces must bracket the parsed inner span (inner_span.hi includes the
    // closing brace in this nightly).
    if !(lo.0 < inner_span.lo().0 && inner_span.hi().0 <= hi.0) {
        return None;
    }
    Some(Span::new(lo, hi, data.ctxt, data.parent))
}

/// Matches `cfg` attributes containing a positive `test` predicate anywhere in
/// their predicate tree: bare `test` matches; `all(...)`/`any(...)` recurse;
/// `not(...)` subtrees are excluded; name-value leaves (`feature = "x"`) never
/// match. `cfg_attr` does not match (its path's last segment is `cfg_attr`).
fn is_test_cfg_attr(attr: &Attribute) -> bool {
    let AttrKind::Normal(normal) = &attr.kind else {
        return false;
    };
    let last = normal.item.path.segments.last().map(|seg| seg.ident.name.as_str());
    if last != Some("cfg") {
        return false;
    }
    let Some(list) = normal.item.meta_item_list() else {
        return false;
    };
    positive_test_ident(&list)
}

fn positive_test_ident(list: &[MetaItemInner]) -> bool {
    list.iter().any(|inner| match inner.meta_item() {
        Some(mi) => {
            let last = mi.path.segments.last().unwrap().ident;
            let name = last.name.as_str().strip_prefix("r#").unwrap_or(last.name.as_str());
            if name == "test" && mi.is_word() {
                true
            } else if name == "not" {
                // Negated predicate subtree: excluded entirely.
                false
            } else {
                mi.meta_item_list().is_some_and(positive_test_ident)
            }
        }
        None => false,
    })
}

/// Does this attribute's path terminal identifier name a test-registering
/// attribute? Handles scoped forms (`#[tokio::test]`) by taking the last path
/// segment; companion-only attrs (`serial`, `fixture`, `should_panic`) do not
/// match.
pub(crate) fn attr_is_test_attr(attr: &Attribute) -> bool {
    let AttrKind::Normal(normal) = &attr.kind else {
        return false;
    };
    let last = normal.item.path.segments.last().unwrap();
    let name = last.ident.name.as_str();
    let name = name.strip_prefix("r#").unwrap_or(name);
    matches!(
        name,
        "test"
            | "rstest"
            | "test_case"
            | "parameterized"
            | "test_with"
            | "wasm_bindgen_test"
            | "quickcheck"
            | "proptest"
            | "test_matrix"
            | "test_each_path"
    )
}

pub(crate) fn attrs_have_test_attr(attrs: &AttrVec) -> bool {
    attrs.iter().any(attr_is_test_attr)
}
