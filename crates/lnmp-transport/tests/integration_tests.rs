#[cfg(any(
    feature = "http",
    feature = "kafka",
    feature = "grpc",
    feature = "nats"
))]
use lnmp_core::{LnmpField, LnmpRecord, LnmpValue};
#[cfg(any(
    feature = "http",
    feature = "kafka",
    feature = "grpc",
    feature = "nats"
))]
use lnmp_envelope::{EnvelopeMetadata, LnmpEnvelope};

#[cfg(feature = "grpc")]
use lnmp_transport::grpc;
#[cfg(feature = "http")]
use lnmp_transport::http;
#[cfg(feature = "kafka")]
use lnmp_transport::kafka;
#[cfg(feature = "nats")]
use lnmp_transport::nats;

#[cfg(any(
    feature = "http",
    feature = "kafka",
    feature = "grpc",
    feature = "nats"
))]
fn create_test_envelope() -> LnmpEnvelope {
    let mut labels = std::collections::HashMap::new();
    labels.insert("env".to_string(), "prod".to_string());

    let meta = EnvelopeMetadata {
        timestamp: Some(1627849200000),
        source: Some("test-source".to_string()),
        trace_id: Some("test-trace-id".to_string()),
        sequence: Some(12345),
        labels,
    };

    let mut record = LnmpRecord::new();
    record.add_field(LnmpField {
        fid: 1,
        value: LnmpValue::Int(100),
    });

    LnmpEnvelope {
        metadata: meta,
        record,
    }
}

#[cfg(feature = "http")]
#[test]
fn test_http_mapping() {
    let env = create_test_envelope();
    let headers = http::envelope_to_headers(&env).unwrap();

    assert_eq!(
        headers.get("x-lnmp-timestamp").unwrap().to_str().unwrap(),
        "1627849200000"
    );
    assert_eq!(
        headers.get("x-lnmp-source").unwrap().to_str().unwrap(),
        "test-source"
    );
    assert_eq!(
        headers.get("x-lnmp-trace-id").unwrap().to_str().unwrap(),
        "test-trace-id"
    );
    assert_eq!(
        headers.get("x-lnmp-sequence").unwrap().to_str().unwrap(),
        "12345"
    );
    assert_eq!(
        headers.get("x-lnmp-label-env").unwrap().to_str().unwrap(),
        "prod"
    );

    let meta = http::headers_to_envelope_metadata(&headers).unwrap();
    assert_eq!(meta.timestamp, env.metadata.timestamp);
    assert_eq!(meta.source, env.metadata.source);
    assert_eq!(meta.trace_id, env.metadata.trace_id);
    assert_eq!(meta.sequence, env.metadata.sequence);
    assert_eq!(meta.labels.get("env"), env.metadata.labels.get("env"));
}

#[cfg(feature = "kafka")]
#[test]
fn test_kafka_mapping() {
    let env = create_test_envelope();
    let headers = kafka::envelope_to_kafka_headers(&env).unwrap();

    assert_eq!(headers.get("lnmp.timestamp").unwrap(), b"1627849200000");
    assert_eq!(headers.get("lnmp.source").unwrap(), b"test-source");
    assert_eq!(headers.get("lnmp.trace_id").unwrap(), b"test-trace-id");
    assert_eq!(headers.get("lnmp.sequence").unwrap(), b"12345");
    assert_eq!(headers.get("lnmp.label.env").unwrap(), b"prod");

    let meta = kafka::kafka_headers_to_envelope_metadata(&headers).unwrap();
    assert_eq!(meta.timestamp, env.metadata.timestamp);
    assert_eq!(meta.source, env.metadata.source);
    assert_eq!(meta.trace_id, env.metadata.trace_id);
    assert_eq!(meta.sequence, env.metadata.sequence);
    assert_eq!(meta.labels.get("env"), env.metadata.labels.get("env"));
}

#[cfg(feature = "grpc")]
#[test]
fn test_grpc_mapping() {
    let env = create_test_envelope();
    let metadata = grpc::envelope_to_metadata(&env).unwrap();

    assert_eq!(metadata.get("lnmp-timestamp").unwrap(), "1627849200000");
    assert_eq!(metadata.get("lnmp-source").unwrap(), "test-source");
    assert_eq!(metadata.get("lnmp-trace-id").unwrap(), "test-trace-id");
    assert_eq!(metadata.get("lnmp-sequence").unwrap(), "12345");
    assert_eq!(metadata.get("lnmp-label-env").unwrap(), "prod");

    let meta = grpc::metadata_to_envelope_metadata(&metadata).unwrap();
    assert_eq!(meta.timestamp, env.metadata.timestamp);
    assert_eq!(meta.source, env.metadata.source);
    assert_eq!(meta.trace_id, env.metadata.trace_id);
    assert_eq!(meta.sequence, env.metadata.sequence);
    assert_eq!(meta.labels.get("env"), env.metadata.labels.get("env"));
}

#[cfg(feature = "nats")]
#[test]
fn test_nats_mapping() {
    let env = create_test_envelope();
    let headers = nats::envelope_to_nats_headers(&env).unwrap();

    assert_eq!(headers.get("lnmp-timestamp").unwrap(), "1627849200000");
    assert_eq!(headers.get("lnmp-source").unwrap(), "test-source");
    assert_eq!(headers.get("lnmp-trace-id").unwrap(), "test-trace-id");
    assert_eq!(headers.get("lnmp-sequence").unwrap(), "12345");
    assert_eq!(headers.get("lnmp-label-env").unwrap(), "prod");

    let meta = nats::nats_headers_to_envelope_metadata(&headers).unwrap();
    assert_eq!(meta.timestamp, env.metadata.timestamp);
    assert_eq!(meta.source, env.metadata.source);
    assert_eq!(meta.trace_id, env.metadata.trace_id);
    assert_eq!(meta.sequence, env.metadata.sequence);
    assert_eq!(meta.labels.get("env"), env.metadata.labels.get("env"));
}
