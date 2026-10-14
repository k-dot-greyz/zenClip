import { describe, expect, it } from 'vitest';
import {
  asEngineError,
  assertSanitizeResult,
  isEngineErrorKind,
  SHA256_HEX_RE,
  ZenClipEngineError,
} from './ZenClipEngine';

describe('ZenClipEngine contract', () => {
  it('accepts the three error kinds and no others', () => {
    expect(isEngineErrorKind('decode')).toBe(true);
    expect(isEngineErrorKind('too_large')).toBe(true);
    expect(isEngineErrorKind('unsupported')).toBe(true);
    expect(isEngineErrorKind('boom')).toBe(false);
  });

  it('wraps thrown Errors that already have kind', () => {
    const thrown = new Error('nope') as Error & { kind: 'decode' };
    thrown.kind = 'decode';
    const wrapped = asEngineError(thrown);
    expect(wrapped).toBeInstanceOf(ZenClipEngineError);
    expect(wrapped.kind).toBe('decode');
    expect(wrapped.message).toBe('nope');
  });

  it('validates SanitizeResult sha256 as 64 lowercase hex', () => {
    const png = new Uint8Array([1, 2, 3]);
    const ok = assertSanitizeResult({
      png,
      sha256: 'a'.repeat(64),
      width: 1,
      height: 1,
      removedChunks: ['tEXt'],
    });
    expect(ok.sha256).toMatch(SHA256_HEX_RE);
    expect(ok.removedChunks).toEqual(['tEXt']);

    expect(() =>
      assertSanitizeResult({
        png,
        sha256: 'ABCDEF'.toLowerCase().padEnd(64, 'G'),
        width: 1,
        height: 1,
        removedChunks: [],
      }),
    ).toThrow(ZenClipEngineError);
  });
});
