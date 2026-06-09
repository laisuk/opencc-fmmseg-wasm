import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

import init, { OpenccWasm } from "../pkg/opencc_fmmseg_wasm.js";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const wasmPath = join(__dirname, "../pkg/opencc_fmmseg_wasm_bg.wasm");
const wasmBytes = readFileSync(wasmPath);

await init(wasmBytes);

const cc = OpenccWasm.newWithCustomDicts("s2t", [
    {
        slot: "STPhrases",
        mode: "Append",
        pairs: [
            ["帕兰蒂尔", "柏蘭蒂爾"],
            ["软件", "軟體"],
        ],
    },
]);

const input = "帕兰蒂尔软件";
const output = cc.convert(input, false);

console.log(output);

if (output !== "柏蘭蒂爾軟體") {
    throw new Error(`Unexpected output: ${output}`);
}

console.log("Custom dict WASM test passed.");