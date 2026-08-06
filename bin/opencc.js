#!/usr/bin/env node

import fs from "fs";
import path from "path";
import process from "process";

import init, {
    OpenccWasm,
    DetofuLevelWasm
} from "../opencc_fmmseg_wasm.js";

const OFFICE_FORMATS = new Set([
    "docx",
    "xlsx",
    "pptx",
    "odt",
    "ods",
    "odp",
    "epub"
]);

let wasmInitialized = false;

async function ensureWasmInitialized() {
    if (wasmInitialized) {
        return;
    }

    const wasmPath = new URL("../opencc_fmmseg_wasm_bg.wasm", import.meta.url);
    const wasmBytes = fs.readFileSync(wasmPath);

    await init({
        module_or_path: wasmBytes
    });

    wasmInitialized = true;
}

function printHelp() {
    console.log(`
opencc-fmmseg WASM CLI

Usage:
  npx opencc-fmmseg convert [options]
  npx opencc-fmmseg office  [options]

Commands:
  convert                     Convert plain text
  office                      Convert Office / EPUB documents

Convert options:
  -i, --input <file>          Input text file; stdin if omitted
  -o, --output <file>         Output text file; stdout if omitted
  -c, --config <conversion>   Conversion config (default: s2t)
  -p, --punct                 Enable punctuation conversion
  --detofu [level]            Replace tofu-risk rare CJK extension chars after conversion
                              level: all | ext-b | ext-c | ext-d | ext-e | ext-f | ext-g | ext-h | ext-i
                              default when omitted value: all
  --keep-ids                  Preserve complete IDS expressions during conversion (default: false)
  -n, --norm-compat           Normalize CJK Compatibility Ideographs before conversion (default: false)
  -D, --custom-dict <slot:mode:file>
                              Load a custom dictionary.
                              May be specified multiple times.
                              Examples:
                                --custom-dict hkphrasesrev:append:my_hk_dict.txt
                                --custom-dict stphrases:override:terms.txt
  --in-enc <encoding>         Input encoding (default: utf8)
  --out-enc <encoding>        Output encoding (default: utf8)
  
Supported encodings:
  utf8
  utf16le
  latin1
  ascii
  
  Note: utf8 and utf16le are recommended for CJK text.

Supported configs:
  s2t, s2tw, s2twp, s2hk, s2hkp, t2s, t2tw, t2twp, t2hk, t2hkp,
  tw2s, tw2sp, tw2t, tw2tp, hk2s, hk2sp, hk2t, hk2tp, jp2t, t2jp

Office options:
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

General options:
  -h, --help                  Show help

Examples:
  npx opencc-fmmseg convert -i a.txt -o b.txt -c s2t
  npx opencc-fmmseg convert -i a.txt -o b.txt -c s2tw -p
  cat a.txt | npx opencc-fmmseg convert -c t2s
  echo "别随便录影侵犯个人隐私权" | npx opencc-fmmseg convert -c s2hkp
  npx opencc-fmmseg convert -i a.txt -o b.txt -c t2s --detofu
  npx opencc-fmmseg convert -i a.txt -o b.txt -c t2s --detofu ext-c
  echo "⿰氵漢" | npx opencc-fmmseg convert -c t2s
  echo "⿰氵漢" | npx opencc-fmmseg convert -c t2s --keep-ids

  npx opencc-fmmseg office -i a.docx -o b.docx -c s2t -p
  npx opencc-fmmseg office -i a.epub -c s2tw
  npx opencc-fmmseg office -i 软件手册.docx -c s2t --convert-filename
`);
}

function getArg(args, shortName, longName, defaultValue = null) {
    const candidates = [shortName, longName].filter(Boolean);

    for (let i = 0; i < args.length; i++) {
        if (!candidates.includes(args[i])) {
            continue;
        }

        const value = args[i + 1];

        if (value === undefined || value.startsWith("-")) {
            throw new Error(`Missing value for option: ${args[i]}`);
        }

        return value;
    }

    return defaultValue;
}

function getArgs(args, shortName, longName) {
    const values = [];
    const candidates = [shortName, longName].filter(Boolean);

    for (let i = 0; i < args.length; i++) {
        if (!candidates.includes(args[i])) {
            continue;
        }

        const value = args[i + 1];

        if (value === undefined || value.startsWith("-")) {
            throw new Error(`Missing value for option: ${args[i]}`);
        }

        values.push(value);
        i++;
    }

    return values;
}

function validateEncoding(value, optionName) {
    const encoding = String(value).trim().toLowerCase();

    if (!Buffer.isEncoding(encoding)) {
        throw new Error(
            `Unsupported encoding for ${optionName}: ${value}. ` +
            "Supported values: utf8, utf16le, latin1, ascii"
        );
    }

    return encoding;
}

function validateInputFile(filePath) {
    let stats;

    try {
        stats = fs.statSync(filePath);
    } catch (err) {
        if (err?.code === "ENOENT") {
            throw new Error(`Input file not found: ${filePath}`);
        }

        throw new Error(
            `Cannot access input file ${filePath}: ${err?.message || err}`
        );
    }

    if (!stats.isFile()) {
        throw new Error(`Input path is not a file: ${filePath}`);
    }
}

function parseCustomDictSpec(value) {
    const first = value.indexOf(":");
    const second = value.indexOf(":", first + 1);

    if (first < 0 || second < 0) {
        throw new Error(
            `Invalid custom dictionary specification: ${value}\n` +
            "Expected: <slot>:<append|override>:<file>"
        );
    }

    const slot = value.substring(0, first).trim();
    const rawMode = value.substring(first + 1, second).trim();
    const mode = rawMode.toLowerCase();
    const file = value.substring(second + 1).trim();

    if (!slot) {
        throw new Error("Custom dictionary slot is empty.");
    }

    if (!rawMode) {
        throw new Error("Custom dictionary mode is empty.");
    }

    if (mode !== "append" && mode !== "override") {
        throw new Error(
            `Invalid custom dictionary mode: ${rawMode}. ` +
            "Expected: append or override"
        );
    }

    if (!file) {
        throw new Error("Custom dictionary file is empty.");
    }

    return {
        slot,
        mode,
        pairs: loadCustomDictPairs(file)
    };
}

function readUtf8File(file, description) {
    let bytes;

    try {
        bytes = fs.readFileSync(file);
    } catch (err) {
        if (err?.code === "ENOENT") {
            throw new Error(`${description} not found: ${file}`);
        }

        if (err?.code === "EISDIR") {
            throw new Error(`${description} path is not a file: ${file}`);
        }

        throw new Error(
            `Cannot read ${description.toLowerCase()} ${file}: ` +
            `${err?.message || err}`
        );
    }

    // Reject invalid UTF-8.
    const decoder = new TextDecoder("utf-8", {fatal: true});

    try {
        return decoder.decode(bytes);
    } catch {
        throw new Error(
            `${description} must be encoded as UTF-8: ${file}`
        );
    }
}

function loadCustomDictPairs(file) {
    const text = readUtf8File(file, "Custom dictionary file");
    const lines = text.split(/\r?\n/);
    const pairs = [];

    for (let i = 0; i < lines.length; i++) {
        let line = lines[i];

        if (i === 0 && line.charCodeAt(0) === 0xfeff) {
            line = line.slice(1);
        }

        line = line.trimEnd();

        if (!line || line.trimStart().startsWith("#")) {
            continue;
        }

        const tab = line.indexOf("\t");

        if (tab < 0) {
            throw new Error(
                `Invalid custom dictionary file ${file}:${i + 1}: ` +
                "missing TAB separator"
            );
        }

        const source = line.substring(0, tab).trim();
        const values = line.substring(tab + 1).trim().split(/\s+/);
        const target = values[0] || "";

        if (!source || !target) {
            throw new Error(
                `Invalid custom dictionary file ${file}:${i + 1}: ` +
                "empty source or target"
            );
        }

        pairs.push([source, target]);
    }

    if (pairs.length === 0) {
        throw new Error(
            `Custom dictionary file contains no usable entries: ${file}`
        );
    }

    return pairs;
}

function hasFlag(args, shortName, longName) {
    return (
        (shortName && args.includes(shortName)) ||
        (longName && args.includes(longName))
    );
}

function readInputText(filePath, encoding) {
    if (!filePath) {
        return fs.readFileSync(0, encoding);
    }

    try {
        return fs.readFileSync(filePath, encoding);
    } catch (err) {
        if (err?.code === "ENOENT") {
            throw new Error(`Input file not found: ${filePath}`);
        }

        if (err?.code === "EISDIR") {
            throw new Error(`Input path is not a file: ${filePath}`);
        }

        throw new Error(
            `Cannot read input file ${filePath}: ${err?.message || err}`
        );
    }
}

function writeOutputText(filePath, text, encoding) {
    if (!filePath) {
        process.stdout.write(text);
        return;
    }

    fs.writeFileSync(filePath, text, encoding);
}

function inferOfficeFormat(inputFile, explicitFormat) {
    if (explicitFormat) {
        const normalized = explicitFormat.trim().toLowerCase();

        if (!OFFICE_FORMATS.has(normalized)) {
            throw new Error(
                `Invalid office format: ${explicitFormat}. ` +
                `Valid formats: ${Array.from(OFFICE_FORMATS).join(", ")}`
            );
        }

        return normalized;
    }

    const ext = path.extname(inputFile).slice(1).toLowerCase();

    if (!OFFICE_FORMATS.has(ext)) {
        throw new Error(
            `Invalid Office file extension: .${ext || "(none)"}. ` +
            "Valid extensions: .docx | .xlsx | .pptx | .odt | .ods | .odp | .epub"
        );
    }

    return ext;
}

function makeDefaultOfficeOutput(inputFile, officeFormat, convertFilename, cc, config, punct) {
    const parsed = path.parse(inputFile);
    const stem = convertFilename
        ? cc.convert(parsed.name, punct)
        : parsed.name;

    return path.join(
        parsed.dir || process.cwd(),
        `${stem}_converted.${officeFormat}`
    );
}

function applyOutputExtension(outputFile, officeFormat) {
    if (path.extname(outputFile)) {
        return outputFile;
    }

    return `${outputFile}.${officeFormat}`;
}

function parseDetofuLevel(value) {
    if (value === null || value === undefined || value === true) {
        return DetofuLevelWasm.ExtB; // "all"
    }

    const normalized = String(value).trim().toLowerCase();

    switch (normalized) {
        case "":
        case "all":
        case "ext-b":
        case "extb":
        case "b":
            return DetofuLevelWasm.ExtB;
        case "ext-c":
        case "extc":
        case "c":
            return DetofuLevelWasm.ExtC;
        case "ext-d":
        case "extd":
        case "d":
            return DetofuLevelWasm.ExtD;
        case "ext-e":
        case "exte":
        case "e":
            return DetofuLevelWasm.ExtE;
        case "ext-f":
        case "extf":
        case "f":
            return DetofuLevelWasm.ExtF;
        case "ext-g":
        case "extg":
        case "g":
            return DetofuLevelWasm.ExtG;
        case "ext-h":
        case "exth":
        case "h":
            return DetofuLevelWasm.ExtH;
        case "ext-i":
        case "exti":
        case "i":
            return DetofuLevelWasm.ExtI;
        default:
            throw new Error(
                `Invalid detofu level: ${value}. ` +
                `Valid values: all | ext-b | ext-c | ext-d | ext-e | ext-f | ext-g | ext-h | ext-i`
            );
    }
}

async function runConvert(args) {
    const input = getArg(args, "-i", "--input");
    const output = getArg(args, "-o", "--output");
    const config = getArg(args, "-c", "--config", "s2t");
    const inEnc = validateEncoding(
        getArg(args, null, "--in-enc", "utf8"),
        "--in-enc"
    );

    const outEnc = validateEncoding(
        getArg(args, null, "--out-enc", "utf8"),
        "--out-enc"
    );
    const punct = hasFlag(args, "-p", "--punct");
    const keepIds = hasFlag(args, null, "--keep-ids");
    const normCompat = hasFlag(args, "-n", "--norm-compat");
    const customDicts = getArgs(args, "-D", "--custom-dict")
        .map(parseCustomDictSpec);

    const detofuIndex = args.indexOf("--detofu");
    const detofuEnabled = detofuIndex !== -1;
    let detofuLevel = null;

    if (detofuEnabled) {
        const next = args[detofuIndex + 1];

        if (!next || next.startsWith("-")) {
            detofuLevel = parseDetofuLevel("all");
        } else {
            detofuLevel = parseDetofuLevel(next);
        }
    }

    await ensureWasmInitialized();

    // const cc = new OpenccWasm(config);
    const cc = customDicts.length === 0
        ? new OpenccWasm(config)
        : OpenccWasm.newWithCustomDicts(config, customDicts);

    if (keepIds) {
        cc.setPreserveIds(true);
    }

    // Prompt user if reading from interactive terminal
    if (!input && process.stdin.isTTY) {
        console.error("Input text to convert, <Ctrl+Z>/<Ctrl+D> to submit:");
    }

    let inputText = readInputText(input, inEnc);

    if (normCompat) {
        inputText = cc.normalizeCompat(inputText);
    }

    let outputText = cc.convert(inputText, punct);

    if (detofuEnabled) {
        outputText = cc.detofu(outputText, detofuLevel);
    }

    writeOutputText(output, outputText, outEnc);

    const inFrom = input || "<stdin>";
    const outTo = output || "stdout";

    if (process.stderr.isTTY) {
        if (!output && outputText && !outputText.endsWith("\n")) {
            console.error();
        }

        const suffixParts = [];
        if (normCompat) suffixParts.push("normalized");
        if (detofuEnabled) suffixParts.push("detofu");
        if (keepIds) suffixParts.push("keep-ids");

        const suffix = suffixParts.length ? `, ${suffixParts.join(", ")}` : "";
        console.error(`Conversion completed (${cc.getConfig()}${suffix}): ${inFrom} -> ${outTo}`);
    }
}

async function runOffice(args) {
    const input = getArg(args, "-i", "--input");
    let output = getArg(args, "-o", "--output");
    const config = getArg(args, "-c", "--config", "s2t");
    const explicitFormat = getArg(args, "-f", "--format");
    const punct = hasFlag(args, "-p", "--punct");
    const convertFilename = hasFlag(args, "-F", "--convert-filename");
    const keepFont = !hasFlag(args, null, "--no-keep-font");
    const customDicts = getArgs(args, "-D", "--custom-dict")
        .map(parseCustomDictSpec);

    if (!input) {
        throw new Error("Input file is missing.");
    }

    validateInputFile(input);

    const officeFormat = inferOfficeFormat(input, explicitFormat);

    await ensureWasmInitialized();

    // const cc = new OpenccWasm(config);
    const cc = customDicts.length === 0
        ? new OpenccWasm(config)
        : OpenccWasm.newWithCustomDicts(config, customDicts);

    if (!output) {
        output = makeDefaultOfficeOutput(input, officeFormat, convertFilename, cc, config, punct);
        console.error(`Output file not specified. Using: ${output}`);
    } else {
        output = applyOutputExtension(output, officeFormat);
    }

    const inputBytes = fs.readFileSync(input);

    const outputBytes = cc.convertOfficeBytes(
        inputBytes,
        officeFormat,
        punct,
        keepFont
    );

    fs.writeFileSync(output, outputBytes);

    console.error(`Conversion completed (${cc.getConfig()}, ${officeFormat}): ${input} -> ${output}`);
}

async function main() {
    const args = process.argv.slice(2);

    if (args.length === 0 || hasFlag(args, "-h", "--help")) {
        printHelp();
        return;
    }

    const command = args[0];

    switch (command) {
        case "convert":
            if (hasFlag(args.slice(1), "-h", "--help")) {
                printHelp();
                return;
            }
            await runConvert(args.slice(1));
            break;

        case "office":
            if (hasFlag(args.slice(1), "-h", "--help")) {
                printHelp();
                return;
            }
            await runOffice(args.slice(1));
            break;

        default:
            console.error(`Unknown command: ${command}`);
            printHelp();
            process.exit(1);
    }
}

main().catch(err => {
    console.error(`Error: ${err && err.message ? err.message : err}`);
    process.exitCode = 1;
});
