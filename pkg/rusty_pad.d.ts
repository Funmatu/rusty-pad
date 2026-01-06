/* tslint:disable */
/* eslint-disable */

export class RustySession {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  static new(): RustySession;
  evaluate(expression: string): string;
}

export class TextStats {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  chars: number;
  words: number;
  lines: number;
}

export function wasm_text_stats(text: string): TextStats;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_get_textstats_chars: (a: number) => number;
  readonly __wbg_get_textstats_lines: (a: number) => number;
  readonly __wbg_get_textstats_words: (a: number) => number;
  readonly __wbg_rustysession_free: (a: number, b: number) => void;
  readonly __wbg_set_textstats_chars: (a: number, b: number) => void;
  readonly __wbg_set_textstats_lines: (a: number, b: number) => void;
  readonly __wbg_set_textstats_words: (a: number, b: number) => void;
  readonly __wbg_textstats_free: (a: number, b: number) => void;
  readonly rustysession_evaluate: (a: number, b: number, c: number) => [number, number];
  readonly rustysession_new: () => number;
  readonly wasm_text_stats: (a: number, b: number) => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
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
