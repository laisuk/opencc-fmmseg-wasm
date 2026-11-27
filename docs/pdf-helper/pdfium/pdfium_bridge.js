const initialized = new WeakSet();
const active = new WeakSet();
const decoder = new TextDecoder('utf-16le', {ignoreBOM: true});
const textObjectType = 1;
const maxObjectBytes = 20_000_000;

const openErrors = [
    'no error reported', 'unknown error', 'file access failed', 'invalid PDF format',
    'password required or incorrect', 'unsupported security handler', 'page processing error'
];

export class PdfiumBridge {
    constructor(module, normalizePage, filterOverlay) {
        this.m = module;
        this.normalizePage = normalizePage;
        this.filterOverlay = filterOverlay;
        if (!initialized.has(module)) {
            module._FPDF_InitLibrary();
            initialized.add(module);
        }
    }

    async extract(bytes, {
        pageHeaders = false, ignoreUntrustedPdfText = false, onPage = () => {
        }, signal
    } = {}) {
        const m = this.m;
        if (active.has(m)) throw new Error('Extraction in progress');
        if (!(bytes instanceof Uint8Array) || !bytes.length || bytes.length > 0x7fffffff) {
            throw new Error('Invalid PDF length');
        }
        if (ignoreUntrustedPdfText && typeof this.filterOverlay !== 'function') {
            throw new Error('The Rust WASM overlay filter is unavailable');
        }

        active.add(m);
        let data = 0, doc = 0, buf = 0, capacity = 0, bounds = 0;
        let batch = new Uint16Array();
        const buffer = size => {
            if (size > 0x7fffffff) throw new Error('Page text is too large');
            if (size > capacity) {
                const replacement = m._malloc(size) >>> 0;
                if (!replacement) throw new Error('Text allocation failed');
                if (buf) m._free(buf);
                buf = replacement;
                capacity = size;
            }
            return buf;
        };

        const wholePage = tp => {
            const count = m._FPDFText_CountChars(tp);
            if (count < 0) throw new Error('PDFium could not count page characters');
            if (count === 0) return this.normalizePage('');
            const ptr = buffer((count + 1) * 2);
            const units = m._FPDFText_GetText(tp, 0, count, ptr);
            if (units < 0 || units > count + 1) throw new Error('Invalid PDFium page text length');
            const start = ptr >>> 1;
            const end = start + units;
            const terminated = units > 0 && m.HEAPU16[end - 1] === 0;
            return this.normalizePage(decoder.decode(m.HEAPU16.subarray(start, end - Number(terminated))));
        };

        const filteredPage = (page, tp) => {
            const count = m._FPDFPage_CountObjects(page);
            if (count < 0) throw new Error('PDFium could not enumerate page objects');
            const ends = [];
            const midpoints = [];
            let used = 0;
            for (let i = 0; i < count; i++) {
                const obj = m._FPDFPage_GetObject(page, i);
                if (!obj || m._FPDFPageObj_GetType(obj) !== textObjectType) continue;

                const bytes = m._FPDFTextObj_GetText(obj, tp, 0, 0) >>> 0;
                // Match the native extractor's empty-object and size limits.
                if (bytes <= 2 || bytes > maxObjectBytes) continue;
                if (bytes % 2) throw new Error('Invalid PDFium text object size');
                const ptr = buffer(bytes);
                const written = m._FPDFTextObj_GetText(obj, tp, ptr, bytes) >>> 0;
                if (written === 0) continue;
                if (written > bytes || written % 2) throw new Error('Invalid PDFium text object length');
                const start = ptr >>> 1;
                const end = start + written / 2;
                const units = m.HEAPU16.subarray(start, end - Number(m.HEAPU16[end - 1] === 0));
                if (!units.length) continue;
                if (used + units.length > batch.length) {
                    const grown = new Uint16Array(Math.max(1024, batch.length * 2, used + units.length));
                    grown.set(batch.subarray(0, used));
                    batch = grown;
                }
                batch.set(units, used);
                used += units.length;

                if (!bounds) {
                    bounds = m._malloc(16) >>> 0;
                    if (!bounds) throw new Error('Object bounds allocation failed');
                }
                let midpoint = 0;
                if (m._FPDFPageObj_GetBounds(obj, bounds, bounds + 4, bounds + 8, bounds + 12)) {
                    const base = bounds >>> 2;
                    midpoint = Math.fround(m.HEAPF32[base + 1] + m.HEAPF32[base + 3]) * 0.5;
                }
                ends.push(used);
                midpoints.push(midpoint);
            }
            return this.filterOverlay(
                batch.subarray(0, used), Uint32Array.from(ends), Float32Array.from(midpoints)
            );
        };

        try {
            signal?.throwIfAborted();
            data = m._malloc(bytes.length) >>> 0;
            if (!data) throw new Error('PDF allocation failed');
            m.HEAPU8.set(bytes, data);
            doc = m._FPDF_LoadMemDocument(data, bytes.length, 0);
            if (!doc) {
                const code = m._FPDF_GetLastError();
                throw new Error(`PDFium open failed (code ${code}: ${openErrors[code] ?? 'unrecognized error'})`);
            }
            const total = m._FPDF_GetPageCount(doc);
            if (total <= 0) {
                onPage(1, 1, '\n');
                signal?.throwIfAborted();
                return '\n';
            }

            const chunks = [];
            for (let i = 0; i < total; i++) {
                signal?.throwIfAborted();
                let page = 0, tp = 0, text = '\n';
                try {
                    page = m._FPDF_LoadPage(doc, i);
                    if (page) tp = m._FPDFText_LoadPage(page);
                    if (tp) {
                        text = ignoreUntrustedPdfText ? filteredPage(page, tp) : wholePage(tp);
                        if (pageHeaders) text = `=== [Page ${i + 1}/${total}] ===\n` + text;
                    }
                } catch (error) {
                    throw new Error(`PDFium extraction failed on page ${i + 1}: ${error.message}`, {cause: error});
                } finally {
                    if (tp) m._FPDFText_ClosePage(tp);
                    if (page) m._FPDF_ClosePage(page);
                }
                chunks.push(text);
                onPage(i + 1, total, text);
                await new Promise(resolve => setTimeout(resolve, 0));
            }
            signal?.throwIfAborted();
            return chunks.join('');
        } finally {
            try {
                if (bounds) m._free(bounds);
                if (buf) m._free(buf);
                if (doc) m._FPDF_CloseDocument(doc);
                if (data) m._free(data);
            } finally {
                active.delete(m);
            }
        }
    }
}

/** Build a lazy, retryable loader from application-provided runtime loaders. */
export function createPdfiumLoader({loadPdfium, loadRust}) {
    let pdfiumPromise, rustPromise, ready;
    return () => {
        if (!ready) {
            pdfiumPromise ??= Promise.resolve().then(loadPdfium).catch(error => {
                pdfiumPromise = undefined;
                throw error;
            });
            rustPromise ??= Promise.resolve().then(loadRust).catch(error => {
                rustPromise = undefined;
                throw error;
            });
            ready = Promise.all([pdfiumPromise, rustPromise]).then(([module, rust]) => ({
                bridge: new PdfiumBridge(module, rust.normalize_page, rust.filter_untrusted_overlay),
                rust
            })).catch(error => {
                ready = undefined;
                throw error;
            });
        }
        return ready;
    };
}
