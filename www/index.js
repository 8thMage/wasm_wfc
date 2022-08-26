import * as wasm_wfc from "../pkg/wasm_wfc.js";
import init from "../pkg/wasm_wfc.js";
await init("../pkg/wasm_game_of_life_bg.wasm");
wasm_wfc.start()