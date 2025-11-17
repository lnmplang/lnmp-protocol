pub fn version() -> &'static str {
    "lnmp-sdk-rust v0.1.0"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(version().len() > 0);
    }
}
