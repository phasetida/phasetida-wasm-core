/* @ts-self-types="./phasetida_wasm_core.d.ts" */

import * as wasm from "./phasetida_wasm_core_bg.wasm";
import { __wbg_set_wasm } from "./phasetida_wasm_core_bg.js";
__wbg_set_wasm(wasm);
wasm.__wbindgen_start();
export {
    load_image_offset, load_level, pre_draw, reset_note_state
} from "./phasetida_wasm_core_bg.js";
