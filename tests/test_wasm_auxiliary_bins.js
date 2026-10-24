import assert from "node:assert/strict";
import {readFileSync} from "node:fs";
import test from "node:test";

const wasm = readFileSync(new URL("../pkg/opencc_fmmseg_wasm_bg.wasm", import.meta.url));

for (const name of [
    "CharactersTofu",
    "CJK_Compatibility_Ideographs",
    "Unicode_Compatibility",
]) {
    test(`WASM embeds ${name} BIN instead of TXT`, () => {
        const data = new URL(`../vendor/opencc-fmmseg/src/data/${name}`, import.meta.url);
        const bin = readFileSync(new URL(`${data.href}.bin`));
        const txt = readFileSync(new URL(`${data.href}.txt`));
        assert.ok(wasm.includes(bin), `Missing or stale ${name}.bin payload; rebuild WASM`);
        assert.ok(!wasm.includes(txt), `${name}.txt embedded; check BIN feature wiring`);
    });
}
