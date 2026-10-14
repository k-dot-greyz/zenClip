/// <reference types="astro/client" />

interface ImportMetaEnv {
  readonly PUBLIC_ZENCLIP_WASM?: string;
  readonly PUBLIC_ZENCLIP_MAX_BYTES?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
