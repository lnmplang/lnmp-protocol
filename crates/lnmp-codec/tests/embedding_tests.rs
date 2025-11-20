use lnmp_codec::binary::entry::BinaryEntry;
use lnmp_codec::binary::types::{BinaryValue, TypeTag};
use lnmp_embedding::{EmbeddingType, Vector};

#[test]
fn test_embedding_encode_decode() {
    let vec_data = Vector::from_f32(vec![1.0, 2.0, 3.0]);
    let entry = BinaryEntry {
        fid: 60,
        tag: TypeTag::Embedding,
        value: BinaryValue::Embedding(vec_data.clone()),
    };

    let bytes = entry.encode();
    let (decoded, consumed) = BinaryEntry::decode(&bytes).unwrap();

    assert_eq!(decoded.fid, 60);
    assert_eq!(decoded.tag, TypeTag::Embedding);
    match decoded.value {
        BinaryValue::Embedding(v) => {
            assert_eq!(v.dim, 3);
            assert_eq!(v.dtype, EmbeddingType::F32);
            assert_eq!(v.as_f32().unwrap(), vec![1.0, 2.0, 3.0]);
        }
        _ => panic!("Expected Embedding value"),
    }
    assert_eq!(consumed, bytes.len());
}
