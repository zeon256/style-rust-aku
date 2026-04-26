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
extern crate rustc_lint;
extern crate rustc_session;

pub mod turbofish_collect;
pub mod minimal_imports;
pub mod literal_suffix;
pub mod prefer_vec_macro;

dylint_linting::dylint_library!();

#[unsafe(no_mangle)]
pub fn register_lints(_sess: &rustc_session::Session, lint_store: &mut rustc_lint::LintStore) {
    lint_store.register_lints(&[
        turbofish_collect::PREFER_COLLECT_TURBOFISH,
        minimal_imports::MINIMAL_IMPORTS,
        literal_suffix::LITERAL_SUFFIX,
        prefer_vec_macro::PREFER_VEC_MACRO,
    ]);
    lint_store.register_late_pass(|_| Box::new(turbofish_collect::PreferCollectTurbofish));
    lint_store.register_late_pass(|_| Box::new(minimal_imports::MinimalImports));
    lint_store.register_late_pass(|_| Box::new(literal_suffix::LiteralSuffix));
    lint_store.register_late_pass(|_| Box::new(prefer_vec_macro::PreferVecMacro));
}

#[test]
fn ui() {
    dylint_testing::ui_test(env!("CARGO_PKG_NAME"), "ui");
}
