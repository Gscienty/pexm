export function initPreview() {
  const btnSave = document.getElementById('btnSave');
  const docContentWrapper = document.getElementById('docContentWrapper');
  const docPreviewPanel = document.getElementById('docPreviewPanel');
  const docOutline = document.getElementById('docOutline');
  const docAnnotations = document.getElementById('docAnnotations');
  const btnFullscreen = document.getElementById('btnFullscreen');

  let isPreviewMode = localStorage.getItem('preview-mode') === 'true';
  let isFullscreen = localStorage.getItem('preview-fullscreen') === 'true';

  const svgFullscreen = '<svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M2 6V2h4M10 2h4v4M14 10v4h-4M6 14H2v-4"/></svg>';
  const svgRestore = '<svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 2v2H2M12 2v2h2M14 12h-2v2M4 14v-2H2"/></svg>';

  function applyPreviewMode(on: boolean) {
    if (!docContentWrapper) return;
    if (on) {
      docContentWrapper.style.width = '50%';
      docContentWrapper.style.flexShrink = '0';
      docContentWrapper.style.gridTemplateColumns = '1fr';
      docOutline && (docOutline.style.display = 'none');
      docAnnotations && (docAnnotations.style.display = 'none');
      if (docPreviewPanel) {
        docPreviewPanel.style.display = 'block';
        docPreviewPanel.style.position = 'relative';
      }
      if (btnSave) {
        btnSave.innerHTML = '<span class="action-icon">✕</span> 关闭预览';
        btnSave.classList.remove('primary');
      }
      // Restore fullscreen state
      if (isFullscreen) {
        if (docContentWrapper) docContentWrapper.style.display = 'none';
        if (docPreviewPanel) docPreviewPanel.style.width = '100%';
      }
    } else {
      docContentWrapper.style.width = '';
      docContentWrapper.style.flexShrink = '';
      docContentWrapper.style.gridTemplateColumns = '';
      docContentWrapper.style.display = '';
      docOutline && (docOutline.style.display = '');
      docAnnotations && (docAnnotations.style.display = '');
      if (docPreviewPanel) {
        docPreviewPanel.style.display = '';
        docPreviewPanel.style.position = '';
        docPreviewPanel.style.width = '';
      }
      if (btnSave) {
        btnSave.innerHTML = '<span class="action-icon">👁</span> 预览';
        btnSave.classList.add('primary');
      }
    }
  }

  // Apply initial state
  applyPreviewMode(isPreviewMode);

  if (btnSave) {
    btnSave.addEventListener('click', () => {
      isPreviewMode = !isPreviewMode;
      localStorage.setItem('preview-mode', isPreviewMode ? 'true' : 'false');
      if (!isPreviewMode) {
        isFullscreen = false;
        localStorage.setItem('preview-fullscreen', 'false');
      }
      applyPreviewMode(isPreviewMode);
    });
  }

  // Fullscreen preview
  if (btnFullscreen) {
    if (isFullscreen) btnFullscreen.innerHTML = svgRestore;
    btnFullscreen.addEventListener('click', () => {
      isFullscreen = !isFullscreen;
      localStorage.setItem('preview-fullscreen', isFullscreen ? 'true' : 'false');
      btnFullscreen.innerHTML = isFullscreen ? svgRestore : svgFullscreen;
      if (isFullscreen) {
        if (docContentWrapper) docContentWrapper.style.display = 'none';
        if (docPreviewPanel) docPreviewPanel.style.width = '100%';
      } else {
        if (docContentWrapper) docContentWrapper.style.display = '';
        if (docPreviewPanel) docPreviewPanel.style.width = '';
      }
    });
  }
}
