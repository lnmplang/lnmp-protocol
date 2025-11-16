use crc::{
    Crc,
    CRC_32_AIXM,
    CRC_32_AUTOSAR,
    CRC_32_BASE91_D,
    CRC_32_BZIP2,
    CRC_32_CD_ROM_EDC,
    CRC_32_CKSUM,
    CRC_32_ISCSI,
    CRC_32_ISO_HDLC,
    CRC_32_JAMCRC,
    CRC_32_MPEG_2,
    CRC_32_XFER,
};

fn compute_crc(c: &Crc<u32>, s: &str) -> u32 {
    let mut digest = c.digest();
    digest.update(s.as_bytes());
    digest.finalize()
}

fn rev_bytes(v: u32) -> u32 {
    let b = v.to_be_bytes();
    u32::from_le_bytes(b)
}

fn main() {
    let s = "12:i:14532";
    let expected: u32 = 0x36AAE667;
    println!("Testing canonical: {} expecting {:08X}", s, expected);

    let algos: Vec<(&str, Crc<u32>)> = vec![
        ("CRC_32_AIXM", Crc::<u32>::new(&CRC_32_AIXM)),
        ("CRC_32_AUTOSAR", Crc::<u32>::new(&CRC_32_AUTOSAR)),
        ("CRC_32_BASE91_D", Crc::<u32>::new(&CRC_32_BASE91_D)),
        ("CRC_32_BZIP2", Crc::<u32>::new(&CRC_32_BZIP2)),
        ("CRC_32_CD_ROM_EDC", Crc::<u32>::new(&CRC_32_CD_ROM_EDC)),
        ("CRC_32_CKSUM", Crc::<u32>::new(&CRC_32_CKSUM)),
        ("CRC_32_ISCSI", Crc::<u32>::new(&CRC_32_ISCSI)),
        ("CRC_32_ISO_HDLC", Crc::<u32>::new(&CRC_32_ISO_HDLC)),
        ("CRC_32_JAMCRC", Crc::<u32>::new(&CRC_32_JAMCRC)),
        ("CRC_32_MPEG_2", Crc::<u32>::new(&CRC_32_MPEG_2)),
        ("CRC_32_XFER", Crc::<u32>::new(&CRC_32_XFER)),
    ];

    for (name, c) in algos.iter() {
        let raw = compute_crc(c, s);
        let inv = !raw;
        let rev = rev_bytes(raw);
        let rev_inv = !rev;
        println!("{} → raw={:08X}, ~raw={:08X}, rev={:08X}, ~rev={:08X}", name, raw, inv, rev, rev_inv);

        if raw == expected || inv == expected || rev == expected || rev_inv == expected { 
            println!("MATCH: {} produced expected {:08X}", name, expected);
        }
    }
}
