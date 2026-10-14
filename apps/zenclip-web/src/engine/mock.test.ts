import { describe, expect, it } from 'vitest';
import { createMockEngine, MOCK_VERSION } from './mock';
import { sha256Hex } from './sha256';
import { pngHeader } from './testPng';
import { SHA256_HEX_RE, ZenClipEngineError } from './ZenClipEngine';

describe('mock engine', () => {
  it('implements the engine methods', () => {
    const engine = createMockEngine();
    expect(typeof engine.sanitize).toBe('function');
    expect(typeof engine.sha256).toBe('function');
    expect(typeof engine.version).toBe('function');
    expect(engine.version()).toBe(MOCK_VERSION);
  });

  it('passes PNG bytes through and hashes with WebCrypto', async () => {
    const engine = createMockEngine();
    const png = pngHeader(64, 32);
    const result = await engine.sanitize(png);
    const expected = await sha256Hex(png);

    expect(result.width).toBe(64);
    expect(result.height).toBe(32);
    expect(result.removedChunks).toEqual([]);
    expect(result.sha256).toMatch(SHA256_HEX_RE);
    expect(result.sha256).toBe(expected);
    expect(result.sha256).toBe(await engine.sha256(png));
    expect(result.png).toEqual(png);
    expect(result.png).not.toBe(png);
  });

  it('throws kind=too_large', async () => {
    const engine = createMockEngine(8);
    await expect(engine.sanitize(pngHeader(1, 1))).rejects.toMatchObject({
      kind: 'too_large',
    });
  });

  it('throws kind=unsupported for non-PNG', async () => {
    const engine = createMockEngine();
    const junk = new Uint8Array(33).fill(7);
    try {
      await engine.sanitize(junk);
      expect.unreachable();
    } catch (error) {
      expect(error).toBeInstanceOf(ZenClipEngineError);
      expect((error as ZenClipEngineError).kind).toBe('unsupported');
    }
  });

  it('throws kind=decode for a truncated PNG', async () => {
    const engine = createMockEngine();
    const truncated = pngHeader(10, 10).slice(0, 12);
    try {
      await engine.sanitize(truncated);
      expect.unreachable();
    } catch (error) {
      expect(error).toBeInstanceOf(ZenClipEngineError);
      expect((error as ZenClipEngineError).kind).toBe('decode');
    }
  });
});
