/* tslint:disable */
/* eslint-disable */

export class ThrongletsWeb {
    free(): void;
    [Symbol.dispose](): void;
    camera_x(): number;
    camera_y(): number;
    drop_food(): void;
    eggs(): number;
    faded(): number;
    ideas(): number;
    move_cursor(dx: number, dy: number): void;
    constructor(seed: number, start_pop: number);
    next_theme(): void;
    place_egg(): void;
    population(): number;
    render_rgba(width: number, height: number): Uint8Array;
    seed_idea(): void;
    set_cursor(x: number, y: number): void;
    step(ticks: number): void;
    theme_name(): string;
    tick(): bigint;
    view_height(): number;
    view_width(): number;
    world_height(): number;
    world_width(): number;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_throngletsweb_free: (a: number, b: number) => void;
    readonly throngletsweb_camera_x: (a: number) => number;
    readonly throngletsweb_camera_y: (a: number) => number;
    readonly throngletsweb_drop_food: (a: number) => void;
    readonly throngletsweb_eggs: (a: number) => number;
    readonly throngletsweb_faded: (a: number) => number;
    readonly throngletsweb_ideas: (a: number) => number;
    readonly throngletsweb_move_cursor: (a: number, b: number, c: number) => void;
    readonly throngletsweb_new: (a: number, b: number) => number;
    readonly throngletsweb_next_theme: (a: number) => void;
    readonly throngletsweb_place_egg: (a: number) => void;
    readonly throngletsweb_population: (a: number) => number;
    readonly throngletsweb_render_rgba: (a: number, b: number, c: number) => [number, number];
    readonly throngletsweb_seed_idea: (a: number) => void;
    readonly throngletsweb_set_cursor: (a: number, b: number, c: number) => void;
    readonly throngletsweb_step: (a: number, b: number) => void;
    readonly throngletsweb_theme_name: (a: number) => [number, number];
    readonly throngletsweb_tick: (a: number) => bigint;
    readonly throngletsweb_view_height: (a: number) => number;
    readonly throngletsweb_view_width: (a: number) => number;
    readonly throngletsweb_world_height: (a: number) => number;
    readonly throngletsweb_world_width: (a: number) => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
