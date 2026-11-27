import {createPdfiumLoader} from "./pdfium/pdfium_bridge.js";
import {loadClassicPdfium} from "./pdfium/load_classic_pdfium.js";

import {attachHeadingValidation} from "./pdfiumHeadingValidation.js";
import {loadDocumentExtractor as loadRust} from "./load.js";

const load = createPdfiumLoader({
    loadPdfium: () => loadClassicPdfium(
        new URL("./pdfium/pdfium.js", import.meta.url)),
    loadRust
});

async function loadWithCancellation(signal, start) {
    signal.throwIfAborted();
    let onAbort;
    try {
        const cancelled = new Promise((_, reject) => {
            onAbort = () => reject(signal.reason);
            signal.addEventListener("abort", onAbort, {once: true});
        });
        // Keep shared initialization alive for the next selection.
        return await Promise.race([start(), cancelled]);
    } finally {
        signal.removeEventListener("abort", onAbort);
    }
}

async function readPdf(file, signal) {
    signal.throwIfAborted();
    const reader = new FileReader();
    const abort = () => reader.abort();
    signal.addEventListener("abort", abort, {once: true});
    try {
        const buffer = await new Promise((resolve, reject) => {
            reader.onload = () => resolve(reader.result);
            reader.onerror = () => reject(reader.error);
            reader.onabort = () => reject(signal.reason);
            reader.readAsArrayBuffer(file);
        });
        return new Uint8Array(buffer);
    } finally {
        signal.removeEventListener("abort", abort);
    }
}

function createProgressPanel(id) {
    const panel = document.getElementById(id);
    const progress = panel.querySelector("progress");
    const text = panel.querySelector('[role="status"]');
    const cancel = panel.querySelector("button");
    return {
        cancel,
        stage(message, cancellable = true) {
            progress.removeAttribute("value");
            text.textContent = message;
            cancel.disabled = !cancellable;
            panel.hidden = false;
        },
        page(page, total) {
            progress.max = total;
            progress.value = page;
            text.textContent = `Page ${page} of ${total} (${Math.floor(page * 100 / total)}%)`;
        },
        hide() {
            panel.hidden = true;
        }
    };
}

async function extractPdf(file, options, validation, progress, signal) {
    if (file.size === 0) throw new Error("The PDF file is empty.");
    if (file.size > 512 * 1024 * 1024) throw new Error("PDF files are limited to 512 MiB.");

    progress.stage("Initializing PDF extraction…");
    if (options.headingPattern) {
        await loadWithCancellation(signal, () => validation.validate(options.headingPattern));
    }
    const {bridge, rust} = await loadWithCancellation(signal, load);

    progress.stage("Reading PDF…");
    const bytes = await readPdf(file, signal);
    const raw = await bridge.extract(bytes, {
        pageHeaders: options.headers,
        ignoreUntrustedPdfText: options.ignoreUntrusted,
        signal,
        onPage: progress.page
    });
    signal.throwIfAborted();
    if (!options.reflow) return raw;

    await showSynchronousStage(progress, "Reflowing PDF text…", signal);
    return options.headingPattern
        ? rust.reflow_with_heading_regex(raw, options.headers, options.compact, options.headingPattern)
        : rust.reflow(raw, options.headers, options.compact);
}

async function showSynchronousStage(progress, message, signal) {
    signal.throwIfAborted();
    progress.stage(message, false);
    // Allow the browser to draw the stage before synchronous WASM work.
    await new Promise(resolve => setTimeout(resolve, 50));
    signal.throwIfAborted();
}

export function createPdfExtraction() {
    const openProgress = createProgressPanel("pdfOpenProgress");
    const dropProgress = createProgressPanel("pdfDropProgress");
    const headers = document.getElementById("pdf-page-headers");
    const ignoreUntrusted = document.getElementById("pdf-ignore-untrusted");
    const reflow = document.getElementById("pdf-reflow");
    const compact = document.getElementById("pdf-compact");
    const heading = document.getElementById("pdf-heading-regex");
    const validation = attachHeadingValidation(
        heading, reflow, document.getElementById("pdf-heading-feedback"), loadRust
    );

    function updateOptions() {
        compact.disabled = heading.disabled = !reflow.checked;
    }

    reflow.addEventListener("change", updateOptions);
    updateOptions();

    let active = null;
    for (const {cancel} of [openProgress, dropProgress]) {
        cancel.addEventListener("click", () => {
            active?.abort();
            cancel.disabled = true;
        });
    }

    async function process(file, isDrop, complete) {
        if (active) throw new Error("Another PDF operation is already running.");
        const controller = new AbortController();
        active = controller;
        const {signal} = controller;
        const progress = isDrop ? dropProgress : openProgress;
        try {
            const options = {
                headers: headers.checked,
                ignoreUntrusted: ignoreUntrusted.checked,
                reflow: reflow.checked,
                compact: reflow.checked && compact.checked,
                headingPattern: validation.pattern()
            };
            const text = await extractPdf(file, options, validation, progress, signal);
            signal.throwIfAborted();
            if (isDrop) await showSynchronousStage(progress, "Converting with OpenCC…", signal);
            complete(text);
        } finally {
            progress.hide();
            active = null;
        }
    }

    return {
        get busy() {
            return active !== null;
        },
        process
    };
}
