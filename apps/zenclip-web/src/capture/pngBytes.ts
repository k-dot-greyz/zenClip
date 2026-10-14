/** Canvas → PNG bytes. Capture path only; not a sanitizer. */
export async function pngBytesFromCanvas(canvas: HTMLCanvasElement): Promise<Uint8Array> {
  const blob = await new Promise<Blob>((resolve, reject) => {
    canvas.toBlob((result) => {
      if (result) {
        resolve(result);
      } else {
        reject(new Error('Canvas failed to encode PNG bytes'));
      }
    }, 'image/png');
  });
  return new Uint8Array(await blob.arrayBuffer());
}

export async function bytesFromBlob(blob: Blob): Promise<Uint8Array> {
  return new Uint8Array(await blob.arrayBuffer());
}
