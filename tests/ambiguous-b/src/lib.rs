// This crate depends on hashbrown 0.16

// Should be excluded: 0.16.x does not match ^0.14
#[cfg_version::cfg_version(hashbrown = "^0.14")]
pub fn has_014() -> bool {
    true
}

// Should be included: 0.16.x matches ^0.16
#[cfg_version::cfg_version(hashbrown = "^0.16")]
pub fn has_016() -> bool {
    true
}

// Should be included: 0.16.x matches >=0.13
#[cfg_version::cfg_version(hashbrown = ">=0.13")]
pub fn has_gte_013() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_016() {
        assert!(has_016());
    }

    #[test]
    fn test_has_gte_013() {
        assert!(has_gte_013());
    }
}
