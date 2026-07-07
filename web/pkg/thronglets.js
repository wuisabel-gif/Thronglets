/* @ts-self-types="./thronglets.d.ts" */

export class ThrongletsWeb {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ThrongletsWebFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_throngletsweb_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    camera_x() {
        const ret = wasm.throngletsweb_camera_x(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    camera_y() {
        const ret = wasm.throngletsweb_camera_y(this.__wbg_ptr);
        return ret >>> 0;
    }
    drop_food() {
        wasm.throngletsweb_drop_food(this.__wbg_ptr);
    }
    /**
     * @returns {number}
     */
    eggs() {
        const ret = wasm.throngletsweb_eggs(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    faded() {
        const ret = wasm.throngletsweb_faded(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    food_units() {
        const ret = wasm.throngletsweb_food_units(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    ideas() {
        const ret = wasm.throngletsweb_ideas(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} dx
     * @param {number} dy
     */
    move_cursor(dx, dy) {
        wasm.throngletsweb_move_cursor(this.__wbg_ptr, dx, dy);
    }
    /**
     * @param {number} seed
     * @param {number} start_pop
     */
    constructor(seed, start_pop) {
        const ret = wasm.throngletsweb_new(seed, start_pop);
        this.__wbg_ptr = ret;
        ThrongletsWebFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    next_theme() {
        wasm.throngletsweb_next_theme(this.__wbg_ptr);
    }
    place_egg() {
        wasm.throngletsweb_place_egg(this.__wbg_ptr);
    }
    /**
     * @returns {number}
     */
    population() {
        const ret = wasm.throngletsweb_population(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} width
     * @param {number} height
     * @returns {Uint8Array}
     */
    render_rgba(width, height) {
        const ret = wasm.throngletsweb_render_rgba(this.__wbg_ptr, width, height);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @returns {number}
     */
    scarcity() {
        const ret = wasm.throngletsweb_scarcity(this.__wbg_ptr);
        return ret;
    }
    seed_idea() {
        wasm.throngletsweb_seed_idea(this.__wbg_ptr);
    }
    /**
     * @param {number} x
     * @param {number} y
     */
    set_cursor(x, y) {
        wasm.throngletsweb_set_cursor(this.__wbg_ptr, x, y);
    }
    /**
     * @param {number} ticks
     */
    step(ticks) {
        wasm.throngletsweb_step(this.__wbg_ptr, ticks);
    }
    /**
     * @returns {string}
     */
    theme_name() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.throngletsweb_theme_name(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {bigint}
     */
    tick() {
        const ret = wasm.throngletsweb_tick(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * @returns {number}
     */
    view_height() {
        const ret = wasm.throngletsweb_view_height(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    view_width() {
        const ret = wasm.throngletsweb_view_width(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    world_height() {
        const ret = wasm.throngletsweb_world_height(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    world_width() {
        const ret = wasm.throngletsweb_world_width(this.__wbg_ptr);
        return ret >>> 0;
    }
}
if (Symbol.dispose) ThrongletsWeb.prototype[Symbol.dispose] = ThrongletsWeb.prototype.free;
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_throw_344f42d3211c4765: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./thronglets_bg.js": import0,
    };
}

const ThrongletsWebFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_throngletsweb_free(ptr, 1));

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

function getStringFromWasm0(ptr, len) {
    return decodeText(ptr >>> 0, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

let wasmModule, wasmInstance, wasm;
function __wbg_finalize_init(instance, module) {
    wasmInstance = instance;
    wasm = instance.exports;
    wasmModule = module;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('thronglets_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
