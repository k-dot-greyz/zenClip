import { describe, expect, it } from 'vitest';
import { pngHeader } from './testPng';
import { wrapWasmModule } from './wasm';
import { ZenClipEngineError } from './ZenClipEngine';

describe('wasm-bindgen wrapper', () => {
  it('maps a wasm sanitize payload onto SanitizeResult', async () => {
    const png = pngHeader(8, 4);
    const engine = wrapWasmModule({
      sanitize: (input) => ({
        png: input,
        sha256: 'b'.repeat(64),
        width: 8,
        height: 4,
        removedChunks: ['tEXt', 'iTXt'],
      }),
      sha256: () => 'c'.repeat(64),
      version: () => '0.1.0-wasm',
    });

    const result = await engine.sanitize(png);
    expect(result.width).toBe(8);
    expect(result.height).toBe(4);
    expect(result.removedChunks).toEqual(['tEXt', 'iTXt']);
    expect(result.sha256).toBe('b'.repeat(64));
    expect(await engine.sha256(png)).toBe('c'.repeat(64));
    expect(engine.version()).toBe('0.1.0-wasm');
  });

  it('rewrites wasm throws that carry kind', async () => {
    const engine = wrapWasmModule({
      sanitize: () => {
        const error = new Error('cap') as Error & { kind: 'too_large' };
        error.kind = 'too_large';
        throw error;
      },
    });

    try {
      await engine.sanitize(pngHeader(1, 1));
      expect.unreachable();
    } catch (error) {
      expect(error).toBeInstanceOf(ZenClipEngineError);
      expect((error as ZenClipEngineError).kind).toBe('too_large');
    }
  });
});
