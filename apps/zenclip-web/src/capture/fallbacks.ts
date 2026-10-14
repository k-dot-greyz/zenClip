import { toArrayBuffer } from '../engine/bytes';
import { hasPngSignature } from '../engine/pngPeek';
import { bytesFromBlob, pngBytesFromCanvas } from './pngBytes';

const IMAGE_TYPES = ['image/png', 'image/jpeg', 'image/webp', 'image/gif', 'image/bmp'];

export function isLikelyImageFile(file: { type: string; name: string }): boolean {
  if (IMAGE_TYPES.includes(file.type)) {
    return true;
  }
  return /\.(png|jpe?g|webp|gif|bmp)$/i.test(file.name);
}

export async function pngBytesFromFile(file: File): Promise<Uint8Array> {
  const bytes = await bytesFromBlob(file);
  if (file.type === 'image/png' || hasPngSignature(bytes)) {
    return bytes;
  }
  if (!isLikelyImageFile(file)) {
    throw new Error('Choose a PNG or another still image file');
  }
  return rasterizeImageToPng(bytes);
}

export async function pngBytesFromClipboardEvent(event: ClipboardEvent): Promise<Uint8Array | null> {
  const items = event.clipboardData?.items;
  if (!items) {
    return null;
  }
  for (const item of items) {
    if (item.kind === 'file' && item.type.startsWith('image/')) {
      const file = item.getAsFile();
      if (file) {
        return pngBytesFromFile(file);
      }
    }
  }
  return null;
}

async function rasterizeImageToPng(bytes: Uint8Array): Promise<Uint8Array> {
  const blob = new Blob([toArrayBuffer(bytes)]);
  const url = URL.createObjectURL(blob);
  try {
    const image = await loadImage(url);
    const canvas = document.createElement('canvas');
    canvas.width = image.naturalWidth;
    canvas.height = image.naturalHeight;
    if (canvas.width < 1 || canvas.height < 1) {
      throw new Error('Image has empty dimensions');
    }
    const context = canvas.getContext('2d', { alpha: true });
    if (!context) {
      throw new Error('Canvas 2D context is unavailable');
    }
    context.drawImage(image, 0, 0);
    return pngBytesFromCanvas(canvas);
  } finally {
    URL.revokeObjectURL(url);
  }
}

function loadImage(url: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const image = new Image();
    image.onload = () => resolve(image);
    image.onerror = () => reject(new Error('Could not decode the image file'));
    image.src = url;
  });
}
