//! Internal CJK Compatibility Ideograph normalization.
//!
//! The built-in UnicodeData-derived table is parsed once and cached in dense
//! lookup tables. Public callers use [`crate::OpenCC::normalize_compat`].

use std::borrow::Cow;
#[cfg(any(feature = "compat-bin", test))]
use std::io;
use std::sync::OnceLock;

#[cfg(not(feature = "compat-bin"))]
static COMPAT_DATA: &[u8] = include_bytes!("data/CJK_Compatibility_Ideographs.txt");
#[cfg(feature = "compat-bin")]
static COMPAT_DATA: &[u8] = include_bytes!("data/CJK_Compatibility_Ideographs.bin");

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
        COMPAT_TABLE.get_or_init(|| {
            #[cfg(feature = "compat-bin")]
            {
                let entries = parse_compat_bin(COMPAT_DATA).unwrap_or_else(|err| {
                    panic!("invalid built-in CJK_Compatibility_Ideographs.bin: {err}")
                });
                Self::from_entries(&entries).unwrap_or_else(|err| {
                    panic!("invalid built-in CJK_Compatibility_Ideographs.bin: {err}")
                })
            }
            #[cfg(not(feature = "compat-bin"))]
            {
                let text = std::str::from_utf8(COMPAT_DATA)
                    .expect("CJK_Compatibility_Ideographs.txt must be valid UTF-8");

                Self::from_text(text)
                    .unwrap_or_else(|err| panic!("invalid CJK_Compatibility_Ideographs.txt: {err}"))
            }
        })
    }

    #[cfg(any(feature = "compat-bin", test))]
    fn from_entries(entries: &[(char, char)]) -> Result<Self, String> {
        let mut table = Self::default();
        for (index, &(src, dst)) in entries.iter().enumerate() {
            table.set(src, dst, index + 1)?;
        }
        Ok(table)
    }

    /// Builds a compatibility ideograph normalizer from UTF-8 mapping text.
    ///
    /// This is mainly useful for tests or custom data. The expected format is
    /// one tab-separated `source<TAB>target` pair per line, with `#` comments
    /// and blank lines ignored.
    #[cfg(any(not(feature = "compat-bin"), feature = "dictionary-build", test))]
    pub(crate) fn from_text(text: &str) -> Result<Self, String> {
        let mut table = Self::default();

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
            let dst = single_char(dst_text, line_no, "target")?;

            table.set(src, dst, line_no)?;
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

        Err(format!(
            "line {line_no}: source U+{u:04X} is outside CJK Compatibility Ideograph ranges"
        ))
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
    #[cfg(test)]
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
        self.normalize_cow(input).into_owned()
    }
    /// Normalizes mapped characters, borrowing unchanged input.
    pub(crate) fn normalize_cow<'a>(&self, input: &'a str) -> Cow<'a, str> {
        // This table can only contain sources in the two CJK compatibility
        // ranges. Byte scanning avoids scalar decoding for all-ASCII input.
        if input.is_ascii() {
            return Cow::Borrowed(input);
        }
        crate::lazy_replace::replace_chars(input, |ch| self.normalize_char(ch))
    }
}

#[cfg(any(not(feature = "compat-bin"), feature = "dictionary-build", test))]
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

    #[test]
    fn rejects_multi_char_source_or_target() {
        assert!(CompatIdeographs::from_text("豈x\t豈\n").is_err());
        assert!(CompatIdeographs::from_text("豈\t豈x\n").is_err());
    }

    #[test]
    fn rejects_source_outside_supported_ranges() {
        assert!(CompatIdeographs::from_text("金\t金\n").is_err());
    }
}

#[cfg(any(feature = "compat-bin", test))]
const COMPAT_BIN_MAGIC: &[u8; 8] = b"OCCOMPAT";
#[cfg(any(feature = "compat-bin", test))]
const COMPAT_BIN_VERSION: u8 = 1;
#[cfg(any(feature = "compat-bin", test))]
const COMPAT_BIN_HEADER_LEN: usize = 13;
#[cfg(any(feature = "compat-bin", test))]
const COMPAT_BIN_RECORD_LEN: usize = 8;

/// Decodes the original version-1 auxiliary BIN format with strict validation.
#[cfg(any(feature = "compat-bin", test))]
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

#[cfg(any(feature = "compat-bin", test))]
fn validate_compat_source(src: char, line_no: usize) -> Result<(), String> {
    let u = src as u32;

    if (BMP_START..=BMP_END).contains(&u) || (SUPP_START..=SUPP_END).contains(&u) {
        return Ok(());
    }

    Err(format!(
        "line {line_no}: source U+{u:04X} is outside CJK Compatibility Ideograph ranges"
    ))
}

#[cfg(test)]
mod bin_tests {
    use super::*;
    const BIN: &[u8] = include_bytes!("data/CJK_Compatibility_Ideographs.bin");
    const TXT: &str = include_str!("data/CJK_Compatibility_Ideographs.txt");

    #[test]
    fn artifact_matches_txt_and_both_dense_ranges() {
        let entries = parse_compat_bin(BIN).unwrap();
        assert_eq!(
            entries,
            crate::auxiliary_bin_tests::pairs(TXT),
            "stale compatibility BIN; regenerate it"
        );
        let txt = CompatIdeographs::from_text(TXT).unwrap();
        let bin = CompatIdeographs::from_entries(&entries).unwrap();
        assert!(entries.iter().any(|(src, _)| (*src as u32) >= SUPP_START));
        for scalar in (BMP_START..=BMP_END).chain(SUPP_START..=SUPP_END) {
            let ch = char::from_u32(scalar).unwrap();
            assert_eq!(txt.normalize_char(ch), bin.normalize_char(ch));
            assert_eq!(
                CompatIdeographs::builtin().normalize_char(ch),
                bin.normalize_char(ch)
            );
        }
        assert!(std::ptr::eq(
            CompatIdeographs::builtin(),
            CompatIdeographs::builtin()
        ));
    }

    #[test]
    fn rejects_invalid_bins() {
        crate::auxiliary_bin_tests::rejects_corruption(BIN, 8, |bytes| {
            parse_compat_bin(bytes).map(|_| ())
        });
        let mut bad = BIN.to_vec();
        bad[13..17].copy_from_slice(&('金' as u32).to_le_bytes());
        assert!(parse_compat_bin(&bad).is_err());
    }
}
