use crate::vector::{EmbeddingType, Vector};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;

pub struct Decoder;

impl Decoder {
    pub fn decode(data: &[u8]) -> Result<Vector, std::io::Error> {
        let mut rdr = Cursor::new(data);

        let dim = rdr.read_u16::<LittleEndian>()?;
        let dtype_byte = rdr.read_u8()?;
        let _similarity = rdr.read_u8()?; // Read and ignore for now

        let dtype = match dtype_byte {
            0x01 => EmbeddingType::F32,
            0x02 => EmbeddingType::F16,
            0x03 => EmbeddingType::I8,
            0x04 => EmbeddingType::U8,
            0x05 => EmbeddingType::Binary,
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid dtype",
                ))
            }
        };

        let pos = rdr.position() as usize;
        let vector_data = data[pos..].to_vec();

        // Validation could happen here (check length matches dim * type_size)

        Ok(Vector {
            dim,
            dtype,
            data: vector_data,
        })
    }
}
