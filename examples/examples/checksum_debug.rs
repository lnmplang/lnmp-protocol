use crc::{Crc, CRC_32_BZIP2, CRC_32_ISO_HDLC, CRC_32_ISCSI, CRC_32_CKSUM};

fn compute_crc(c: &Crc<u32>, s: &str) -> u32 {
    let mut digest = c.digest();
    digest.update(s.as_bytes());
    digest.finalize()
}

fn main() {
    let s = "12:i:14532";
    println!("input: {}", s);

    let c1 = Crc::<u32>::new(&CRC_32_ISO_HDLC); // standard
    let c2 = Crc::<u32>::new(&CRC_32_ISCSI); // castagnoli
    let c3 = Crc::<u32>::new(&CRC_32_BZIP2);
    let c4 = Crc::<u32>::new(&CRC_32_CKSUM);

    println!("CRC_32_ISO_HDLC: {:08X}", compute_crc(&c1, s));
    println!("CRC_32_ISCSI (Castagnoli): {:08X}", compute_crc(&c2, s));
    println!("CRC_32_BZIP2: {:08X}", compute_crc(&c3, s));
    println!("CRC_32_CKSUM: {:08X}", compute_crc(&c4, s));
}
