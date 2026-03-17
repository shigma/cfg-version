use cfg_version::cfg_version;

// Caret: >= 2.0.0, < 3.0.0
#[cfg_version(indexmap = "^2")]
pub fn needs_indexmap_2() -> &'static str {
    "indexmap 2.x"
}

// Exact match won't work for most versions
#[cfg_version(indexmap = "=99.0.0")]
pub fn needs_indexmap_99() -> &'static str {
    "should never be present"
}

// Greater-than-or-equal
#[cfg_version(indexmap = ">=2")]
pub fn needs_indexmap_gte_2() -> &'static str {
    "at least indexmap 2"
}

// Less-than (should be true for any current indexmap)
#[cfg_version(indexmap = "<99")]
pub fn indexmap_lt_99() -> &'static str {
    "below 99"
}

// Non-existent dependency
#[cfg_version(nonexistent = ">=0")]
pub fn needs_nonexistent() -> &'static str {
    "should never be present"
}

// Tilde: >= 2.13.0, < 2.14.0
#[cfg_version(indexmap = "~2.13")]
pub fn needs_indexmap_2_13() -> &'static str {
    "indexmap 2.13.x"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caret_indexmap_2() {
        assert_eq!(needs_indexmap_2(), "indexmap 2.x");
    }

    #[test]
    fn gte_indexmap_2() {
        assert_eq!(needs_indexmap_gte_2(), "at least indexmap 2");
    }

    #[test]
    fn lt_99() {
        assert_eq!(indexmap_lt_99(), "below 99");
    }

    #[test]
    fn tilde_2_13() {
        assert_eq!(needs_indexmap_2_13(), "indexmap 2.13.x");
    }
}

#[cfg_version(indexmap = "2")]
pub fn bare_2() -> &'static str {
    "bare 2"
}

// Hyphenated crate name
// #[cfg_version(cfg-cfg_version = "^0.1")]
// pub fn has_cfg_version() -> &'static str {
//     "yes"
// }

#[cfg(test)]
#[test]
fn test_bare_2() {
    assert_eq!(bare_2(), "bare 2");
}

#[cfg(test)]
#[test]
fn test_hyphenated_name() {
    // assert_eq!(has_cfg_version(), "yes");
}
