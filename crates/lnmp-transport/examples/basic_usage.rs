use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};
use lnmp_envelope::{EnvelopeMetadata, LnmpEnvelope};
use lnmp_transport::{grpc, http, kafka, nats};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create an Envelope
    let meta = EnvelopeMetadata {
        timestamp: Some(1732373147000),
        source: Some("example-service".to_string()),
        trace_id: Some("abc-123-xyz".to_string()),
        sequence: None,
        labels: std::collections::HashMap::new(),
    };
    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 12,
        value: LnmpValue::Int(42),
    });

    let env = LnmpEnvelope {
        metadata: meta,
        record,
    };

    println!("Original Envelope Metadata: {:?}", env.metadata);

    // 2. HTTP Binding
    #[cfg(feature = "http")]
    {
        let headers = http::envelope_to_headers(&env)?;
        println!("\n--- HTTP Headers ---");
        for (k, v) in &headers {
            println!("{}: {:?}", k, v);
        }
    }

    // 3. Kafka Binding
    let kafka_headers = kafka::envelope_to_kafka_headers(&env)?;
    println!("\n--- Kafka Headers ---");
    for (k, v) in &kafka_headers {
        println!("{}: {:?}", k, String::from_utf8_lossy(v));
    }

    // 4. gRPC Binding
    let grpc_metadata = grpc::envelope_to_metadata(&env)?;
    println!("\n--- gRPC Metadata ---");
    for (k, v) in &grpc_metadata {
        println!("{}: {}", k, v);
    }

    // 5. NATS Binding
    let nats_headers = nats::envelope_to_nats_headers(&env)?;
    println!("\n--- NATS Headers ---");
    for (k, v) in &nats_headers {
        println!("{}: {}", k, v);
    }

    Ok(())
}
