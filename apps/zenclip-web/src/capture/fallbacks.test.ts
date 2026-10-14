import { describe, expect, it } from 'vitest';
import { isLikelyImageFile } from './fallbacks';

describe('capture fallbacks', () => {
  it('accepts PNG and common still-image files', () => {
    expect(isLikelyImageFile({ type: 'image/png', name: 'shot.png' })).toBe(true);
    expect(isLikelyImageFile({ type: '', name: 'frame.WEBP' })).toBe(true);
    expect(isLikelyImageFile({ type: 'text/plain', name: 'notes.txt' })).toBe(false);
  });
});
