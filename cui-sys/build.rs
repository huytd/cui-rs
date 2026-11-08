use std::env;
use std::path::{Path, PathBuf};

#[cfg(feature = "bindgen")]
include!("bindgen_config.rs");

fn main() {
    let crate_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let cui = crate_dir.join("CUI");
    let cmake = cui.join("CMakeLists.txt");
    if !cmake.exists() {
        panic!(
            "CUI sources not found at {}. Run `git submodule update --init`.",
            cui.display()
        );
    }
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=bindgen_config.rs");
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed={}", cmake.display());
    println!("cargo:rerun-if-changed={}", cui.join("include").display());
    println!("cargo:rerun-if-changed={}", cui.join("src").display());

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let feat = |name: &str| env::var_os(format!("CARGO_FEATURE_{}", name.to_uppercase().replace('-', "_"))).is_some();

    // The source list comes from upstream's CMakeLists.txt, so a CUI update
    // that adds or removes files needs no change here.
    let cmake_text = std::fs::read_to_string(&cmake).unwrap();
    let mut sources = cmake_list(&cmake_text, "CUI_CORE_SOURCES");
    assert!(!sources.is_empty(), "could not read CUI_CORE_SOURCES from CMakeLists.txt");

    let mut build = cc::Build::new();
    build.include(cui.join("include")).warnings(false);
    if target_env == "msvc" {
        build.flag_if_supported("/utf-8").define("_CRT_SECURE_NO_WARNINGS", None);
    } else {
        build.std("c99").flag_if_supported("-ffunction-sections").flag_if_supported("-fdata-sections");
    }

    if feat("default-fonts") {
        sources.push("src/cui_font_data.c".into());
        sources.push("src/cui_inflate.c".into());
    } else {
        build.define("CUI_NO_DEFAULT_FONTS", None);
    }
    if !feat("stb-image") {
        build.define("CUI_NO_STB_IMAGE", None);
    } else if feat("image-all-formats") {
        build.define("CUI_IMAGE_ALL_FORMATS", None);
    }
    if feat("text-shaping") {
        sources.push("src/cui_bidi.c".into());
        sources.push("src/cui_shape.c".into());
        build.define("CUI_TEXT_SHAPING", None);
    }

    if feat("app") {
        sources.push("src/platform/cui_app.c".into());
        sources.push("src/platform/cui_dialogs.c".into());
        let backend = if feat("sdl2") {
            "sdl2"
        } else if feat("glfw") {
            "glfw"
        } else {
            match target_os.as_str() {
                "windows" => "win32",
                "macos" => "cocoa",
                _ => "x11",
            }
        };
        println!("cargo:backend={backend}");
        match backend {
            "win32" => {
                sources.push("src/platform/cui_app_win32.c".into());
                for lib in ["user32", "gdi32", "opengl32", "shell32", "imm32", "dwmapi", "comdlg32", "ole32"] {
                    println!("cargo:rustc-link-lib={lib}");
                }
            }
            "cocoa" => {
                build.define("GL_SILENCE_DEPRECATION", None);
                // cc compiles .m with the system clang; ARC is required.
                let mut objc = cc::Build::new();
                objc.include(cui.join("include"))
                    .define("GL_SILENCE_DEPRECATION", None)
                    .flag("-fobjc-arc")
                    .warnings(false)
                    .file(cui.join("src/platform/cui_app_cocoa.m"));
                objc.compile("cui_cocoa");
                for fw in ["Cocoa", "QuartzCore", "OpenGL"] {
                    println!("cargo:rustc-link-lib=framework={fw}");
                }
            }
            "x11" => {
                sources.push("src/platform/cui_app_x11.c".into());
                println!("cargo:rustc-link-lib=X11");
                println!("cargo:rustc-link-lib=dl");
            }
            "sdl2" => {
                sources.push("src/platform/cui_app_sdl2.c".into());
                println!("cargo:rustc-link-lib=SDL2");
            }
            "glfw" => {
                sources.push("src/platform/cui_app_glfw.c".into());
                println!("cargo:rustc-link-lib=glfw");
            }
            _ => unreachable!(),
        }
    }

    for src in &sources {
        build.file(cui.join(src));
    }
    build.compile("cui");

    if target_os != "windows" {
        println!("cargo:rustc-link-lib=m");
        println!("cargo:rustc-link-lib=pthread");
    }
    println!("cargo:include={}", cui.join("include").display());

    bindings(&crate_dir);
}

/// Reads `set(NAME a.c b.c ...)` from a CMake file.
fn cmake_list(text: &str, name: &str) -> Vec<String> {
    let start = match text.find(&format!("set({name}")) {
        Some(i) => i + 4 + name.len(),
        None => return Vec::new(),
    };
    let end = start + text[start..].find(')').unwrap_or(0);
    text[start..end].split_whitespace().map(str::to_owned).collect()
}

#[cfg(feature = "bindgen")]
fn bindings(crate_dir: &Path) {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");
    cui_bindgen_builder(crate_dir)
        .generate()
        .expect("bindgen failed")
        .write_to_file(out)
        .unwrap();
}

#[cfg(not(feature = "bindgen"))]
fn bindings(_crate_dir: &Path) {}
