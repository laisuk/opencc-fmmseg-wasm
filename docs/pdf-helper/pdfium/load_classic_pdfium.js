const runtimes = new Map();

/** Load the vendored classic Emscripten script in an isolated browser realm. */
export function loadClassicPdfium(scriptUrl, wasmUrl) {
    const script = new URL(scriptUrl, document.baseURI);
    const wasm = new URL(wasmUrl ?? 'pdfium.wasm', wasmUrl == null ? script : document.baseURI);
    const key = JSON.stringify([script.href, wasm.href]);
    if (runtimes.has(key)) return runtimes.get(key);

    const promise = new Promise((resolve, reject) => {
        const frame = document.createElement('iframe');
        frame.hidden = true;
        frame.title = 'PDFium runtime';
        let ready = false;

        function fail(error) {
            if (ready) return;
            frame.remove();
            reject(error);
        }

        try {
            document.body.append(frame);
            const context = frame.contentWindow;
            const element = context.document.createElement('script');
            const config = {
                locateFile: name => name === 'pdfium.wasm' ? wasm.href : new URL(name, script).href,
                onRuntimeInitialized: () => {
                    ready = true;
                    resolve(config);
                },
                onAbort: reason => fail(new Error(`PDFium initialization failed: ${reason}`))
            };
            context.Module = config;
            // An evaluation error can occur before Emscripten installs its abort hook.
            context.addEventListener('error', event => {
                fail(new Error(`PDFium initialization failed: ${event.message}`));
            });
            element.src = script.href;
            element.onerror = () => fail(new Error(`Could not load PDFium script: ${script.href}`));
            context.document.head.append(element);
        } catch (error) {
            fail(error);
        }
    }).catch(error => {
        runtimes.delete(key);
        throw error;
    });
    runtimes.set(key, promise);
    return promise;
}
