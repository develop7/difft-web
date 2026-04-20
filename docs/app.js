import {
  WASI,
  File,
  OpenFile,
  ConsoleStdout,
  PreopenDirectory,
} from "https://cdn.jsdelivr.net/npm/@bjorn3/browser_wasi_shim@0.4.2/+esm";

const root = document.getElementById("demo-root");

async function renderDemoWithWasi() {
  const stdoutChunks = [];

  const wasi = new WASI(["demo_wasi.wasm"], [], [
    new OpenFile(new File([])),
    ConsoleStdout.lineBuffered((line) => stdoutChunks.push(line)),
    ConsoleStdout.lineBuffered((line) => console.error(`[stderr] ${line}`)),
    new PreopenDirectory(".", []),
  ]);

  const wasm = await WebAssembly.compileStreaming(fetch("./demo_wasi.wasm"));
  const instance = await WebAssembly.instantiate(wasm, {
    wasi_snapshot_preview1: wasi.wasiImport,
  });

  wasi.start(instance);
  root.innerHTML = stdoutChunks.join("\n");
}

renderDemoWithWasi().catch((error) => {
  console.error(error);
  root.textContent = `Failed to render WASI demo: ${error}`;
});
