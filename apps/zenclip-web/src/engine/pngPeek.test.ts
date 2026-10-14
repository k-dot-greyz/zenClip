import { describe, expect, it } from 'vitest';
import { assertNotTooLarge, hasPngSignature, peekPngSize } from './pngPeek';
import { pngHeader } from './testPng';
import { ZenClipEngineError } from './ZenClipEngine';

describe('pngPeek', () => {
  it('reads IHDR width and height', () => {
    const png = pngHeader(1920, 1080);
    expect(hasPngSignature(png)).toBe(true);
    expect(peekPngSize(png)).toEqual({ width: 1920, height: 1080 });
  });

  it('throws unsupported when the signature is not PNG', () => {
    const jpegLike = new Uint8Array([0xff, 0xd8, 0xff, 0xe0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    try {
      peekPngSize(jpegLike);
      expect.unreachable();
    } catch (error) {
      expect(error).toBeInstanceOf(ZenClipEngineError);
      expect((error as ZenClipEngineError).kind).toBe('unsupported');
    }
  });

  it('throws decode when IHDR is truncated', () => {
    const truncated = pngHeader(8, 8).slice(0, 20);
    try {
      peekPngSize(truncated);
      expect.unreachable();
    } catch (error) {
      expect(error).toBeInstanceOf(ZenClipEngineError);
      expect((error as ZenClipEngineError).kind).toBe('decode');
    }
  });

  it('throws too_large when the buffer exceeds the cap', () => {
    const png = pngHeader(1, 1);
    try {
      assertNotTooLarge(png, 4);
      expect.unreachable();
    } catch (error) {
      expect(error).toBeInstanceOf(ZenClipEngineError);
      expect((error as ZenClipEngineError).kind).toBe('too_large');
    }
  });
});
