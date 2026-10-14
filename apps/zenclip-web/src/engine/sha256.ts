import { toArrayBuffer } from './bytes';

/** WebCrypto SHA-256 as 64 lowercase hex. Not a sanitizer. */
export async function sha256Hex(bytes: Uint8Array): Promise<string> {
  const digest = await crypto.subtle.digest('SHA-256', toArrayBuffer(bytes));
  const hash = new Uint8Array(digest);
  let hex = '';
  for (const octet of hash) {
    hex += octet.toString(16).padStart(2, '0');
  }
  return hex;
}
