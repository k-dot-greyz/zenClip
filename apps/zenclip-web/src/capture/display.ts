import { pngBytesFromCanvas } from './pngBytes';

export type DisplayCapture = {
  getDisplayMedia: (constraints?: DisplayMediaStreamOptions) => Promise<MediaStream>;
};

/**
 * getDisplayMedia → video frame → canvas → PNG bytes.
 * Stops the stream after one frame so we do not keep a live capture session.
 */
export async function captureDisplayPng(
  api: DisplayCapture = navigator.mediaDevices,
): Promise<Uint8Array> {
  if (typeof api.getDisplayMedia !== 'function') {
    throw new Error('Display capture is not available in this browser');
  }

  const stream = await api.getDisplayMedia({
    video: { frameRate: 1 },
    audio: false,
  });

  try {
    const [track] = stream.getVideoTracks();
    if (!track) {
      throw new Error('Display capture returned no video track');
    }

    const video = document.createElement('video');
    video.muted = true;
    video.playsInline = true;
    video.setAttribute('playsinline', '');
    video.srcObject = stream;
    await video.play();
    await waitForVideoFrame(video);

    const width = video.videoWidth;
    const height = video.videoHeight;
    if (width < 1 || height < 1) {
      throw new Error('Display capture produced an empty frame');
    }

    const canvas = document.createElement('canvas');
    canvas.width = width;
    canvas.height = height;
    const context = canvas.getContext('2d', { alpha: false });
    if (!context) {
      throw new Error('Canvas 2D context is unavailable');
    }
    context.drawImage(video, 0, 0, width, height);
    return await pngBytesFromCanvas(canvas);
  } finally {
    for (const track of stream.getTracks()) {
      track.stop();
    }
  }
}

function waitForVideoFrame(video: HTMLVideoElement): Promise<void> {
  if (video.videoWidth > 0 && video.videoHeight > 0) {
    return Promise.resolve();
  }
  return new Promise((resolve, reject) => {
    const onReady = () => {
      cleanup();
      resolve();
    };
    const onError = () => {
      cleanup();
      reject(new Error('Display capture video failed to load'));
    };
    const cleanup = () => {
      video.removeEventListener('loadeddata', onReady);
      video.removeEventListener('error', onError);
    };
    video.addEventListener('loadeddata', onReady);
    video.addEventListener('error', onError);
  });
}
