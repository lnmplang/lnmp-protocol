use crc::Crc;
use crc::{
    CRC_32_ISO_HDLC,
    CRC_32_ISCSI,
    CRC_32_BZIP2,
    CRC_32_CKSUM,
};

fn compute_crc(crc: &Crc<u32>, s: &str) -> u32 {
    let mut digest = crc.digest();
    digest.update(s.as_bytes());
    digest.finalize()
}

fn main() {
    let s = "50:r:{7:i:1;12:i:1}"; // compute CRC for nested record example (7:int)
    println!("Searching CRC match for input: {}\n", s);
    let algos: Vec<(&str, Crc<u32>)> = vec![
        ("ISO_HDLC", Crc::<u32>::new(&CRC_32_ISO_HDLC)),
        ("ISCSI", Crc::<u32>::new(&CRC_32_ISCSI)),
        ("BZIP2", Crc::<u32>::new(&CRC_32_BZIP2)),
        ("CKSUM", Crc::<u32>::new(&CRC_32_CKSUM)),
    ];
    for (name, c) in algos.iter() {
        let value = compute_crc(c, s);
        println!("{} -> {:08X}", name, value);
    }
}
