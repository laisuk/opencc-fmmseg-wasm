# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/).

---

## [0.3.4] - Unreleased

### Changed

- CLI: Optimized `opencc.js office`

---

## [0.3.3] - Unreleased

### Added

- Added optional IDS (Ideographic Description Sequence) preservation support:
    - `OpenccWasm.getPreserveIds()`
    - `OpenccWasm.setPreserveIds(bool)`
- Added `opencc.js convert --keep-ids` to preserve complete IDS expressions during conversion.

### Changed

- Update dictionary data.
- Refactored serial and parallel conversion paths to share the same text segmentation logic.
- Complete IDS expressions can now be preserved consistently across Rust, WASM, and CLI conversions when IDS
  preservation is enabled.

---

## [0.3.2] - 2026-06-17

### Changed

- Update dictionary data.

---

## [0.3.1] - 2026-06-16

### Added

- Added `OpenccWasm.version()`.

### Changed

- Update dictionary data.

---

## [0.3.0] - 2026-06-14

### Added

* Added Hong Kong phrase conversion configs:

    * `s2hkp` / `OpenccConfigWasm.S2hkp` (`17`)
    * `hk2sp` / `OpenccConfigWasm.Hk2sp` (`18`)
* Added WASM, TypeScript, and CLI support for the new HK phrase configs.
* Added vendored `dict-generate` support for `HKPhrases.txt` and `HKPhrasesRev.txt`, including JSON serde output.
* Added WebAssembly custom dictionary support via in-memory custom dictionary pairs.
* Added `OpenccWasm.newWithCustomDicts(...)` for constructing converters from the embedded CBOR dictionary with
  post-load custom dictionary injection.
* Added `WasmCustomDictSpec` support for JavaScript and TypeScript custom dictionary configuration.
* Added support for all OpenCC dictionary slots through `DictSlot`-compatible slot names.
* Added tests covering custom dictionary pair injection and slot validation.

### Changed

* Updated dictionary date.
* Updated embedded dictionary artifacts with HK phrase slots.
* Refactored WASM custom dictionary parsing to reuse core `DictSlot` parsing logic as the single source of truth.
* Custom dictionaries are now applied to `DictionaryMaxlength` before `OpenCC` construction, matching the core Rust
  ownership model and immutable conversion pipeline.
* Sync new config chain for JP slot with opencc-fmmseg upstream.

---

## [0.2.5] - 2026-06-08

### Added

- CLI: Added convert --detofu option

### Changed

- Update dictionary date.

---

## [0.2.4] - 2026-06-06

### Changed

- Update dictionary date.

---

## [0.2.3] - 2026-06-05

### Changed

- Update dictionary data
- Renamed the public WASM-facing config enum to `OpenccConfigWasm` so it appears alongside `OpenccWasm` in IDE
  autocomplete.

---

## [0.2.2] - 2026-06-03

### Fixed

* Fixed npm package CLI layout so:

    * `npx opencc-fmmseg ...`
    * `node pkg/bin/opencc.js`

  correctly locate and execute the packaged CLI entrypoint.

* Fixed npm publish artifact synchronization between:

    * `bin/opencc.js`
    * `pkg/bin/opencc.js`

* Refactored npm packaging workflow to use a single-source-of-truth (SSOT) CLI script copied into the generated npm
  package.

* Added PowerShell packaging helpers for stable npm release preparation.

---

## [0.2.1] - 2026-06-03

### Changed

* Internal npm packaging and CLI layout adjustments.

### Notes

* This version was a short-lived transitional packaging release while stabilizing the npm CLI structure.

---

## [0.2.0] - 2026-06-03

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

### Notes

* Browser WASM functionality remained operational throughout the 0.2.x transition.
* Early npm CLI packaging/layout issues were stabilized in later patch releases.

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
