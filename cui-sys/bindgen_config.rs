// Shared by build.rs (feature "bindgen") and `cargo xtask bindgen`, so the
// committed bindings and build-time bindings always use the same settings.

pub fn cui_bindgen_builder(crate_dir: &std::path::Path) -> bindgen::Builder {
    let include = crate_dir.join("CUI").join("include");
    bindgen::Builder::default()
        .header(crate_dir.join("wrapper.h").to_string_lossy())
        .clang_arg(format!("-I{}", include.display()))
        .allowlist_function("cui_.*")
        .allowlist_type("cui_.*")
        .allowlist_var("CUI_.*")
        // va_list differs per target; the variadic and plain forms cover it.
        .blocklist_function(".*appendv")
        .blocklist_type("va_list|__gnuc_va_list")
        .use_core()
        .ctypes_prefix("::core::ffi")
        .prepend_enum_name(false)
        .derive_default(true)
        .derive_debug(true)
        .derive_copy(true)
        .layout_tests(false)
        .generate_comments(true)
        .sort_semantically(true)
        .merge_extern_blocks(true)
}
