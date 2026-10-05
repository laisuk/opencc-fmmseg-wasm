import assert from "node:assert/strict";
import {readFileSync} from "node:fs";
import init, {OpenccWasm, OpenccConfigWasm} from "../pkg/opencc_fmmseg_wasm.js";

await init({module_or_path: readFileSync(new URL("../pkg/opencc_fmmseg_wasm_bg.wasm", import.meta.url))});

const traditional = "你好，小篆國際編碼18";
const simplified = "你好，小篆国际编码18";
const seal = "你𿒛，𽌠𽴖𾇓𿭖𿛛碼18";
for (const [name, variant, id, input, expected] of [
    ["s2seal", "S2seal", 21, simplified, seal],
    ["t2seal", "T2seal", 22, traditional, seal],
    ["seal2s", "Seal2s", 23, seal, simplified],
    ["seal2t", "Seal2t", 24, seal, traditional],
]) {
    assert.equal(OpenccConfigWasm[variant], id);
    assert.ok(OpenccWasm.getSupportedConfigs().includes(name));
    assert.ok(OpenccWasm.isValidConfig(name.toUpperCase()));
    const cc = new OpenccWasm(name);
    assert.equal(cc.convert(input, false), expected);
    assert.ok(cc.setConfig(name.toUpperCase()));
    assert.equal(cc.getConfig(), name);
    cc.setConfigEnum(OpenccConfigWasm[variant]);
    assert.equal(cc.convert(input, false), expected);
    cc.free();
}
assert.equal(OpenccWasm.getSupportedConfigs().length, 24);

// Seal inputs use astral scalars; custom slots must work through JS serialization.
const cc = OpenccWasm.newWithCustomDicts("seal2t", [{
    slot: "SealCharacters", mode: "Override", pairs: [["𿒛", "好"]],
}]);
assert.equal(cc.convert("𿒛", false), "好");
cc.free();
console.log("Seal WASM tests passed.");
