# Browser document extractor

This directory contains the browser build of `pdfium-helper-wasm` from the
[laisuk/pdfium-helper source checkout](https://github.com/laisuk/pdfium-helper/tree/1e3cf0aaea590b4c8bfc726946ca48c16aec27db/crates/pdfium-helper-wasm)
at revision `1e3cf0aaea590b4c8bfc726946ca48c16aec27db`. Its EPUB, DOCX, and ODT
exports use the shared `opencc-office-converter` extractor. The separate PDFium
runtime is outside this package.

To refresh the assets, build a pinned helper checkout with the WebAssembly target
installed and the version of `wasm-bindgen` specified by the helper crate. This
package was built with `wasm-bindgen` 0.2.117 and the pinned Cargo dependencies.
The offline build staged the helper and extractor crates in a temporary workspace
without the CLI crates. To build from the full helper workspace, run:

```sh
cargo build --locked --release --target wasm32-unknown-unknown -p pdfium-helper-wasm
wasm-bindgen --target web --no-typescript --out-dir target/pdfium-helper-web \
  target/wasm32-unknown-unknown/release/pdfium_helper_wasm.wasm
```

Copy `pdfium_helper_wasm.js` and `pdfium_helper_wasm_bg.wasm` from the same
output directory. Initialize the pair over HTTP and compare all three extraction
exports against the default golden files under
`crates/opencc-office-converter/tests/extraction` before deploying. Refresh the
license and dependency notices when updating the package.

| Asset | SHA-256 |
| --- | --- |
| `pdfium_helper_wasm.js` | `af35e047237cb7beff7cebf12cfc021d27f58fd141648c1f445b6a4f61af8f9e` |
| `pdfium_helper_wasm_bg.wasm` | `1dd2e813ac017013b0a11f6488f86dfaff4585b2866ecead34c0ff21bd4fcdab` |

Serve the repository root to use the local OpenCC package, or serve `docs/`
to use the published OpenCC package. Both pages resolve the helper assets from
this directory. GitHub Pages uploads `docs/` directly.
