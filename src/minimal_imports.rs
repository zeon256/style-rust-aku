use clippy_utils::diagnostics::span_lint_and_help;
use rustc_hir::{HirId, ItemKind, Node, Path};
use rustc_lint::{LateContext, LateLintPass};
use rustc_session::{declare_lint, declare_lint_pass};

declare_lint! {
    /// ### What it does
    /// Checks for deeply nested absolute paths and suggests minimal imports.
    pub MINIMAL_IMPORTS,
    Warn,
    "Suggests minimal imports for deeply nested paths"
}

declare_lint_pass!(MinimalImports => [MINIMAL_IMPORTS]);

impl<'tcx> LateLintPass<'tcx> for MinimalImports {
    fn check_path(&mut self, cx: &LateContext<'tcx>, path: &Path<'tcx>, hir_id: HirId) {
        if path.segments.len() < 3 {
            return;
        }

        if path.span.from_expansion() {
            return;
        }

        // We use parent_hir_node or try hir_node directly
        let node = cx.tcx.hir_node(hir_id);
        if let Node::Item(item) = node
            && let ItemKind::Use(..) = item.kind {
                return;
            }
        
        let path_str = path.segments.iter()
            .map(|s| s.ident.as_str())
            .collect::<Vec<_>>()
            .join("::");

        let use_path = path.segments[..path.segments.len() - 1]
            .iter()
            .map(|s| s.ident.as_str())
            .collect::<Vec<_>>()
            .join("::");
            
        let last_mod = path.segments[path.segments.len() - 2].ident.as_str();
        let item_name = path.segments.last().unwrap().ident.as_str();
        let replacement = format!("{}::{}", last_mod, item_name);

        span_lint_and_help(
            cx,
            MINIMAL_IMPORTS,
            path.span,
            format!("fully qualified path `{}` is too long (>= 3 segments)", path_str),
            None,
            format!("consider adding `use {};` and using `{}` instead", use_path, replacement),
        );
    }
}
