import init, {start} from "../pkg/wasm_wfc.js";
await init("../pkg/wasm_game_of_life_bg.wasm");
start()