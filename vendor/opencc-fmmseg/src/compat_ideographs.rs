//! Internal CJK Compatibility Ideograph normalization.
//!
//! The built-in UnicodeData-derived table is parsed once and cached in dense
//! lookup tables. Public callers use [`crate::OpenCC::normalize_compat`].

#[cfg(feature = "compat-bin")]
use std::io::{self, Write};
#[cfg(feature = "compat-bin")]
use std::path::Path;
use std::sync::OnceLock;

#[cfg(feature = "compat-bin")]
static COMPAT_DATA: &[u8] = include_bytes!("data/CJK_Compatibility_Ideographs.bin");

#[cfg(not(feature = "compat-bin"))]
static COMPAT_DATA: &str = include_str!("data/CJK_Compatibility_Ideographs.txt");

#[cfg(feature = "compat-bin")]
const COMPAT_BIN_MAGIC: &[u8; 8] = b"OCCOMPAT";
#[cfg(feature = "compat-bin")]
const COMPAT_BIN_VERSION: u8 = 1;
#[cfg(feature = "compat-bin")]
const COMPAT_BIN_HEADER_LEN: usize = 13;
#[cfg(feature = "compat-bin")]
const COMPAT_BIN_RECORD_LEN: usize = 8;

const BMP_START: u32 = 0xF900;
const BMP_END: u32 = 0xFAFF;
const BMP_LEN: usize = (BMP_END - BMP_START + 1) as usize;

const SUPP_START: u32 = 0x2F800;
const SUPP_END: u32 = 0x2FA1F;
const SUPP_LEN: usize = (SUPP_END - SUPP_START + 1) as usize;

static COMPAT_TABLE: OnceLock<CompatIdeographs> = OnceLock::new();

/// Dense lookup tables for CJK Compatibility Ideograph normalization.
///
/// The built-in table maps compatibility ideographs to their UnicodeData
/// decomposition targets. Each supported range is stored densely for fast
/// character lookup. Characters without a mapping are initialized to themselves,
/// so normalization preserves unmapped compatibility ideographs unchanged.
#[derive(Debug, Clone)]
pub(crate) struct CompatIdeographs {
    bmp: [char; BMP_LEN],
    supp: [char; SUPP_LEN],
}

impl Default for CompatIdeographs {
    fn default() -> Self {
        let mut bmp = ['\0'; BMP_LEN];
        let mut supp = ['\0'; SUPP_LEN];

        for (i, slot) in bmp.iter_mut().enumerate() {
            *slot = char::from_u32(BMP_START + i as u32).unwrap();
        }

        for (i, slot) in supp.iter_mut().enumerate() {
            *slot = char::from_u32(SUPP_START + i as u32).unwrap();
        }

        Self { bmp, supp }
    }
}

impl CompatIdeographs {
    /// Returns the cached built-in compatibility ideograph normalizer.
    ///
    /// The bundled mapping data is parsed at most once per process. Subsequent
    /// calls reuse the same dense lookup tables.
    pub(crate) fn builtin() -> &'static Self {
        COMPAT_TABLE.get_or_init(load_builtin_compat_table)
    }

    /// Builds a compatibility ideograph normalizer from UTF-8 mapping text.
    ///
    /// This is mainly useful for tests or custom data. The expected format is
    /// one tab-separated `source<TAB>target` pair per line, with `#` comments
    /// and blank lines ignored.
    #[cfg(not(feature = "compat-bin"))]
    pub(crate) fn from_text(text: &str) -> Result<Self, String> {
        Self::from_entries(&parse_compat_entries(text)?)
    }

    fn from_entries(entries: &[(char, char)]) -> Result<Self, String> {
        let mut table = Self::default();

        for (index, &(src, dst)) in entries.iter().enumerate() {
            table.set(src, dst, index + 1)?;
        }

        Ok(table)
    }

    fn set(&mut self, src: char, dst: char, line_no: usize) -> Result<(), String> {
        let u = src as u32;

        if (BMP_START..=BMP_END).contains(&u) {
            self.bmp[(u - BMP_START) as usize] = dst;
            return Ok(());
        }

        if (SUPP_START..=SUPP_END).contains(&u) {
            self.supp[(u - SUPP_START) as usize] = dst;
            return Ok(());
        }

        validate_compat_source(src, line_no)
    }

    /// Normalizes one character if it has a compatibility mapping.
    ///
    /// Characters outside the CJK Compatibility Ideograph ranges, and
    /// compatibility ideographs without UnicodeData decomposition targets, are
    /// returned unchanged.
    ///
    #[inline(always)]
    pub(crate) fn normalize_char(&self, ch: char) -> char {
        let u = ch as u32;

        if (BMP_START..=BMP_END).contains(&u) {
            return self.bmp[(u - BMP_START) as usize];
        }

        if (SUPP_START..=SUPP_END).contains(&u) {
            return self.supp[(u - SUPP_START) as usize];
        }

        ch
    }

    /// Normalizes a mutable character slice in place.
    ///
    /// This is useful when text has already been collected into a reusable
    /// `Vec<char>` before segmentation.
    ///
    #[allow(dead_code)]
    pub(crate) fn normalize_in_place(&self, chars: &mut [char]) {
        for ch in chars {
            *ch = self.normalize_char(*ch);
        }
    }

    /// Normalizes all mapped CJK Compatibility Ideographs in `input`.
    ///
    /// This returns a new string and leaves ordinary Chinese text unchanged.
    ///
    pub(crate) fn normalize(&self, input: &str) -> String {
        let mut output = String::with_capacity(input.len());

        for ch in input.chars() {
            output.push(self.normalize_char(ch));
        }

        output
    }
}

/// Parses canonical tab-separated CJK compatibility ideograph entries.
///
/// `CJK_Compatibility_Ideographs.txt` uses one mapping per non-comment line:
/// `compat_char<TAB>unified_char`.
///
/// This parser remains unconditional because the text file is the canonical
/// source and `dict-generate --compat` uses it to create the generated binary
/// runtime artifact.
pub(crate) fn parse_compat_entries(text: &str) -> Result<Vec<(char, char)>, String> {
    let mut entries = Vec::new();

    for (index, raw_line) in text.lines().enumerate() {
        let line_no = index + 1;
        let line = raw_line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut parts = line.split('\t');

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
        let dst = single_char(dst_text, line_no, "target")?;
        validate_compat_source(src, line_no)?;

        entries.push((src, dst));
    }

    Ok(entries)
}

/// Parses built-in CJK compatibility ideograph binary data.
///
/// The binary format is intentionally compatibility-table-specific and stable:
///
/// - magic: `OCCOMPAT`
/// - version: `1`
/// - record count: `u32` little-endian
/// - records: `compat: u32`, `unified: u32`
///
/// This parser is used by the optional `compat-bin` runtime loader. The
/// `CJK_Compatibility_Ideographs.bin` file it reads is a generated runtime
/// artifact; the canonical source remains `CJK_Compatibility_Ideographs.txt`.
#[cfg(feature = "compat-bin")]
pub fn parse_compat_bin(bytes: &[u8]) -> io::Result<Vec<(char, char)>> {
    if bytes.len() < COMPAT_BIN_HEADER_LEN {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "compat binary is too short: expected at least {COMPAT_BIN_HEADER_LEN} bytes, got {}",
                bytes.len()
            ),
        ));
    }

    if &bytes[..COMPAT_BIN_MAGIC.len()] != COMPAT_BIN_MAGIC {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid compat binary magic",
        ));
    }

    let version = bytes[COMPAT_BIN_MAGIC.len()];
    if version != COMPAT_BIN_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unsupported compat binary version: {version}"),
        ));
    }

    let count_start = COMPAT_BIN_MAGIC.len() + 1;
    let count = u32::from_le_bytes(
        bytes[count_start..count_start + 4]
            .try_into()
            .expect("count slice length is fixed"),
    ) as usize;

    let expected_len = COMPAT_BIN_HEADER_LEN
        .checked_add(count.checked_mul(COMPAT_BIN_RECORD_LEN).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "compat binary record count overflows",
            )
        })?)
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "compat binary length overflows")
        })?;

    if bytes.len() != expected_len {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "invalid compat binary length: expected {expected_len} bytes for {count} records, got {}",
                bytes.len()
            ),
        ));
    }

    let mut entries = Vec::with_capacity(count);
    let mut pos = COMPAT_BIN_HEADER_LEN;

    for index in 0..count {
        let compat_u32 = u32::from_le_bytes(
            bytes[pos..pos + 4]
                .try_into()
                .expect("compat slice length is fixed"),
        );
        let unified_u32 = u32::from_le_bytes(
            bytes[pos + 4..pos + 8]
                .try_into()
                .expect("unified slice length is fixed"),
        );
        pos += COMPAT_BIN_RECORD_LEN;

        let compat = char::from_u32(compat_u32).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("record {index}: invalid compatibility Unicode scalar: U+{compat_u32:04X}"),
            )
        })?;

        let unified = char::from_u32(unified_u32).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("record {index}: invalid unified Unicode scalar: U+{unified_u32:04X}"),
            )
        })?;

        if let Err(err) = validate_compat_source(compat, index + 1) {
            return Err(io::Error::new(io::ErrorKind::InvalidData, err));
        }

        entries.push((compat, unified));
    }

    Ok(entries)
}

/// Writes CJK compatibility ideograph entries in the compact binary format.
///
/// This helper writes the generated representation consumed when the optional
/// `compat-bin` feature is enabled. The output should be derived from canonical
/// `CJK_Compatibility_Ideographs.txt` data and committed as
/// `CJK_Compatibility_Ideographs.bin`.
#[cfg(feature = "compat-bin")]
pub fn write_compat_bin<W: Write>(entries: &[(char, char)], mut writer: W) -> io::Result<()> {
    let count = u32::try_from(entries.len()).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "too many compat entries for binary format: {}",
                entries.len()
            ),
        )
    })?;

    writer.write_all(COMPAT_BIN_MAGIC)?;
    writer.write_all(&[COMPAT_BIN_VERSION])?;
    writer.write_all(&count.to_le_bytes())?;

    for &(compat, unified) in entries {
        writer.write_all(&(compat as u32).to_le_bytes())?;
        writer.write_all(&(unified as u32).to_le_bytes())?;
    }

    Ok(())
}

/// Writes CJK compatibility ideograph entries to a binary file.
///
/// Prefer [`write_compat_bin_from_txt_file`] when regenerating the checked-in
/// runtime artifact from canonical text data.
#[cfg(feature = "compat-bin")]
pub fn write_compat_bin_file<P: AsRef<Path>>(entries: &[(char, char)], path: P) -> io::Result<()> {
    let file = std::fs::File::create(path)?;
    let mut writer = io::BufWriter::new(file);
    write_compat_bin(entries, &mut writer)?;
    writer.flush()
}

/// Generates a CJK compatibility ideograph binary file from canonical text data.
///
/// This is the public helper used by `dict-generate --compat`. The input text is
/// the canonical source of truth; the output binary is only the generated
/// runtime artifact used when the optional `compat-bin` feature is enabled.
#[cfg(feature = "compat-bin")]
pub fn write_compat_bin_from_txt_file<P: AsRef<Path>, Q: AsRef<Path>>(
    input_txt: P,
    output_bin: Q,
) -> io::Result<()> {
    let text = std::fs::read_to_string(input_txt)?;
    let entries = parse_compat_entries(&text)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    write_compat_bin_file(&entries, output_bin)
}

#[cfg(feature = "compat-bin")]
fn load_builtin_compat_table() -> CompatIdeographs {
    let entries = parse_compat_bin(COMPAT_DATA)
        .unwrap_or_else(|err| panic!("invalid built-in CJK_Compatibility_Ideographs.bin: {err}"));

    CompatIdeographs::from_entries(&entries)
        .unwrap_or_else(|err| panic!("invalid built-in CJK_Compatibility_Ideographs.bin: {err}"))
}

#[cfg(not(feature = "compat-bin"))]
fn load_builtin_compat_table() -> CompatIdeographs {
    CompatIdeographs::from_text(COMPAT_DATA)
        .unwrap_or_else(|err| panic!("invalid built-in CJK_Compatibility_Ideographs.txt: {err}"))
}

fn validate_compat_source(src: char, line_no: usize) -> Result<(), String> {
    let u = src as u32;

    if (BMP_START..=BMP_END).contains(&u) || (SUPP_START..=SUPP_END).contains(&u) {
        return Ok(());
    }

    Err(format!(
        "line {line_no}: source U+{u:04X} is outside CJK Compatibility Ideograph ranges"
    ))
}

fn single_char(text: &str, line_no: usize, field: &str) -> Result<char, String> {
    let mut chars = text.chars();

    let ch = chars
        .next()
        .ok_or_else(|| format!("line {line_no}: empty {field}"))?;

    if chars.next().is_some() {
        return Err(format!(
            "line {line_no}: {field} must be exactly one character"
        ));
    }

    Ok(ch)
}

/// Normalizes mapped CJK Compatibility Ideographs using the built-in table.
///
/// This is a convenience wrapper around [`CompatIdeographs::builtin`] and
/// [`CompatIdeographs::normalize`]. It performs Unicode compatibility
/// normalization as an optional pre-pass before OpenCC conversion.
///
pub(crate) fn normalize_compat_ideographs(input: &str) -> String {
    CompatIdeographs::builtin().normalize(input)
}

#[cfg(all(test, feature = "compat-bin"))]
mod compat_bin_tests {
    use super::{parse_compat_bin, parse_compat_entries};

    #[test]
    fn builtin_compat_bin_matches_builtin_compat_txt() {
        let txt_entries =
            parse_compat_entries(include_str!("data/CJK_Compatibility_Ideographs.txt"))
                .expect("built-in CJK_Compatibility_Ideographs.txt should parse");

        let bin_entries = parse_compat_bin(include_bytes!("data/CJK_Compatibility_Ideographs.bin"))
            .expect("built-in CJK_Compatibility_Ideographs.bin should parse");

        assert_eq!(
            txt_entries, bin_entries,
            "CJK_Compatibility_Ideographs.bin must be regenerated from CJK_Compatibility_Ideographs.txt"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_bmp_compat_ideographs() {
        let table = CompatIdeographs::builtin();

        assert_eq!(table.normalize("金庸"), "金庸");
        assert_eq!(table.normalize("龜龜"), "龜龜");
        assert_eq!(table.normalize("樂天"), "樂天");
    }

    #[test]
    fn leaves_normal_text_unchanged() {
        let table = CompatIdeographs::builtin();

        assert_eq!(table.normalize("金庸寫小說"), "金庸寫小說");
        assert_eq!(table.normalize("abc123，測試。"), "abc123，測試。");
    }

    #[test]
    fn normalizes_in_place() {
        let table = CompatIdeographs::builtin();

        let mut chars: Vec<char> = "金龜樂".chars().collect();
        table.normalize_in_place(&mut chars);

        assert_eq!(chars.iter().collect::<String>(), "金龜樂");
    }

    #[test]
    fn unmapped_compat_ideographs_stay_self() {
        let table = CompatIdeographs::builtin();

        // U+FA11 is documented as having no UnicodeData decomposition mapping.
        assert_eq!(table.normalize_char('﨑'), '﨑');
    }

    #[cfg(not(feature = "compat-bin"))]
    #[test]
    fn parses_custom_table() {
        let table = CompatIdeographs::from_text(
            "\
# comment
豈\t豈
金\t金
",
        )
        .unwrap();

        assert_eq!(table.normalize("豈金"), "豈金");
    }

    #[cfg(not(feature = "compat-bin"))]
    #[test]
    fn rejects_multi_char_source_or_target() {
        assert!(CompatIdeographs::from_text("豈x\t豈\n").is_err());
        assert!(CompatIdeographs::from_text("豈\t豈x\n").is_err());
    }

    #[cfg(not(feature = "compat-bin"))]
    #[test]
    fn rejects_source_outside_supported_ranges() {
        assert!(CompatIdeographs::from_text("金\t金\n").is_err());
    }
}
