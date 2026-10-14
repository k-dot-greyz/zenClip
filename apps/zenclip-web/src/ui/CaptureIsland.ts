import { captureDisplayPng } from '../capture/display';
import { isLikelyImageFile, pngBytesFromClipboardEvent, pngBytesFromFile } from '../capture/fallbacks';
import { toArrayBuffer } from '../engine/bytes';
import { asEngineError, type LoadedEngine } from '../engine/ZenClipEngine';
import { loadEngine } from '../engine/load';
import {
  engineLabel,
  previewAlt,
  toErrorViewModel,
  toResultViewModel,
  type ResultViewModel,
} from './result';

type CaptureRoot = {
  engineStatus: HTMLElement;
  captureButton: HTMLButtonElement;
  fileInput: HTMLInputElement;
  pasteHint: HTMLElement;
  status: HTMLElement;
  result: HTMLElement;
  preview: HTMLImageElement;
  dimensions: HTMLElement;
  sha256: HTMLElement;
  chunks: HTMLElement;
};

export async function mountCaptureApp(root: HTMLElement): Promise<void> {
  const ui = bind(root);
  const loaded = await loadEngine();
  paintEngine(ui, loaded);

  ui.captureButton.addEventListener('click', () => {
    void runCapture(ui, loaded, () => captureDisplayPng());
  });

  ui.fileInput.addEventListener('change', () => {
    const file = ui.fileInput.files?.[0];
    ui.fileInput.value = '';
    if (!file) {
      return;
    }
    if (!isLikelyImageFile(file)) {
      paintError(ui, { message: 'Choose a PNG or another still image file' });
      return;
    }
    void runCapture(ui, loaded, () => pngBytesFromFile(file));
  });

  document.addEventListener('paste', (event) => {
    void onPaste(event, ui, loaded);
  });
}

function bind(root: HTMLElement): CaptureRoot {
  return {
    engineStatus: must(root, '[data-engine-status]'),
    captureButton: must(root, '[data-capture]') as HTMLButtonElement,
    fileInput: must(root, '[data-file]') as HTMLInputElement,
    pasteHint: must(root, '[data-paste-hint]'),
    status: must(root, '[data-status]'),
    result: must(root, '[data-result]'),
    preview: must(root, '[data-preview]') as HTMLImageElement,
    dimensions: must(root, '[data-dimensions]'),
    sha256: must(root, '[data-sha256]'),
    chunks: must(root, '[data-chunks]'),
  };
}

function must(root: HTMLElement, selector: string): HTMLElement {
  const node = root.querySelector<HTMLElement>(selector);
  if (!node) {
    throw new Error(`Missing capture UI node ${selector}`);
  }
  return node;
}

function paintEngine(ui: CaptureRoot, loaded: LoadedEngine): void {
  const label = engineLabel(loaded.source, loaded.engine.version());
  ui.engineStatus.textContent =
    loaded.source === 'mock'
      ? `Engine: ${label}. The wasm pkg is not loaded; PNG bytes pass through unchanged.`
      : `Engine: ${label}`;
}

async function onPaste(
  event: ClipboardEvent,
  ui: CaptureRoot,
  loaded: LoadedEngine,
): Promise<void> {
  try {
    const bytes = await pngBytesFromClipboardEvent(event);
    if (!bytes) {
      return;
    }
    event.preventDefault();
    await runCapture(ui, loaded, async () => bytes);
  } catch (error) {
    paintError(ui, error instanceof Error ? error : { message: 'Paste failed' });
  }
}

async function runCapture(
  ui: CaptureRoot,
  loaded: LoadedEngine,
  grab: () => Promise<Uint8Array>,
): Promise<void> {
  ui.captureButton.disabled = true;
  ui.status.textContent = 'Working…';
  try {
    const png = await grab();
    const result = await loaded.engine.sanitize(png);
    paintResult(ui, toResultViewModel(loaded.source, loaded.engine.version(), result), result.png);
  } catch (error) {
    const engineError = isLikelyEngineError(error) ? asEngineError(error) : null;
    paintError(ui, engineError ?? (error instanceof Error ? error : { message: 'Capture failed' }));
  } finally {
    ui.captureButton.disabled = false;
  }
}

function isLikelyEngineError(error: unknown): boolean {
  return (
    typeof error === 'object' &&
    error !== null &&
    'kind' in error &&
    (error as { kind: unknown }).kind !== undefined
  );
}

function paintResult(ui: CaptureRoot, view: ResultViewModel, png: Uint8Array): void {
  ui.status.textContent = 'Sanitize finished.';
  ui.result.hidden = false;
  ui.dimensions.textContent = view.dimensions;
  ui.sha256.textContent = view.sha256;
  ui.chunks.textContent = view.removedChunksLabel;

  const blob = new Blob([toArrayBuffer(png)], { type: 'image/png' });
  const previous = ui.preview.dataset.objectUrl;
  if (previous) {
    URL.revokeObjectURL(previous);
  }
  const url = URL.createObjectURL(blob);
  ui.preview.dataset.objectUrl = url;
  ui.preview.src = url;
  ui.preview.alt = previewAlt(view.dimensions);
  ui.preview.hidden = false;
}

function paintError(ui: CaptureRoot, error: { kind?: 'decode' | 'too_large' | 'unsupported'; message: string }): void {
  const view = toErrorViewModel(error);
  ui.status.textContent = view.kind === 'capture' ? view.message : `[${view.kind}] ${view.message}`;
  ui.result.hidden = true;
  ui.preview.hidden = true;
  ui.preview.removeAttribute('src');
  ui.preview.alt = '';
}
