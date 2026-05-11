use clippy_utils::diagnostics::span_lint_and_sugg;
use clippy_utils::is_from_proc_macro;
use clippy_utils::is_path_diagnostic_item;
use rustc_errors::Applicability;
use rustc_hir::{BindingMode, Expr, ExprKind, Node, PatKind, QPath, TyKind};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_session::{declare_lint, declare_lint_pass};
use rustc_span::sym;

declare_lint! {
    /// ### What it does
    /// Checks for usages of `Vec::new()` without turbofish and suggests using the `vec![]` macro.
    pub PREFER_VEC_MACRO,
    Warn,
    "Use `vec![]` instead of `Vec::new()`"
}

declare_lint_pass!(PreferVecMacro => [PREFER_VEC_MACRO]);

impl<'tcx> LateLintPass<'tcx> for PreferVecMacro {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) {
        if let ExprKind::Call(func, []) = expr.kind {
            if is_path_diagnostic_item(cx, func, sym::vec_new) {
                // Check if it has any generic arguments (e.g. `Vec::<u32>::new()`).
                let has_turbofish = match func.kind {
                    ExprKind::Path(QPath::Resolved(_, path)) => {
                        path.segments.iter().any(|seg| seg.args.is_some())
                    }
                    ExprKind::Path(QPath::TypeRelative(ty, segment)) => {
                        segment.args.is_some()
                            || match ty.kind {
                                TyKind::Path(QPath::Resolved(_, path)) => {
                                    path.segments.iter().any(|seg| seg.args.is_some())
                                }
                                _ => true, // Conservatively skip if complex type is used
                            }
                    }
                    _ => false,
                };

                if !has_turbofish
                    && !expr.span.from_expansion()
                    && !expr.span.in_external_macro(cx.sess().source_map())
                    && !is_from_proc_macro(cx, expr)
                    && !is_mutable_local_init(cx, expr)
                {
                    span_lint_and_sugg(
                        cx,
                        PREFER_VEC_MACRO,
                        expr.span,
                        "consider using the `vec![]` macro",
                        "replace with",
                        "vec![]".to_string(),
                        Applicability::MachineApplicable,
                    );
                }
            }
        }
    }
}

fn is_mutable_local_init(cx: &LateContext<'_>, expr: &Expr<'_>) -> bool {
    let Node::LetStmt(local) = cx.tcx.parent_hir_node(expr.hir_id) else {
        return false;
    };

    local.init.is_some_and(|init| init.hir_id == expr.hir_id)
        && matches!(local.pat.kind, PatKind::Binding(BindingMode::MUT, ..))
}
