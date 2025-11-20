use crate::vector::Vector;
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::Write;

pub struct Encoder;

impl Encoder {
    pub fn encode(vector: &Vector) -> Result<Vec<u8>, std::io::Error> {
        let mut buf = Vec::new();

        // Spec: u16 dim, u8 dtype, u8 reserved (or similarity default?), [vector payload]
        // The user spec said: u16 dim, u8 dtype, u8 similarity
        // But similarity is usually a query parameter, not intrinsic to the vector storage.
        // However, the user request says: "u16 dim, u8 dtype, u8 similarity, [vector payload]"
        // Let's stick to the user spec for now, maybe 0x00 as default similarity if not specified?
        // Or maybe the vector carries a preferred similarity metric?
        // Let's assume 0x00 (Cosine) as default if not provided, but the Vector struct doesn't have it.
        // I'll add a placeholder byte for now or maybe I should update Vector to hold it?
        // The user spec: "u16 dim, u8 dtype, u8 similarity"
        // I'll just write 0x01 (Cosine) as a default for now.

        buf.write_u16::<LittleEndian>(vector.dim)?;
        buf.write_u8(vector.dtype as u8)?;
        buf.write_u8(0x01)?; // Default to Cosine for now
        buf.write_all(&vector.data)?;

        Ok(buf)
    }
}
