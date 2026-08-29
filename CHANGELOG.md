# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/).

---

## [0.4.0] - 2026-08-29

### Added

- Added `OpenccWasm.normalizeCompatExtended()` and `OpenccWasm.normalizeUnicodeCompat()`.

### Changed

- Updated dictionary data.
- Updated `opencc-fmmseg` native to `v0.12.0`.
- CLI: Optimized error handling.

---

## [0.3.9] - 2026-08-06

### Added

- Added `OpenccWasm.getAvailableSlots()` to return the canonical dictionary slot names accepted by
  `newWithCustomDicts(...)`.
- Added the backend allocation-reuse API `OpenCC::detofu_into(...)`, allowing callers to append DeTofu results into an
  existing `String` buffer and reuse allocations across multiple conversions.

### Changed

- Optimized DeTofu by replacing per-call built-in fallback table construction with a shared, lazily initialized
  `FxHashMap` reused by all conversions. `DetofuMap` now stores only custom override mappings, substantially reducing
  initialization overhead and memory usage while preserving the existing public API and behavior.
- Optimized `OpenccWasm.convertDetofu()` to use the backend `OpenCC::detofu_into(...)` API internally, avoiding an
  additional intermediate DeTofu result allocation without changing the JavaScript API.
- Updated dictionary data.
- Improved Node.js CLI argument parsing and validation with clearer diagnostics for missing option values, invalid
  option values, and malformed custom dictionary specifications.

### Fixed

- Fixed the Node.js CLI `office` command incorrectly forwarding arguments to the WASM API. The `-p` (`--punct`) and
  `--no-keep-font` options now behave as documented during Office document conversion.
- Fixed custom dictionary loading to reject non-UTF-8 dictionary files with a clear error message, matching the
  documented UTF-8 requirement.
- Improved custom dictionary file validation with clearer diagnostics for missing files, invalid specifications, empty
  dictionaries, and malformed dictionary entries.

---

## [0.3.8] -2026-07-28

### Changed

- Update dictionary data

---

## [0.3.7] - 2026-07-13

### Added

- Added WASM-facing `OpenccConfigWasm.T2hkp` and `OpenccConfigWasm.Hk2tp` configs, mapped to the stable backend IDs `19`
  and `20`.
- Added direct vendored backend conversions `OpenCC::t2hkp()` and `OpenCC::hk2tp()` for phrase-aware Traditional ↔ Hong
  Kong Traditional conversion without a punctuation parameter.
- Added `t2hkp` and `hk2tp` to the Node.js CLI supported-config help and public WASM documentation.

### Changed

- Updated conversion dictionary data.
- Refactored the vendored direct Taiwan phrase conversions `t2twp` and `tw2tp` from two dictionary rounds to one using
  the combined `TwTriple` and `TwRevTriple` unions.
- Generalized the vendored Hong Kong phrase union caches to `HkTriple` and `HkRevTriple`; phrase dictionaries remain
  first in every forward and reverse triple-union lookup order.
- Expanded WASM/backend config parity validation from IDs `1..=18` to `1..=20`.

---

## [0.3.6] -2026-07-08

### Added

- Added optional `tofu-bin` feature for loading the built-in DeTofu dictionary from compact `CharactersTofu.bin` data
  via `include_bytes!()`.
- Added binary serialization helpers for the built-in DeTofu dictionary.
- Added regression test verifying `CharactersTofu.bin` produces identical entries to canonical `CharactersTofu.txt`.
- Added `dict-generate --tofu` to generate `CharactersTofu.bin` from `CharactersTofu.txt`.
- Added internal optional `compat-bin` runtime feature for loading CJK Compatibility Ideograph mappings from generated
  `CJK_Compatibility_Ideographs.bin` data.

### Changed

- Built-in DeTofu dictionary loading now uses:
    - embedded `CharactersTofu.txt` by default;
    - embedded `CharactersTofu.bin` when `tofu-bin` is enabled.
- Built-in CJK Compatibility Ideograph mapping loading now mirrors DeTofu runtime packaging:
    - embedded canonical `CJK_Compatibility_Ideographs.txt` by default;
    - embedded generated `CJK_Compatibility_Ideographs.bin` when internal `compat-bin` is enabled.
- Update dictionary data.

---

## [0.3.5] 2026-07-02

### Added

- Added CJK Compatibility Normalization feature in WASM core and CLI.
-
    - Added `opencc.js --norm-compat` feature.
- Documented the public WASM `OpenccWasm.normalizeCompat(...)` API for CJK Compatibility Ideograph normalization.
- Documented `opencc-fmmseg convert --norm-compat` usage for normalizing compatibility ideographs before conversion.
- Update dictionary data.

---

## [0.3.4] - 2026-06-27

### Added

- Added `opencc.js --custom-dict <slot>:<mode>:<file>` feature.
- Added instance-level `OpenccWasm.convertOfficeBytes(...)` for in-memory Office / EPUB conversion using the converter's
  current config and custom dictionaries.

### Changed

- CLI: Optimized `opencc.js office`
- README Office / EPUB examples now recommend `OpenccWasm.convertOfficeBytes(...)`; the existing
  `convert_office_bytes(...)` free function remains available for compatibility.
- Custom dictionary slot names are now trimmed and normalized case-insensitively for known slots, while file-style names
  such as `STPhrases.txt` remain invalid.
- Update dictionary data.

---

## [0.3.3] - 2026-06-18

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
