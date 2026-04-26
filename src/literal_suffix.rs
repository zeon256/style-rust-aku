use clippy_utils::diagnostics::span_lint_and_then;
use clippy_utils::source::snippet_opt;
use rustc_errors::Applicability;
use rustc_hir::{ExprKind, LetStmt, TyKind, QPath};
use rustc_ast::ast::{LitKind, LitIntType, LitFloatType};
use rustc_lint::{LateContext, LateLintPass};
use rustc_session::{declare_lint, declare_lint_pass};

declare_lint! {
    /// ### What it does
    /// Checks for explicit type annotations on `let` bindings that use unsuffixed literals,
    /// and suggests using suffixed literals instead.
    pub LITERAL_SUFFIX,
    Warn,
    "Use suffixed literals instead of explicit type annotations"
}

declare_lint_pass!(LiteralSuffix => [LITERAL_SUFFIX]);

impl<'tcx> LateLintPass<'tcx> for LiteralSuffix {
    fn check_local(&mut self, cx: &LateContext<'tcx>, local: &'tcx LetStmt<'tcx>) {
        let Some(ty) = local.ty else { return; };
        let Some(init) = local.init else { return; };

        let TyKind::Path(QPath::Resolved(None, path)) = ty.kind else { return; };
        if path.segments.len() != 1 { return; }
        let type_name = path.segments[0].ident.name.as_str();

        let is_numeric = matches!(
            type_name,
            "u8" | "u16" | "u32" | "u64" | "u128" | "usize" |
            "i8" | "i16" | "i32" | "i64" | "i128" | "isize" |
            "f32" | "f64"
        );
        if !is_numeric { return; }

        let ExprKind::Lit(lit) = init.kind else { return; };

        let is_unsuffixed = match lit.node {
            LitKind::Int(_, LitIntType::Unsuffixed) => true,
            LitKind::Float(_, LitFloatType::Unsuffixed) => true,
            _ => false,
        };

        if !is_unsuffixed { return; }

        let Some(init_snippet) = snippet_opt(cx, init.span) else { return; };
        let new_lit = format!("{}{}", init_snippet, type_name);
        
        let remove_span = ty.span.with_lo(local.pat.span.hi());

        span_lint_and_then(
            cx,
            LITERAL_SUFFIX,
            local.span,
            "use suffixed literal instead of explicit type annotation",
            |diag| {
                diag.multipart_suggestion(
                    "use a suffixed literal",
                    vec![
                        (remove_span, String::new()),
                        (init.span, new_lit),
                    ],
                    Applicability::MachineApplicable,
                );
            }
        );
    }
}
