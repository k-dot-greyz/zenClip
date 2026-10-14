import { createMockEngine } from './mock';
import type { LoadedEngine } from './ZenClipEngine';
import { tryLoadWasmEngine } from './wasm';

export async function loadEngine(): Promise<LoadedEngine> {
  const wasm = await tryLoadWasmEngine();
  if (wasm) {
    return { source: 'wasm', engine: wasm };
  }
  return { source: 'mock', engine: createMockEngine() };
}
