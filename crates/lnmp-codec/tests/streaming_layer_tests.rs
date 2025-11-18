//! Integration tests for Streaming Frame Layer (SFL)

use lnmp_codec::binary::{
    BackpressureController, FrameType, StreamingConfig, StreamingDecoder, StreamingEncoder,
    StreamingError, StreamingEvent, StreamingFrame,
};

#[test]
fn test_individual_frame_encoding_decoding() {
    let mut encoder = StreamingEncoder::new();

    // Test BEGIN frame
    let begin_bytes = encoder.begin_stream().unwrap();
    assert!(!begin_bytes.is_empty());
    assert_eq!(begin_bytes[0], FrameType::Begin.to_u8());

    // Test CHUNK frame
    let chunk_data = vec![1, 2, 3, 4, 5];
    let chunk_bytes = encoder.write_chunk(&chunk_data).unwrap();
    assert!(!chunk_bytes.is_empty());
    assert_eq!(chunk_bytes[0], FrameType::Chunk.to_u8());

    // Test END frame
    let end_bytes = encoder.end_stream().unwrap();
    assert!(!end_bytes.is_empty());
    assert_eq!(end_bytes[0], FrameType::End.to_u8());

    // Test ERROR frame
    let mut encoder2 = StreamingEncoder::new();
    let error_bytes = encoder2.error_frame("Test error").unwrap();
    assert!(!error_bytes.is_empty());
    assert_eq!(error_bytes[0], FrameType::Error.to_u8());
}

#[test]
fn test_streaming_small_payload() {
    // Payload smaller than chunk size
    let mut encoder = StreamingEncoder::new();
    let mut decoder = StreamingDecoder::new();

    let payload = vec![1, 2, 3, 4, 5];

    // Begin stream
    let begin_bytes = encoder.begin_stream().unwrap();
    let event = decoder.feed_frame(&begin_bytes).unwrap();
    assert_eq!(event, StreamingEvent::StreamStarted);

    // Send single chunk
    let chunk_bytes = encoder.write_chunk(&payload).unwrap();
    let event = decoder.feed_frame(&chunk_bytes).unwrap();
    assert_eq!(event, StreamingEvent::ChunkReceived { bytes: 5 });

    // End stream
    let end_bytes = encoder.end_stream().unwrap();
    let event = decoder.feed_frame(&end_bytes).unwrap();
    match event {
        StreamingEvent::StreamComplete { total_bytes } => {
            assert_eq!(total_bytes, 5);
        }
        _ => panic!("Expected StreamComplete event"),
    }

    // Verify payload
    let received = decoder.get_complete_payload().unwrap();
    assert_eq!(received, &payload[..]);
}

#[test]
fn test_streaming_large_payload() {
    // Payload larger than chunk size
    let config = StreamingConfig::new().with_chunk_size(1024);
    let mut encoder = StreamingEncoder::with_config(config.clone());
    let mut decoder = StreamingDecoder::with_config(config);

    // Create a large payload (5KB)
    let large_payload: Vec<u8> = (0..5120).map(|i| (i % 256) as u8).collect();

    // Begin stream
    let begin_bytes = encoder.begin_stream().unwrap();
    decoder.feed_frame(&begin_bytes).unwrap();

    // Send in chunks
    let mut total_sent = 0;
    for chunk in large_payload.chunks(1024) {
        let chunk_bytes = encoder.write_chunk(chunk).unwrap();
        let event = decoder.feed_frame(&chunk_bytes).unwrap();
        match event {
            StreamingEvent::ChunkReceived { bytes } => {
                total_sent += bytes;
            }
            _ => panic!("Expected ChunkReceived event"),
        }
    }

    // End stream
    let end_bytes = encoder.end_stream().unwrap();
    let event = decoder.feed_frame(&end_bytes).unwrap();
    match event {
        StreamingEvent::StreamComplete { total_bytes } => {
            assert_eq!(total_bytes, total_sent);
            assert_eq!(total_bytes, 5120);
        }
        _ => panic!("Expected StreamComplete event"),
    }

    // Verify payload
    let received = decoder.get_complete_payload().unwrap();
    assert_eq!(received, &large_payload[..]);
}

#[test]
fn test_checksum_validation() {
    let config = StreamingConfig::new().with_checksums(true);
    let mut encoder = StreamingEncoder::with_config(config.clone());
    let mut decoder = StreamingDecoder::with_config(config);

    let payload = vec![1, 2, 3, 4, 5, 6, 7, 8];

    // Begin stream
    let begin_bytes = encoder.begin_stream().unwrap();
    decoder.feed_frame(&begin_bytes).unwrap();

    // Send chunk with valid checksum
    let chunk_bytes = encoder.write_chunk(&payload).unwrap();
    let event = decoder.feed_frame(&chunk_bytes).unwrap();
    assert!(matches!(event, StreamingEvent::ChunkReceived { .. }));

    // Corrupt checksum and verify detection
    let mut corrupted_bytes = encoder.write_chunk(&payload).unwrap();
    if corrupted_bytes.len() > 10 {
        corrupted_bytes[5] ^= 0xFF; // Corrupt a byte in the checksum area
    }
    let result = decoder.feed_frame(&corrupted_bytes);
    assert!(matches!(
        result,
        Err(StreamingError::ChecksumMismatch { .. })
    ));
}

#[test]
fn test_backpressure_mechanism() {
    let mut controller = BackpressureController::with_window_size(4096);

    // Initially can send
    assert!(controller.can_send());
    assert_eq!(controller.available_window(), 4096);

    // Send chunks
    controller.on_chunk_sent(1024);
    assert!(controller.can_send());
    assert_eq!(controller.available_window(), 3072);

    controller.on_chunk_sent(1024);
    assert!(controller.can_send());
    assert_eq!(controller.available_window(), 2048);

    controller.on_chunk_sent(1024);
    assert!(controller.can_send());
    assert_eq!(controller.available_window(), 1024);

    controller.on_chunk_sent(1024);
    assert!(!controller.can_send());
    assert_eq!(controller.available_window(), 0);

    // Acknowledge chunks
    controller.on_chunk_acked(1024);
    assert!(controller.can_send());
    assert_eq!(controller.available_window(), 1024);

    controller.on_chunk_acked(2048);
    assert!(controller.can_send());
    assert_eq!(controller.available_window(), 3072);
}

#[test]
fn test_error_frames() {
    let mut encoder = StreamingEncoder::new();
    let mut decoder = StreamingDecoder::new();

    // Send error frame
    let error_bytes = encoder.error_frame("Connection timeout").unwrap();
    let event = decoder.feed_frame(&error_bytes).unwrap();

    match event {
        StreamingEvent::StreamError { message } => {
            assert_eq!(message, "Connection timeout");
        }
        _ => panic!("Expected StreamError event"),
    }
}

#[test]
fn test_incomplete_streams() {
    let mut encoder = StreamingEncoder::new();
    let mut decoder = StreamingDecoder::new();

    // Begin stream
    let begin_bytes = encoder.begin_stream().unwrap();
    decoder.feed_frame(&begin_bytes).unwrap();

    // Send chunk
    let chunk_bytes = encoder.write_chunk(&[1, 2, 3]).unwrap();
    decoder.feed_frame(&chunk_bytes).unwrap();

    // Try to get payload before END frame
    assert!(decoder.get_complete_payload().is_none());

    // Now end the stream
    let end_bytes = encoder.end_stream().unwrap();
    decoder.feed_frame(&end_bytes).unwrap();

    // Now payload should be available
    assert!(decoder.get_complete_payload().is_some());
}

#[test]
fn test_multiple_streams_sequential() {
    let mut encoder = StreamingEncoder::new();
    let mut decoder = StreamingDecoder::new();

    // First stream
    let begin1_bytes = encoder.begin_stream().unwrap();
    let chunk1_bytes = encoder.write_chunk(&[1, 2, 3]).unwrap();
    let end1_bytes = encoder.end_stream().unwrap();

    decoder.feed_frame(&begin1_bytes).unwrap();
    decoder.feed_frame(&chunk1_bytes).unwrap();
    decoder.feed_frame(&end1_bytes).unwrap();

    let payload1 = decoder.get_complete_payload().unwrap().to_vec();
    assert_eq!(payload1, vec![1, 2, 3]);

    // Second stream
    let begin2_bytes = encoder.begin_stream().unwrap();
    let chunk2_bytes = encoder.write_chunk(&[4, 5, 6]).unwrap();
    let end2_bytes = encoder.end_stream().unwrap();

    decoder.feed_frame(&begin2_bytes).unwrap();
    decoder.feed_frame(&chunk2_bytes).unwrap();
    decoder.feed_frame(&end2_bytes).unwrap();

    let payload2 = decoder.get_complete_payload().unwrap().to_vec();
    assert_eq!(payload2, vec![4, 5, 6]);
}

#[test]
fn test_streaming_with_empty_chunks() {
    let mut encoder = StreamingEncoder::new();
    let mut decoder = StreamingDecoder::new();

    // Begin stream
    let begin_bytes = encoder.begin_stream().unwrap();
    decoder.feed_frame(&begin_bytes).unwrap();

    // Send empty chunk
    let empty_chunk_bytes = encoder.write_chunk(&[]).unwrap();
    let event = decoder.feed_frame(&empty_chunk_bytes).unwrap();
    assert_eq!(event, StreamingEvent::ChunkReceived { bytes: 0 });

    // End stream
    let end_bytes = encoder.end_stream().unwrap();
    decoder.feed_frame(&end_bytes).unwrap();

    let payload = decoder.get_complete_payload().unwrap();
    assert!(payload.is_empty());
}

#[test]
fn test_streaming_with_various_chunk_sizes() {
    let chunk_sizes = vec![512, 1024, 2048, 4096, 8192];

    for chunk_size in chunk_sizes {
        let config = StreamingConfig::new().with_chunk_size(chunk_size);
        let mut encoder = StreamingEncoder::with_config(config.clone());
        let mut decoder = StreamingDecoder::with_config(config);

        let payload: Vec<u8> = (0..chunk_size).map(|i| (i % 256) as u8).collect();

        let begin_bytes = encoder.begin_stream().unwrap();
        let chunk_bytes = encoder.write_chunk(&payload).unwrap();
        let end_bytes = encoder.end_stream().unwrap();

        decoder.feed_frame(&begin_bytes).unwrap();
        decoder.feed_frame(&chunk_bytes).unwrap();
        decoder.feed_frame(&end_bytes).unwrap();

        let received = decoder.get_complete_payload().unwrap();
        assert_eq!(received, &payload[..]);
    }
}

#[test]
fn test_streaming_binary_data() {
    let mut encoder = StreamingEncoder::new();
    let mut decoder = StreamingDecoder::new();

    // Create binary data with all byte values
    let binary_data: Vec<u8> = (0..=255).collect();

    let begin_bytes = encoder.begin_stream().unwrap();
    let chunk_bytes = encoder.write_chunk(&binary_data).unwrap();
    let end_bytes = encoder.end_stream().unwrap();

    decoder.feed_frame(&begin_bytes).unwrap();
    decoder.feed_frame(&chunk_bytes).unwrap();
    decoder.feed_frame(&end_bytes).unwrap();

    let received = decoder.get_complete_payload().unwrap();
    assert_eq!(received, &binary_data[..]);
}

#[test]
fn test_checksum_with_different_data_patterns() {
    // Test checksum with various data patterns
    let patterns = vec![
        vec![0u8; 100],                            // All zeros
        vec![0xFF; 100],                           // All ones
        (0..100).map(|i| i as u8).collect(),       // Sequential
        (0..100).map(|i| (i * 7) as u8).collect(), // Pattern
    ];

    for pattern in patterns {
        let checksum = StreamingFrame::compute_xor_checksum(&pattern);
        let frame = StreamingFrame::chunk(pattern.clone(), false);
        assert_eq!(frame.checksum, checksum);
        assert!(frame.validate_checksum().is_ok());
    }
}

#[test]
fn test_backpressure_with_realistic_scenario() {
    // Simulate a realistic streaming scenario with backpressure
    let window_size = 16384; // 16KB window
    let chunk_size = 4096; // 4KB chunks
    let total_data = 65536; // 64KB total data

    let mut controller = BackpressureController::with_window_size(window_size);
    let mut chunks_sent = 0;
    let mut chunks_acked = 0;
    let mut bytes_sent = 0;

    while bytes_sent < total_data {
        if controller.can_send() && controller.available_window() >= chunk_size {
            controller.on_chunk_sent(chunk_size);
            chunks_sent += 1;
            bytes_sent += chunk_size;
        } else {
            // Simulate acknowledgment
            if chunks_acked < chunks_sent {
                controller.on_chunk_acked(chunk_size);
                chunks_acked += 1;
            }
        }
    }

    // Acknowledge remaining chunks
    while chunks_acked < chunks_sent {
        controller.on_chunk_acked(chunk_size);
        chunks_acked += 1;
    }

    assert_eq!(controller.bytes_in_flight(), 0);
    assert_eq!(chunks_sent, total_data / chunk_size);
}
