use lnmp_codec::container::{ContainerFrame, ContainerFrameError};
use lnmp_core::{LnmpContainerError, LnmpFileMode, LNMP_FLAG_EXT_META_BLOCK};
use std::path::Path;

fn fixture(name: &str) -> Vec<u8> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("fixtures");
    let path = root.join(name);
    std::fs::read(path).expect("fixture should exist")
}

#[test]
fn valid_frames_parse() {
    let cases = [
        "valid-text-checksum.lnmp",
        "valid-binary-meta16.lnmp",
        "valid-stream-4k-sc32.lnmp",
        "valid-delta-base1.lnmp",
        "valid-stream-4k-sc32-chunks.lnmp",
    ];

    for name in cases {
        let bytes = fixture(name);
        ContainerFrame::parse(&bytes).unwrap();
    }
}

#[test]
fn invalid_bad_magic() {
    let bytes = fixture("invalid-bad-magic.lnmp");
    let err = ContainerFrame::parse(&bytes).unwrap_err();
    assert!(matches!(
        err,
        ContainerFrameError::Header(LnmpContainerError::InvalidMagic)
    ));
}

#[test]
fn invalid_version() {
    let bytes = fixture("invalid-version-ff.lnmp");
    let err = ContainerFrame::parse(&bytes).unwrap_err();
    assert!(matches!(
        err,
        ContainerFrameError::Header(LnmpContainerError::UnsupportedVersion(0xFF))
    ));
}

#[test]
fn invalid_mode() {
    let bytes = fixture("invalid-mode-ff.lnmp");
    let err = ContainerFrame::parse(&bytes).unwrap_err();
    assert!(matches!(
        err,
        ContainerFrameError::Header(LnmpContainerError::UnknownMode(0xFF))
    ));
}

#[test]
fn invalid_reserved_flags() {
    let bytes = fixture("invalid-reserved-flag.lnmp");
    let err = ContainerFrame::parse(&bytes).unwrap_err();
    assert!(matches!(err, ContainerFrameError::ReservedFlags(_)));
}

#[test]
fn invalid_reserved_extension_flag() {
    let bytes = fixture("invalid-reserved-ext-meta-flag.lnmp");
    let err = ContainerFrame::parse(&bytes).unwrap_err();
    assert!(
        matches!(err, ContainerFrameError::ReservedFlags(flags) if flags & LNMP_FLAG_EXT_META_BLOCK != 0)
    );
}

#[test]
fn invalid_stream_missing_metadata() {
    let bytes = fixture("invalid-stream-no-meta.lnmp");
    let err = ContainerFrame::parse(&bytes).unwrap_err();
    assert!(matches!(
        err,
        ContainerFrameError::InvalidMetadataLength {
            expected: 6,
            actual: 0,
            ..
        }
    ));
}

#[test]
fn invalid_delta_missing_metadata() {
    let bytes = fixture("invalid-delta-no-meta.lnmp");
    let err = ContainerFrame::parse(&bytes).unwrap_err();
    assert!(matches!(
        err,
        ContainerFrameError::InvalidMetadataLength {
            expected: 10,
            actual: 0,
            ..
        }
    ));
}

#[test]
fn invalid_delta_unknown_algorithm() {
    let bytes = fixture("invalid-delta-unknown-algorithm.lnmp");
    let err = ContainerFrame::parse(&bytes).unwrap_err();
    assert!(matches!(
        err,
        ContainerFrameError::InvalidMetadataValue {
            mode: LnmpFileMode::Delta,
            field: "algorithm",
            value: 0xFF
        }
    ));
}

#[test]
fn invalid_delta_unknown_compression() {
    let bytes = fixture("invalid-delta-unknown-compression.lnmp");
    let err = ContainerFrame::parse(&bytes).unwrap_err();
    assert!(matches!(
        err,
        ContainerFrameError::InvalidMetadataValue {
            mode: LnmpFileMode::Delta,
            field: "compression",
            value: 0xFF
        }
    ));
}

#[test]
fn invalid_metadata_overflow() {
    let bytes = fixture("invalid-meta-overflow.lnmp");
    let err = ContainerFrame::parse(&bytes).unwrap_err();
    assert!(matches!(
        err,
        ContainerFrameError::TruncatedMetadata {
            expected: 16,
            available: 0
        }
    ));
}
