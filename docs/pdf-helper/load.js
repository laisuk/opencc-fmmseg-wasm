let initialization;
const moduleUrl = new URL("./pdfium_helper_wasm.js", import.meta.url);
let failedImports = 0;

export function loadDocumentExtractor() {
    initialization ??= import(moduleUrl.href)
        .catch(error => {
            // Browsers cache failed module loads by URL.
            moduleUrl.searchParams.set("retry", String(++failedImports));
            throw error;
        })
        .then(async helper => {
            await helper.default();
            return helper;
        })
        .catch(error => {
            initialization = undefined;
            throw error;
        });

    return initialization;
}
