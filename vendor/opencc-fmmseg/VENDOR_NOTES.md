# Vendored upstream

Synced on 2026-10-10 from `D:\codes\Rust\opencc-fmmseg`, commit
`54910b9f124c6beddbb5e066f62c9a1b5b941da2` (v0.13.0).

Includes upstream library sources, dictionary/data assets, documentation, tests,
`dict-generate`, and its shared `opencc-tool-common` dependency.

Local adaptations:

- The parent workspace owns workspace membership and release profiles.
- Rayon remains optional through `parallel`, enabled for native tools and disabled
  by the WASM dependency. Parallel setters stay disabled when Rayon is absent.
- `ruzstd` remains a compatibility feature alias; upstream now always uses its
  built-in pure-Rust Zstandard decoder. `dictionary-build` enables native encoding.
- `OpenCC::new_embedded` and `DictionaryMaxlength::from_embedded_cbor` remain
  compatibility APIs backed by the bundled compressed dictionary.
- Upstream text-based compatibility tables replace the former generated `.bin`
  assets and their generator feature flags.
- The repository-only Seal diagnostic harness is omitted. Upstream integration
  tests are included. Small lint fixes accommodate the local newer toolchain;
  decoder modulo expressions retain MSRV-compatible syntax.

The upstream Office converter lives in `tools/opencc-rs`, outside the library.
Its `TextConverter` abstraction and helper/lint refactors are adapted in the
parent project's `src/converter.rs` and `src/text_converter.rs`. Existing public
Office APIs and WASM byte/pipeline entry points remain intact. Local differences
retain rejection of unsafe ZIP paths, full rebuilt-entry validation, strict EPUB
mimetype validation, case-insensitive matching, PPTX comment-author coverage,
and the established font marker spelling. Only the reusable transformer is
imported; this project already owns its normalization/conversion/DeTofu pipeline.
