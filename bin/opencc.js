#!/usr/bin/env node

import fs from "fs";
import path from "path";
import process from "process";

import init, {
    OpenccWasm,
    convert_office_bytes
} from "../pkg/opencc_fmmseg_wasm.js";

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

    const wasmPath = new URL("../pkg/opencc_fmmseg_wasm_bg.wasm", import.meta.url);
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
  opencc.js convert [options]
  opencc.js office  [options]

Commands:
  convert                     Convert plain text
  office                      Convert Office / EPUB documents

Convert options:
  -i, --input <file>          Input text file; stdin if omitted
  -o, --output <file>         Output text file; stdout if omitted
  -c, --config <conversion>   Conversion config (default: s2t)
  -p, --punct                 Enable punctuation conversion
  --in-enc <encoding>         Input encoding (default: utf8)
  --out-enc <encoding>        Output encoding (default: utf8)

Office options:
  -i, --input <file>          Input Office / EPUB file
  -o, --output <file>         Output file
  -c, --config <conversion>   Conversion config (default: s2t)
  -p, --punct                 Enable punctuation conversion
  --format <format>           docx | xlsx | pptx | odt | ods | odp | epub
  --auto-ext                  Append extension to output if missing
  --keep-font                 Preserve font-family information (default)
  --no-keep-font              Do not preserve font-family information

General options:
  -h, --help                  Show help

Examples:
  node ./bin/opencc.js convert -i a.txt -o b.txt -c s2t
  node ./bin/opencc.js convert -i a.txt -o b.txt -c s2tw -p
  cat a.txt | node ./bin/opencc.js convert -c t2s

  node ./bin/opencc.js office -i a.docx -o b.docx -c s2t -p
  node ./bin/opencc.js office -i a.epub -c s2tw --auto-ext
`);
}

function getArg(args, shortName, longName, defaultValue = null) {
    const candidates = [];

    if (shortName) {
        candidates.push(shortName);
    }
    if (longName) {
        candidates.push(longName);
    }

    for (const name of candidates) {
        const index = args.indexOf(name);

        if (index !== -1 && index + 1 < args.length) {
            return args[index + 1];
        }
    }

    return defaultValue;
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

    if (!fs.existsSync(filePath)) {
        console.error(`Error: Input file not found: ${filePath}`);
        process.exit(1);
    }

    return fs.readFileSync(filePath, encoding);
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

function makeDefaultOfficeOutput(inputFile, officeFormat, autoExt) {
    const parsed = path.parse(inputFile);
    const ext = autoExt && OFFICE_FORMATS.has(officeFormat)
        ? `.${officeFormat}`
        : parsed.ext;

    return path.join(
        parsed.dir || process.cwd(),
        `${parsed.name}_converted${ext}`
    );
}

function applyAutoExt(outputFile, officeFormat, autoExt) {
    if (!autoExt || path.extname(outputFile)) {
        return outputFile;
    }

    if (!OFFICE_FORMATS.has(officeFormat)) {
        return outputFile;
    }

    return `${outputFile}.${officeFormat}`;
}

async function runConvert(args) {
    const input = getArg(args, "-i", "--input");
    const output = getArg(args, "-o", "--output");
    const config = getArg(args, "-c", "--config", "s2t");
    const inEnc = getArg(args, null, "--in-enc", "utf8");
    const outEnc = getArg(args, null, "--out-enc", "utf8");
    const punct = hasFlag(args, "-p", "--punct");

    await ensureWasmInitialized();

    const cc = new OpenccWasm(config);

    const inputText = readInputText(input, inEnc);
    const outputText = cc.convert(inputText, punct);

    writeOutputText(output, outputText, outEnc);

    const inFrom = input || "<stdin>";
    const outTo = output || "stdout";

    if (process.stderr.isTTY) {
        if (!output && outputText && !outputText.endsWith("\n")) {
            console.error();
        }

        console.error(`Conversion completed (${config}): ${inFrom} -> ${outTo}`);
    }
}

async function runOffice(args) {
    const input = getArg(args, "-i", "--input");
    let output = getArg(args, "-o", "--output");
    const config = getArg(args, "-c", "--config", "s2t");
    const explicitFormat = getArg(args, null, "--format");
    const punct = hasFlag(args, "-p", "--punct");
    const autoExt = hasFlag(args, null, "--auto-ext");
    const keepFont = !hasFlag(args, null, "--no-keep-font");

    if (!input) {
        throw new Error("Input file is missing.");
    }

    if (!fs.existsSync(input) || !fs.statSync(input).isFile()) {
        throw new Error(`Input file not found: ${input}`);
    }

    const officeFormat = inferOfficeFormat(input, explicitFormat);

    if (!output) {
        output = makeDefaultOfficeOutput(input, officeFormat, autoExt);
        console.error(`Output file not specified. Using: ${output}`);
    } else {
        output = applyAutoExt(output, officeFormat, autoExt);
    }

    await ensureWasmInitialized();

    const inputBytes = fs.readFileSync(input);

    const outputBytes = convert_office_bytes(
        inputBytes,
        officeFormat,
        config,
        punct,
        keepFont
    );

    fs.writeFileSync(output, outputBytes);

    console.error(`Conversion completed (${config}, ${officeFormat}): ${input} -> ${output}`);
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
    console.error(err && err.message ? err.message : err);
    process.exit(1);
});
