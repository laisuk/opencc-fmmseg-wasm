import assert from "node:assert/strict";
import {readFileSync} from "node:fs";
import test from "node:test";
import init, {OpenccWasm} from "../pkg/opencc_fmmseg_wasm.js";

await init({
    module_or_path: readFileSync(new URL("../pkg/opencc_fmmseg_wasm_bg.wasm", import.meta.url))
});

const document = readFileSync(new URL("./OneDay.docx", import.meta.url));

test("vendored ZIP converts a real DOCX through the WASM API", () => {
    const cc = new OpenccWasm("s2t");
    try {
        const output = cc.convertOfficeBytes(document, "docx", false, true);
        assert.ok(output instanceof Uint8Array);
        assert.ok(output.length > 0);
        // Reopening the rebuilt package exercises the mandatory header validator.
        assert.ok(cc.convertOfficeBytes(output, "docx", false, true).length > 0);
    } finally {
        cc.free();
    }
});

for (const name of ["word/document.xml", "[Content_Types].xml"]) {
    test(`WASM rejects inconsistent headers for ${name}`, () => {
        const malformed = Buffer.from(document);
        const offset = malformed.indexOf(name);
        assert.ok(offset >= 0, `fixture must contain ${name}`);
        // Change the local filename without changing the central directory name.
        malformed[offset] ^= 1;
        const cc = new OpenccWasm("s2t");
        try {
            assert.throws(
                () => cc.convertOfficeBytes(malformed, "docx", false, true),
                error => String(error).includes("ZIP header inconsistency")
            );
        } finally {
            cc.free();
        }
    });
}
