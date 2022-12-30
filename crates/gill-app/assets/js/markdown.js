import {default as init, render_markdown} from './gill_web_markdown.js'

async function run() {
    await init();
}

run().catch(err => console.error("failed to init wasm module: " + err));

let input = document.getElementById("description");

input.oninput = () => {
    document.getElementById("preview-pull-request").innerHTML = render_markdown(input.value, owner, repository);
};