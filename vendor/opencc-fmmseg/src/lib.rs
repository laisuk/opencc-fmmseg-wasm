/// Unicode CJK Compatibility Ideograph normalization utilities.
mod compat_ideographs;
/// Delimiters helper for splitting and matching delimiters.
mod delimiter_set;
mod detofu;
/// Bridge helper for conversion plan and core converter functions.
mod dict_refs;
/// Dictionary utilities for managing multiple OpenCC lexicons.
mod dictionary_lib;
mod ids;
/// Core converter
mod opencc;
/// Configurations for conversion.
mod opencc_config;
mod unicode_compat;
/// Common helpers for opencc-fmmseg.
mod utils;

pub use crate::delimiter_set::{is_delimiter, DelimiterSet};
pub use crate::dict_refs::DictRefs;
pub use crate::dictionary_lib::{CustomDictFileSpec, CustomDictMode, CustomDictSpec, DictSlot};
pub use crate::dictionary_lib::{DictMaxLen, DictionaryError, DictionaryMaxlength};
pub use crate::opencc::OpenCC;
pub use crate::opencc_config::OpenccConfig;
#[cfg(feature = "compat-bin")]
pub use compat_ideographs::write_compat_bin_from_txt_file;
/// Converts rare non-BMP CJK extension characters to compatibility fallbacks.
pub use detofu::detofu;
pub use detofu::detofu_into;
#[cfg(feature = "tofu-bin")]
pub use detofu::write_tofu_bin_from_txt_file;
/// Threshold level used by detofu display-compatibility fallback.
pub use detofu::DetofuLevel;
/// Reusable and customizable detofu fallback map.
pub use detofu::DetofuMap;
#[cfg(feature = "unicode-bin")]
pub use unicode_compat::write_unicode_compat_bin_from_txt_file;
pub(crate) use utils::*;
