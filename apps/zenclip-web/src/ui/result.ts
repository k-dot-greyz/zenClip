import type { EngineErrorKind, EngineSource, SanitizeResult } from '../engine/ZenClipEngine';

export type ResultViewModel = {
  engineLabel: string;
  width: number;
  height: number;
  dimensions: string;
  sha256: string;
  removedChunks: string[];
  removedChunksLabel: string;
};

export type ErrorViewModel = {
  kind: EngineErrorKind | 'capture';
  message: string;
};

export function engineLabel(source: EngineSource, version: string): string {
  if (source === 'mock') {
    return `MOCK (pass-through) · ${version}`;
  }
  return `zenclip-wasm · ${version}`;
}

export function toResultViewModel(
  source: EngineSource,
  version: string,
  result: SanitizeResult,
): ResultViewModel {
  return {
    engineLabel: engineLabel(source, version),
    width: result.width,
    height: result.height,
    dimensions: `${result.width} × ${result.height}`,
    sha256: result.sha256,
    removedChunks: result.removedChunks,
    removedChunksLabel: formatRemovedChunks(source, result.removedChunks),
  };
}

export function userMessageForKind(kind: EngineErrorKind): string {
  switch (kind) {
    case 'decode':
      return 'Could not decode this PNG.';
    case 'too_large':
      return 'This PNG is larger than the configured max size.';
    case 'unsupported':
      return 'This file is not a supported PNG.';
    default: {
      const _exhaustive: never = kind;
      return _exhaustive;
    }
  }
}

export function toErrorViewModel(error: {
  kind?: EngineErrorKind;
  message: string;
}): ErrorViewModel {
  if (error.kind === 'decode' || error.kind === 'too_large' || error.kind === 'unsupported') {
    return {
      kind: error.kind,
      message: `${userMessageForKind(error.kind)} ${error.message}`.trim(),
    };
  }
  return { kind: 'capture', message: error.message };
}

export function formatRemovedChunks(source: EngineSource, chunks: string[]): string {
  if (chunks.length > 0) {
    return chunks.join(', ');
  }
  if (source === 'mock') {
    return 'None — MOCK passes the PNG through and does not strip chunks';
  }
  return 'None';
}

export function previewAlt(dimensions: string): string {
  return `Captured PNG preview, ${dimensions}`;
}
