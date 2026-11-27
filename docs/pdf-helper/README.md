# Browser document extractor

This directory contains the browser build of `pdfium-helper-wasm` from the
[laisuk/pdfium-helper source checkout](https://github.com/laisuk/pdfium-helper/tree/1e3cf0aaea590b4c8bfc726946ca48c16aec27db/crates/pdfium-helper-wasm)
at revision `1e3cf0aaea590b4c8bfc726946ca48c16aec27db`. Its EPUB, DOCX, and ODT
exports use the shared `opencc-office-converter` extractor.

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

| Asset                        | SHA-256                                                            |
|------------------------------|--------------------------------------------------------------------|
| `pdfium_helper_wasm.js`      | `af35e047237cb7beff7cebf12cfc021d27f58fd141648c1f445b6a4f61af8f9e` |
| `pdfium_helper_wasm_bg.wasm` | `1dd2e813ac017013b0a11f6488f86dfaff4585b2866ecead34c0ff21bd4fcdab` |

Serve the repository root to use the local `OpenCC` package, or serve `docs/`
to use the published `OpenCC` package. Both pages resolve the helper assets from
this directory. GitHub Pages uploads `docs/` directly.

## PDFium runtime

The `pdfium/` directory contains PDFium 159.0.8152.0, the classic iframe loader,
the extraction bridge, and their licenses from the pinned
[OpenccNetLibWasmDemo distribution](https://github.com/laisuk/OpenccNetLibWasmDemo/tree/ec6e15d7e1847fd290acebbde0095cd350255a21/wwwroot/pdfium).
The runtime loads on the first PDF operation. The bridge uses the Rust helper
above for page normalization, overlay filtering, and optional reflow.

| Runtime asset        |     Bytes | SHA-256                                                            |
|----------------------|----------:|--------------------------------------------------------------------|
| `pdfium/pdfium.js`   |   165,929 | `4fb0a212e4e5856f0cd7a9877365d0f5d04f6efb170a334d764af1524198146a` |
| `pdfium/pdfium.wasm` | 5,590,002 | `8f12dd340d93ce8e46044d74bbb1587eaa7044ef42622fa1072b1b8fb1acfb05` |

PDF extraction processes one file at a time, up to the reference limit of
512 MiB. The browser holds the complete file, a copy in the PDFium heap,
and extracted text. Available browser memory can impose a lower limit.
Cancellation stops the current operation during loading, file reading, or
between pages. Shared initialization can finish in the background. A single
page extraction, Rust reflow, and `OpenCC` conversion run synchronously.
Extraction uses the PDF text layer. Password entry and OCR are unavailable.
