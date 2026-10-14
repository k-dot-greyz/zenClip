import { describe, expect, it } from 'vitest';
import { captureDisplayPng } from './display';

describe('display capture', () => {
  it('throws when getDisplayMedia is missing', async () => {
    await expect(
      captureDisplayPng({
        getDisplayMedia: undefined as unknown as (constraints?: DisplayMediaStreamOptions) => Promise<MediaStream>,
      }),
    ).rejects.toThrow(/not available/i);
  });
});
