/** Minimal DOM mirror of `index.astro` for `mountCaptureApp` integration tests. */
export function createCaptureDom(): HTMLElement {
  const main = document.createElement('main');
  main.innerHTML = `
    <p class="engine" data-engine-status>Loading engine…</p>
    <button type="button" data-capture>Capture display</button>
    <input data-file type="file" />
    <p data-paste-hint>Paste hint</p>
    <p class="status" data-status role="status">Idle.</p>
    <section data-result hidden>
      <img class="preview" data-preview alt="" hidden />
      <dd data-dimensions></dd>
      <dd data-sha256></dd>
      <dd data-chunks></dd>
    </section>
  `;
  document.body.append(main);
  return main;
}
