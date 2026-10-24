//! Build helpers for the original version-1 auxiliary BIN artifacts.
//! Runtime table initialization remains independent of these generators.

use crate::{
    compat_ideographs::CompatIdeographs, detofu::parse_tofu_entries, unicode_compat::UnicodeCompat,
};
use std::fs::{read_to_string, File};
use std::io::{self, BufWriter, Write};
use std::path::Path;

fn invalid_data(message: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn write_header(writer: &mut impl Write, magic: &[u8; 8], count: usize) -> io::Result<()> {
    let count = u32::try_from(count)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "too many BIN entries"))?;
    writer.write_all(magic)?;
    writer.write_all(&[1])?;
    writer.write_all(&count.to_le_bytes())
}

/// Generates `CharactersTofu.bin` from canonical TXT data.
/// Records are two little-endian Unicode scalars followed by the extension ID (0–7).
pub fn write_tofu_bin_from_txt_file(
    input_txt: impl AsRef<Path>,
    output_bin: impl AsRef<Path>,
) -> io::Result<()> {
    let entries = parse_tofu_entries(&read_to_string(input_txt)?).map_err(invalid_data)?;
    let mut writer = BufWriter::new(File::create(output_bin)?);
    write_header(&mut writer, b"OCTFTOFU", entries.len())?;
    for (src, dst, level) in entries {
        writer.write_all(&(src as u32).to_le_bytes())?;
        writer.write_all(&(dst as u32).to_le_bytes())?;
        writer.write_all(&[level as u8])?;
    }
    writer.flush()
}

/// Generates `CJK_Compatibility_Ideographs.bin` in its original version-1 format.
pub fn write_compat_bin_from_txt_file(
    input_txt: impl AsRef<Path>,
    output_bin: impl AsRef<Path>,
) -> io::Result<()> {
    let text = read_to_string(input_txt)?;
    CompatIdeographs::from_text(&text).map_err(invalid_data)?;
    write_pairs(&text, output_bin.as_ref(), b"OCCOMPAT")
}

/// Generates `Unicode_Compatibility.bin` in its original version-1 format.
/// Source order is preserved, including duplicate rows and last-wins semantics.
pub fn write_unicode_compat_bin_from_txt_file(
    input_txt: impl AsRef<Path>,
    output_bin: impl AsRef<Path>,
) -> io::Result<()> {
    let text = read_to_string(input_txt)?;
    UnicodeCompat::from_text(&text).map_err(invalid_data)?;
    write_pairs(&text, output_bin.as_ref(), b"OCUNICOD")
}

fn write_pairs(text: &str, output: &Path, magic: &[u8; 8]) -> io::Result<()> {
    // The caller validated every row with the corresponding runtime TXT parser.
    // Read the original rows rather than the lookup map to retain duplicate order.
    let entries: Vec<_> = text
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
        .map(|line| {
            let mut columns = line.split('\t');
            (
                columns.next().unwrap().trim().chars().next().unwrap(),
                columns.next().unwrap().trim().chars().next().unwrap(),
            )
        })
        .collect();
    let mut writer = BufWriter::new(File::create(output)?);
    write_header(&mut writer, magic, entries.len())?;
    for (src, dst) in entries {
        writer.write_all(&(src as u32).to_le_bytes())?;
        writer.write_all(&(dst as u32).to_le_bytes())?;
    }
    writer.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regenerated_artifacts_match_and_load_with_runtime_decoders() {
        let dir = tempfile::tempdir().unwrap();
        let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/data");
        let output = dir.path().join("generated.bin");
        write_tofu_bin_from_txt_file(data.join("CharactersTofu.txt"), &output).unwrap();
        let bytes = std::fs::read(&output).unwrap();
        assert_eq!(bytes, include_bytes!("data/CharactersTofu.bin"));
        assert_eq!(
            crate::detofu::parse_tofu_bin(&bytes).unwrap(),
            parse_tofu_entries(include_str!("data/CharactersTofu.txt")).unwrap()
        );

        write_compat_bin_from_txt_file(data.join("CJK_Compatibility_Ideographs.txt"), &output)
            .unwrap();
        let bytes = std::fs::read(&output).unwrap();
        assert_eq!(
            bytes,
            include_bytes!("data/CJK_Compatibility_Ideographs.bin")
        );
        assert_eq!(
            crate::compat_ideographs::parse_compat_bin(&bytes).unwrap(),
            crate::auxiliary_bin_tests::pairs(include_str!(
                "data/CJK_Compatibility_Ideographs.txt"
            ))
        );

        write_unicode_compat_bin_from_txt_file(data.join("Unicode_Compatibility.txt"), &output)
            .unwrap();
        let bytes = std::fs::read(&output).unwrap();
        assert_eq!(bytes, include_bytes!("data/Unicode_Compatibility.bin"));
        assert_eq!(
            crate::unicode_compat::parse_unicode_compat_bin(&bytes).unwrap(),
            crate::auxiliary_bin_tests::pairs(include_str!("data/Unicode_Compatibility.txt"))
        );
    }
}
