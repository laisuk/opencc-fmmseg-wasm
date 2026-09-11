mod converter;
pub use converter::OfficeConverter;

use opencc_fmmseg::{
    CustomDictMode, CustomDictSpec, DetofuLevel, DictSlot, DictionaryMaxlength, OpenCC,
    OpenccConfig,
};
use wasm_bindgen::prelude::*;

/// OpenCC conversion configurations exposed to JavaScript/WebAssembly callers.
///
/// The numeric values are kept in sync with the backend [`OpenccConfig`] FFI IDs.
#[wasm_bindgen]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenccConfigWasm {
    /// Simplified Chinese to Traditional Chinese.
    S2t = 1,
    /// Simplified Chinese to Traditional Chinese (Taiwan).
    S2tw = 2,
    /// Simplified Chinese to Traditional Chinese (Taiwan, with phrases).
    S2twp = 3,
    /// Simplified Chinese to Traditional Chinese (Hong Kong).
    S2hk = 4,
    /// Traditional Chinese to Simplified Chinese.
    T2s = 5,
    /// Traditional Chinese to Traditional Chinese (Taiwan).
    T2tw = 6,
    /// Traditional Chinese to Traditional Chinese (Taiwan, with phrases).
    T2twp = 7,
    /// Traditional Chinese to Traditional Chinese (Hong Kong).
    T2hk = 8,
    /// Traditional Chinese (Taiwan) to Simplified Chinese.
    Tw2s = 9,
    /// Traditional Chinese (Taiwan) to Simplified Chinese, with phrase conversion.
    Tw2sp = 10,
    /// Traditional Chinese (Taiwan) to generic Traditional Chinese.
    Tw2t = 11,
    /// Traditional Chinese (Taiwan) to generic Traditional Chinese, with phrases.
    Tw2tp = 12,
    /// Traditional Chinese (Hong Kong) to Simplified Chinese.
    Hk2s = 13,
    /// Traditional Chinese (Hong Kong) to generic Traditional Chinese.
    Hk2t = 14,
    /// Japanese Shinjitai forms to Traditional Chinese forms.
    Jp2t = 15,
    /// Traditional Chinese forms to Japanese Shinjitai forms.
    T2jp = 16,
    /// Simplified Chinese to Hong Kong Traditional Chinese, with phrase conversion.
    S2hkp = 17,
    /// Hong Kong Traditional Chinese to Simplified Chinese, with phrase conversion.
    Hk2sp = 18,
    /// Traditional Chinese to Hong Kong Traditional Chinese, with phrase conversion.
    T2hkp = 19,
    /// Hong Kong Traditional Chinese to generic Traditional Chinese, with phrase conversion.
    Hk2tp = 20,
}

impl OpenccConfigWasm {
    fn into_backend(self) -> OpenccConfig {
        OpenccConfig::from_ffi(self as u32)
            .expect("OpenccConfigWasm must match backend OpenccConfig")
    }
}

impl From<OpenccConfigWasm> for OpenccConfig {
    fn from(value: OpenccConfigWasm) -> Self {
        value.into_backend()
    }
}

/// CJK Extension support level used by DeTofu fallback processing.
///
/// Characters beyond the selected extension level may be replaced with safer
/// fallback forms when mappings are available.
#[wasm_bindgen]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DetofuLevelWasm {
    /// Treat CJK Extension B as the maximum supported extension level.
    ExtB = 0,
    /// Treat CJK Extension C as the maximum supported extension level.
    ExtC = 1,
    /// Treat CJK Extension D as the maximum supported extension level.
    ExtD = 2,
    /// Treat CJK Extension E as the maximum supported extension level.
    ExtE = 3,
    /// Treat CJK Extension F as the maximum supported extension level.
    ExtF = 4,
    /// Treat CJK Extension G as the maximum supported extension level.
    ExtG = 5,
    /// Treat CJK Extension H as the maximum supported extension level.
    ExtH = 6,
    /// Treat CJK Extension I as the maximum supported extension level.
    ExtI = 7,
}

impl From<DetofuLevelWasm> for DetofuLevel {
    fn from(value: DetofuLevelWasm) -> Self {
        match value {
            DetofuLevelWasm::ExtB => DetofuLevel::ExtB,
            DetofuLevelWasm::ExtC => DetofuLevel::ExtC,
            DetofuLevelWasm::ExtD => DetofuLevel::ExtD,
            DetofuLevelWasm::ExtE => DetofuLevel::ExtE,
            DetofuLevelWasm::ExtF => DetofuLevel::ExtF,
            DetofuLevelWasm::ExtG => DetofuLevel::ExtG,
            DetofuLevelWasm::ExtH => DetofuLevel::ExtH,
            DetofuLevelWasm::ExtI => DetofuLevel::ExtI,
        }
    }
}

/// Optional normalization stage used by Office/EPUB pipeline conversion.
///
/// Pipeline order is always normalization -> OpenCC conversion -> optional DeTofu.
/// `None` skips normalization entirely. The remaining variants map directly to
/// the corresponding normalization APIs exposed by [`OpenccWasm`].
#[wasm_bindgen]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NormalizeModeWasm {
    /// Do not normalize before OpenCC conversion.
    None = 0,
    /// Normalize CJK Compatibility Ideographs.
    Compat = 1,
    /// Apply the extended Unicode compatibility table only.
    UnicodeCompat = 2,
    /// Apply extended Unicode compatibility normalization together with CJK
    /// Compatibility Ideograph normalization.
    CompatExtended = 3,
}

/// JavaScript-facing custom dictionary specification.
///
/// `slot` selects the OpenCC dictionary slot, `pairs` contains source/replacement
/// entries, and `mode` accepts `"Append"`/`"append"` or `"Override"`/`"override"`.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WasmCustomDictSpec {
    pub slot: String,
    pub pairs: Vec<(String, String)>,
    pub mode: Option<String>,
}

fn parse_dict_slot(slot: &str) -> Result<DictSlot, String> {
    DictSlot::from_name_ignore_ascii_case(slot).ok_or_else(|| format!("Invalid DictSlot: {slot}"))
}

fn parse_custom_dict_mode(mode: Option<&str>) -> Result<CustomDictMode, String> {
    match mode.unwrap_or("Append") {
        "Append" | "append" => Ok(CustomDictMode::Append),
        "Override" | "override" => Ok(CustomDictMode::Override),
        other => Err(format!("Invalid CustomDictMode: {other}")),
    }
}

impl TryFrom<WasmCustomDictSpec> for CustomDictSpec {
    type Error = String;

    fn try_from(value: WasmCustomDictSpec) -> Result<Self, Self::Error> {
        Ok(CustomDictSpec {
            slot: parse_dict_slot(&value.slot)?,
            pairs: value.pairs,
            mode: parse_custom_dict_mode(value.mode.as_deref())?,
        })
    }
}

/// WebAssembly wrapper around the embedded `opencc-fmmseg` conversion engine.
///
/// Each instance owns its conversion configuration and dictionary state.
#[wasm_bindgen]
pub struct OpenccWasm {
    inner: OpenCC,
    config: OpenccConfig,
}

fn parse_wasm_config(config: Option<String>) -> Result<OpenccConfig, JsValue> {
    let Some(config) = config.as_deref() else {
        return Ok(OpenccConfig::S2t);
    };

    OpenccConfig::parse(config)
        .ok_or_else(|| JsValue::from_str(&format!("Invalid OpenCC config: {config}")))
}

/// Shared Office/EPUB pipeline adapter.
///
/// The Office package layer owns only ZIP/package mechanics. Text-processing
/// policy is composed here in the fixed order normalization -> OpenCC -> DeTofu
/// and supplied to [`OfficeConverter`] as a generic `&str -> String` closure.
fn convert_office_bytes_pipeline_core(
    input: &[u8],
    format: &str,
    inner: &OpenCC,
    config: &str,
    punctuation: bool,
    keep_font: bool,
    norm_mode: NormalizeModeWasm,
    detofu_level: Option<DetofuLevelWasm>,
) -> std::io::Result<(Vec<u8>, usize)> {
    let text_converter = |text: &str| {
        let converted = match norm_mode {
            NormalizeModeWasm::None => inner.convert(text, config, punctuation),
            NormalizeModeWasm::Compat => {
                let normalized = inner.normalize_compat(text);
                inner.convert(&normalized, config, punctuation)
            }
            NormalizeModeWasm::UnicodeCompat => {
                let normalized = inner.normalize_unicode_compat(text);
                inner.convert(&normalized, config, punctuation)
            }
            NormalizeModeWasm::CompatExtended => {
                let normalized = inner.normalize_compat_extended(text);
                inner.convert(&normalized, config, punctuation)
            }
        };

        match detofu_level {
            Some(level) => inner.detofu(&converted, level.into()),
            None => converted,
        }
    };

    OfficeConverter::convert_bytes_with(input, format, keep_font, &text_converter)
}

#[wasm_bindgen]
impl OpenccWasm {
    /// Returns the `opencc-fmmseg-wasm` package version.
    pub fn version() -> String {
        env!("CARGO_PKG_VERSION").to_owned()
    }

    /// Creates a converter using the embedded dictionaries.
    ///
    /// `config` is an OpenCC configuration name such as `"s2t"` or `"t2s"`.
    /// When omitted, the default configuration is `"s2t"`. Invalid names throw
    /// a JavaScript error.
    #[wasm_bindgen(constructor)]
    pub fn new(config: Option<String>) -> Result<OpenccWasm, JsValue> {
        let config = parse_wasm_config(config)?;

        let mut inner = OpenCC::new_embedded();

        // IMPORTANT for wasm first version
        inner.set_parallel(false);

        Ok(OpenccWasm { inner, config })
    }

    /// Creates a converter with one or more custom dictionary specifications.
    ///
    /// `config` behaves like the main constructor and defaults to `"s2t"`.
    /// `specs` must be an array of objects containing `slot`, `pairs`, and an
    /// optional `mode`. Invalid specs throw a JavaScript error.
    #[wasm_bindgen(js_name = newWithCustomDicts)]
    pub fn new_with_custom_dicts(
        config: Option<String>,
        specs: JsValue,
    ) -> Result<OpenccWasm, JsValue> {
        let config = parse_wasm_config(config)?;

        let specs: Vec<WasmCustomDictSpec> = serde_wasm_bindgen::from_value(specs)
            .map_err(|e| JsValue::from_str(&format!("Invalid custom dict specs: {e}")))?;

        let specs: Vec<CustomDictSpec> = specs
            .into_iter()
            .map(WasmCustomDictSpec::try_into)
            .collect::<Result<_, _>>()?;

        let dictionary = DictionaryMaxlength::from_embedded_cbor()
            .with_custom_dicts(&specs)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let mut inner = OpenCC::from_dictionary(dictionary);
        inner.set_parallel(false);

        Ok(OpenccWasm { inner, config })
    }

    /// Converts text using the instance's current OpenCC configuration.
    ///
    /// `punctuation` enables the configuration's punctuation conversion when
    /// supported. The instance configuration can be changed with [`Self::set_config`].
    pub fn convert(&self, text: &str, punctuation: bool) -> String {
        self.inner.convert(text, self.config.as_str(), punctuation)
    }

    /// Returns the current OpenCC configuration name.
    #[wasm_bindgen(js_name = getConfig)]
    pub fn get_config(&self) -> String {
        self.config.as_str().to_string()
    }

    /// Changes the current OpenCC configuration by name.
    ///
    /// Returns `true` when `config` is valid. On failure, returns `false` and
    /// leaves the current configuration unchanged.
    #[wasm_bindgen(js_name = setConfig)]
    pub fn set_config(&mut self, config: &str) -> bool {
        match OpenccConfig::parse(config) {
            Some(cfg) => {
                self.config = cfg;
                true
            }
            None => false,
        }
    }

    /// Returns whether a configuration name is supported.
    #[wasm_bindgen(js_name = isValidConfig)]
    pub fn is_valid_config(config: &str) -> bool {
        OpenccConfig::is_valid_config(config)
    }

    /// Returns all supported OpenCC configuration names.
    #[wasm_bindgen(js_name = getSupportedConfigs)]
    pub fn get_supported_configs() -> Vec<String> {
        OpenccConfig::ALL
            .iter()
            .map(|config| config.as_str().to_string())
            .collect()
    }

    /// Returns all canonical dictionary slot names available for custom dictionaries.
    #[wasm_bindgen(js_name = getAvailableSlots)]
    pub fn get_available_slots() -> Vec<String> {
        DictSlot::ALL
            .iter()
            .map(|slot| slot.canonical_name().to_string())
            .collect()
    }

    /// Creates a converter using an [`OpenccConfigWasm`] enum value.
    ///
    /// When omitted, the configuration defaults to [`OpenccConfigWasm::S2t`].
    #[wasm_bindgen(js_name = newWithEnum)]
    pub fn new_with_enum(config: Option<OpenccConfigWasm>) -> Result<OpenccWasm, JsValue> {
        let config = config.map(OpenccConfig::from).unwrap_or(OpenccConfig::S2t);

        let mut inner = OpenCC::new_embedded();
        inner.set_parallel(false);

        Ok(OpenccWasm { inner, config })
    }

    /// Changes the current configuration using an [`OpenccConfigWasm`] enum value.
    #[wasm_bindgen(js_name = setConfigEnum)]
    pub fn set_config_enum(&mut self, config: OpenccConfigWasm) {
        self.config = OpenccConfig::from(config);
    }

    /// Returns the numeric FFI ID of the current OpenCC configuration.
    #[wasm_bindgen(js_name = getConfigId)]
    pub fn get_config_id(&self) -> u32 {
        self.config.to_ffi()
    }

    /// Returns whether ID-preservation behavior is enabled in the backend.
    #[wasm_bindgen(js_name = getPreserveIds)]
    pub fn get_preserve_ids(&self) -> bool {
        self.inner.get_preserve_ids()
    }

    /// Enables or disables ID-preservation behavior in the backend.
    #[wasm_bindgen(js_name = setPreserveIds)]
    pub fn set_preserve_ids(&mut self, value: bool) {
        self.inner.set_preserve_ids(value);
    }

    /// Detects whether text is predominantly Traditional or Simplified Chinese.
    ///
    /// Returns `1` for Traditional Chinese, `2` for Simplified Chinese, and `0`
    /// when the text cannot be classified as either.
    #[wasm_bindgen(js_name = zhoCheck)]
    pub fn zho_check(&self, text: &str) -> i32 {
        self.inner.zho_check(text)
    }

    /// Normalizes Unicode CJK Compatibility Ideographs.
    ///
    /// This is an optional pre-conversion pass. Mapped compatibility ideographs
    /// are replaced with their unified forms; unmapped characters are preserved.
    #[wasm_bindgen(js_name = normalizeCompat)]
    pub fn normalize_compat(&self, text: &str) -> String {
        self.inner.normalize_compat(text)
    }

    /// Performs extended Unicode compatibility normalization.
    ///
    /// This applies the extended Unicode compatibility table together with CJK
    /// Compatibility Ideograph normalization. It is intended as the broadest
    /// optional normalization pre-pass before OpenCC conversion.
    #[wasm_bindgen(js_name = normalizeCompatExtended)]
    pub fn normalize_compat_extended(&self, text: &str) -> String {
        self.inner.normalize_compat_extended(text)
    }

    /// Normalizes characters using the extended Unicode compatibility table only.
    ///
    /// This includes selected radicals, allographs, legacy glyphs, and
    /// compatibility-like punctuation defined by the bundled extended table.
    #[wasm_bindgen(js_name = normalizeUnicodeCompat)]
    pub fn normalize_unicode_compat(&self, text: &str) -> String {
        self.inner.normalize_unicode_compat(text)
    }

    /// Applies DeTofu fallback processing for rare CJK extension characters.
    ///
    /// `level` specifies the highest CJK Extension block considered safe for
    /// display. Characters beyond that level are replaced when a fallback mapping
    /// is available.
    #[wasm_bindgen(js_name = detofu)]
    pub fn detofu(&self, text: &str, level: DetofuLevelWasm) -> String {
        self.inner.detofu(text, level.into())
    }

    /// Converts text and then applies DeTofu fallback processing.
    ///
    /// This is equivalent to calling [`Self::convert`] followed by [`Self::detofu`],
    /// while reusing an internal output buffer for the DeTofu pass.
    #[wasm_bindgen(js_name = convertDetofu)]
    pub fn convert_detofu(&self, text: &str, punctuation: bool, level: DetofuLevelWasm) -> String {
        let converted = self.convert(text, punctuation);
        let mut output = String::with_capacity(converted.len());

        self.inner
            .detofu_into(&converted, level.into(), &mut output);

        output
    }

    /// Converts a ZIP-based Office or EPUB document entirely in memory.
    ///
    /// `input` contains the complete document bytes. `format` accepts `"docx"`,
    /// `"xlsx"`, `"pptx"`, `"odt"`, `"ods"`, `"odp"`, or `"epub"`.
    /// `punctuation` controls punctuation conversion and `keep_font` preserves
    /// supported font declarations. The returned byte array is the rebuilt file.
    /// Invalid or unsupported input throws a JavaScript error.
    #[wasm_bindgen(js_name = convertOfficeBytes)]
    pub fn convert_office_bytes(
        &self,
        input: &[u8],
        format: &str,
        punctuation: bool,
        keep_font: bool,
    ) -> Result<Vec<u8>, JsValue> {
        OfficeConverter::convert_bytes(
            input,
            format,
            &self.inner,
            self.config.as_str(),
            punctuation,
            keep_font,
        )
        .map(|(bytes, _)| bytes)
        .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Converts a ZIP-based Office or EPUB document using the full text pipeline.
    ///
    /// Processing order is fixed and explicit:
    /// 1. normalization selected by `norm_mode`
    /// 2. OpenCC conversion using this instance's current configuration
    /// 3. optional DeTofu processing selected by `detofu_level`
    ///
    /// `NormalizeModeWasm::None` skips normalization. Passing `None` for
    /// `detofu_level` skips DeTofu. Package parsing, XLSX inline-string handling,
    /// font preservation, EPUB rules, and ZIP validation remain owned by
    /// [`OfficeConverter`].
    #[wasm_bindgen(js_name = convertOfficeBytesPipeline)]
    pub fn convert_office_bytes_pipeline(
        &self,
        input: &[u8],
        format: &str,
        punctuation: bool,
        keep_font: bool,
        norm_mode: NormalizeModeWasm,
        detofu_level: Option<DetofuLevelWasm>,
    ) -> Result<Vec<u8>, JsValue> {
        convert_office_bytes_pipeline_core(
            input,
            format,
            &self.inner,
            self.config.as_str(),
            punctuation,
            keep_font,
            norm_mode,
            detofu_level,
        )
        .map(|(bytes, _)| bytes)
        .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

/// Converts a ZIP-based Office or EPUB document entirely in memory without
/// creating an [`OpenccWasm`] instance.
///
/// `config` is an OpenCC configuration name. `format` accepts `"docx"`, `"xlsx"`,
/// `"pptx"`, `"odt"`, `"ods"`, `"odp"`, or `"epub"`. The returned byte array is
/// the rebuilt document; invalid input produces a JavaScript error.
#[wasm_bindgen]
pub fn convert_office_bytes(
    input: &[u8],
    format: &str,
    config: &str,
    punctuation: bool,
    keep_font: bool,
) -> Result<Vec<u8>, JsValue> {
    let mut opencc = OpenCC::new_embedded();
    opencc.set_parallel(false);

    OfficeConverter::convert_bytes(input, format, &opencc, config, punctuation, keep_font)
        .map(|(bytes, _)| bytes)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Converts a ZIP-based Office or EPUB document with the full normalization ->
/// OpenCC -> optional DeTofu pipeline without creating an [`OpenccWasm`] instance.
///
/// `config` is an OpenCC configuration name. `norm_mode` selects the optional
/// normalization stage, and `detofu_level` selects the optional DeTofu stage.
/// Package processing itself remains conversion-policy-independent.
#[wasm_bindgen]
pub fn convert_office_bytes_pipeline(
    input: &[u8],
    format: &str,
    config: &str,
    punctuation: bool,
    keep_font: bool,
    norm_mode: NormalizeModeWasm,
    detofu_level: Option<DetofuLevelWasm>,
) -> Result<Vec<u8>, JsValue> {
    let config = OpenccConfig::parse(config)
        .ok_or_else(|| JsValue::from_str(&format!("Invalid OpenCC config: {config}")))?;

    let mut opencc = OpenCC::new_embedded();
    opencc.set_parallel(false);

    convert_office_bytes_pipeline_core(
        input,
        format,
        &opencc,
        config.as_str(),
        punctuation,
        keep_font,
        norm_mode,
        detofu_level,
    )
    .map(|(bytes, _)| bytes)
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Read, Write};
    use zip::{
        CompressionMethod, ZipArchive, ZipWriter,
        write::{ExtendedFileOptions, FileOptions},
    };

    fn make_test_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut cursor = Cursor::new(Vec::<u8>::new());
        {
            let mut zip = ZipWriter::new(&mut cursor);
            let opts: FileOptions<'_, ExtendedFileOptions> =
                FileOptions::default().compression_method(CompressionMethod::Deflated);

            for (name, bytes) in entries {
                zip.start_file(*name, opts.clone()).unwrap();
                zip.write_all(bytes).unwrap();
            }

            zip.finish().unwrap();
        }
        cursor.into_inner()
    }

    fn read_test_zip_entry(zip_bytes: &[u8], name: &str) -> String {
        let cursor = Cursor::new(zip_bytes);
        let mut zip = ZipArchive::new(cursor).expect("ZIP archive should be readable");
        let mut entry = zip.by_name(name).expect("ZIP entry should exist");
        let mut content = String::new();
        entry
            .read_to_string(&mut content)
            .expect("ZIP entry should be UTF-8 text");
        content
    }

    #[test]
    fn test_cbor_load_and_convert() {
        let cc = OpenccWasm::new(None).unwrap();

        assert_eq!(cc.convert("汉字", false), "漢字");

        let mut cc2 = OpenccWasm::new(Some("t2s".to_string())).unwrap();

        assert_eq!(cc2.convert("漢字", false), "汉字");

        cc2.set_config("s2t");

        assert_eq!(cc2.convert("汉字", false), "漢字");
    }

    #[test]
    fn test_zho_check() {
        let cc = OpenccWasm::new(None).unwrap();

        assert_eq!(cc.zho_check("漢字"), 1);
        assert_eq!(cc.zho_check("汉字"), 2);
    }

    #[test]
    fn test_config_validation() {
        assert!(OpenccWasm::is_valid_config("s2t"));
        assert!(OpenccWasm::is_valid_config("T2JP"));
        assert!(!OpenccWasm::is_valid_config("bad"));
    }
    #[test]
    fn wasm_config_enum_matches_backend() {
        for config in OpenccConfig::ALL {
            let id = config.to_ffi();

            assert!(matches!(id, 1..=20));
            assert_eq!(OpenccConfig::from_ffi(id), Some(config));
        }

        assert_eq!(OpenccConfigWasm::S2t as u32, OpenccConfig::S2t.to_ffi());
        assert_eq!(OpenccConfigWasm::T2jp as u32, OpenccConfig::T2jp.to_ffi());
        assert_eq!(OpenccConfigWasm::S2hkp as u32, OpenccConfig::S2hkp.to_ffi());
        assert_eq!(OpenccConfigWasm::Hk2sp as u32, OpenccConfig::Hk2sp.to_ffi());
        assert_eq!(OpenccConfigWasm::T2hkp as u32, OpenccConfig::T2hkp.to_ffi());
        assert_eq!(OpenccConfigWasm::Hk2tp as u32, OpenccConfig::Hk2tp.to_ffi());
    }

    #[test]
    fn test_hk_phrase_configs() {
        let s2hkp = OpenccWasm::new(Some("s2hkp".to_string())).unwrap();
        assert_eq!(
            s2hkp.convert("别随便录影侵犯个人隐私权", false),
            "別隨便錄影侵犯個人私隱權"
        );

        let hk2sp = OpenccWasm::new(Some("hk2sp".to_string())).unwrap();
        assert_eq!(
            hk2sp.convert("別隨便錄影侵犯個人私隱權", false),
            "别随便录影侵犯个人隐私权"
        );
    }

    #[test]
    fn test_convert_bytes_docx_real_file() {
        use std::fs;
        use std::io::{Cursor, Read};
        use zip::ZipArchive;

        let input_path = "tests/OneDay.docx";

        let input_bytes = fs::read(input_path).expect("Failed to read tests/OneDay.docx");

        let mut opencc = OpenCC::new_embedded();
        opencc.set_parallel(false);

        let (out_bytes, converted_count) =
            OfficeConverter::convert_bytes(&input_bytes, "docx", &opencc, "s2t", true, true)
                .expect("convert_bytes failed");

        assert!(
            converted_count > 0,
            "Expected at least one converted XML entry"
        );

        // Optional debug output
        #[cfg(debug_assertions)]
        let _ = fs::write("tests/OneDay_s2t.docx", &out_bytes);

        // Verify output is a valid ZIP/docx
        let cursor = Cursor::new(out_bytes);
        let mut zip = ZipArchive::new(cursor).expect("Output is not a valid ZIP archive");

        let mut doc = zip
            .by_name("word/document.xml")
            .expect("Missing word/document.xml");

        let mut content = String::new();
        doc.read_to_string(&mut content)
            .expect("Failed to read document.xml");

        assert!(
            content.contains("碼頭"),
            "Expected converted Traditional Chinese phrase"
        );
    }

    #[test]
    fn test_detofu() {
        let cc = OpenccWasm::new(Some("t2s".to_string())).unwrap();

        let converted = cc.convert("儼驂騑於上路，訪風景於崇阿", false);
        assert_eq!(converted, "俨骖𬴂于上路，访风景于崇阿");

        let safe = cc.detofu(&converted, DetofuLevelWasm::ExtB);
        assert_eq!(safe, "俨骖騑于上路，访风景于崇阿");
    }

    #[test]
    fn detofu_level_wasm_values_are_zero_based() {
        assert_eq!(DetofuLevelWasm::ExtB as u32, 0);
        assert_eq!(DetofuLevelWasm::ExtC as u32, 1);
        assert_eq!(DetofuLevelWasm::ExtD as u32, 2);
        assert_eq!(DetofuLevelWasm::ExtE as u32, 3);
        assert_eq!(DetofuLevelWasm::ExtF as u32, 4);
        assert_eq!(DetofuLevelWasm::ExtG as u32, 5);
        assert_eq!(DetofuLevelWasm::ExtH as u32, 6);
        assert_eq!(DetofuLevelWasm::ExtI as u32, 7);
    }

    #[test]
    fn test_wasm_custom_dict_spec_to_custom_dict_spec() {
        let spec = WasmCustomDictSpec {
            slot: "STPhrases".to_string(),
            mode: Some("Append".to_string()),
            pairs: vec![("帕兰蒂尔".to_string(), "柏蘭蒂爾".to_string())],
        };

        let spec: CustomDictSpec = spec.try_into().unwrap();

        assert_eq!(spec.slot, DictSlot::STPhrases);
        assert_eq!(spec.mode, CustomDictMode::Append);
        assert_eq!(
            spec.pairs,
            vec![("帕兰蒂尔".to_string(), "柏蘭蒂爾".to_string())]
        );
    }

    #[test]
    fn test_new_with_custom_dicts_append_pairs() {
        let spec = CustomDictSpec {
            slot: DictSlot::STPhrases,
            mode: CustomDictMode::Append,
            pairs: vec![("帕兰蒂尔".to_string(), "柏蘭蒂爾".to_string())],
        };

        let dictionary = DictionaryMaxlength::from_embedded_cbor()
            .with_custom_dicts(&[spec])
            .unwrap();

        let mut inner = OpenCC::from_dictionary(dictionary);
        inner.set_parallel(false);

        let output = inner.convert_with_config("帕兰蒂尔", OpenccConfig::S2t, false);
        assert_eq!(output, "柏蘭蒂爾");
    }

    #[test]
    fn test_wasm_custom_dict_spec_invalid_slot() {
        let spec = WasmCustomDictSpec {
            slot: "STPhrases.txt".to_string(),
            mode: Some("Append".to_string()),
            pairs: vec![],
        };

        assert!(CustomDictSpec::try_from(spec).is_err());
    }
    #[test]
    fn normalize_mode_wasm_values_are_stable() {
        assert_eq!(NormalizeModeWasm::None as u32, 0);
        assert_eq!(NormalizeModeWasm::Compat as u32, 1);
        assert_eq!(NormalizeModeWasm::UnicodeCompat as u32, 2);
        assert_eq!(NormalizeModeWasm::CompatExtended as u32, 3);
    }

    #[test]
    fn test_office_pipeline_applies_normalization_before_opencc() {
        let input = make_test_zip(&[(
            "word/document.xml",
            "<w:document><w:t>天龍八部</w:t></w:document>".as_bytes(),
        )]);
        let cc = OpenccWasm::new(Some("t2s".to_string())).unwrap();

        let output = cc
            .convert_office_bytes_pipeline(
                &input,
                "docx",
                false,
                false,
                NormalizeModeWasm::Compat,
                None,
            )
            .expect("Office pipeline conversion should succeed");

        let document = read_test_zip_entry(&output, "word/document.xml");
        assert!(
            document.contains("天龙八部"),
            "expected compatibility normalization before t2s conversion, got: {document}"
        );
    }

    #[test]
    fn test_office_pipeline_applies_detofu_after_opencc() {
        let input = make_test_zip(&[(
            "word/document.xml",
            "<w:document><w:t>儼驂騑於上路</w:t></w:document>".as_bytes(),
        )]);
        let cc = OpenccWasm::new(Some("t2s".to_string())).unwrap();

        let output = cc
            .convert_office_bytes_pipeline(
                &input,
                "docx",
                false,
                false,
                NormalizeModeWasm::None,
                Some(DetofuLevelWasm::ExtB),
            )
            .expect("Office pipeline conversion should succeed");

        let document = read_test_zip_entry(&output, "word/document.xml");
        assert!(
            document.contains("俨骖騑于上路"),
            "expected DeTofu after t2s conversion, got: {document}"
        );
    }

    #[test]
    fn test_office_pipeline_none_matches_legacy_office_conversion() {
        let input = make_test_zip(&[(
            "word/document.xml",
            "<w:document><w:t>汉语</w:t></w:document>".as_bytes(),
        )]);
        let cc = OpenccWasm::new(Some("s2t".to_string())).unwrap();

        let legacy = cc
            .convert_office_bytes(&input, "docx", false, false)
            .expect("legacy Office conversion should succeed");
        let pipeline = cc
            .convert_office_bytes_pipeline(
                &input,
                "docx",
                false,
                false,
                NormalizeModeWasm::None,
                None,
            )
            .expect("pipeline Office conversion should succeed");

        assert_eq!(
            read_test_zip_entry(&legacy, "word/document.xml"),
            read_test_zip_entry(&pipeline, "word/document.xml")
        );
    }
}
