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

const input = "天龍八部書裡的喬峰是契丹人";
const normalized = cc.normalizeCompat(input);

expectEqual(
    normalized,
    "天龍八部書裡的喬峰是契丹人",
    "normalizeCompat() failed"
);

const converted = cc.convert(normalized);

expectEqual(
    converted,
    "天龙八部书里的乔峰是契丹人",
    "convert() failed"
);

console.log("Input: " + input);
console.log("Normalized Input: " + normalized);
console.log("Converted: " + converted);

console.log("Normalize Compatability Ideographs WASM test passed.");