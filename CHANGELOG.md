# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/).

---

## [0.1.0] - Unreleased

### Added

* Initial WebAssembly (WASM) bindings for `opencc-fmmseg`.
* Browser-compatible Simplified/Traditional Chinese conversion powered by `wasm-bindgen`.
* Support for OpenCC-compatible conversion configs:

    * `s2t`
    * `s2tw`
    * `s2twp`
    * `s2hk`
    * `t2s`
    * `t2tw`
    * `t2twp`
    * `t2hk`
    * `tw2s`
    * `tw2sp`
    * `tw2t`
    * `tw2tp`
    * `hk2s`
    * `hk2t`
    * `jp2t`
    * `t2jp`
* Browser-accessible punctuation conversion support.
* Browser-accessible `zho_check` language detection helper.
* Embedded precompiled dictionary support via vendored `opencc-fmmseg`.
* Local workspace integration for the `dict-generate` tool.

### Changed

* Updated vendored `opencc-fmmseg` dictionary chains to support:

    * `TWVariantsPhrases`
    * `HKVariantsPhrases`
* Forward Taiwanese and Hong Kong regional conversions now apply phrase dictionaries before character dictionaries,
  matching upstream OpenCC behavior.
* Refactored internal union cache logic from `*_variants_only` to `*_variants_pair`.
* Updated vendored dictionary generator to include the new upstream phrase dictionaries.
* Improved workspace test reliability using `CARGO_MANIFEST_DIR`-based dictionary path resolution.
