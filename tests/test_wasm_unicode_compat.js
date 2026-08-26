import {readFileSync} from "node:fs";
import {fileURLToPath} from "node:url";
import {dirname, join} from "node:path";

import init, {OpenccWasm} from "../pkg/opencc_fmmseg_wasm.js";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const wasmPath = join(__dirname, "../pkg/opencc_fmmseg_wasm_bg.wasm");
const wasmBytes = readFileSync(wasmPath);

await init({
    module_or_path: wasmBytes
});

function expectEqual(actual, expected, message) {
    if (actual !== expected) {
        throw new Error(
            `${message}\nExpected: ${expected}\nActual:   ${actual}`
        );
    }
}

const cc = new OpenccWasm("t2s");

// Unicode_Compatibility.txt only
const unicodeInput = "聼聼竒羙⽟䂖甁噐⾳";
const unicodeNormalized = cc.normalizeUnicodeCompat(unicodeInput);

expectEqual(
    unicodeNormalized,
    "聽聽奇美玉石瓶器音",
    "normalizeUnicodeCompat() failed"
);

// Extended normalization:
// Unicode_Compatibility + CJK_Compatibility_Ideographs
const extendedInput = "天龍八部書裡的聼眾";
const extendedNormalized = cc.normalizeCompatExtended(extendedInput);

expectEqual(
    extendedNormalized,
    "天龍八部書裡的聽眾",
    "normalizeCompatExtended() failed"
);

const converted = cc.convert(extendedNormalized, false);

expectEqual(
    converted,
    "天龙八部书里的听众",
    "convert() after normalizeCompatExtended() failed"
);

console.log("Unicode Input: " + unicodeInput);
console.log("Unicode Normalized: " + unicodeNormalized);
console.log("Extended Input: " + extendedInput);
console.log("Extended Normalized: " + extendedNormalized);
console.log("Converted: " + converted);

console.log("Unicode Compatibility WASM test passed.");