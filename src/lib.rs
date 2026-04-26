#![feature(rustc_private)]
#![allow(unused_extern_crates)]

extern crate rustc_arena;
extern crate rustc_ast;
extern crate rustc_ast_pretty;
extern crate rustc_data_structures;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_hir_pretty;
extern crate rustc_index;
extern crate rustc_infer;
extern crate rustc_lexer;
extern crate rustc_middle;
extern crate rustc_mir_dataflow;
extern crate rustc_parse;
extern crate rustc_span;
extern crate rustc_target;
extern crate rustc_trait_selection;

use clippy_utils::diagnostics::span_lint_and_then;
use clippy_utils::is_trait_method;
use clippy_utils::source::snippet_opt;
use rustc_errors::Applicability;
use rustc_hir::{ExprKind, LetStmt};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::sym;

dylint_linting::declare_late_lint! {
    /// ### What it does
    /// Checks for explicit type annotations on `let` bindings that use `Iterator::collect`,
    /// and suggests using the turbofish syntax instead.
    ///
    /// ### Why is this bad?
    /// Using turbofish syntax is considered more idiomatic in this codebase.
    ///
    /// ### Example
    /// ```rust
    /// let x: Vec<u32> = data.into_iter().collect();
    /// ```
    ///
    /// Use instead:
    /// ```rust
    /// let x = data.into_iter().collect::<Vec<u32>>();
    /// ```
    pub STYLE_RUST_AKU,
    Warn,
    "Use turbofish for collect() instead of type annotations"
}

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

        // `ty.span.with_lo(local.pat.span.hi())` spans from the end of the pattern to the end of the type
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

#[test]
fn ui() {
    dylint_testing::ui_test(env!("CARGO_PKG_NAME"), "ui");
}
