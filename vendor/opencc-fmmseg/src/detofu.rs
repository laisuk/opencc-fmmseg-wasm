//! Display compatibility fallback utilities.
//!
//! This module provides optional "detofu" processing for non-BMP
//! CJK extension characters that may not render correctly on some
//! systems, fonts, browsers, or e-book readers.
//!
//! DeToFu data is built from `src/data/TSCharactersTofu.txt`. That text file is
//! the canonical source of the built-in fallback table and is embedded by
//! default with `include_str!()`.
//!
//! # Cargo feature
//!
//! Enabling the optional `tofu-bin` feature switches the built-in runtime loader
//! from the canonical text file to `src/data/TSCharactersTofu.bin`, a compact
//! generated artifact committed for size-sensitive builds such as WebAssembly.
//! The binary file must be regenerated from `TSCharactersTofu.txt` with
//! `dict-generate --tofu` whenever the canonical text data changes.
//!
//! The feature model is intentionally binary:
//!
//! - without `tofu-bin`, the runtime loads embedded TXT data;
//! - with `tofu-bin`, the runtime loads embedded BIN data.

use std::collections::HashMap;
#[cfg(not(feature = "tofu-bin"))]
use std::io;
#[cfg(feature = "tofu-bin")]
use std::io::{self, Write};
use std::path::Path;
use std::sync::OnceLock;

#[cfg(feature = "tofu-bin")]
static TOFU_DATA: &[u8] = include_bytes!("data/TSCharactersTofu.bin");

#[cfg(not(feature = "tofu-bin"))]
static TOFU_DATA: &str = include_str!("data/TSCharactersTofu.txt");

#[cfg(feature = "tofu-bin")]
const TOFU_BIN_MAGIC: &[u8; 8] = b"OCTFTOFU";
#[cfg(feature = "tofu-bin")]
const TOFU_BIN_VERSION: u8 = 1;
#[cfg(feature = "tofu-bin")]
const TOFU_BIN_HEADER_LEN: usize = 13;
#[cfg(feature = "tofu-bin")]
const TOFU_BIN_RECORD_LEN: usize = 9;

/// Controls which CJK extension ranges are replaced by detofu.
///
/// Detofu levels are threshold-based: the selected level is the earliest
/// extension block to replace, and all supported later extension blocks are
/// replaced too.
///
/// - [`DetofuLevel::ExtB`] means ExtB+ and replaces all supported non-BMP
///   mappings: ExtB, ExtC, ExtD, ExtE, ExtF, ExtG, ExtH, and ExtI.
/// - [`DetofuLevel::ExtC`] means ExtC+ and replaces ExtC through ExtI.
/// - [`DetofuLevel::ExtD`] means ExtD+ and replaces ExtD through ExtI.
/// - [`DetofuLevel::ExtE`] means ExtE+ and replaces ExtE through ExtI.
///
/// The CLI alias `all` maps to [`DetofuLevel::ExtB`], so `ExtB` is the
/// broadest built-in fallback level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DetofuLevel {
    /// Replace CJK Extension B and all supported later extension mappings.
    ExtB,
    /// Replace CJK Extension C and all supported later extension mappings.
    ExtC,
    /// Replace CJK Extension D and all supported later extension mappings.
    ExtD,
    /// Replace CJK Extension E and all supported later extension mappings.
    ExtE,
    /// Replace CJK Extension F and all supported later extension mappings.
    ExtF,
    /// Replace CJK Extension G and all supported later extension mappings.
    ExtG,
    /// Replace CJK Extension H and all supported later extension mappings.
    ExtH,
    /// Replace CJK Extension I mappings.
    ExtI,
}

impl DetofuLevel {
    /// Parses a detofu level from a CLI/API string.
    ///
    /// Accepted aliases include `all`, `b`, `ext-b`, and `extb` for
    /// [`DetofuLevel::ExtB`]. The parser is case-insensitive.
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.trim().to_ascii_lowercase().as_str() {
            "all" | "ext-b" | "extb" | "b" => Ok(Self::ExtB),
            "ext-c" | "extc" | "c" => Ok(Self::ExtC),
            "ext-d" | "extd" | "d" => Ok(Self::ExtD),
            "ext-e" | "exte" | "e" => Ok(Self::ExtE),
            "ext-f" | "extf" | "f" => Ok(Self::ExtF),
            "ext-g" | "extg" | "g" => Ok(Self::ExtG),
            "ext-h" | "exth" | "h" => Ok(Self::ExtH),
            "ext-i" | "exti" | "i" => Ok(Self::ExtI),
            _ => Err(
                "supported detofu levels: all, ext-b, ext-c, ext-d, ext-e, ext-f, ext-g, ext-h, ext-i"
                    .to_string(),
            ),
        }
    }

    #[cfg(feature = "tofu-bin")]
    #[inline]
    pub(crate) fn to_bin_id(self) -> u8 {
        match self {
            Self::ExtB => 0,
            Self::ExtC => 1,
            Self::ExtD => 2,
            Self::ExtE => 3,
            Self::ExtF => 4,
            Self::ExtG => 5,
            Self::ExtH => 6,
            Self::ExtI => 7,
        }
    }

    #[cfg(feature = "tofu-bin")]
    #[inline]
    pub(crate) fn from_bin_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(Self::ExtB),
            1 => Some(Self::ExtC),
            2 => Some(Self::ExtD),
            3 => Some(Self::ExtE),
            4 => Some(Self::ExtF),
            5 => Some(Self::ExtG),
            6 => Some(Self::ExtH),
            7 => Some(Self::ExtI),
            _ => None,
        }
    }
}

static TOFU_ENTRIES: OnceLock<Vec<(char, char, DetofuLevel)>> = OnceLock::new();

/// Parses canonical tab-separated DeToFu text entries.
///
/// `TSCharactersTofu.txt` uses one mapping per non-comment line:
/// `tofu_char<TAB>fallback_char<TAB>extension`.
///
/// This parser remains crate-visible because both the default TXT runtime
/// loader and `dict-generate --tofu` use the same canonical text format.
pub(crate) fn parse_tofu_entries(text: &str) -> Result<Vec<(char, char, DetofuLevel)>, String> {
    let mut entries = Vec::new();

    for (index, raw_line) in text.lines().enumerate() {
        let line_no = index + 1;
        let line = raw_line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut parts = line.split('\t');

        let tofu = parts
            .next()
            .and_then(|s| s.trim().chars().next())
            .ok_or_else(|| format!("line {line_no}: missing tofu character"))?;

        let fallback = parts
            .next()
            .and_then(|s| s.trim().chars().next())
            .ok_or_else(|| format!("line {line_no}: missing fallback character"))?;

        let ext_text = parts
            .next()
            .map(str::trim)
            .ok_or_else(|| format!("line {line_no}: missing extension"))?;

        let ext = DetofuLevel::parse(ext_text)
            .map_err(|err| format!("line {line_no}: invalid extension `{ext_text}`: {err}"))?;

        entries.push((tofu, fallback, ext));
    }

    Ok(entries)
}

/// Parses built-in DeToFu binary data.
///
/// The binary format is intentionally DeToFu-specific and stable:
///
/// - magic: `OCTFTOFU`
/// - version: `1`
/// - record count: `u32` little-endian
/// - records: `tofu: u32`, `fallback: u32`, `level: u8`
///
/// This parser is used by the optional `tofu-bin` runtime loader. The
/// `TSCharactersTofu.bin` file it reads is a generated runtime artifact; the
/// canonical source remains `TSCharactersTofu.txt`.
#[cfg(feature = "tofu-bin")]
pub fn parse_tofu_bin(bytes: &[u8]) -> io::Result<Vec<(char, char, DetofuLevel)>> {
    if bytes.len() < TOFU_BIN_HEADER_LEN {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "tofu binary is too short: expected at least {TOFU_BIN_HEADER_LEN} bytes, got {}",
                bytes.len()
            ),
        ));
    }

    if &bytes[..TOFU_BIN_MAGIC.len()] != TOFU_BIN_MAGIC {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid tofu binary magic",
        ));
    }

    let version = bytes[TOFU_BIN_MAGIC.len()];
    if version != TOFU_BIN_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unsupported tofu binary version: {version}"),
        ));
    }

    let count_start = TOFU_BIN_MAGIC.len() + 1;
    let count = u32::from_le_bytes(
        bytes[count_start..count_start + 4]
            .try_into()
            .expect("count slice length is fixed"),
    ) as usize;

    let expected_len = TOFU_BIN_HEADER_LEN
        .checked_add(count.checked_mul(TOFU_BIN_RECORD_LEN).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "tofu binary record count overflows",
            )
        })?)
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "tofu binary length overflows")
        })?;

    if bytes.len() != expected_len {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "invalid tofu binary length: expected {expected_len} bytes for {count} records, got {}",
                bytes.len()
            ),
        ));
    }

    let mut entries = Vec::with_capacity(count);
    let mut pos = TOFU_BIN_HEADER_LEN;

    for index in 0..count {
        let tofu_u32 = u32::from_le_bytes(
            bytes[pos..pos + 4]
                .try_into()
                .expect("tofu slice length is fixed"),
        );
        let fallback_u32 = u32::from_le_bytes(
            bytes[pos + 4..pos + 8]
                .try_into()
                .expect("fallback slice length is fixed"),
        );
        let level_id = bytes[pos + 8];
        pos += TOFU_BIN_RECORD_LEN;

        let tofu = char::from_u32(tofu_u32).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("record {index}: invalid tofu Unicode scalar: U+{tofu_u32:04X}"),
            )
        })?;

        let fallback = char::from_u32(fallback_u32).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("record {index}: invalid fallback Unicode scalar: U+{fallback_u32:04X}"),
            )
        })?;

        let level = DetofuLevel::from_bin_id(level_id).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("record {index}: invalid detofu level id: {level_id}"),
            )
        })?;

        entries.push((tofu, fallback, level));
    }

    Ok(entries)
}

/// Writes DeToFu entries in the compact built-in binary format.
///
/// This helper writes the generated representation consumed when the optional
/// `tofu-bin` feature is enabled. The output should be derived from canonical
/// `TSCharactersTofu.txt` data and committed as `TSCharactersTofu.bin`.
#[cfg(feature = "tofu-bin")]
pub fn write_tofu_bin<W: Write>(
    entries: &[(char, char, DetofuLevel)],
    mut writer: W,
) -> io::Result<()> {
    let count = u32::try_from(entries.len()).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("too many tofu entries for binary format: {}", entries.len()),
        )
    })?;

    writer.write_all(TOFU_BIN_MAGIC)?;
    writer.write_all(&[TOFU_BIN_VERSION])?;
    writer.write_all(&count.to_le_bytes())?;

    for &(tofu, fallback, level) in entries {
        writer.write_all(&(tofu as u32).to_le_bytes())?;
        writer.write_all(&(fallback as u32).to_le_bytes())?;
        writer.write_all(&[level.to_bin_id()])?;
    }

    Ok(())
}

/// Writes DeToFu entries to a `TSCharactersTofu.bin`-style binary file.
///
/// Prefer [`write_tofu_bin_from_txt_file`] when regenerating the checked-in
/// runtime artifact from canonical text data.
#[cfg(feature = "tofu-bin")]
pub fn write_tofu_bin_file<P: AsRef<Path>>(
    entries: &[(char, char, DetofuLevel)],
    path: P,
) -> io::Result<()> {
    let file = std::fs::File::create(path)?;
    let mut writer = io::BufWriter::new(file);
    write_tofu_bin(entries, &mut writer)?;
    writer.flush()
}

/// Generates a DeToFu binary file from canonical `TSCharactersTofu.txt` data.
///
/// This is the public helper used by `dict-generate --tofu`. The input text is
/// the canonical source of truth; the output binary is only the generated
/// runtime artifact used when the optional `tofu-bin` feature is enabled.
#[cfg(feature = "tofu-bin")]
pub fn write_tofu_bin_from_txt_file<P: AsRef<Path>, Q: AsRef<Path>>(
    input_txt: P,
    output_bin: Q,
) -> io::Result<()> {
    let text = std::fs::read_to_string(input_txt)?;
    let entries =
        parse_tofu_entries(&text).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    write_tofu_bin_file(&entries, output_bin)
}

#[cfg(feature = "tofu-bin")]
fn load_builtin_tofu_entries() -> Vec<(char, char, DetofuLevel)> {
    parse_tofu_bin(TOFU_DATA)
        .unwrap_or_else(|err| panic!("invalid built-in TSCharactersTofu.bin: {err}"))
}

#[cfg(not(feature = "tofu-bin"))]
fn load_builtin_tofu_entries() -> Vec<(char, char, DetofuLevel)> {
    parse_tofu_entries(TOFU_DATA)
        .unwrap_or_else(|err| panic!("invalid built-in TSCharactersTofu.txt: {err}"))
}

fn tofu_entries() -> &'static [(char, char, DetofuLevel)] {
    TOFU_ENTRIES.get_or_init(load_builtin_tofu_entries)
}

/// A reusable map for detofu display-compatibility fallback.
///
/// `DetofuMap` is an advanced API for callers that want to build a fallback
/// table once and reuse it across many strings, or layer application-specific
/// fallbacks on top of the built-in map.
///
/// Detofu is independent of OpenCC conversion dictionaries. It does not
/// participate in Simplified/Traditional phrase matching, regional variant
/// selection, punctuation conversion, or any other OpenCC conversion logic.
/// It is best treated as a display compatibility pass that can run after
/// conversion when the target renderer has incomplete rare-character coverage.
///
/// # Examples
///
/// ```rust
/// use opencc_fmmseg::{DetofuLevel, DetofuMap};
///
/// let map = DetofuMap::builtin(DetofuLevel::ExtB)
///     .with_custom_pairs(&[
///         ('𣭲', '氄'),
///     ]);
///
/// let safe = map.detofu("這隻小狗有𣭲毛");
///
/// assert_eq!(safe, "這隻小狗有氄毛");
/// ```
#[derive(Debug, Clone)]
pub struct DetofuMap {
    level: DetofuLevel,
    map: HashMap<char, char>,
}

impl DetofuMap {
    /// Builds a detofu map from the crate's built-in compatibility data.
    ///
    /// The selected [`DetofuLevel`] is threshold-based. For example,
    /// [`DetofuLevel::ExtB`] loads all supported non-BMP mappings, while
    /// [`DetofuLevel::ExtE`] loads only ExtE and later supported mappings.
    ///
    /// The built-in detofu map is independent of the OpenCC conversion
    /// dictionaries bundled with this crate.
    pub fn builtin(level: DetofuLevel) -> Self {
        let map = tofu_entries()
            .iter()
            .filter(|(_, _, ext)| *ext >= level)
            .map(|(tofu, fallback, _)| (*tofu, *fallback))
            .collect();

        Self { level, map }
    }

    /// Adds or overrides compatibility fallback entries from a tofu mapping file.
    ///
    /// The file uses the same tab-separated format as the built-in generated
    /// data: `tofu_char<TAB>fallback_char<TAB>extension`.
    ///
    /// The extension field may use either the compact form (`B`, `C`, `D`, ...)
    /// or the full form (`ExtB`, `ExtC`, `ExtD`, ...). Extension parsing is
    /// case-insensitive, so `b`, `ext-b`, and `ExtB` are accepted.
    ///
    /// Blank lines and lines starting with `#` are ignored. Malformed entries,
    /// missing fields, or unsupported extension values return
    /// [`io::ErrorKind::InvalidData`] with the source line number.
    ///
    /// File entries are applied post-load. If a file entry already exists in
    /// the built-in detofu map, the file fallback wins. Entries below this
    /// map's threshold level are ignored, matching [`DetofuMap::builtin`].
    pub fn with_custom_file<P: AsRef<Path>>(mut self, path: P) -> io::Result<Self> {
        let text = std::fs::read_to_string(path)?;

        for (tofu, fallback, ext) in parse_tofu_entries(&text)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?
        {
            if ext >= self.level {
                self.map.insert(tofu, fallback);
            }
        }

        Ok(self)
    }

    /// Adds or overrides compatibility fallback pairs after loading the map.
    ///
    /// Custom pairs are applied post-load. If a custom key already exists in
    /// the built-in detofu map, the custom fallback wins.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opencc_fmmseg::{DetofuLevel, DetofuMap};
    ///
    /// let map = DetofuMap::builtin(DetofuLevel::ExtB)
    ///     .with_custom_pairs(&[('𣭲', '氄')]);
    ///
    /// assert_eq!(map.detofu("𣭲"), "氄");
    /// ```
    pub fn with_custom_pairs(mut self, pairs: &[(char, char)]) -> Self {
        for &(tofu, fallback) in pairs {
            self.map.insert(tofu, fallback);
        }
        self
    }

    /// Replaces mapped non-BMP CJK extension characters with compatibility fallbacks.
    ///
    /// Characters not present in this map are copied unchanged. This is a
    /// display compatibility operation only; it does not modify OpenCC
    /// conversion dictionaries or conversion behavior.
    pub fn detofu(&self, input: &str) -> String {
        let mut output = String::with_capacity(input.len());

        for ch in input.chars() {
            if let Some(fallback) = self.map.get(&ch) {
                output.push(*fallback);
            } else {
                output.push(ch);
            }
        }

        output
    }
}

/// Converts non-BMP CJK extension characters to compatibility fallbacks.
///
/// This convenience function builds the built-in [`DetofuMap`] for `level` and
/// applies it to `input`. It is intended for environments with incomplete font
/// coverage where rare CJK extension characters may render as tofu boxes on
/// some systems, fonts, browsers, or e-book readers.
///
/// Detofu is independent of OpenCC conversion dictionaries and does not
/// modify OpenCC conversion logic. In a typical workflow, run OpenCC
/// conversion first and then apply detofu to the converted text.
///
/// # Examples
///
/// ```rust
/// use opencc_fmmseg::{detofu, DetofuLevel};
///
/// let safe = detofu("骖𬴂", DetofuLevel::ExtB);
///
/// assert_eq!(safe, "骖騑");
/// ```
pub fn detofu(input: &str, level: DetofuLevel) -> String {
    DetofuMap::builtin(level).detofu(input)
}

// Tests

#[cfg(all(test, feature = "tofu-bin"))]
mod tofu_bin_tests {
    use super::{parse_tofu_bin, parse_tofu_entries};

    #[test]
    fn builtin_tofu_bin_matches_builtin_tofu_txt() {
        let txt_entries = parse_tofu_entries(include_str!("data/TSCharactersTofu.txt"))
            .expect("built-in TSCharactersTofu.txt should parse");

        let bin_entries = parse_tofu_bin(include_bytes!("data/TSCharactersTofu.bin"))
            .expect("built-in TSCharactersTofu.bin should parse");

        assert_eq!(
            txt_entries, bin_entries,
            "TSCharactersTofu.bin must be regenerated from TSCharactersTofu.txt"
        );
    }
}
