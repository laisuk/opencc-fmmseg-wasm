//! Wire-format checks shared by the three auxiliary decoders.
use std::io;

pub(crate) fn pairs(text: &str) -> Vec<(char, char)> {
    text.lines()
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
        .map(|line| {
            let mut columns = line.split('\t');
            (
                columns.next().unwrap().trim().chars().next().unwrap(),
                columns.next().unwrap().trim().chars().next().unwrap(),
            )
        })
        .collect()
}

pub(crate) fn rejects_corruption(
    bytes: &[u8],
    width: usize,
    decode: impl Fn(&[u8]) -> io::Result<()>,
) {
    decode(bytes).unwrap();
    let reject = |data: &[u8]| {
        assert_eq!(decode(data).unwrap_err().kind(), io::ErrorKind::InvalidData);
    };
    for length in [0, 8, 12, 13, bytes.len() - 1] {
        reject(&bytes[..length]);
    }
    for (offset, value) in [(0, b'X'), (8, 2)] {
        let mut bad = bytes.to_vec();
        bad[offset] = value;
        reject(&bad);
    }
    for count in [0, 1, u32::MAX] {
        let mut bad = bytes.to_vec();
        bad[9..13].copy_from_slice(&count.to_le_bytes());
        reject(&bad);
    }
    let mut trailing = bytes.to_vec();
    trailing.push(0);
    reject(&trailing);
    for offset in [13, 17] {
        for invalid in [0xD800_u32, 0x11_0000] {
            let mut bad = bytes.to_vec();
            bad[offset..offset + 4].copy_from_slice(&invalid.to_le_bytes());
            reject(&bad);
        }
    }
    if width == 9 {
        for id in [8, 255] {
            let mut bad = bytes.to_vec();
            bad[21] = id;
            reject(&bad);
        }
    }
    let mut empty = bytes[..13].to_vec();
    empty[9..13].copy_from_slice(&0_u32.to_le_bytes());
    decode(&empty).unwrap();
}
