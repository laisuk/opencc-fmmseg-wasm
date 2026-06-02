# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/).

---

## [0.1.1] - Unreleased

### Added

* Added browser- and Node.js-compatible Office / EPUB document conversion support powered by Rust WebAssembly (WASM).

* Added `convert_office_bytes()` WASM API for in-memory Office and EPUB conversion.

* Added support for converting:

    * `.docx`
    * `.xlsx`
    * `.pptx`
    * `.odt`
    * `.ods`
    * `.odp`
    * `.epub`

* Added browser-friendly in-memory ZIP conversion pipeline with no filesystem dependency.

* Added zero-dependency Node.js CLI (`opencc.js`) with subcommands:

    * `convert`
    * `office`

* Added local Office / EPUB conversion support for Node.js:

    * punctuation conversion
    * OpenCC config selection
    * automatic Office format inference
    * optional output extension handling
    * optional font preservation

* Added real `.docx` integration tests validating:

    * OpenXML ZIP repacking
    * phrase conversion correctness
    * Traditional Chinese phrase conversion (`码头` → `碼頭`)
    * font preservation behavior

### Changed

* Refactored `converter.rs` to support `wasm32-unknown-unknown` builds.

* Gated native filesystem/path APIs behind:

    * `#[cfg(not(target_arch = "wasm32"))]`

* Replaced `Path`-based ZIP-slip validation with pure string-based ZIP entry validation suitable for browser/WASM
  environments.

* Preserved the existing native Rust Office conversion APIs while exposing the in-memory conversion core to WASM.

* Reduced WASM portability risks by removing unnecessary filesystem coupling from the Office conversion pipeline.

* Updated Node.js WASM initialization to use explicit `.wasm` byte loading compatible with local filesystem execution.

---

## [0.1.0] - 2026-06-02

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
