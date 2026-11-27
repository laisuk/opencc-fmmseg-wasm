export function attachHeadingValidation(heading, reflow, feedback, loadRust) {
    let timer;
    let version = 0;

    // Unicode White_Space matches Rust str::trim(), including U+0085.
    const pattern = () => reflow.checked && !/^\p{White_Space}*$/u.test(heading.value)
        ? heading.value
        : "";

    async function validate(value) {
        if (value) {
            const rust = await loadRust();
            rust.validate_heading_regex(value);
        }
    }

    function reset() {
        clearTimeout(timer);
        version++;
        feedback.hidden = true;
        feedback.textContent = "";
        delete feedback.dataset.state;
        heading.removeAttribute("aria-invalid");
    }

    function schedule() {
        reset();
        const value = pattern();
        if (!value) return;
        const scheduledVersion = version;
        timer = setTimeout(async () => {
            let state = "valid";
            let message = "✓ Valid heading regex";
            try {
                await validate(value);
            } catch (error) {
                state = "invalid";
                message = "✗ " + (error?.message ?? String(error));
            }

            if (scheduledVersion !== version || value !== pattern()) return;
            feedback.dataset.state = state;
            feedback.textContent = message;
            feedback.hidden = false;
            if (state === "invalid") heading.setAttribute("aria-invalid", "true");
        }, 300);
    }

    heading.addEventListener("input", schedule);
    reflow.addEventListener("change", schedule);
    schedule();

    return {
        pattern,
        validate,
        dispose() {
            reset();
            heading.removeEventListener("input", schedule);
            reflow.removeEventListener("change", schedule);
        }
    };
}
