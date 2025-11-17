//! Example demonstrating LNMP v0.5 Streaming Frame Layer (SFL)
//!
//! This example shows:
//! - Streaming large payloads in chunks
//! - Frame types: BEGIN, CHUNK, END, ERROR
//! - Checksum validation for data integrity
//! - Backpressure flow control
//! - Handling incomplete streams
//! - Error recovery scenarios

use lnmp_codec::binary::{
    BinaryEncoder, StreamingEncoder, StreamingDecoder, StreamingConfig,
    StreamingEvent, BackpressureController, FrameType,
};
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};

fn main() {
    println!("=== LNMP v0.5 Streaming Frame Layer Example ===\n");

    // Example 1: Stream a small payload
    println!("1. Streaming Small Payload:");
    stream_small_payload();
    println!();

    // Example 2: Stream a large payload with multiple chunks
    println!("2. Streaming Large Payload:");
    stream_large_payload();
    println!();

    // Example 3: Checksum validation
    println!("3. Checksum Validation:");
    checksum_validation_example();
    println!();

    // Example 4: Backpressure flow control
    println!("4. Backpressure Flow Control:");
    backpressure_example();
    println!();

    // Example 5: Error frame handling
    println!("5. Error Frame Handling:");
    error_frame_example();
    println!();

    // Example 6: Incomplete stream detection
    println!("6. Incomplete Stream Detection:");
    incomplete_stream_example();
    println!();
}

fn stream_small_payload() {
    // Create a small record
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 7,
        value: LnmpValue::Bool(true),
    });
    record.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::Int(14532),
    });

    // Encode to binary first
    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();
    
    println!("   Original binary size: {} bytes", binary.len());

    // Stream it with default chunk size (4KB)
    let config = StreamingConfig::new()
        .with_chunk_size(4096)
        .with_checksums(true);
    let mut streaming_encoder = StreamingEncoder::with_config(config);
    
    // Begin stream
    let begin_frame = streaming_encoder.begin_stream().unwrap();
    println!("   BEGIN frame: {} bytes", begin_frame.len());
    
    // Write data (will fit in one chunk since it's small)
    let chunk_frame = streaming_encoder.write_chunk(&binary).unwrap();
    println!("   CHUNK frame: {} bytes", chunk_frame.len());
    
    // End stream
    let end_frame = streaming_encoder.end_stream().unwrap();
    println!("   END frame: {} bytes", end_frame.len());
    
    // Decode the stream
    let decoder_config = StreamingConfig::new().with_checksums(true);
    let mut streaming_decoder = StreamingDecoder::with_config(decoder_config);
    
    // Feed frames
    match streaming_decoder.feed_frame(&begin_frame).unwrap() {
        StreamingEvent::StreamStarted => println!("   ✓ Stream started"),
        _ => println!("   ✗ Unexpected event"),
    }
    
    match streaming_decoder.feed_frame(&chunk_frame).unwrap() {
        StreamingEvent::ChunkReceived { bytes } => {
            println!("   ✓ Chunk received: {} bytes", bytes);
        }
        _ => println!("   ✗ Unexpected event"),
    }
    
    match streaming_decoder.feed_frame(&end_frame).unwrap() {
        StreamingEvent::StreamComplete { total_bytes } => {
            println!("   ✓ Stream complete: {} bytes total", total_bytes);
        }
        _ => println!("   ✗ Unexpected event"),
    }
    
    // Get the complete payload
    if let Some(payload) = streaming_decoder.get_complete_payload() {
        println!("   ✓ Payload matches original: {}", payload == binary);
    }
}

fn stream_large_payload() {
    // Create a large record with many fields
    let mut record = LnmpRecord::new();
    for i in 1..=200 {
        record.add_field(LnmpField {
            fid: i,
            value: LnmpValue::String(format!("Field {} with some substantial data content to make it larger", i)),
        });
    }

    // Encode to binary
    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();
    
    println!("   Large record binary size: {} bytes", binary.len());

    // Stream with small chunk size to demonstrate multiple chunks
    let chunk_size = 1024; // 1KB chunks
    let config = StreamingConfig::new()
        .with_chunk_size(chunk_size)
        .with_checksums(true);
    let mut streaming_encoder = StreamingEncoder::with_config(config);
    
    // Begin stream
    let begin_frame = streaming_encoder.begin_stream().unwrap();
    println!("   BEGIN frame sent");
    
    // Write data in chunks
    let mut offset = 0;
    let mut chunk_count = 0;
    let mut frames = vec![begin_frame];
    
    while offset < binary.len() {
        let end = (offset + chunk_size).min(binary.len());
        let chunk = &binary[offset..end];
        let chunk_frame = streaming_encoder.write_chunk(chunk).unwrap();
        frames.push(chunk_frame);
        chunk_count += 1;
        offset = end;
    }
    
    println!("   Sent {} CHUNK frames", chunk_count);
    
    // End stream
    let end_frame = streaming_encoder.end_stream().unwrap();
    frames.push(end_frame);
    println!("   END frame sent");
    
    // Decode the stream
    let decoder_config = StreamingConfig::new().with_checksums(true);
    let mut streaming_decoder = StreamingDecoder::with_config(decoder_config);
    
    let mut received_chunks = 0;
    for frame in frames {
        match streaming_decoder.feed_frame(&frame).unwrap() {
            StreamingEvent::StreamStarted => {}
            StreamingEvent::ChunkReceived { bytes } => {
                received_chunks += 1;
                println!("   Received chunk {}: {} bytes", received_chunks, bytes);
            }
            StreamingEvent::StreamComplete { total_bytes } => {
                println!("   ✓ Stream complete: {} bytes total", total_bytes);
            }
            StreamingEvent::StreamError { message } => {
                println!("   ✗ Error: {}", message);
            }
        }
    }
    
    // Verify payload
    if let Some(payload) = streaming_decoder.get_complete_payload() {
        println!("   ✓ Payload integrity verified: {} bytes", payload.len());
    }
}

fn checksum_validation_example() {
    // Create a record
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::Int(14532),
    });

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();
    
    // Stream with checksums enabled
    let config = StreamingConfig::new()
        .with_chunk_size(4096)
        .with_checksums(true);
    let mut streaming_encoder = StreamingEncoder::with_config(config);
    
    let begin_frame = streaming_encoder.begin_stream().unwrap();
    let mut chunk_frame = streaming_encoder.write_chunk(&binary).unwrap();
    let end_frame = streaming_encoder.end_stream().unwrap();
    
    println!("   Original chunk frame: {} bytes", chunk_frame.len());
    
    // Corrupt the checksum (last 4 bytes before payload)
    if chunk_frame.len() > 10 {
        let idx = chunk_frame.len() - 5;
        chunk_frame[idx] ^= 0xFF; // Flip bits
        println!("   Corrupted checksum in chunk frame");
    }
    
    // Try to decode with corrupted checksum
    let decoder_config = StreamingConfig::new().with_checksums(true);
    let mut streaming_decoder = StreamingDecoder::with_config(decoder_config);
    
    streaming_decoder.feed_frame(&begin_frame).unwrap();
    
    match streaming_decoder.feed_frame(&chunk_frame) {
        Ok(_) => println!("   ✗ Unexpected success with corrupted checksum"),
        Err(e) => println!("   ✓ Correctly detected corruption: {}", e),
    }
}

fn backpressure_example() {
    println!("   Simulating backpressure flow control");
    
    // Create backpressure controller with 8KB window
    let mut controller = BackpressureController::with_window_size(8192);
    
    println!("   Window size: 8192 bytes");
    println!("   Initial state: can_send = {}", controller.can_send());
    
    // Send some chunks
    let chunk_size = 2048;
    for i in 1..=3 {
        if controller.can_send() {
            controller.on_chunk_sent(chunk_size);
            println!("   Sent chunk {}: {} bytes (in-flight: {} bytes)", 
                     i, chunk_size, controller.bytes_in_flight());
        } else {
            println!("   Cannot send chunk {}: backpressure active", i);
        }
    }
    
    // Try to send more - should hit backpressure
    if controller.can_send() {
        controller.on_chunk_sent(chunk_size);
        println!("   Sent chunk 4: {} bytes", chunk_size);
    } else {
        println!("   ✓ Backpressure activated: cannot send chunk 4");
    }
    
    // Acknowledge some chunks
    controller.on_chunk_acked(chunk_size);
    println!("   Acknowledged chunk (in-flight: {} bytes)", controller.bytes_in_flight());
    
    // Now we can send again
    if controller.can_send() {
        controller.on_chunk_sent(chunk_size);
        println!("   ✓ Can send again after acknowledgment");
    }
}

fn error_frame_example() {
    // Create streaming encoder
    let config = StreamingConfig::new();
    let mut streaming_encoder = StreamingEncoder::with_config(config);
    
    // Begin stream
    let begin_frame = streaming_encoder.begin_stream().unwrap();
    
    // Simulate an error condition
    let error_frame = streaming_encoder.error_frame("Simulated encoding error").unwrap();
    println!("   ERROR frame created: {} bytes", error_frame.len());
    
    // Decode the stream
    let decoder_config = StreamingConfig::new();
    let mut streaming_decoder = StreamingDecoder::with_config(decoder_config);
    
    streaming_decoder.feed_frame(&begin_frame).unwrap();
    
    match streaming_decoder.feed_frame(&error_frame).unwrap() {
        StreamingEvent::StreamError { message } => {
            println!("   ✓ Error received: {}", message);
        }
        _ => println!("   ✗ Unexpected event"),
    }
}

fn incomplete_stream_example() {
    // Create a record
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::Int(14532),
    });

    let encoder = BinaryEncoder::new();
    let binary = encoder.encode(&record).unwrap();
    
    // Stream it
    let config = StreamingConfig::new();
    let mut streaming_encoder = StreamingEncoder::with_config(config);
    
    let begin_frame = streaming_encoder.begin_stream().unwrap();
    let chunk_frame = streaming_encoder.write_chunk(&binary).unwrap();
    // Intentionally don't send END frame
    
    println!("   Sent BEGIN and CHUNK frames, but no END frame");
    
    // Decode the incomplete stream
    let decoder_config = StreamingConfig::new();
    let mut streaming_decoder = StreamingDecoder::with_config(decoder_config);
    
    streaming_decoder.feed_frame(&begin_frame).unwrap();
    streaming_decoder.feed_frame(&chunk_frame).unwrap();
    
    // Try to get payload before stream is complete
    match streaming_decoder.get_complete_payload() {
        Some(_) => println!("   ✗ Unexpected payload from incomplete stream"),
        None => println!("   ✓ Correctly detected incomplete stream"),
    }
    
    println!("   Stream state: waiting for END frame");
}
