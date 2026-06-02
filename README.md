# opencc-fmmseg-wasm

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

---

## Installation

```bash
npm install @laisuk/opencc-fmmseg-wasm
```

---

## Quick Start

```javascript
import init, {
    OpenccWasm
} from "@laisuk/opencc-fmmseg-wasm";

await init();

const cc = new OpenccWasm("s2t");

console.log(cc.convert("汉字", false));
// 漢字
```

---

## Using Config Enums

```javascript
import init, {
    OpenccWasm,
    WasmOpenccConfig
} from "@laisuk/opencc-fmmseg-wasm";

await init();

const cc = OpenccWasm.newWithEnum(
    WasmOpenccConfig.S2t
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

## TypeScript Support

The package includes generated TypeScript definitions from `wasm-bindgen`.

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
