//! Conditional compilation based on dependency versions.
//!
//! Parses `Cargo.lock` at macro expansion time to determine the resolved version of any dependency,
//! then conditionally includes or excludes the annotated item. When a dependency appears multiple
//! times in the lock file (e.g., `syn` 1.x and 2.x), only the version actually used by the current
//! crate is considered.
//!
//! # Usage
//!
//! ```toml
//! # Cargo.toml
//! [dependencies]
//! cfg-version = "0.1"
//! ```
//!
//! ```rust,ignore
//! use cfg_version::cfg_version;
//!
//! // Include only when indexmap >= 2.8.0, < 3.0.0
//! #[cfg_version(indexmap = "^2.8")]
//! fn needs_indexmap_2_8() { }
//!
//! // Include only when indexmap >= 2.0.0
//! #[cfg_version(indexmap = ">=2")]
//! fn needs_indexmap_2() { }
//!
//! // Include only when indexmap < 2.8.0
//! #[cfg_version(indexmap = "<2.8")]
//! fn old_indexmap_fallback() { }
//! ```
//!
//! Version requirements use [Cargo's semver syntax](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html):
//! `^`, `~`, `*`, `>=`, `>`, `<`, `<=`, `=`, and comma-separated combinations.

use std::collections::HashMap;
use std::sync::OnceLock;

use proc_macro::TokenStream;
use proc_macro2::{Delimiter, TokenStream as TokenStream2, TokenTree};
use semver::{Version, VersionReq};
use syn::LitStr;
use syn::parse::{Parse, ParseStream};

/// A resolved package from Cargo.lock.
struct Package {
    name: String,
    version: Version,
    /// Direct dependencies. Each entry is either `"name"` or `"name version"`.
    deps: Vec<String>,
}

/// Parsed Cargo.lock: all packages + index by name.
struct LockFile {
    packages: Vec<Package>,
}

impl LockFile {
    fn parse(content: &str) -> Self {
        let mut packages = Vec::new();
        let mut current_name: Option<String> = None;
        let mut current_version: Option<Version> = None;
        let mut current_deps: Vec<String> = Vec::new();
        let mut in_deps = false;

        let flush = |name: &mut Option<String>,
                     version: &mut Option<Version>,
                     deps: &mut Vec<String>,
                     packages: &mut Vec<Package>| {
            if let (Some(name), Some(version)) = (name.take(), version.take()) {
                packages.push(Package {
                    name,
                    version,
                    deps: std::mem::take(deps),
                });
            }
        };

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed == "[[package]]" {
                flush(
                    &mut current_name,
                    &mut current_version,
                    &mut current_deps,
                    &mut packages,
                );
                in_deps = false;
                continue;
            }

            if let Some(rest) = trimmed.strip_prefix("name = ") {
                current_name = Some(rest.trim_matches('"').to_string());
                in_deps = false;
            } else if let Some(rest) = trimmed.strip_prefix("version = ") {
                let ver_str = rest.trim_matches('"');
                current_version = Version::parse(ver_str).ok();
                in_deps = false;
            } else if trimmed == "dependencies = [" {
                in_deps = true;
            } else if in_deps && trimmed == "]" {
                in_deps = false;
            } else if in_deps {
                // Lines like: "indexmap", or "syn 1.0.109",
                let dep = trimmed.trim_matches(['"', ',', ' '].as_slice());
                if !dep.is_empty() {
                    current_deps.push(dep.to_string());
                }
            }
        }

        flush(
            &mut current_name,
            &mut current_version,
            &mut current_deps,
            &mut packages,
        );

        LockFile { packages }
    }

    /// Resolve the dependency versions for a given crate.
    ///
    /// For each dependency in the crate's `dependencies` list, find the matching
    /// package in the lock file and return `(dep_name, dep_version)`.
    fn resolve_deps_for(&self, pkg_name: &str) -> Vec<(String, Version)> {
        // Build index: name -> list of versions
        let mut by_name: HashMap<&str, Vec<&Package>> = HashMap::new();
        for pkg in &self.packages {
            by_name.entry(&pkg.name).or_default().push(pkg);
        }

        // Find the current crate's package entry
        let current = match by_name.get(pkg_name).and_then(|pkgs| pkgs.first()) {
            Some(pkg) => pkg,
            None => return Vec::new(),
        };

        let mut result = Vec::new();
        for dep_entry in &current.deps {
            // dep_entry is either "name" or "name version"
            let mut parts = dep_entry.splitn(2, ' ');
            let dep_name = parts.next().unwrap();
            let dep_version = parts.next();

            if let Some(candidates) = by_name.get(dep_name) {
                let resolved = if let Some(ver_str) = dep_version {
                    // Exact version specified (disambiguation)
                    candidates.iter().find(|p| p.version.to_string() == ver_str)
                } else if candidates.len() == 1 {
                    // Only one version, no ambiguity
                    Some(&candidates[0])
                } else {
                    // Multiple versions but no disambiguation — shouldn't happen
                    // in a well-formed Cargo.lock, but fall back to first
                    candidates.first()
                };

                if let Some(pkg) = resolved {
                    result.push((dep_name.to_string(), pkg.version.clone()));
                }
            }
        }

        result
    }
}

// Cache only the lockfile parse (shared across crates in one compilation session).
// Do NOT cache the per-crate dep resolution — CARGO_PKG_NAME changes between crates.
static LOCK_CACHE: OnceLock<LockFile> = OnceLock::new();

fn lockfile() -> &'static LockFile {
    LOCK_CACHE.get_or_init(load_lockfile)
}

fn load_lockfile() -> LockFile {
    let manifest_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(d) => d,
        Err(_) => {
            return LockFile { packages: Vec::new() };
        }
    };

    let mut dir = std::path::PathBuf::from(manifest_dir);
    loop {
        let candidate = dir.join("Cargo.lock");
        if let Ok(content) = std::fs::read_to_string(&candidate) {
            return LockFile::parse(&content);
        }
        if !dir.pop() {
            return LockFile { packages: Vec::new() };
        }
    }
}

fn dep_matches(name: &str, req: &VersionReq) -> bool {
    let pkg_name = std::env::var("CARGO_PKG_NAME").unwrap_or_default();
    lockfile()
        .resolve_deps_for(&pkg_name)
        .iter()
        .any(|(n, v)| n == name && req.matches(v))
}

struct Args {
    name: String,
    req: VersionReq,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name = String::new();
        loop {
            if input.is_empty() {
                return Err(input.error("expected `=` after crate name"));
            }
            let tt: TokenTree = input.parse()?;
            if let TokenTree::Punct(ref p) = tt
                && p.as_char() == '='
            {
                break;
            }
            name.push_str(&tt.to_string());
        }
        if name.is_empty() {
            return Err(input.error("expected a crate name"));
        }

        let lit: LitStr = input.parse()?;
        let req = VersionReq::parse(&lit.value())
            .map_err(|e| syn::Error::new(lit.span(), format!("invalid version requirement: {e}")))?;
        Ok(Args { name, req })
    }
}

fn flatten_transparent_group(ts: TokenStream2) -> TokenStream2 {
    let mut output = TokenStream2::new();
    for tt in ts {
        match tt {
            TokenTree::Group(ref g) if g.delimiter() == Delimiter::None => {
                output.extend(flatten_transparent_group(g.stream()));
            }
            other => output.extend(std::iter::once(other)),
        }
    }
    output
}

/// Conditionally include an item based on a dependency's resolved version.
///
/// Uses [Cargo's semver syntax](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html)
/// for version requirements. Only considers direct dependencies of the current crate.
///
/// # Examples
///
/// ```rust,ignore
/// use cfg_version::cfg_version;
///
/// #[cfg_version(indexmap = "^2.8")]
/// fn needs_indexmap_2_8() { }
///
/// #[cfg_version(indexmap = ">=2, <3")]
/// fn needs_indexmap_2_x() { }
///
/// #[cfg_version(indexmap = "<2.8")]
/// fn old_indexmap_fallback() { }
/// ```
///
/// If the dependency is not present in the current crate's dependencies, the item is excluded.
#[proc_macro_attribute]
pub fn cfg_version(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = match syn::parse2::<Args>(flatten_transparent_group(args.into())) {
        Ok(args) => args,
        Err(e) => return e.to_compile_error().into(),
    };

    let keep = dep_matches(&args.name, &args.req);

    if keep { input } else { TokenStream::new() }
}
