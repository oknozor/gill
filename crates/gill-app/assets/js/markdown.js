import {default as init, render_markdown} from './gill_web_markdown.js'

async function run() {
    await init();
}

run()
    .then(() => {
        window.render_markdown = render_markdown;
        window.dispatchEvent(new CustomEvent("WasmLoaded", {}));
    })
    .catch(err => console.error("failed to init wasm module: " + err));