import { describe, expect, it } from 'vitest';
import {
  engineLabel,
  formatRemovedChunks,
  previewAlt,
  toErrorViewModel,
  toResultViewModel,
  userMessageForKind,
} from './result';

describe('UI logic', () => {
  it('labels MOCK vs wasm clearly', () => {
    expect(engineLabel('mock', 'mock-0.1.0')).toContain('MOCK (pass-through)');
    expect(engineLabel('wasm', '0.1.0')).toBe('zenclip-wasm · 0.1.0');
  });

  it('formats sanitize output for the result panel', () => {
    const view = toResultViewModel('wasm', '0.1.0', {
      png: new Uint8Array([1]),
      sha256: 'd'.repeat(64),
      width: 800,
      height: 600,
      removedChunks: ['tEXt', 'tIME'],
    });
    expect(view.dimensions).toBe('800 × 600');
    expect(view.sha256).toBe('d'.repeat(64));
    expect(view.removedChunksLabel).toBe('tEXt, tIME');
    expect(previewAlt(view.dimensions)).toBe('Captured PNG preview, 800 × 600');
  });

  it('explains empty removedChunks for MOCK vs wasm', () => {
    expect(formatRemovedChunks('mock', [])).toMatch(/MOCK passes the PNG through/i);
    expect(formatRemovedChunks('wasm', [])).toBe('None');
  });

  it('maps engine error kinds to copy', () => {
    expect(userMessageForKind('decode')).toMatch(/decode/i);
    expect(userMessageForKind('too_large')).toMatch(/max size/i);
    expect(userMessageForKind('unsupported')).toMatch(/supported PNG/i);

    const mapped = toErrorViewModel({ kind: 'decode', message: 'PNG is truncated before IHDR' });
    expect(mapped.kind).toBe('decode');
    expect(mapped.message).toMatch(/truncated/i);

    const capture = toErrorViewModel({ message: 'Display capture is not available in this browser' });
    expect(capture.kind).toBe('capture');
  });
});
