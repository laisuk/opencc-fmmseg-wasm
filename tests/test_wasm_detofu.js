import {readFileSync} from "node:fs";
import {fileURLToPath} from "node:url";
import {dirname, join} from "node:path";

import init, {DetofuLevelWasm, OpenccWasm} from "../pkg/opencc_fmmseg_wasm.js";

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

// Round 2

cc.setConfig("s2t");
const input2 = "㓆䘞";
const converted2 = cc.convert(input2, false);

expectEqual(
    converted2,
    "𠗣𧜗",
    "convert() failed"
);

const safe2 = cc.detofu(converted2, DetofuLevelWasm.ExtB);

expectEqual(
    safe2,
    "㓆䘞",
    "detofu() failed"
);

const convertedSafe2 = cc.convertDetofu(input2, false, DetofuLevelWasm.ExtB);

expectEqual(
    convertedSafe2,
    "㓆䘞",
    "convertDetofu() failed"
);

console.log("Input2: " + input2);
console.log("Converted2: " + converted2);
console.log("DeTofu2: " + safe2);

console.log("DeTofu WASM test passed.");
