use crc32fast::Hasher;

/// Computes CRC32 checksum for the given data.
pub fn compute_checksum(data: &[u8]) -> u32 {
    let mut hasher = Hasher::new();
    hasher.update(data);
    hasher.finalize()
}

/// Verifies that the data matches the expected checksum.
pub fn verify_checksum(data: &[u8], expected: u32) -> bool {
    compute_checksum(data) == expected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum() {
        let data = b"Hello, World!";
        let checksum = compute_checksum(data);
        assert!(verify_checksum(data, checksum));

        // Corrupted data
        let corrupted = b"Hello, world!"; // lowercase 'w'
        assert!(!verify_checksum(corrupted, checksum));
    }
}
