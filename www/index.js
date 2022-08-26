import init, {start} from "../pkg/wasm_wfc.js";
await init("../pkg/wasm_wfc_bg.wasm");
start()