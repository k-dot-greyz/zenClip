import {
  asEngineError,
  assertSanitizeResult,
  type SanitizeResult,
  type ZenClipEngine,
  ZenClipEngineError,
} from './ZenClipEngine';

type WasmBindgenModule = {
  default?: (moduleOrPath?: unknown) => Promise<unknown>;
  sanitize?: (png: Uint8Array) => unknown;
  sha256?: (bytes: Uint8Array) => unknown;
  version?: () => unknown;
};

/**
 * Loader for a wasm-pack `--target web` pkg named `zenclip-wasm`.
 *
 * Build (when the crate exists):
 *   wasm-pack build crates/zenclip-wasm --target web --release
 *
 * Plug in via npm (`zenclip-wasm`), a git/file path, or PUBLIC_ZENCLIP_WASM.
 * Missing pkg → caller falls back to the labelled MOCK.
 */
export async function tryLoadWasmEngine(): Promise<ZenClipEngine | null> {
  try {
    const specifier = wasmSpecifier();
    const imported: WasmBindgenModule = await import(/* @vite-ignore */ specifier);
    if (typeof imported.default === 'function') {
      await imported.default();
    }
    if (typeof imported.sanitize !== 'function') {
      return null;
    }
    return wrapWasmModule(imported);
  } catch {
    return null;
  }
}

export function wrapWasmModule(mod: WasmBindgenModule): ZenClipEngine {
  const sanitizeFn = mod.sanitize;
  if (typeof sanitizeFn !== 'function') {
    throw new ZenClipEngineError('unsupported', 'zenclip-wasm is missing sanitize()');
  }

  return {
    async sanitize(png: Uint8Array): Promise<SanitizeResult> {
      try {
        const raw = sanitizeFn(png);
        const resolved = isThenable(raw) ? await raw : raw;
        return assertSanitizeResult(resolved);
      } catch (error) {
        throw asEngineError(error);
      }
    },
    async sha256(bytes: Uint8Array): Promise<string> {
      if (typeof mod.sha256 !== 'function') {
        throw new ZenClipEngineError('unsupported', 'zenclip-wasm is missing sha256()');
      }
      try {
        const raw = mod.sha256(bytes);
        const resolved = isThenable(raw) ? await raw : raw;
        if (typeof resolved !== 'string') {
          throw new ZenClipEngineError('decode', 'zenclip-wasm sha256() did not return a string');
        }
        return resolved;
      } catch (error) {
        throw asEngineError(error);
      }
    },
    version(): string {
      if (typeof mod.version !== 'function') {
        return 'wasm-unknown';
      }
      const value = mod.version();
      return typeof value === 'string' ? value : 'wasm-unknown';
    },
  };
}

function wasmSpecifier(): string {
  const fromEnv = import.meta.env.PUBLIC_ZENCLIP_WASM;
  if (fromEnv && fromEnv.trim().length > 0) {
    return fromEnv.trim();
  }
  return 'zenclip-wasm';
}

function isThenable(value: unknown): value is Promise<unknown> {
  return typeof value === 'object' && value !== null && 'then' in value;
}
