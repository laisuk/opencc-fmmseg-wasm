# opencc-fmmseg-wasm

[![npm version](https://img.shields.io/npm/v/@laisuk/opencc-fmmseg-wasm)](https://www.npmjs.com/package/@laisuk/opencc-fmmseg-wasm)
[![npm downloads](https://img.shields.io/npm/dm/@laisuk/opencc-fmmseg-wasm)](https://www.npmjs.com/package/@laisuk/opencc-fmmseg-wasm)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![WebAssembly](https://img.shields.io/badge/WebAssembly-enabled-blue)](https://webassembly.org/)

OpenCC FMM segmentation WebAssembly bindings for browsers and JavaScript runtimes.

This package provides high-quality Simplified Chinese ↔ Traditional Chinese conversion powered by the Rust [
`opencc-fmmseg`](https://github.com/laisuk/opencc-fmmseg) engine.

Features:

* OpenCC-compatible conversion configs
* Pure WebAssembly (no native binaries)
* Browser-friendly
* TypeScript-friendly APIs
* Fast Rust backend
* FMM-based phrase segmentation
* Traditional Chinese regional variants
* Japanese Shinjitai conversion support
* Chinese script detection (`zho_check`)
* In-memory Office / EPUB document conversion
* Zero-dependency Node.js CLI

---

## Installation

```bash
npm install @laisuk/opencc-fmmseg-wasm
```

---

## Quick Start

```javascript
import init, {
    OpenccWasm,
    DetofuLevelWasm
} from "@laisuk/opencc-fmmseg-wasm";

await init();

const cc = new OpenccWasm("s2t");

console.log(cc.convert("汉字", false));
// 漢字

console.log(cc.convertDetofu("儼驂騑於上路", false, DetofuLevelWasm.ExtB));
// 俨骖騑于上路
```

---

## Using Config Enums

```javascript
import init, {
    OpenccWasm,
    OpenccConfigWasm
} from "@laisuk/opencc-fmmseg-wasm";

await init();

const cc = OpenccWasm.newWithEnum(
    OpenccConfigWasm.S2t
);

console.log(cc.convert("汉字", false));
// 漢字
```

---

## Supported Configs

| Config  | Description                                        |
|---------|----------------------------------------------------|
| `s2t`   | Simplified Chinese → Traditional Chinese           |
| `s2tw`  | Simplified Chinese → Taiwan Traditional            |
| `s2twp` | Simplified Chinese → Taiwan Traditional (phrases)  |
| `s2hk`  | Simplified Chinese → Hong Kong Traditional         |
| `t2s`   | Traditional Chinese → Simplified Chinese           |
| `t2tw`  | Traditional Chinese → Taiwan Traditional           |
| `t2twp` | Traditional Chinese → Taiwan Traditional (phrases) |
| `t2hk`  | Traditional Chinese → Hong Kong Traditional        |
| `tw2s`  | Taiwan Traditional → Simplified Chinese            |
| `tw2sp` | Taiwan Traditional → Simplified Chinese (phrases)  |
| `tw2t`  | Taiwan Traditional → Traditional Chinese           |
| `tw2tp` | Taiwan Traditional → Traditional Chinese (phrases) |
| `hk2s`  | Hong Kong Traditional → Simplified Chinese         |
| `hk2t`  | Hong Kong Traditional → Traditional Chinese        |
| `jp2t`  | Japanese Shinjitai → Traditional Chinese           |
| `t2jp`  | Traditional Chinese → Japanese Shinjitai           |

---

## API

### Constructor

```javascript
const cc = new OpenccWasm("s2t");
```

Parameters:

* `config` (optional): OpenCC config string
* default: `"s2t"`

Example:

```javascript
const cc = new OpenccWasm("t2s");
```

---

### convert

```javascript
cc.convert(text, punctuation)
```

Parameters:

* `text`: input string
* `punctuation`: whether to convert punctuation variants

Returns:

* converted string

Example:

```javascript
cc.convert("汉字", false);
```

---

### detofu

Replace tofu-risk rare CJK extension characters with display-compatible fallbacks.

```javascript
cc.detofu(text, level)
```

Parameters:

* `text`: input string
* `level`: `DetofuLevelWasm` threshold for the CJK extension ranges to replace

Returns:

* detofu-safe string

Supported levels:

| Enum                   | CLI value |
|------------------------|-----------|
| `DetofuLevelWasm.ExtB` | `ext-b`   |
| `DetofuLevelWasm.ExtC` | `ext-c`   |
| `DetofuLevelWasm.ExtD` | `ext-d`   |
| `DetofuLevelWasm.ExtE` | `ext-e`   |
| `DetofuLevelWasm.ExtF` | `ext-f`   |
| `DetofuLevelWasm.ExtG` | `ext-g`   |
| `DetofuLevelWasm.ExtH` | `ext-h`   |
| `DetofuLevelWasm.ExtI` | `ext-i`   |

Example:

```javascript
import init, {
    OpenccWasm,
    DetofuLevelWasm
} from "@laisuk/opencc-fmmseg-wasm";

await init();

const cc = new OpenccWasm("t2s");
const converted = cc.convert("儼驂騑於上路", false);

console.log(converted);
// 俨骖𬴂于上路

console.log(cc.detofu(converted, DetofuLevelWasm.ExtB));
// 俨骖騑于上路
```

---

### convertDetofu

Convert text and apply detofu in one call.

```javascript
cc.convertDetofu(text, punctuation, level)
```

Parameters:

* `text`: input string
* `punctuation`: whether to convert punctuation variants
* `level`: `DetofuLevelWasm` threshold for the CJK extension ranges to replace

Returns:

* converted detofu-safe string

Example:

```javascript
cc.convertDetofu("儼驂騑於上路", false, DetofuLevelWasm.ExtB);
// 俨骖騑于上路
```

---

### setConfig

```javascript
cc.setConfig("t2s");
```

Returns:

* `true` if valid
* `false` if invalid

---

### getConfig

```javascript
cc.getConfig();
```

Returns current config string.

---

### isValidConfig

```javascript
OpenccWasm.isValidConfig("s2t");
```

---

### getSupportedConfigs

```javascript
OpenccWasm.getSupportedConfigs();
```

Returns all supported config strings.

---

### zhoCheck

Detect Chinese script type.

```javascript
cc.zhoCheck(text);
```

Returns:

| Value | Meaning             |
|-------|---------------------|
| `0`   | Unknown / mixed     |
| `1`   | Traditional Chinese |
| `2`   | Simplified Chinese  |

---

## Office / EPUB Conversion

Office and EPUB conversion runs fully locally in the browser or Node.js. Files are passed in and returned as bytes;
nothing is uploaded to a backend server.

This is useful for converting text inside:

```text
docx, xlsx, pptx, odt, ods, odp, epub
```

File size is limited by available browser or Node.js memory, but there is no upload or server-side limit. Font
preservation is supported with the `keepFont` option.

```javascript
convert_office_bytes(inputBytes, format, config, punctuation, keepFont)
```

Parameters:

* `inputBytes`: `Uint8Array` document bytes
* `format`: `docx`, `xlsx`, `pptx`, `odt`, `ods`, `odp`, or `epub`
* `config`: OpenCC config string, such as `"s2t"`
* `punctuation`: whether to convert punctuation variants
* `keepFont`: whether to preserve font declarations where supported

Returns:

* converted output bytes

### Browser Office Example

```javascript
import init, {convert_office_bytes} from "@laisuk/opencc-fmmseg-wasm";

await init();

const file = document.querySelector("input[type=file]").files[0];
const inputBytes = new Uint8Array(await file.arrayBuffer());

const outputBytes = convert_office_bytes(
    inputBytes,
    "docx",
    "s2t",
    true,
    true
);

const blob = new Blob([outputBytes], {
    type: "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
});

const a = document.createElement("a");
a.href = URL.createObjectURL(blob);
a.download = "converted.docx";
a.click();
URL.revokeObjectURL(a.href);
```

### Node.js Office Example

```javascript
import fs from "fs";
import init, {convert_office_bytes} from "@laisuk/opencc-fmmseg-wasm";

await init();

const inputBytes = fs.readFileSync("input.docx");

const outputBytes = convert_office_bytes(
    inputBytes,
    "docx",
    "s2t",
    true,
    true
);

fs.writeFileSync("output.docx", outputBytes);
```

---

## Browser Example

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>OpenCC WASM Demo</title>
</head>
<body>

<script type="module">
    import init, {
        OpenccWasm
    } from "./pkg/opencc_fmmseg_wasm.js";

    await init();

    const cc = new OpenccWasm("s2t");

    console.log(
            cc.convert("汉字", false)
    );
</script>

</body>
</html>
```

---

## Node.js CLI

The package includes a zero-dependency Node.js CLI:

```bash
opencc-fmmseg convert -i input.txt -o output.txt -c s2t -p
opencc-fmmseg convert -i input.txt -o output.txt -c t2s -p --detofu all
```

```bash
opencc-fmmseg office -i input.docx -o output.docx -c s2t -p --keep-font
```

### Text Conversion Options

```text
-i, --input <file>          Input text file
-o, --output <file>         Output text file
-c, --config <conversion>   Conversion config
-p, --punct                 Enable punctuation conversion
--detofu [level]            Replace tofu-risk rare CJK extension chars after conversion
                              level: all | ext-b | ext-c | ext-d | ext-e | ext-f | ext-g | ext-h | ext-i
                              default when omitted value: all
--in-enc <encoding>         Input encoding
--out-enc <encoding>        Output encoding
```

### Office / EPUB Options

```text
-i, --input <file>          Input Office / EPUB file
-o, --output <file>         Output file
-c, --config <conversion>   Conversion config
-p, --punct                 Enable punctuation conversion
--format <format>           docx | xlsx | pptx | odt | ods | odp | epub
--auto-ext                  Append extension to output if missing
--keep-font                 Preserve font-family information
--no-keep-font              Do not preserve font-family information
```

For `office`, the format is inferred from the input file extension when `--format` is omitted.

If `-o, --output` is omitted, `office` writes:

```text
<input-name>_converted.<ext>
```

---

## TypeScript Support

The package includes generated TypeScript definitions from `wasm-bindgen`.

The WASM-facing enum is exported as `OpenccConfigWasm`, alongside `OpenccWasm`.

---

## Performance Notes

* WebAssembly build disables Rayon parallelism by default.
* Dictionaries are embedded into the WASM binary.
* Browser caching significantly improves subsequent loads.

---

## Related Projects

* Rust backend: https://github.com/laisuk/opencc-fmmseg
* C API: https://github.com/laisuk/opencc-fmmseg/tree/master/capi/opencc-fmmseg-capi
* .NET: https://github.com/laisuk/OpenccNet
* Python: https://github.com/laisuk/opencc_purepy

---

## License

MIT
