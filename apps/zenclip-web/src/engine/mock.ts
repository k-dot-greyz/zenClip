import {
  type SanitizeResult,
  type ZenClipEngine,
  SHA256_HEX_RE,
} from './ZenClipEngine';
import { assertNotTooLarge, defaultMaxBytes, peekPngSize } from './pngPeek';
import { sha256Hex } from './sha256';

export const MOCK_VERSION = 'mock-0.1.0';

export function createMockEngine(maxBytes: number = defaultMaxBytes()): ZenClipEngine {
  return {
    async sanitize(png: Uint8Array): Promise<SanitizeResult> {
      assertNotTooLarge(png, maxBytes);
      const { width, height } = peekPngSize(png);
      const digest = await sha256Hex(png);
      if (!SHA256_HEX_RE.test(digest)) {
        throw new Error('WebCrypto SHA-256 did not return 64 lowercase hex');
      }
      return {
        png: png.slice(),
        sha256: digest,
        width,
        height,
        removedChunks: [],
      };
    },
    sha256(bytes: Uint8Array): Promise<string> {
      return sha256Hex(bytes);
    },
    version(): string {
      return MOCK_VERSION;
    },
  };
}
