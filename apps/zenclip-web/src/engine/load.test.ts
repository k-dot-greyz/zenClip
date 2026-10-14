import { describe, expect, it } from 'vitest';
import { loadEngine } from './load';
import { MOCK_VERSION } from './mock';

describe('loadEngine', () => {
  it('falls back to the labelled MOCK when zenclip-wasm is absent', async () => {
    const loaded = await loadEngine();
    expect(loaded.source).toBe('mock');
    expect(loaded.engine.version()).toBe(MOCK_VERSION);
  });
});
