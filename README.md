# opencc-fmmseg-wasm

[![npm version](https://img.shields.io/npm/v/@laisuk/opencc-fmmseg-wasm)](https://www.npmjs.com/package/@laisuk/opencc-fmmseg-wasm)
[![npm downloads](https://img.shields.io/npm/dm/@laisuk/opencc-fmmseg-wasm)](https://www.npmjs.com/package/@laisuk/opencc-fmmseg-wasm)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![WebAssembly](https://img.shields.io/badge/WebAssembly-enabled-blue)](https://webassembly.org/)

OpenCC FMM segmentation WebAssembly bindings for browsers and JavaScript runtimes.

This package provides high-quality Simplified Chinese ↔ Traditional Chinese conversion powered by the Rust [
`opencc-fmmseg`](https://github.com/laisuk/opencc-fmmseg) engine.

Features:

- OpenCC-compatible conversion configs
- Pure WebAssembly (no native binaries)
- Browser-friendly
- TypeScript-friendly APIs
- Fast Rust backend
- FMM-based phrase segmentation
- Traditional Chinese regional variants
- Japanese Shinjitai conversion support
- Chinese script detection (`zho_check`)
- Optional CJK Compatibility Ideograph normalization
- In-memory Office / EPUB document conversion
- Zero-dependency Node.js CLI

Package profile:

- 0 runtime dependencies
- 1 WASM file
- 20 conversion configs
- 100% offline

### 🌐 Live Demo

Try `opencc-fmmseg-wasm` directly in your browser:

**[Open the live WASM demo: CJK Conversion Tool](https://laisuk.github.io/opencc-fmmseg-wasm/)**

No installation or server backend required.

---

## Installation

```bash
npm install @laisuk/opencc-fmmseg-wasm
```

Or installation from the mirror package:

```bash
npm install opencc-fmmseg-wasm
```

---

## Quick Start

```javascript
import init, {
    OpenccWasm
} from "@laisuk/opencc-fmmseg-wasm";

await init();

const cc = new OpenccWasm("s2t");

console.log(cc.convert("汉字转换测试", false));
// 漢字轉換測試
```

---

## API

### Constructor

```javascript
const cc = new OpenccWasm("s2t");
```

Parameters:

- `config` (optional): OpenCC config string
- default: `"s2t"`

Example:

```javascript
const cc = new OpenccWasm("t2s");
```

Taiwan phrase config example:

```javascript
const cc = new OpenccWasm("s2twp");

cc.convert("预订‘奔驰’品牌出租车", true);
// 預訂『賓士』品牌計程車
```

Hong Kong phrase config example:

```javascript
const cc = new OpenccWasm("hk2sp");

cc.convert("作業系統加密保護個人私隱權", false);
// 操作系统加密保护个人隐私权
```

---

### convert

```javascript
cc.convert(text, punctuation)
```

Parameters:

- `text`: input string
- `punctuation`: whether to convert punctuation variants

Returns:

- converted string

Example:

```javascript
cc.convert("汉字", false);
```

---

### setConfig

```javascript
cc.setConfig("t2s");
```

Returns:

- `true` if valid
- `false` if invalid

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

Includes `s2hkp`, `hk2sp`, `t2hkp`, and `hk2tp`.

---

### getAvailableSlots

```javascript
const slots = OpenccWasm.getAvailableSlots();
```

Returns all canonical dictionary slot names accepted by `newWithCustomDicts` as a string array. The list is sourced from
the core `DictSlot` definitions, so callers can use it to populate selectors or validate custom dictionary input without
maintaining their own slot list.

```javascript
if (!OpenccWasm.getAvailableSlots().includes(slot)) {
    throw new Error(`Unsupported dictionary slot: ${slot}`);
}
```

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

### normalizeCompat

Normalize Unicode CJK Compatibility Ideographs before conversion.

```javascript
cc.normalizeCompat(text)
```

Parameters:

- `text`: input string

Returns:

- normalized string

Example:

```javascript
const cc = new OpenccWasm("t2s");

const input = "天龍八部書裡的喬峰是契丹人";
const normalized = cc.normalizeCompat(input);

console.log(normalized);
// 天龍八部書裡的喬峰是契丹人

console.log(cc.convert(normalized, false));
// 天龙八部书里的乔峰是契丹人
```

This is an optional pre-conversion pass for text that contains CJK Compatibility Ideographs. Unmapped characters are
preserved unchanged. Normal OpenCC conversion does not automatically run this pass, so call it explicitly when
compatibility normalization is desired.

### normalizeUnicodeCompat

Normalize additional Unicode compatibility forms, CJK radicals, allographs, legacy glyphs, and selected
compatibility-like punctuation before conversion.

```javascript
cc.normalizeUnicodeCompat(text)
```

Parameters:

- `text`: input string

Returns:

- normalized string

Example:

```javascript
const cc = new OpenccWasm("t2s");

const input = "聼聼竒羙⽟䂖甁噐⾳";
const normalized = cc.normalizeUnicodeCompat(input);

console.log(normalized);
// 聽聽奇美玉石瓶器音

console.log(cc.convert(normalized, false));
// 听听奇美玉石瓶器音
```

This pass uses the extended Unicode compatibility table and is separate from `normalizeCompat()`. It is useful for text
containing radical forms, historical or allographic Han forms, and other compatibility-like characters that are not
covered by the CJK Compatibility Ideograph ranges.

Unmapped characters are preserved unchanged.

### normalizeCompatExtended

Apply complete compatibility normalization before conversion.

```javascript
cc.normalizeCompatExtended(text)
```

Parameters:

- `text`: input string

Returns:

- normalized string

Example:

```javascript
const cc = new OpenccWasm("t2s");

const input = "天龍八部書裡的聼眾";
const normalized = cc.normalizeCompatExtended(input);

console.log(normalized);
// 天龍八部書裡的聽眾

console.log(cc.convert(normalized, false));
// 天龙八部书里的听众
```

`normalizeCompatExtended()` combines the extended Unicode compatibility table with CJK Compatibility Ideograph
normalization. Use this when input may contain characters handled by either normalization set.

The normalization order is:

1. extended Unicode compatibility normalization;
2. CJK Compatibility Ideograph normalization.

Normal OpenCC conversion does not automatically perform compatibility normalization. Call this method explicitly before
`convert()` when complete compatibility normalization is desired.

---

### detofu

Replace tofu-risk rare CJK extension characters with display-compatible fallbacks.

```javascript
cc.detofu(text, level)
```

Parameters:

- `text`: input string
- `level`: `DetofuLevelWasm` threshold for the CJK extension ranges to replace

Returns:

- detofu-safe string

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

- `text`: input string
- `punctuation`: whether to convert punctuation variants
- `level`: `DetofuLevelWasm` threshold for the CJK extension ranges to replace

Returns:

- converted detofu-safe string

Example:

```javascript
cc.convertDetofu("儼驂騑於上路", false, DetofuLevelWasm.ExtB);
// 俨骖騑于上路
```

---

### newWithCustomDicts

Construct a converter with in-memory custom dictionary pairs.

```javascript
const cc = OpenccWasm.newWithCustomDicts(config, specs);
```

Parameters:

- `config`: OpenCC config string, such as `"s2t"`
- `specs`: array of custom dictionary specs

TypeScript-style spec shape:

```typescript
type WasmCustomDictSpec = {
    slot: string;
    mode?: "Append" | "Override";
    pairs: Array<[string, string]>;
};
```

`mode` defaults to `"Append"` when omitted.

Each `pairs` entry is a `[source, target]` string tuple for the selected slot.

TypeScript example:

```typescript
import init, {OpenccWasm} from "@laisuk/opencc-fmmseg-wasm";

await init();

const specs: WasmCustomDictSpec[] = [
    {
        slot: "STPhrases",
        pairs: [
            ["云端", "雲端"]
        ]
    }
];

const cc = OpenccWasm.newWithCustomDicts("s2t", specs);
```

Practical example:

```javascript
import init, {OpenccWasm} from "@laisuk/opencc-fmmseg-wasm";

await init();

const cc = OpenccWasm.newWithCustomDicts("s2t", [
    {
        slot: "STPhrases",
        mode: "Append",
        pairs: [
            ["帕兰蒂尔", "柏蘭蒂爾"],
            ["软件", "軟體"]
        ]
    }
]);

console.log(cc.convert("帕兰蒂尔软件", false));
// 柏蘭蒂爾軟體
```

Override example:

```javascript
const cc = OpenccWasm.newWithCustomDicts("s2t", [
    {
        slot: "STPhrases",
        mode: "Override",
        pairs: [
            ["软件", "軟體"]
        ]
    }
]);
```

`Override` replaces the selected slot before inserting the provided pairs. It is powerful and should be used only when
the caller intentionally wants to discard built-in entries for that slot.

Custom dictionary specs identify the target dictionary slot by `DictSlot` name. Slot names are trimmed and matched
case-insensitively, so `"stphrases"`, `" STPhrases "`, and `"STPhrases"` all select `STPhrases`. Canonical names are
recommended in TypeScript code and docs; use `OpenccWasm.getAvailableSlots()` to retrieve the current list.

Suffixes such as `.txt` are not accepted, even though case and surrounding whitespace are normalized. Use
`"STPhrases"` or `"stphrases"`, not `"STPhrases.txt"`.

Merge contract:

- Custom dictionaries are loaded from in-memory pairs only; no file I/O is involved.
- The embedded compressed CBOR dictionary is loaded first.
- Custom specs are applied to `DictionaryMaxlength` before `OpenCC::from_dictionary(...)`.
- Conversion hot paths remain immutable after construction.
- `Append` mode merges into the selected slot.
- Duplicate or conflicting keys use last-wins semantics.
- `Override` mode clears the selected slot first, then inserts the provided custom pairs.
- Multiple specs are applied in array order.

This API is useful for browser apps, user-defined terminology, database-loaded terms, generated dictionaries,
`localStorage` or `IndexedDB` terms, testing, and embedded WASM environments. Customization happens at construction
time, not during conversion.

---

## Supported Configs

| Config  | Enum                     | Description                                           |
|---------|--------------------------|-------------------------------------------------------|
| `s2t`   | `OpenccConfigWasm.S2t`   | Simplified Chinese → Traditional Chinese              |
| `s2tw`  | `OpenccConfigWasm.S2tw`  | Simplified Chinese → Taiwan Traditional               |
| `s2twp` | `OpenccConfigWasm.S2twp` | Simplified Chinese → Taiwan Traditional (phrases)     |
| `s2hk`  | `OpenccConfigWasm.S2hk`  | Simplified Chinese → Hong Kong Traditional            |
| `s2hkp` | `OpenccConfigWasm.S2hkp` | Simplified Chinese → Hong Kong Traditional (phrases)  |
| `t2s`   | `OpenccConfigWasm.T2s`   | Traditional Chinese → Simplified Chinese              |
| `t2tw`  | `OpenccConfigWasm.T2tw`  | Traditional Chinese → Taiwan Traditional              |
| `t2twp` | `OpenccConfigWasm.T2twp` | Traditional Chinese → Taiwan Traditional (phrases)    |
| `t2hk`  | `OpenccConfigWasm.T2hk`  | Traditional Chinese → Hong Kong Traditional           |
| `t2hkp` | `OpenccConfigWasm.T2hkp` | Traditional Chinese → Hong Kong Traditional (phrases) |
| `tw2s`  | `OpenccConfigWasm.Tw2s`  | Taiwan Traditional → Simplified Chinese               |
| `tw2sp` | `OpenccConfigWasm.Tw2sp` | Taiwan Traditional → Simplified Chinese (phrases)     |
| `tw2t`  | `OpenccConfigWasm.Tw2t`  | Taiwan Traditional → Traditional Chinese              |
| `tw2tp` | `OpenccConfigWasm.Tw2tp` | Taiwan Traditional → Traditional Chinese (phrases)    |
| `hk2s`  | `OpenccConfigWasm.Hk2s`  | Hong Kong Traditional → Simplified Chinese            |
| `hk2sp` | `OpenccConfigWasm.Hk2sp` | Hong Kong Traditional → Simplified Chinese (phrases)  |
| `hk2t`  | `OpenccConfigWasm.Hk2t`  | Hong Kong Traditional → Traditional Chinese           |
| `hk2tp` | `OpenccConfigWasm.Hk2tp` | Hong Kong Traditional → Traditional Chinese (phrases) |
| `jp2t`  | `OpenccConfigWasm.Jp2t`  | Japanese Shinjitai → Traditional Chinese              |
| `t2jp`  | `OpenccConfigWasm.T2jp`  | Traditional Chinese → Japanese Shinjitai              |

The numeric enum values match the vendored Rust backend. Existing values are unchanged; `S2hkp = 17`, `Hk2sp = 18`,
`T2hkp = 19`, and `Hk2tp = 20`.

---

## Using Config Enums

```javascript
import init, {
    OpenccWasm,
    OpenccConfigWasm
} from "@laisuk/opencc-fmmseg-wasm";

await init();

const cc = OpenccWasm.newWithEnum(
    OpenccConfigWasm.S2hkp
);

console.log(cc.convert("操作系统加密保护个人隐私权", false));
// 作業系統加密保護個人私隱權
```

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

Use the instance method when possible. It reuses the converter configuration and any custom dictionaries already held by
the `OpenccWasm` instance.

```javascript
cc.convertOfficeBytes(inputBytes, format, punctuation, keepFont)
```

Parameters:

- `inputBytes`: `Uint8Array` document bytes
- `format`: `docx`, `xlsx`, `pptx`, `odt`, `ods`, `odp`, or `epub`
- `punctuation`: whether to convert punctuation variants
- `keepFont`: whether to preserve font declarations where supported

Returns:

- converted output bytes

The older free function remains available for compatibility:

```javascript
convert_office_bytes(inputBytes, format, config, punctuation, keepFont)
```

### Browser Office Example

```javascript
import init, {OpenccWasm} from "@laisuk/opencc-fmmseg-wasm";

await init();

const cc = new OpenccWasm("s2t");
const file = document.querySelector("input[type=file]").files[0];
const inputBytes = new Uint8Array(await file.arrayBuffer());

const outputBytes = cc.convertOfficeBytes(
    inputBytes,
    "docx",
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
import init, {OpenccWasm} from "@laisuk/opencc-fmmseg-wasm";

await init();

const cc = new OpenccWasm("s2t");
const inputBytes = fs.readFileSync("input.docx");

const outputBytes = cc.convertOfficeBytes(
    inputBytes,
    "docx",
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

> **Note**
>
> Normally, `await init();` is sufficient when using the published npm package.
>
> When running directly from a local repository checkout (for example in tests
> or development scripts), initialize using explicit WASM bytes:
>
> ```javascript
> import fs from "fs";
> import init from "../pkg/opencc_fmmseg_wasm.js";
>
> const wasmBytes = fs.readFileSync(
>     "../pkg/opencc_fmmseg_wasm_bg.wasm"
> );
>
> await init({
>     module_or_path: wasmBytes
> });
> ```

---

## Node.js CLI

The package includes a zero-dependency Node.js CLI:

```bash
opencc-fmmseg convert -i input.txt -o output.txt -c s2t -p
opencc-fmmseg convert -i input.txt -o output.txt -c t2s -p --detofu all
echo "操作系统加密保护个人隐私权" | opencc-fmmseg convert -c s2hkp
echo "天龍八部書裡的喬峰是契丹人" | opencc-fmmseg convert -c t2s --norm-compat
// 天龙八部书里的乔峰是契丹人
echo "這個細路哥很靈活" | opencc-fmmseg convert -c hk2sp --custom-dict hkphrasesrev:append:my_hk_dict.txt  
// 这个小男孩很灵活
```

my_hk_dict.txt:

```
# Custom Dictionary

細路哥	小男孩
```

```bash
opencc-fmmseg office -i input.docx -o output.docx -c s2t -p --keep-font
```

### Text Conversion Options

```text
-i, --input <file>          Input text file; stdin if omitted
-o, --output <file>         Output text file; stdout if omitted
-c, --config <conversion>   Conversion config (default: s2t)
-p, --punct                 Enable punctuation conversion
--detofu [level]            Replace tofu-risk rare CJK extension chars after conversion
                              level: all | ext-b | ext-c | ext-d | ext-e | ext-f | ext-g | ext-h | ext-i
                              default when omitted value: all
--keep-ids                  Preserve complete IDS expressions during conversion (default: false)
-n, --norm-compat           Normalize CJK Compatibility Ideographs before conversion (default: false)
-E, --norm-compat-extended  Normalize extended Unicode compatibility forms before conversion (default: false)
-D, --custom-dict <slot:mode:file>
                            Load a custom dictionary.
                            May be specified multiple times.
                            Examples:
                              --custom-dict hkphrasesrev:append:my_hk_dict.txt
                              --custom-dict stphrases:override:terms.txt
--in-enc <encoding>         Input encoding (default: utf8)
--out-enc <encoding>        Output encoding (default: utf8)
```

Supported conversion configs:

```text
s2t, s2tw, s2twp, s2hk, s2hkp, t2s, t2tw, t2twp, t2hk, t2hkp,
tw2s, tw2sp, tw2t, tw2tp, hk2s, hk2sp, hk2t, hk2tp, jp2t, t2jp
```

### Office / EPUB Options

```text
-i, --input <file>          Input Office / EPUB file
-o, --output <file>         Output file
-c, --config <conversion>   Conversion config (default: s2t)
-p, --punct                 Enable punctuation conversion
-f, --format <format>       docx | xlsx | pptx | odt | ods | odp | epub
-F, --convert-filename      Convert generated output filename stem (default: false)
--keep-font                 Preserve font-family information (default)
--no-keep-font              Do not preserve font-family information
--custom-dict <slot:mode:file>
                            Load a custom dictionary.
                            May be specified multiple times.
                            Examples:
                              --custom-dict hkphrasesrev:append:my_hk_dict.txt
                              --custom-dict stphrases:override:terms.txt
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

`OpenccConfigWasm.S2hkp`, `OpenccConfigWasm.Hk2sp`, `OpenccConfigWasm.T2hkp`, and
`OpenccConfigWasm.Hk2tp` are available for Hong Kong phrase conversions and map to backend config IDs `17` through `20`.

---

## Performance Notes

- WebAssembly build disables Rayon parallelism by default.
- Dictionaries are embedded into the WASM binary.
- Browser caching significantly improves subsequent loads.

---

## Related Projects

- Rust backend: https://github.com/laisuk/opencc-fmmseg
- C API: https://github.com/laisuk/opencc-fmmseg/tree/master/capi/opencc-fmmseg-capi
- .NET: https://github.com/laisuk/OpenccNet
- Python: https://github.com/laisuk/opencc_purepy
- Java: https://github.com/laisuk/OpenccJava

---

## License

MIT
