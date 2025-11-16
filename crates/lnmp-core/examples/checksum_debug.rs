use crc::{Crc, CRC_32_BZIP2, CRC_32_ISO_HDLC, CRC_32_ISCSI, CRC_32_CKSUM, CRC_32_JAMCRC, CRC_32_MPEG_2, CRC_32_XFER};

fn compute_crc(c: &Crc<u32>, s: &str) -> u32 {
    let mut digest = c.digest();
    digest.update(s.as_bytes());
    digest.finalize()
}

fn main() {
    let variants = vec![
        "12:i:14532",
        "12:i=14532",
        "F12:i=14532",
        "F12=14532",
        "12:i:14532\n",
        "12:i:14532\r\n",
        "12:i:14532 ",
    ];
    for s in variants.iter() {
        println!("input: {}", s);

    let algos = vec![
        ("CRC_32_ISO_HDLC", Crc::<u32>::new(&CRC_32_ISO_HDLC)),
        ("CRC_32_ISCSI", Crc::<u32>::new(&CRC_32_ISCSI)),
        ("CRC_32_BZIP2", Crc::<u32>::new(&CRC_32_BZIP2)),
        ("CRC_32_CKSUM", Crc::<u32>::new(&CRC_32_CKSUM)),
        ("CRC_32_JAMCRC", Crc::<u32>::new(&CRC_32_JAMCRC)),
        ("CRC_32_MPEG_2", Crc::<u32>::new(&CRC_32_MPEG_2)),
        // POSIX/CKSUM maps to CRC_32_CKSUM in the catalog
        ("CRC_32_XFER", Crc::<u32>::new(&CRC_32_XFER)),
    ];

        for (name, c) in algos.iter() {
            println!("  {}: {:08X}", name, compute_crc(c, s));
        }
        println!("");
    }
}
