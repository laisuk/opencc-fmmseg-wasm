//! Internal curated Unicode compatibility normalization.
//!
//! This is not general-purpose NFC, NFD, NFKC, or NFKD normalization. Public
//! callers use [`crate::OpenCC::normalize_unicode_compat`] or
//! [`crate::OpenCC::normalize_compat_extended`].
//!
//! The canonical built-in source is `data/Unicode_Compatibility.txt`.
//! Enabling the optional `unicode-bin` Cargo feature switches the built-in
//! runtime loader to the generated `data/Unicode_Compatibility.bin` artifact.
//! Regenerate the binary whenever the canonical text table changes.

use crate::compat_ideographs::CompatIdeographs;
use rustc_hash::FxHashMap;
#[cfg(feature = "unicode-bin")]
use std::io::{self, Write};
#[cfg(feature = "unicode-bin")]
use std::path::Path;
use std::sync::OnceLock;

#[cfg(feature = "unicode-bin")]
static UNICODE_COMPAT_DATA: &[u8] = include_bytes!("data/Unicode_Compatibility.bin");

#[cfg(not(feature = "unicode-bin"))]
static UNICODE_COMPAT_DATA: &str = include_str!("data/Unicode_Compatibility.txt");

#[cfg(feature = "unicode-bin")]
const UNICODE_BIN_MAGIC: &[u8; 8] = b"OCUNICOD";
#[cfg(feature = "unicode-bin")]
const UNICODE_BIN_VERSION: u8 = 1;
#[cfg(feature = "unicode-bin")]
const UNICODE_BIN_HEADER_LEN: usize = 13;
#[cfg(feature = "unicode-bin")]
const UNICODE_BIN_RECORD_LEN: usize = 8;

static UNICODE_COMPAT_TABLE: OnceLock<UnicodeCompat> = OnceLock::new();

/// Curated Unicode compatibility normalizer.
///
/// `UnicodeCompat` combines two independent normalization sources:
///
/// - the existing [`CompatIdeographs`] table for Unicode CJK Compatibility
///   Ideographs; and
/// - a sparse curated table loaded from `data/Unicode_Compatibility.txt`, or
///   from its generated binary artifact when `unicode-bin` is enabled.
///
/// The curated table is stored in an [`FxHashMap`] because its source characters
/// are sparse and are not confined to one compact Unicode range.
///
/// The built-in instance is immutable after initialization and can be shared
/// safely across threads.
#[derive(Debug, Clone)]
pub(crate) struct UnicodeCompat {
    compat: &'static CompatIdeographs,
    extended: FxHashMap<char, char>,
}

impl UnicodeCompat {
    /// Returns the cached built-in Unicode compatibility normalizer.
    ///
    /// Without `unicode-bin`, the canonical `data/Unicode_Compatibility.txt`
    /// table is embedded with [`include_str!`] and parsed once per process.
    /// With `unicode-bin`, the generated `data/Unicode_Compatibility.bin`
    /// artifact is embedded with [`include_bytes!`] and decoded once instead.
    /// Subsequent calls reuse the same immutable table.
    ///
    /// # Panics
    ///
    /// Panics if the bundled TXT or BIN data violates the documented mapping
    /// format. Such a failure indicates invalid crate data rather than invalid
    /// user input.
    pub(crate) fn builtin() -> &'static Self {
        UNICODE_COMPAT_TABLE.get_or_init(load_builtin_unicode_compat)
    }

    /// Builds a Unicode compatibility normalizer from mapping text.
    ///
    /// This constructor is mainly useful for tests, generated data, and advanced
    /// callers that need to validate a table before using it.
    ///
    /// # Format
    ///
    /// Each non-comment line must contain exactly two tab-separated columns:
    ///
    /// ```text
    /// source<TAB>target
    /// ```
    ///
    /// Both `source` and `target` must contain exactly one Unicode scalar value.
    /// Blank lines and lines beginning with `#` are ignored.
    ///
    /// ASCII source characters (`U+0000..=U+007F`) are rejected. This prevents
    /// the compatibility table from accidentally rewriting ASCII markup,
    /// OpenXML/XML syntax, paths, command text, or other structured content.
    ///
    /// Duplicate source entries use **last-wins** semantics, matching the stable
    /// OpenccNet `UnicodeCompat` implementation.
    ///
    /// # Errors
    ///
    /// Returns a descriptive error containing the source line number when a row:
    ///
    /// - is missing a target;
    /// - contains more than two tab-separated columns;
    /// - has an empty source or target;
    /// - has a source or target containing more than one Unicode scalar; or
    /// - uses an ASCII source character.
    #[allow(dead_code)]
    pub(crate) fn from_text(text: &str) -> Result<Self, String> {
        let entries = parse_unicode_compat_entries(text)?;
        Ok(Self::from_entries(&entries))
    }

    /// Builds the runtime lookup map from already validated mapping entries.
    ///
    /// Entry order is deliberately preserved so duplicate sources remain
    /// last-wins for both TXT and BIN loading paths.
    fn from_entries(entries: &[(char, char)]) -> Self {
        let mut extended = FxHashMap::default();

        for &(src, dst) in entries {
            extended.insert(src, dst);
        }

        Self {
            compat: CompatIdeographs::builtin(),
            extended,
        }
    }

    /// Normalizes one character using only the curated extended table.
    ///
    /// CJK Compatibility Ideograph normalization is **not** applied by this
    /// method. Use [`normalize_all_char`](Self::normalize_all_char) when both
    /// tables should participate.
    ///
    /// Characters without an extended mapping are returned unchanged.
    #[inline(always)]
    pub(crate) fn normalize_char(&self, ch: char) -> char {
        if ch.is_ascii() {
            return ch;
        }

        self.extended.get(&ch).copied().unwrap_or(ch)
    }

    /// Normalizes one character using CJK Compatibility Ideographs first, then
    /// the curated extended table.
    ///
    /// The existing [`CompatIdeographs`] mapping has precedence. If it changes
    /// the input character, that result is returned directly and is **not** fed
    /// through the extended table a second time. Otherwise, the curated table is
    /// consulted.
    ///
    /// This precedence matches the stable OpenccNet `UnicodeCompat.NormalizeAll`
    /// behavior and avoids accidental chained remapping between the two tables.
    #[inline(always)]
    pub(crate) fn normalize_all_char(&self, ch: char) -> char {
        if ch.is_ascii() {
            return ch;
        }

        let compat = self.compat.normalize_char(ch);
        if compat != ch {
            return compat;
        }

        self.extended.get(&ch).copied().unwrap_or(ch)
    }

    /// Normalizes text using only the curated mappings from
    /// `Unicode_Compatibility.txt` or its generated BIN equivalent.
    ///
    /// This method does not apply [`CompatIdeographs`]. It allocates one output
    /// [`String`] and preserves every unmapped character unchanged.
    ///
    /// Because every mapping is one Unicode scalar to one Unicode scalar, the
    /// number of Unicode scalar values in the output is identical to the input,
    /// although the UTF-8 byte length may differ.
    pub(crate) fn normalize(&self, input: &str) -> String {
        self.normalize_impl(input, false)
    }

    /// Normalizes text using both the built-in CJK Compatibility Ideograph
    /// mappings and the curated extended table.
    ///
    /// For each Unicode scalar value, [`CompatIdeographs`] is consulted first.
    /// Only when it leaves the character unchanged is the curated extended map
    /// consulted. The result of one table is never passed through the other table
    /// again.
    ///
    /// This is the intended low-level implementation for a higher-level
    /// `normalize_compat_extended()` API.
    pub(crate) fn normalize_all(&self, input: &str) -> String {
        self.normalize_impl(input, true)
    }

    fn normalize_impl(&self, input: &str, include_compat: bool) -> String {
        let mut output = String::with_capacity(input.len());

        for ch in input.chars() {
            output.push(if include_compat {
                self.normalize_all_char(ch)
            } else {
                self.normalize_char(ch)
            });
        }

        output
    }

    /// Normalizes a mutable character slice using only the curated extended
    /// table.
    ///
    /// This is useful when callers already own a reusable `Vec<char>` before
    /// segmentation or conversion.
    #[cfg(test)]
    pub(crate) fn normalize_in_place(&self, chars: &mut [char]) {
        for ch in chars {
            *ch = self.normalize_char(*ch);
        }
    }

    /// Normalizes a mutable character slice using both compatibility tables.
    ///
    /// [`CompatIdeographs`] has precedence over the curated extended table for
    /// each character, exactly as in [`normalize_all`](Self::normalize_all).
    #[cfg(test)]
    pub(crate) fn normalize_all_in_place(&self, chars: &mut [char]) {
        for ch in chars {
            *ch = self.normalize_all_char(*ch);
        }
    }
}

/// Parses canonical tab-separated Unicode compatibility entries.
///
/// `Unicode_Compatibility.txt` is the source of truth. The parser is kept
/// unconditional because `dict-generate --unicode` also uses it to create the
/// optional generated runtime binary artifact.
///
/// Duplicate sources are preserved in the returned vector. Last-wins behavior
/// is applied later by [`UnicodeCompat::from_entries`], so TXT and BIN loading
/// have identical semantics.
pub(crate) fn parse_unicode_compat_entries(text: &str) -> Result<Vec<(char, char)>, String> {
    let mut entries = Vec::new();

    for (index, raw_line) in text.lines().enumerate() {
        let line_no = index + 1;

        if raw_line.trim().is_empty() || raw_line.trim_start().starts_with('#') {
            continue;
        }

        let mut parts = raw_line.split('\t');

        let src_text = parts
            .next()
            .map(str::trim)
            .ok_or_else(|| format!("line {line_no}: missing source"))?;

        let dst_text = parts
            .next()
            .map(str::trim)
            .ok_or_else(|| format!("line {line_no}: missing target"))?;

        if parts.next().is_some() {
            return Err(format!("line {line_no}: too many columns"));
        }

        let src = single_char(src_text, line_no, "source")?;
        validate_unicode_source(src, line_no)?;
        let dst = single_char(dst_text, line_no, "target")?;

        entries.push((src, dst));
    }

    Ok(entries)
}

/// Parses generated Unicode compatibility binary data.
///
/// Binary format:
///
/// - magic: `OCUNICOD`
/// - version: `1`
/// - record count: `u32` little-endian
/// - records: `source: u32`, `target: u32`
///
/// The binary artifact is generated from canonical `Unicode_Compatibility.txt`
/// and preserves entry order so duplicate sources retain last-wins semantics.
#[cfg(feature = "unicode-bin")]
pub fn parse_unicode_compat_bin(bytes: &[u8]) -> io::Result<Vec<(char, char)>> {
    if bytes.len() < UNICODE_BIN_HEADER_LEN {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "unicode compatibility binary is too short: expected at least {UNICODE_BIN_HEADER_LEN} bytes, got {}",
                bytes.len()
            ),
        ));
    }

    if &bytes[..UNICODE_BIN_MAGIC.len()] != UNICODE_BIN_MAGIC {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid unicode compatibility binary magic",
        ));
    }

    let version = bytes[UNICODE_BIN_MAGIC.len()];
    if version != UNICODE_BIN_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unsupported unicode compatibility binary version: {version}"),
        ));
    }

    let count_start = UNICODE_BIN_MAGIC.len() + 1;
    let count = u32::from_le_bytes(
        bytes[count_start..count_start + 4]
            .try_into()
            .expect("count slice length is fixed"),
    ) as usize;

    let expected_len = UNICODE_BIN_HEADER_LEN
        .checked_add(
            count
                .checked_mul(UNICODE_BIN_RECORD_LEN)
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        "unicode compatibility binary record count overflows",
                    )
                })?,
        )
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "unicode compatibility binary length overflows",
            )
        })?;

    if bytes.len() != expected_len {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "invalid unicode compatibility binary length: expected {expected_len} bytes for {count} records, got {}",
                bytes.len()
            ),
        ));
    }

    let mut entries = Vec::with_capacity(count);
    let mut pos = UNICODE_BIN_HEADER_LEN;

    for index in 0..count {
        let src_u32 = u32::from_le_bytes(
            bytes[pos..pos + 4]
                .try_into()
                .expect("source slice length is fixed"),
        );
        let dst_u32 = u32::from_le_bytes(
            bytes[pos + 4..pos + 8]
                .try_into()
                .expect("target slice length is fixed"),
        );
        pos += UNICODE_BIN_RECORD_LEN;

        let src = char::from_u32(src_u32).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("record {index}: invalid source Unicode scalar: U+{src_u32:04X}"),
            )
        })?;

        let dst = char::from_u32(dst_u32).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("record {index}: invalid target Unicode scalar: U+{dst_u32:04X}"),
            )
        })?;

        if let Err(err) = validate_unicode_source(src, index + 1) {
            return Err(io::Error::new(io::ErrorKind::InvalidData, err));
        }

        entries.push((src, dst));
    }

    Ok(entries)
}

/// Writes Unicode compatibility entries in the compact generated binary format.
///
/// Entry order is preserved deliberately so duplicate source rows retain the
/// canonical TXT table's last-wins behavior when loaded into the runtime map.
#[cfg(feature = "unicode-bin")]
pub fn write_unicode_compat_bin<W: Write>(
    entries: &[(char, char)],
    mut writer: W,
) -> io::Result<()> {
    let count = u32::try_from(entries.len()).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "too many unicode compatibility entries for binary format: {}",
                entries.len()
            ),
        )
    })?;

    writer.write_all(UNICODE_BIN_MAGIC)?;
    writer.write_all(&[UNICODE_BIN_VERSION])?;
    writer.write_all(&count.to_le_bytes())?;

    for &(src, dst) in entries {
        writer.write_all(&(src as u32).to_le_bytes())?;
        writer.write_all(&(dst as u32).to_le_bytes())?;
    }

    Ok(())
}

/// Writes Unicode compatibility entries to a generated binary file.
///
/// Prefer [`write_unicode_compat_bin_from_txt_file`] when regenerating the
/// checked-in runtime artifact from canonical text data.
#[cfg(feature = "unicode-bin")]
pub fn write_unicode_compat_bin_file<P: AsRef<Path>>(
    entries: &[(char, char)],
    path: P,
) -> io::Result<()> {
    let file = std::fs::File::create(path)?;
    let mut writer = io::BufWriter::new(file);
    write_unicode_compat_bin(entries, &mut writer)?;
    writer.flush()
}

/// Generates `Unicode_Compatibility.bin` from canonical text data.
///
/// This is the public helper intended for `dict-generate --unicode`. The input
/// TXT file remains the source of truth; the BIN file is only a generated
/// runtime artifact consumed when the optional `unicode-bin` feature is enabled.
#[cfg(feature = "unicode-bin")]
pub fn write_unicode_compat_bin_from_txt_file<P: AsRef<Path>, Q: AsRef<Path>>(
    input_txt: P,
    output_bin: Q,
) -> io::Result<()> {
    let text = std::fs::read_to_string(input_txt)?;
    let entries = parse_unicode_compat_entries(&text)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    write_unicode_compat_bin_file(&entries, output_bin)
}

#[cfg(feature = "unicode-bin")]
fn load_builtin_unicode_compat() -> UnicodeCompat {
    let entries = parse_unicode_compat_bin(UNICODE_COMPAT_DATA)
        .unwrap_or_else(|err| panic!("invalid built-in Unicode_Compatibility.bin: {err}"));

    UnicodeCompat::from_entries(&entries)
}

#[cfg(not(feature = "unicode-bin"))]
fn load_builtin_unicode_compat() -> UnicodeCompat {
    UnicodeCompat::from_text(UNICODE_COMPAT_DATA)
        .unwrap_or_else(|err| panic!("invalid built-in Unicode_Compatibility.txt: {err}"))
}

fn validate_unicode_source(src: char, line_no: usize) -> Result<(), String> {
    if src.is_ascii() {
        return Err(format!(
            "line {line_no}: source must not be an ASCII character"
        ));
    }

    Ok(())
}

fn single_char(text: &str, line_no: usize, field: &str) -> Result<char, String> {
    let mut chars = text.chars();

    let ch = chars
        .next()
        .ok_or_else(|| format!("line {line_no}: empty {field}"))?;

    if chars.next().is_some() {
        return Err(format!(
            "line {line_no}: {field} must be exactly one Unicode scalar value"
        ));
    }

    Ok(ch)
}

/// Normalizes text using only the built-in curated
/// `Unicode_Compatibility.txt` table or its generated BIN equivalent.
///
/// This convenience wrapper does **not** apply CJK Compatibility Ideograph
/// normalization. Use [`normalize_unicode_compat_all`] when both tables are
/// desired.
pub(crate) fn normalize_unicode_compat(input: &str) -> String {
    UnicodeCompat::builtin().normalize(input)
}

/// Normalizes text using both the built-in CJK Compatibility Ideograph table
/// and the curated Unicode compatibility table.
///
/// CJK Compatibility Ideograph mappings have precedence for each input
/// character. The extended table is consulted only when the compatibility
/// ideograph table leaves that character unchanged.
///
/// This function is useful as the implementation behind a higher-level
/// `OpenCC::normalize_compat_extended()` method.
pub(crate) fn normalize_unicode_compat_all(input: &str) -> String {
    UnicodeCompat::builtin().normalize_all(input)
}

#[cfg(all(test, feature = "unicode-bin"))]
mod unicode_bin_tests {
    use super::{parse_unicode_compat_bin, parse_unicode_compat_entries};

    #[test]
    fn builtin_unicode_bin_matches_builtin_unicode_txt() {
        let txt_entries = parse_unicode_compat_entries(include_str!(
            "data/Unicode_Compatibility.txt"
        ))
            .expect("built-in Unicode_Compatibility.txt should parse");

        let bin_entries = parse_unicode_compat_bin(include_bytes!(
            "data/Unicode_Compatibility.bin"
        ))
            .expect("built-in Unicode_Compatibility.bin should parse");

        assert_eq!(
            txt_entries, bin_entries,
            "Unicode_Compatibility.bin must be regenerated from Unicode_Compatibility.txt"
        );
    }

    #[test]
    fn unicode_bin_round_trip_preserves_duplicate_order_and_astral_scalars() {
        use super::write_unicode_compat_bin;

        let entries = vec![
            ('聼', '听'),
            ('𠮷', '𠮟'),
            ('聼', '聽'),
        ];
        let mut bytes = Vec::new();

        write_unicode_compat_bin(&entries, &mut bytes).unwrap();
        let decoded = parse_unicode_compat_bin(&bytes).unwrap();

        assert_eq!(decoded, entries);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_comments_blank_lines_and_pairs() {
        let table = UnicodeCompat::from_text(
            "\
# comment

⺙\t攵
聼\t聽
",
        )
            .unwrap();

        assert_eq!(table.normalize("⺙聼"), "攵聽");
    }

    #[test]
    fn parser_preserves_duplicate_entries_in_source_order() {
        let entries = parse_unicode_compat_entries(
            "\
聼\t听
聼\t聽
",
        )
            .unwrap();

        assert_eq!(entries, vec![('聼', '听'), ('聼', '聽')]);
    }

    #[test]
    fn extended_normalization_does_not_apply_compat_ideographs() {
        let table = UnicodeCompat::from_text("⺙\t攵\n").unwrap();

        assert_eq!(table.normalize("金⺙"), "金攵");
    }

    #[test]
    fn normalize_all_combines_compat_and_extended_tables() {
        let table = UnicodeCompat::from_text("⺙\t攵\n").unwrap();

        assert_eq!(table.normalize_all("金⺙"), "金攵");
    }

    #[test]
    fn compat_mapping_has_precedence_without_chained_remapping() {
        // 金 is normalized by CompatIdeographs to 金. If normalize_all were
        // implemented as two chained full passes, the extended 金 -> 銀 entry
        // would incorrectly turn the result into 銀.
        let table = UnicodeCompat::from_text("金\t銀\n").unwrap();

        assert_eq!(table.normalize("金金"), "金銀");
        assert_eq!(table.normalize_all("金金"), "金銀");
    }

    #[test]
    fn duplicate_sources_are_last_wins() {
        let table = UnicodeCompat::from_text(
            "\
聼\t听
聼\t聽
",
        )
            .unwrap();

        assert_eq!(table.normalize("聼"), "聽");
    }

    #[test]
    fn rejects_ascii_source() {
        let err = UnicodeCompat::from_text("A\tＢ\n").unwrap_err();

        assert_eq!(err, "line 1: source must not be an ASCII character");
    }

    #[test]
    fn rejects_missing_target() {
        let err = UnicodeCompat::from_text("聼\n").unwrap_err();

        assert_eq!(err, "line 1: missing target");
    }

    #[test]
    fn rejects_too_many_columns() {
        let err = UnicodeCompat::from_text("聼\t聽\textra\n").unwrap_err();

        assert_eq!(err, "line 1: too many columns");
    }

    #[test]
    fn rejects_empty_source_or_target() {
        assert_eq!(
            UnicodeCompat::from_text("\t聽\n").unwrap_err(),
            "line 1: empty source"
        );
        assert_eq!(
            UnicodeCompat::from_text("聼\t\n").unwrap_err(),
            "line 1: empty target"
        );
    }

    #[test]
    fn rejects_multi_scalar_source_or_target() {
        assert_eq!(
            UnicodeCompat::from_text("聼x\t聽\n").unwrap_err(),
            "line 1: source must be exactly one Unicode scalar value"
        );
        assert_eq!(
            UnicodeCompat::from_text("聼\t聽x\n").unwrap_err(),
            "line 1: target must be exactly one Unicode scalar value"
        );
    }

    #[test]
    fn supports_astral_source_and_target_scalars() {
        let table = UnicodeCompat::from_text("𠮷\t𠮟\n").unwrap();

        assert_eq!(table.normalize("A𠮷B"), "A𠮟B");
    }

    #[test]
    fn ascii_and_unmapped_text_stay_unchanged() {
        let table = UnicodeCompat::from_text("聼\t聽\n").unwrap();

        assert_eq!(table.normalize("ABC123 中文"), "ABC123 中文");
        assert_eq!(table.normalize_all("ABC123 中文"), "ABC123 中文");
    }

    #[test]
    fn normalize_in_place_uses_extended_table_only() {
        let table = UnicodeCompat::from_text("⺙\t攵\n").unwrap();
        let mut chars: Vec<char> = "金⺙".chars().collect();

        table.normalize_in_place(&mut chars);

        assert_eq!(chars.into_iter().collect::<String>(), "金攵");
    }

    #[test]
    fn normalize_all_in_place_combines_both_tables() {
        let table = UnicodeCompat::from_text("⺙\t攵\n").unwrap();
        let mut chars: Vec<char> = "金⺙".chars().collect();

        table.normalize_all_in_place(&mut chars);

        assert_eq!(chars.into_iter().collect::<String>(), "金攵");
    }

    #[test]
    fn empty_table_is_valid() {
        let table = UnicodeCompat::from_text("# comments only\n\n").unwrap();

        assert_eq!(table.normalize("聼"), "聼");
        assert_eq!(table.normalize_all("金"), "金");
    }

    #[test]
    fn builtin_is_cached() {
        let a = UnicodeCompat::builtin();
        let b = UnicodeCompat::builtin();

        assert!(std::ptr::eq(a, b));
    }
}
