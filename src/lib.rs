mod converter;
pub use converter::OfficeConverter;

use opencc_fmmseg::{
    CustomDictMode, CustomDictSpec, DetofuLevel, DictSlot, DictionaryMaxlength, OpenCC,
    OpenccConfig,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenccConfigWasm {
    S2t = 1,
    S2tw = 2,
    S2twp = 3,
    S2hk = 4,
    T2s = 5,
    T2tw = 6,
    T2twp = 7,
    T2hk = 8,
    Tw2s = 9,
    Tw2sp = 10,
    Tw2t = 11,
    Tw2tp = 12,
    Hk2s = 13,
    Hk2t = 14,
    Jp2t = 15,
    T2jp = 16,
    S2hkp = 17,
    Hk2sp = 18,
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

#[wasm_bindgen]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DetofuLevelWasm {
    ExtB = 2,
    ExtC = 3,
    ExtD = 4,
    ExtE = 5,
    ExtF = 6,
    ExtG = 7,
    ExtH = 8,
    ExtI = 9,
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

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WasmCustomDictSpec {
    pub slot: String,
    pub pairs: Vec<(String, String)>,
    pub mode: Option<String>,
}

fn parse_dict_slot(slot: &str) -> Result<DictSlot, String> {
    DictSlot::try_from(slot).map_err(|_| format!("Invalid DictSlot: {slot}"))
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

#[wasm_bindgen]
impl OpenccWasm {
    pub fn version() -> String {
        env!("CARGO_PKG_VERSION").to_owned()
    }

    #[wasm_bindgen(constructor)]
    pub fn new(config: Option<String>) -> Result<OpenccWasm, JsValue> {
        let config = parse_wasm_config(config)?;

        let mut inner = OpenCC::new_embedded();

        // IMPORTANT for wasm first version
        inner.set_parallel(false);

        Ok(OpenccWasm { inner, config })
    }

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

    pub fn convert(&self, text: &str, punctuation: bool) -> String {
        self.inner.convert(text, self.config.as_str(), punctuation)
    }

    #[wasm_bindgen(js_name = getConfig)]
    pub fn get_config(&self) -> String {
        self.config.as_str().to_string()
    }

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

    #[wasm_bindgen(js_name = isValidConfig)]
    pub fn is_valid_config(config: &str) -> bool {
        OpenccConfig::is_valid_config(config)
    }

    #[wasm_bindgen(js_name = getSupportedConfigs)]
    pub fn get_supported_configs() -> Vec<String> {
        OpenccConfig::ALL
            .iter()
            .map(|c| c.as_str().to_string())
            .collect()
    }

    #[wasm_bindgen(js_name = newWithEnum)]
    pub fn new_with_enum(config: Option<OpenccConfigWasm>) -> Result<OpenccWasm, JsValue> {
        let config = config.map(OpenccConfig::from).unwrap_or(OpenccConfig::S2t);

        let mut inner = OpenCC::new_embedded();
        inner.set_parallel(false);

        Ok(OpenccWasm { inner, config })
    }

    #[wasm_bindgen(js_name = setConfigEnum)]
    pub fn set_config_enum(&mut self, config: OpenccConfigWasm) {
        self.config = OpenccConfig::from(config);
    }

    #[wasm_bindgen(js_name = getConfigId)]
    pub fn get_config_id(&self) -> u32 {
        self.config.to_ffi()
    }

    #[wasm_bindgen(js_name = getPreserveIds)]
    pub fn get_preserve_ids(&self) -> bool {
        self.inner.get_preserve_ids()
    }

    #[wasm_bindgen(js_name = setPreserveIds)]
    pub fn set_preserve_ids(&mut self, value: bool) {
        self.inner.set_preserve_ids(value);
    }

    #[wasm_bindgen(js_name = zhoCheck)]
    pub fn zho_check(&self, text: &str) -> i32 {
        self.inner.zho_check(text)
    }

    #[wasm_bindgen(js_name = detofu)]
    pub fn detofu(&self, text: &str, level: DetofuLevelWasm) -> String {
        self.inner.detofu(text, level.into())
    }

    #[wasm_bindgen(js_name = convertDetofu)]
    pub fn convert_detofu(&self, text: &str, punctuation: bool, level: DetofuLevelWasm) -> String {
        let converted = self.convert(text, punctuation);
        self.inner.detofu(&converted, level.into())
    }

    #[wasm_bindgen(js_name = debugPing)]
    pub fn debug_ping(&self) -> String {
        self.inner.convert("汉字", "s2t", false)
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

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

            assert!(matches!(id, 1..=18));
            assert_eq!(OpenccConfig::from_ffi(id), Some(config));
        }

        assert_eq!(OpenccConfigWasm::S2t as u32, OpenccConfig::S2t.to_ffi());
        assert_eq!(OpenccConfigWasm::T2jp as u32, OpenccConfig::T2jp.to_ffi());
        assert_eq!(OpenccConfigWasm::S2hkp as u32, OpenccConfig::S2hkp.to_ffi());
        assert_eq!(OpenccConfigWasm::Hk2sp as u32, OpenccConfig::Hk2sp.to_ffi());
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
}
