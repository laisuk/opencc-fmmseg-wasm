use wasm_bindgen::prelude::*;
use opencc_fmmseg::{OpenCC, OpenccConfig};

#[wasm_bindgen]
pub struct OpenccWasm {
    inner: OpenCC,
    config: OpenccConfig,
}

#[wasm_bindgen]
impl OpenccWasm {
    #[wasm_bindgen(constructor)]
    pub fn new(config: Option<String>) -> Result<OpenccWasm, JsValue> {
        let config = match config {
            Some(s) => {
                OpenccConfig::parse(&s)
                    .ok_or_else(|| JsValue::from_str("Invalid OpenCC config"))?
            }
            None => OpenccConfig::S2t,
        };

        let mut inner = OpenCC::new_embedded();

        // IMPORTANT for wasm first version
        inner.set_parallel(false);

        Ok(OpenccWasm { inner, config })
    }

    pub fn convert(
        &self,
        text: &str,
        punctuation: bool,
    ) -> String {
        self.inner.convert(
            text,
            self.config.as_str(),
            punctuation,
        )
    }

    pub fn get_config(&self) -> String {
        self.config.as_str().to_string()
    }

    pub fn set_config(&mut self, config: &str) -> bool {
        match OpenccConfig::parse(config) {
            Some(cfg) => {
                self.config = cfg;
                true
            }
            None => false,
        }
    }

    pub fn is_valid_config(config: &str) -> bool {
        OpenccConfig::is_valid_config(config)
    }

    pub fn get_supported_configs() -> Vec<String> {
        OpenccConfig::ALL
            .iter()
            .map(|c| c.as_str().to_string())
            .collect()
    }

    pub fn zho_check(&self, text: &str) -> i32 {
        self.inner.zho_check(text)
    }

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

        let mut cc2 =
            OpenccWasm::new(Some("t2s".to_string())).unwrap();

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
}