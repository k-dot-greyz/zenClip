import { ZenClipEngineError } from './ZenClipEngine';

/** PNG signature: \x89PNG\r\n\x1a\n */
export const PNG_SIGNATURE = new Uint8Array([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);

const IHDR = 0x49484452;

/**
 * Header-only peek for MOCK dimensions. This is not a sanitizer and does not
 * strip chunks — the real engine lives in `zenclip-wasm`.
 */
export function peekPngSize(bytes: Uint8Array): { width: number; height: number } {
  if (bytes.byteLength < 8) {
    throw new ZenClipEngineError('decode', 'PNG is truncated before the signature');
  }
  if (!hasPngSignature(bytes)) {
    throw new ZenClipEngineError('unsupported', 'Not a PNG');
  }
  if (bytes.byteLength < 24) {
    throw new ZenClipEngineError('decode', 'PNG is truncated before IHDR');
  }

  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const length = view.getUint32(8);
  const type = view.getUint32(12);
  if (type !== IHDR || length < 13) {
    throw new ZenClipEngineError('decode', 'PNG is missing a valid IHDR chunk');
  }
  if (bytes.byteLength < 24 + 4) {
    throw new ZenClipEngineError('decode', 'PNG IHDR is truncated');
  }

  const width = view.getUint32(16);
  const height = view.getUint32(20);
  if (width === 0 || height === 0) {
    throw new ZenClipEngineError('decode', 'PNG IHDR has zero dimension');
  }
  return { width, height };
}

export function hasPngSignature(bytes: Uint8Array): boolean {
  if (bytes.byteLength < PNG_SIGNATURE.byteLength) {
    return false;
  }
  for (let i = 0; i < PNG_SIGNATURE.byteLength; i += 1) {
    if (bytes[i] !== PNG_SIGNATURE[i]) {
      return false;
    }
  }
  return true;
}

export function defaultMaxBytes(): number {
  const raw = import.meta.env.PUBLIC_ZENCLIP_MAX_BYTES;
  if (raw && /^\d+$/.test(raw)) {
    const parsed = Number.parseInt(raw, 10);
    if (parsed > 0) {
      return parsed;
    }
  }
  return 20 * 1024 * 1024;
}

export function assertNotTooLarge(bytes: Uint8Array, maxBytes: number): void {
  if (bytes.byteLength > maxBytes) {
    throw new ZenClipEngineError(
      'too_large',
      `PNG is ${bytes.byteLength} bytes; max is ${maxBytes}`,
    );
  }
}
