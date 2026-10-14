declare module 'zenclip-wasm' {
  const init: (moduleOrPath?: unknown) => Promise<unknown>;
  export default init;
  export function sanitize(png: Uint8Array): unknown;
  export function sha256(bytes: Uint8Array): string;
  export function version(): string;
}
