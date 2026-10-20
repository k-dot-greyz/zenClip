import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { pngHeader } from '../engine/testPng';
import { ZenClipEngineError, type LoadedEngine, type ZenClipEngine } from '../engine/ZenClipEngine';
import { createCaptureDom } from './captureDomFixture';
import { mountCaptureApp } from './CaptureIsland';

const loadEngineMock = vi.fn<() => Promise<LoadedEngine>>();

vi.mock('../engine/load', () => ({
  loadEngine: () => loadEngineMock(),
}));

const captureDisplayPngMock = vi.fn<() => Promise<Uint8Array>>();

vi.mock('../capture/display', () => ({
  captureDisplayPng: () => captureDisplayPngMock(),
}));

function stubEngine(overrides: Partial<ZenClipEngine> = {}): ZenClipEngine {
  const png = pngHeader(4, 2);
  return {
    version: () => 'test-engine-0',
    sha256: vi.fn(),
    sanitize: vi.fn(async () => ({
      png,
      sha256: 'c'.repeat(64),
      width: 4,
      height: 2,
      removedChunks: ['tEXt'],
    })),
    ...overrides,
  };
}

function loaded(engine: ZenClipEngine, source: LoadedEngine['source'] = 'mock'): LoadedEngine {
  return { source, engine };
}

function panel(root: HTMLElement, selector: string): HTMLElement {
  const node = root.querySelector(selector);
  if (!(node instanceof HTMLElement)) {
    throw new Error(`Expected ${selector}`);
  }
  return node;
}

describe('ZCW-PR3 CaptureIsland UX + security', () => {
  beforeEach(() => {
    loadEngineMock.mockReset();
    captureDisplayPngMock.mockReset();
    document.body.innerHTML = '';
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('ZCW-PR3-UX-HAPPY-01 — paints labelled MOCK engine status on mount', async () => {
    loadEngineMock.mockResolvedValue(loaded(stubEngine()));
    const root = createCaptureDom();
    await mountCaptureApp(root);

    const status = root.querySelector('[data-engine-status]');
    expect(status?.textContent).toMatch(/MOCK \(pass-through\)/i);
    expect(status?.textContent).toMatch(/wasm pkg is not loaded/i);
  });

  it('ZCW-PR3-UX-HAPPY-02 — wasm source uses compact engine line', async () => {
    loadEngineMock.mockResolvedValue(loaded(stubEngine(), 'wasm'));
    const root = createCaptureDom();
    await mountCaptureApp(root);

    expect(root.querySelector('[data-engine-status]')?.textContent).toBe(
      'Engine: zenclip-wasm · test-engine-0',
    );
  });

  it('ZCW-PR3-UX-HAPPY-03 — display capture shows sanitize result panel', async () => {
    const engine = stubEngine();
    loadEngineMock.mockResolvedValue(loaded(engine));
    captureDisplayPngMock.mockResolvedValue(pngHeader(8, 6));

    const root = createCaptureDom();
    await mountCaptureApp(root);
    const button = root.querySelector<HTMLButtonElement>('[data-capture]')!;

    button.click();
    await vi.waitFor(() => {
      expect(panel(root, '[data-result]').hidden).toBe(false);
    });

    expect(root.querySelector('[data-status]')?.textContent).toBe('Sanitize finished.');
    expect(root.querySelector('[data-dimensions]')?.textContent).toBe('4 × 2');
    expect(root.querySelector('[data-sha256]')?.textContent).toBe('c'.repeat(64));
    expect(root.querySelector('[data-chunks]')?.textContent).toBe('tEXt');
    expect(engine.sanitize).toHaveBeenCalledOnce();
  });

  it('ZCW-PR3-UX-SAD-01 — rejects non-image upload without calling sanitize', async () => {
    const engine = stubEngine();
    loadEngineMock.mockResolvedValue(loaded(engine));
    const root = createCaptureDom();
    await mountCaptureApp(root);

    const input = root.querySelector<HTMLInputElement>('[data-file]')!;
    const file = new File(['not-an-image'], 'readme.txt', { type: 'text/plain' });
    Object.defineProperty(input, 'files', { value: [file], configurable: true });
    input.dispatchEvent(new Event('change', { bubbles: true }));

    expect(root.querySelector('[data-status]')?.textContent).toBe(
      'Choose a PNG or another still image file',
    );
    expect(panel(root, '[data-result]').hidden).toBe(true);
    expect(engine.sanitize).not.toHaveBeenCalled();
  });

  it('ZCW-PR3-UX-SAD-02 — capture failure re-enables the button and surfaces copy', async () => {
    loadEngineMock.mockResolvedValue(loaded(stubEngine()));
    captureDisplayPngMock.mockRejectedValue(new Error('Display capture is not available in this browser'));

    const root = createCaptureDom();
    await mountCaptureApp(root);
    const button = root.querySelector<HTMLButtonElement>('[data-capture]')!;

    button.click();
    await vi.waitFor(() => {
      expect(button.disabled).toBe(false);
    });

    expect(root.querySelector('[data-status]')?.textContent).toMatch(/not available/i);
    expect(panel(root, '[data-result]').hidden).toBe(true);
  });

  it('ZCW-PR3-UX-SAD-03 — engine decode errors use bracketed kind prefix', async () => {
    const engine = stubEngine({
      sanitize: vi.fn(async () => {
        throw new ZenClipEngineError('decode', 'PNG is truncated before IHDR');
      }),
    });
    loadEngineMock.mockResolvedValue(loaded(engine));
    captureDisplayPngMock.mockResolvedValue(pngHeader(1, 1));

    const root = createCaptureDom();
    await mountCaptureApp(root);
    root.querySelector<HTMLButtonElement>('[data-capture]')!.click();

    await vi.waitFor(() => {
      expect(root.querySelector('[data-status]')?.textContent).toMatch(/^\[decode\]/);
    });
  });

  it('ZCW-PR3-UX-ABLAT-01 — paste without image items is a silent no-op', async () => {
    loadEngineMock.mockResolvedValue(loaded(stubEngine()));
    const root = createCaptureDom();
    await mountCaptureApp(root);

    const event = new ClipboardEvent('paste', {
      clipboardData: new DataTransfer(),
    });
    document.dispatchEvent(event);

    expect(root.querySelector('[data-status]')?.textContent).toBe('Idle.');
    expect(panel(root, '[data-result]').hidden).toBe(true);
  });

  it('ZCW-PR3-SEC-01 — error copy is text-only (no HTML injection in status)', async () => {
    loadEngineMock.mockResolvedValue(loaded(stubEngine()));
    captureDisplayPngMock.mockRejectedValue(new Error('<img src=x onerror="alert(1)">'));

    const root = createCaptureDom();
    await mountCaptureApp(root);
    const status = root.querySelector('[data-status]')!;

    root.querySelector<HTMLButtonElement>('[data-capture]')!.click();
    await vi.waitFor(() => {
      expect(status.textContent).toContain('<img');
    });

    expect(status.querySelector('img')).toBeNull();
    expect(status.textContent).toContain('<img');
    expect(status.innerHTML).not.toContain('<img');
  });

  it('ZCW-PR3-SEC-02 — revokes prior preview object URLs on repeat success', async () => {
    loadEngineMock.mockResolvedValue(loaded(stubEngine()));
    captureDisplayPngMock.mockResolvedValue(pngHeader(2, 2));

    const createSpy = vi.spyOn(URL, 'createObjectURL').mockReturnValue('blob:preview-1');
    const revokeSpy = vi.spyOn(URL, 'revokeObjectURL');

    const root = createCaptureDom();
    await mountCaptureApp(root);
    const button = root.querySelector<HTMLButtonElement>('[data-capture]')!;

    button.click();
    await vi.waitFor(() => {
      expect(root.querySelector('[data-status]')?.textContent).toBe('Sanitize finished.');
    });
    expect(revokeSpy).not.toHaveBeenCalled();

    createSpy.mockReturnValueOnce('blob:preview-2');
    button.click();
    await vi.waitFor(() => {
      expect(revokeSpy).toHaveBeenCalledWith('blob:preview-1');
    });

    createSpy.mockRestore();
    revokeSpy.mockRestore();
  });

  it('ZCW-PR3-UX-SAD-04 — missing capture nodes fail fast on mount', async () => {
    loadEngineMock.mockResolvedValue(loaded(stubEngine()));
    const broken = document.createElement('main');
    broken.innerHTML = '<p data-engine-status></p>';

    await expect(mountCaptureApp(broken)).rejects.toThrow(/Missing capture UI node/);
  });
});
