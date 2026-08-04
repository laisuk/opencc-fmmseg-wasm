import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

import init, { DetofuLevelWasm, OpenccWasm } from "../pkg/opencc_fmmseg_wasm.js";

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

const input = "儼驂騑於上路";
const converted = cc.convert(input, false);

expectEqual(
    converted,
    "俨骖𬴂于上路",
    "convert() failed"
);

const safe = cc.detofu(converted, DetofuLevelWasm.ExtB);

expectEqual(
    safe,
    "俨骖騑于上路",
    "detofu() failed"
);

const convertedSafe = cc.convertDetofu(input, false, DetofuLevelWasm.ExtB);

expectEqual(
    convertedSafe,
    "俨骖騑于上路",
    "convertDetofu() failed"
);

console.log("Input: " + input);
console.log("Converted: " + converted);
console.log("DeTofu: " + safe);

console.log("DeTofu WASM test passed.");
