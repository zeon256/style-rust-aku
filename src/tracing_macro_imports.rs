use clippy_utils::diagnostics::span_lint_and_then;
use rustc_ast::MacCall;
use rustc_errors::Applicability;
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use rustc_session::{declare_lint, declare_lint_pass};
use rustc_span::symbol::kw;

declare_lint! {
    /// ### What it does
    /// Checks for qualified calls to `tracing` macros.
    pub TRACING_MACRO_IMPORTS,
    Warn,
    "Import tracing macros instead of calling them through `tracing::`"
}

declare_lint_pass!(TracingMacroImports => [TRACING_MACRO_IMPORTS]);

impl EarlyLintPass for TracingMacroImports {
    fn check_mac(&mut self, cx: &EarlyContext<'_>, mac: &MacCall) {
        if mac.path.span.from_expansion() || mac.path.span.in_external_macro(cx.sess().source_map())
        {
            return;
        }

        let segments = mac.path.segments.as_ref();
        let macro_name = match segments {
            [tracing, macro_name] if tracing.ident.name.as_str() == "tracing" => macro_name,
            [root, tracing, macro_name]
                if root.ident.name == kw::PathRoot && tracing.ident.name.as_str() == "tracing" =>
            {
                macro_name
            }
            _ => return,
        };

        let macro_name = macro_name.ident.name.as_str();
        if !is_tracing_macro(macro_name) {
            return;
        }

        span_lint_and_then(
            cx,
            TRACING_MACRO_IMPORTS,
            mac.path.span,
            format!("avoid calling tracing macro `{macro_name}!` through `tracing::`"),
            |diag| {
                diag.span_suggestion(
                    mac.path.span,
                    format!("import `tracing::{macro_name}` and call the macro directly"),
                    macro_name.to_string(),
                    Applicability::MaybeIncorrect,
                );
            },
        );
    }
}

fn is_tracing_macro(name: &str) -> bool {
    matches!(
        name,
        "trace"
            | "debug"
            | "info"
            | "warn"
            | "error"
            | "event"
            | "span"
            | "trace_span"
            | "debug_span"
            | "info_span"
            | "warn_span"
            | "error_span"
            | "enabled"
    )
}
