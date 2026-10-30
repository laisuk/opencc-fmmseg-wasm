# ZIP module vendoring for opencc-fmmseg-wasm

Synced on 2026-10-30 from the clean local source tree at
`D:\codes\Rust\opencc-fmmseg\tools\opencc-rs\src\office_converter\zip`,
repository commit `87afddd02ecd70c2ebdb4699de4d1ebb87b1182c`.

This private module is the source project's trimmed and hardened adaptation of
zip 3.0.0, not a standalone Cargo crate. The upstream MIT license is preserved
verbatim in [LICENSE](LICENSE). Original source-port notes and the supplied
advisory are retained as [UPSTREAM_VENDOR_NOTES.md](UPSTREAM_VENDOR_NOTES.md)
and [SECURITY_ADVISORY.md](SECURITY_ADVISORY.md). Paths and verification claims
in those historical notes refer to their source repositories, not this WASM port.

## Local integration changes

- Rebased internal paths from `crate::office_converter::zip` to `crate::zip`.
- Added `mod zip` at the crate root; converted production and test imports to
  this module. Kept the existing WASM API, generic text transformer, font handling,
  file replacement lifecycle, and full decoded-read output validation.
- Used the shared portable unsafe-path predicate, while preserving this project's
  error policy rather than the source CLI's skip policy.
- Removed two redundant `ref mut` bindings for Rust 2024 match ergonomics.
- Applied rustfmt and small current-toolchain lint adjustments: removed a blank
  doc-comment line, used `is_some_and`, collapsed the writer drop conditional,
  and marked the platform-dependent symlink target binding with an underscore.
  A function-local Clippy allowance retains upstream leap-year arithmetic without
  introducing a newer integer API. The parent crate's Rust 2024 edition still
  applies; the source module's Rust 1.75 verification is not a WASM-port MSRV claim.
- Retained all eight ZIP regression tests; adapted the three converter security
  regressions to this project's API, native test helpers, and unsafe-path policy.

## Cargo dependency graph

Direct ZIP support dependencies:

```text
crc32fast 1.5
flate2 =1.1.1 (default-features = false, rust_backend)
  crc32fast
  miniz_oxide
    adler2
indexmap =2.7.1
  equivalent
  hashbrown
memchr 2.7
```

The exact flate2/indexmap pins match the supplied source integration. The external
`zip` package, Zopfli, and zlib-rs are absent from the resolved runtime graph.
`bumpalo` remains required by wasm-bindgen's build tooling. Other dependencies of
the WASM API and dictionary tools are unchanged. Cargo.lock records the refreshed
resolution. Compression bytes may change with the miniz_oxide backend.

Inspect with `cargo tree -p opencc-fmmseg-wasm --target wasm32-unknown-unknown`.

## Validation and maintenance

Header consistency validation runs in `ZipArchive::new` and `with_config`, before
entries can be inspected or raw-copied. It checks raw names, UnicodePath identity,
selected flags, compression methods, CRC/sizes, duplicate effective names, checked
offsets, central-directory boundaries, and indexed-entry overlaps. Rebuilt output
is validated again and fully decoded by the converter.

This deliberately rejects some ambiguous archives previously accepted. It does
not impose archive resource quotas, reconcile every parser interpretation, validate
all data-descriptor trailer contents, or prevent concurrent mutation of underlying
file input. The source implementation's additional limitations remain documented
in UPSTREAM_VENDOR_NOTES.md. Native Windows replacement retains its previous
non-atomic behavior.

Vendoring removes automated external `zip` alerts. Maintainers must continue
tracking upstream ZIP/RustSec advisories, reviewing fixes against retained code,
preserving the license and integration notes, and running native regression tests,
WASM compilation, and browser/Node bindings tests when updating this module.
Do not treat removal from Cargo.lock as proof that an advisory is resolved.

## Port verification (2026-10-30, Windows x86_64 MSVC)

- `cargo test --workspace --offline`: PASS, including 48 WASM-crate native tests
  and workspace documentation tests; existing ignored tests remain ignored.
- `cargo clippy -p opencc-fmmseg-wasm --lib --offline -- -D warnings`: PASS.
- The same library Clippy check with `--target wasm32-unknown-unknown`: PASS.
- All-target Clippy passes with `-A clippy::io_other_error` for an existing
  injected-error test in converter.rs; strict all-target Clippy still reports
  that pre-existing lint.
- `cargo fmt -p opencc-fmmseg-wasm -- --check` and `git diff --check`: PASS.
- `wasm-pack build --target web --release -- --offline`: PASS, including wasm-opt.
- `node --test tests/*.js`: PASS, 11 tests. New WASM regressions cover a real
  DOCX round trip and header mismatches in target and non-target entries.
- The resolved WASM graph and Cargo.lock contain no external `zip`, `zopfli`,
  or `zlib-rs` package. No browser UI or cross-platform runtime testing was run.
