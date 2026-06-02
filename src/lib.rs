use opencc_fmmseg::{OpenCC, OpenccConfig};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WasmOpenccConfig {
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
}

impl WasmOpenccConfig {
    fn into_backend(self) -> OpenccConfig {
        OpenccConfig::from_ffi(self as u32)
            .expect("WasmOpenccConfig must match backend OpenccConfig")
    }
}

impl From<WasmOpenccConfig> for OpenccConfig {
    fn from(value: WasmOpenccConfig) -> Self {
        value.into_backend()
    }
}

#[wasm_bindgen]
pub struct OpenccWasm {
    inner: OpenCC,
    config: OpenccConfig,
}

#[wasm_bindgen]
impl OpenccWasm {
    #[wasm_bindgen(constructor)]
    pub fn new(config: Option<String>) -> Result<OpenccWasm, JsValue> {
        let config = match config.as_deref() {
            Some(s) => {
                OpenccConfig::parse(s).ok_or_else(|| JsValue::from_str("Invalid OpenCC config"))?
            }
            None => OpenccConfig::S2t,
        };

        let mut inner = OpenCC::new_embedded();

        // IMPORTANT for wasm first version
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
    pub fn new_with_enum(config: Option<WasmOpenccConfig>) -> Result<OpenccWasm, JsValue> {
        let config = config.map(OpenccConfig::from).unwrap_or(OpenccConfig::S2t);

        let mut inner = OpenCC::new_embedded();
        inner.set_parallel(false);

        Ok(OpenccWasm { inner, config })
    }

    #[wasm_bindgen(js_name = setConfigEnum)]
    pub fn set_config_enum(&mut self, config: WasmOpenccConfig) {
        self.config = OpenccConfig::from(config);
    }

    #[wasm_bindgen(js_name = getConfigId)]
    pub fn get_config_id(&self) -> u32 {
        self.config.to_ffi()
    }

    #[wasm_bindgen(js_name = zhoCheck)]
    pub fn zho_check(&self, text: &str) -> i32 {
        self.inner.zho_check(text)
    }

    #[wasm_bindgen(js_name = debugPing)]
    pub fn debug_ping(&self) -> String {
        self.inner.convert("汉字", "s2t", false)
    }
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

            assert!(matches!(id, 1..=16));
            assert_eq!(OpenccConfig::from_ffi(id), Some(config));
        }

        assert_eq!(WasmOpenccConfig::S2t as u32, OpenccConfig::S2t.to_ffi());
        assert_eq!(WasmOpenccConfig::T2jp as u32, OpenccConfig::T2jp.to_ffi());
    }
}
