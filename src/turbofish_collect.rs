use clippy_utils::diagnostics::span_lint_and_then;
use clippy_utils::is_trait_method;
use clippy_utils::source::snippet_opt;
use rustc_errors::Applicability;
use rustc_hir::{ExprKind, LetStmt};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_session::{declare_lint, declare_lint_pass};
use rustc_span::sym;

declare_lint! {
    /// ### What it does
    /// Checks for explicit type annotations on `let` bindings that use `Iterator::collect`,
    /// and suggests using the turbofish syntax instead.
    pub STYLE_RUST_AKU,
    Warn,
    "Use turbofish for collect() instead of type annotations"
}

declare_lint_pass!(StyleRustAku => [STYLE_RUST_AKU]);

impl<'tcx> LateLintPass<'tcx> for StyleRustAku {
    fn check_local(&mut self, cx: &LateContext<'tcx>, local: &'tcx LetStmt<'tcx>) {
        let Some(ty) = local.ty else { return; };
        let Some(init) = local.init else { return; };

        let ExprKind::MethodCall(path, _receiver, _args, _span) = init.kind else { return; };
        if path.ident.name.as_str() != "collect" { return; }

        if !is_trait_method(cx, init, sym::Iterator) { return; }

        if path.args.is_some() { return; }

        let Some(ty_snippet) = snippet_opt(cx, ty.span) else { return; };
        
        let turbofish = format!("::<{}>", ty_snippet);
        let turbofish_span = path.ident.span.shrink_to_hi();

        let remove_span = ty.span.with_lo(local.pat.span.hi());

        span_lint_and_then(
            cx,
            STYLE_RUST_AKU,
            local.span,
            "use turbofish `collect::<T>()` instead of explicit type annotation",
            |diag| {
                diag.multipart_suggestion(
                    "use turbofish here",
                    vec![
                        (remove_span, String::new()),
                        (turbofish_span, turbofish),
                    ],
                    Applicability::MachineApplicable,
                );
            }
        );
    }
}
