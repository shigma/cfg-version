# cfg-version

Conditional compilation based on dependency versions.

## Usage

```toml
[dependencies]
cfg-version = "0.1"
```

```rs
use cfg_version::cfg_version;

#[cfg_version(indexmap = "^2.8")]
fn needs_indexmap_2_8() { }
```

## Version syntax

Uses [Cargo's semver syntax](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html):

| Syntax | Example | Meaning |
|--------|---------|---------|
| `^` (default) | `"^2.8"` or `"2.8"` | `>= 2.8.0, < 3.0.0` |
| `~` | `"~2.8"` | `>= 2.8.0, < 2.9.0` |
| `>=` | `">=2.8"` | `>= 2.8.0` |
| `>` | `">2.8"` | `> 2.8.0` |
| `<` | `"<3"` | `< 3.0.0` |
| `<=` | `"<=2.8"` | `<= 2.8.0` |
| `=` | `"=2.8.0"` | exactly `2.8.0` |
| `*` | `"*"` | any version |
| combined | `">=2.8, <2.12"` | `>= 2.8.0` and `< 2.12.0` |

## How it works

At macro expansion time, `cfg-version`:

1. Reads `CARGO_PKG_NAME` to identify the current crate.
2. Finds `Cargo.lock` by walking up from `CARGO_MANIFEST_DIR`.
3. Parses the lock file to find the current crate's direct dependencies and their resolved versions.
4. Matches the resolved version against the user-specified requirement.
5. Keeps or removes the annotated item accordingly.

When a dependency appears multiple times in `Cargo.lock` (e.g., `syn` 1.x and 2.x), only the version directly depended on by the current crate is considered.

## License

MIT
