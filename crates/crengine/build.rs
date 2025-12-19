use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

fn main() -> Result<()> {
    emit_rerun_directives();

    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").context("CARGO_MANIFEST_DIR missing")?);
    let vendor_dir = manifest_dir.join("vendor");
    let source_dir = vendor_dir.join("crengine");

    if !source_dir.join("CMakeLists.txt").exists() {
        anyhow::bail!(
            "Vendored CREngine sources are missing. Run scripts/update-crengine.sh before building."
        );
    }

    let min_deps = has_feature("MIN_DEPS");
    let static_requested = env_static_flag("CRENGINE_SYS_STATIC");
    let features = FeatureConfig::detect(min_deps, static_requested);
    let cmake_dst = build_with_cmake(&source_dir, &features)?;

    emit_link_searches(&cmake_dst)?;
    emit_cpp_runtime_link()?;
    println!("cargo:include={}", source_dir.join("include").display());

    Ok(())
}

struct FeatureConfig {
    harfbuzz: DependencyStatus,
    fribidi: DependencyStatus,
    icu: DependencyStatus,
    min_deps: bool,
    static_requested: bool,
}

impl FeatureConfig {
    fn detect(min_deps: bool, static_requested: bool) -> Self {
        let wants_harfbuzz = has_feature("HARFBUZZ") && !min_deps;
        let wants_fribidi = has_feature("FRIBIDI") && !min_deps;
        let wants_icu = has_feature("ICU") && !min_deps;

        let harfbuzz = DependencyStatus::new(
            wants_harfbuzz,
            probe_pkg("harfbuzz", static_requested, "HarfBuzz shaping"),
        );
        let fribidi = DependencyStatus::new(
            wants_fribidi,
            probe_pkg("fribidi", static_requested, "FriBidi RTL support"),
        );
        let icu = DependencyStatus::new(wants_icu, probe_icu(static_requested));

        FeatureConfig {
            harfbuzz,
            fribidi,
            icu,
            min_deps,
            static_requested,
        }
    }
}

#[derive(Clone, Copy)]
struct DependencyStatus {
    requested: bool,
    found: bool,
}

impl DependencyStatus {
    fn new(requested: bool, found: bool) -> Self {
        DependencyStatus { requested, found }
    }

    fn enabled(&self) -> bool {
        self.requested && self.found
    }
}

fn build_with_cmake(source_dir: &Path, features: &FeatureConfig) -> Result<PathBuf> {
    let profile = cmake_profile();
    let mut config = cmake::Config::new(source_dir);
    config
        .profile(profile)
        .define("CRE_BUILD_SHARED", "OFF")
        .define("CRE_BUILD_STATIC", "ON")
        .define("BUILD_TOOLS", "OFF")
        .define("ENABLE_UNITTESTING", "OFF")
        .define("OFFLINE_BUILD_MODE", "ON")
        .define("CMAKE_POSITION_INDEPENDENT_CODE", "ON")
        .define(
            "PKG_CONFIG_USE_STATIC_LIBS",
            on_off(features.static_requested),
        );
    if features.static_requested {
        config.env("PKG_CONFIG_ALL_STATIC", "1");
    }

    // Optional dependency controls.
    if features.min_deps {
        config
            .define("WITH_HARFBUZZ", "OFF")
            .define("WITH_FRIBIDI", "OFF")
            .define("USE_FONTCONFIG", "OFF");
    } else {
        config.define("WITH_HARFBUZZ", on_off(features.harfbuzz.enabled()));
        config.define("WITH_FRIBIDI", on_off(features.fribidi.enabled()));
    }

    config.define("CRENGINE_WITH_ICU", on_off(features.icu.enabled()));

    let dst = config.build();
    Ok(dst)
}

fn emit_link_searches(cmake_dst: &Path) -> Result<()> {
    let mut any = false;
    for lib_dir in ["lib", "lib64"] {
        let path = cmake_dst.join(lib_dir);
        if path.is_dir() {
            println!("cargo:rustc-link-search=native={}", path.display());
            link_archives(&path)?;
            any = true;
        }
    }

    if !any {
        anyhow::bail!(
            "CMake build for CREngine completed but no library directories were found under {}",
            cmake_dst.display()
        );
    }

    Ok(())
}

fn link_archives(dir: &Path) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        if let Some(ext) = path.extension() {
            let stem = match path.file_stem() {
                Some(stem) => stem.to_string_lossy(),
                None => continue,
            };
            let name = stem
                .strip_prefix("lib")
                .map(|s| s.to_string())
                .unwrap_or_else(|| stem.to_string());

            if ext == "a" || ext == "lib" {
                println!("cargo:rustc-link-lib=static={name}");
            } else if ext == "so" || ext == "dylib" {
                println!("cargo:rustc-link-lib=dylib={name}");
            }
        }
    }
    Ok(())
}

fn emit_cpp_runtime_link() -> Result<()> {
    let target = env::var("TARGET").context("TARGET not set")?;
    if target.contains("msvc") {
        // MSVC links the C++ runtime automatically.
        return Ok(());
    }

    if target.contains("apple") {
        println!("cargo:rustc-link-lib=c++");
    } else {
        println!("cargo:rustc-link-lib=stdc++");
    }
    Ok(())
}

fn emit_rerun_directives() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=vendor");
    println!("cargo:rerun-if-env-changed=CRENGINE_SYS_STATIC");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_HARFBUZZ");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_FRIBIDI");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_ICU");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_MIN_DEPS");
    println!("cargo:rerun-if-env-changed=TARGET");
    println!("cargo:rerun-if-env-changed=HOST");
}

fn has_feature(name: &str) -> bool {
    let env_key = format!("CARGO_FEATURE_{name}");
    env::var_os(env_key).is_some()
}

fn env_static_flag(key: &str) -> bool {
    matches!(
        env::var(key).as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE") | Ok("yes") | Ok("on")
    )
}

fn probe_pkg(name: &str, statik: bool, reason: &str) -> bool {
    let mut cfg = pkg_config::Config::new();
    cfg.cargo_metadata(true).statik(statik);
    match cfg.probe(name) {
        Ok(_) => true,
        Err(err) => {
            println!("cargo:warning=Optional dependency {name} not available ({reason}): {err}");
            false
        }
    }
}

fn probe_icu(statik: bool) -> bool {
    let uc = probe_pkg("icu-uc", statik, "Unicode core services");
    let i18n = probe_pkg("icu-i18n", statik, "Unicode i18n services");
    uc && i18n
}

fn cmake_profile() -> &'static str {
    match env::var("PROFILE").as_deref() {
        Ok("debug") => "Debug",
        Ok("release") => "Release",
        Ok("bench") => "RelWithDebInfo",
        _ => "RelWithDebInfo",
    }
}

fn on_off(value: bool) -> &'static str {
    if value {
        "ON"
    } else {
        "OFF"
    }
}
