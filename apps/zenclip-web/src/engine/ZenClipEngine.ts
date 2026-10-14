/**
 * Typed JS binding for the zenClip sanitize engine.
 *
 * The rust/wasm API is sync after init. This facade returns Promises because
 * wasm-bindgen init and the MOCK's WebCrypto digest are async. Do not
 * reimplement sanitize in JS — load `zenclip-wasm` or fall back to the labelled MOCK.
 */

export type EngineErrorKind = 'decode' | 'too_large' | 'unsupported';

export type SanitizeResult = {
  png: Uint8Array;
  /** 64 lowercase hex characters */
  sha256: string;
  width: number;
  height: number;
  removedChunks: string[];
};

export class ZenClipEngineError extends Error {
  readonly kind: EngineErrorKind;

  constructor(kind: EngineErrorKind, message: string) {
    super(message);
    this.name = 'ZenClipEngineError';
    this.kind = kind;
  }
}

export interface ZenClipEngine {
  sanitize(png: Uint8Array): Promise<SanitizeResult>;
  sha256(bytes: Uint8Array): Promise<string>;
  version(): string;
}

export type EngineSource = 'wasm' | 'mock';

export type LoadedEngine = {
  source: EngineSource;
  engine: ZenClipEngine;
};

export const SHA256_HEX_RE = /^[0-9a-f]{64}$/;

export function isEngineErrorKind(value: unknown): value is EngineErrorKind {
  return value === 'decode' || value === 'too_large' || value === 'unsupported';
}

export function asEngineError(error: unknown): ZenClipEngineError {
  if (error instanceof ZenClipEngineError) {
    return error;
  }
  if (error instanceof Error && isEngineErrorKind((error as Error & { kind?: unknown }).kind)) {
    return new ZenClipEngineError(
      (error as Error & { kind: EngineErrorKind }).kind,
      error.message,
    );
  }
  if (typeof error === 'object' && error !== null) {
    const record = error as { kind?: unknown; message?: unknown };
    if (isEngineErrorKind(record.kind) && typeof record.message === 'string') {
      return new ZenClipEngineError(record.kind, record.message);
    }
  }
  const message = error instanceof Error ? error.message : 'Engine failed';
  return new ZenClipEngineError('decode', message);
}

export function assertSanitizeResult(value: unknown): SanitizeResult {
  if (typeof value !== 'object' || value === null) {
    throw new ZenClipEngineError('decode', 'Sanitize result is not an object');
  }
  const record = value as Record<string, unknown>;
  const png = toUint8Array(record.png);
  const sha256 = record.sha256;
  const width = record.width;
  const height = record.height;
  const removedChunks = record.removedChunks;

  if (typeof sha256 !== 'string' || !SHA256_HEX_RE.test(sha256)) {
    throw new ZenClipEngineError('decode', 'Sanitize result sha256 is not 64 lowercase hex');
  }
  if (!Number.isInteger(width) || typeof width !== 'number' || width <= 0) {
    throw new ZenClipEngineError('decode', 'Sanitize result width is invalid');
  }
  if (!Number.isInteger(height) || typeof height !== 'number' || height <= 0) {
    throw new ZenClipEngineError('decode', 'Sanitize result height is invalid');
  }
  if (!Array.isArray(removedChunks) || removedChunks.some((chunk) => typeof chunk !== 'string')) {
    throw new ZenClipEngineError('decode', 'Sanitize result removedChunks is invalid');
  }

  return {
    png,
    sha256,
    width,
    height,
    removedChunks: removedChunks as string[],
  };
}

function toUint8Array(value: unknown): Uint8Array {
  if (value instanceof Uint8Array) {
    return value;
  }
  if (value instanceof ArrayBuffer) {
    return new Uint8Array(value);
  }
  throw new ZenClipEngineError('decode', 'Sanitize result png is not bytes');
}
