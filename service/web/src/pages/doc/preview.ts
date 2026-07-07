import { compilePdf } from './api';
import { initTemplateSettings } from './templateSettings';

let pdfjsLibPromise: Promise<typeof import('pdfjs-dist')> | null = null;

async function getPdfJsLib() {
  if (!pdfjsLibPromise) {
    pdfjsLibPromise = import('pdfjs-dist').then((module) => {
      module.GlobalWorkerOptions.workerSrc = '/pdf.worker.min.mjs';
      return module;
    });
  }
  return pdfjsLibPromise;
}

type ZoomMode = 'fit-width' | 'fit-page' | 'actual' | 'custom';

const ZOOM_LABELS: Record<ZoomMode, string> = {
  'fit-width': '适配宽度',
  'fit-page': '适配页面',
  'actual': '实际大小',
  'custom': '自定义',
};

const ZOOM_CYCLE: ZoomMode[] = ['fit-width', 'fit-page', 'actual'];

export function initPreview(paperToken: string) {
  if (typeof window === 'undefined') return;

  const btnSave = document.getElementById('btnSave');
  const docContentWrapper = document.getElementById('docContentWrapper');
  const docPreviewPanel = document.getElementById('docPreviewPanel');
  const docOutline = document.getElementById('docOutline');
  const docAnnotations = document.getElementById('docAnnotations');
  const btnFullscreen = document.getElementById('btnFullscreen');
  const btnRefresh = document.getElementById('btnRefresh');
  const btnZoom = document.getElementById('btnZoom');
  const btnSettings = document.getElementById('btnSettings');
  const btnSettingsClose = document.getElementById('btnSettingsClose');
  const docSettingsOverlay = document.getElementById('docSettingsOverlay');
  const docSettingsContent = document.getElementById('docSettingsContent');
  const previewContent = document.getElementById('previewContent');
  const docSettingsNavItems = Array.from(document.querySelectorAll<HTMLAnchorElement>('.doc-settings-nav-item'));
  const docSettingsSections = Array.from(document.querySelectorAll<HTMLElement>('.doc-settings-section'));

  let isPreviewMode = localStorage.getItem('preview-mode') === 'true';
  let isFullscreen = localStorage.getItem('preview-fullscreen') === 'true';
  let zoomMode: ZoomMode = (localStorage.getItem('preview-zoom') as ZoomMode) || 'fit-width';
  let customZoomPct = parseInt(localStorage.getItem('preview-zoom-custom') || '100', 10);
  let currentPdfBlob: Blob | null = null;
  let zoomMenu: HTMLDivElement | null = null;
  let renderTimer: ReturnType<typeof setTimeout> | null = null;
  let lastRenderedPct = 100;

  const svgFullscreen = '<svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M2 6V2h4M10 2h4v4M14 10v4h-4M6 14H2v-4"/></svg>';
  const svgRestore = '<svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 2v2H2M12 2v2h2M14 12h-2v2M4 14v-2H2"/></svg>';

  function updateZoomTitle() {
    if (!btnZoom) return;
    const label = zoomMode === 'custom'
      ? `${ZOOM_LABELS[zoomMode]} ${customZoomPct}%`
      : ZOOM_LABELS[zoomMode];
    btnZoom.title = `缩放：${label}`;
  }

  function closeZoomMenu() {
    if (zoomMenu) {
      zoomMenu.remove();
      zoomMenu = null;
    }
  }

  function openSettingsModal() {
    if (!docSettingsOverlay) return;
    docSettingsOverlay.removeAttribute('hidden');
    document.body.classList.add('doc-settings-open');
    setActiveSettingNav('settings-template');
  }

  function closeSettingsModal() {
    if (!docSettingsOverlay) return;
    docSettingsOverlay.setAttribute('hidden', '');
    document.body.classList.remove('doc-settings-open');
  }

  function setActiveSettingNav(sectionId: string) {
    docSettingsNavItems.forEach((item) => {
      const isActive = item.getAttribute('href') === `#${sectionId}`;
      item.classList.toggle('active', isActive);
    });
  }

  function scrollToSettingSection(sectionId: string) {
    const target = docSettingsSections.find((section) => section.id === sectionId);
    if (!target || !docSettingsContent) return;
    setActiveSettingNav(sectionId);
    docSettingsContent.scrollTo({
      top: Math.max(0, target.offsetTop - 16),
      behavior: 'smooth',
    });
  }

  function openZoomMenu() {
    closeZoomMenu();
    if (!btnZoom || !docPreviewPanel) return;

    const menu = document.createElement('div');
    menu.className = 'preview-zoom-menu';

    const btnRect = btnZoom.getBoundingClientRect();
    const panelRect = docPreviewPanel.getBoundingClientRect();
    menu.style.top = (btnRect.bottom - panelRect.top + 4) + 'px';
    menu.style.right = (panelRect.right - btnRect.right) + 'px';

    for (const mode of ZOOM_CYCLE) {
      const item = document.createElement('div');
      item.className = 'preview-zoom-item' + (mode === zoomMode ? ' active' : '');
      item.textContent = ZOOM_LABELS[mode];
      item.addEventListener('click', async () => {
        zoomMode = mode;
        localStorage.setItem('preview-zoom', mode);
        updateZoomTitle();
        closeZoomMenu();
        if (currentPdfBlob && isPreviewMode) {
          renderPdf(await currentPdfBlob.arrayBuffer());
        }
      });
      menu.appendChild(item);
    }

    // Divider
    const divider = document.createElement('div');
    divider.className = 'preview-zoom-divider';
    menu.appendChild(divider);

    // Custom zoom slider row
    const sliderRow = document.createElement('div');
    sliderRow.className = 'preview-zoom-slider-row';

    const label = document.createElement('span');
    label.className = 'preview-zoom-slider-label';
    label.textContent = customZoomPct + '%';

    const slider = document.createElement('input');
    slider.type = 'range';
    slider.min = '25';
    slider.max = '300';
    slider.step = '5';
    slider.value = String(customZoomPct);
    slider.className = 'preview-zoom-slider';

    slider.addEventListener('input', () => {
      customZoomPct = parseInt(slider.value, 10);
      label.textContent = customZoomPct + '%';
      zoomMode = 'custom';
      localStorage.setItem('preview-zoom', 'custom');
      localStorage.setItem('preview-zoom-custom', String(customZoomPct));
      updateZoomTitle();
      applyCssZoom();
      scheduleRender();
    });

    sliderRow.appendChild(slider);
    sliderRow.appendChild(label);
    menu.appendChild(sliderRow);

    // Highlight custom if active
    if (zoomMode === 'custom') {
      sliderRow.classList.add('active');
    }

    docPreviewPanel.appendChild(menu);
    zoomMenu = menu;

    // Close on outside click.
    const onOutsideClick = (e: MouseEvent) => {
      if (zoomMenu && !zoomMenu.contains(e.target as Node) && e.target !== btnZoom) {
        closeZoomMenu();
        document.removeEventListener('click', onOutsideClick);
      }
    };
    setTimeout(() => document.addEventListener('click', onOutsideClick), 0);
  }

  function showLoading() {
    if (!previewContent) return;
    previewContent.innerHTML = `
      <div class="preview-placeholder">
        <div class="preview-placeholder-icon">⏳</div>
        <p>正在编译 PDF...</p>
      </div>
    `;
  }

  function showError(msg: string) {
    if (!previewContent) return;
    previewContent.innerHTML = `
      <div class="preview-placeholder">
        <div class="preview-placeholder-icon">⚠️</div>
        <p>编译失败：${msg}</p>
      </div>
    `;
  }

  /** Schedule a debounced high-quality re-render. */
  function scheduleRender(delay = 300) {
    if (renderTimer) clearTimeout(renderTimer);
    renderTimer = setTimeout(async () => {
      if (currentPdfBlob && isPreviewMode) {
        renderPdf(await currentPdfBlob.arrayBuffer());
      }
    }, delay);
  }

  /** Apply CSS transform to existing canvases for instant visual feedback. */
  function applyCssZoom() {
    if (!previewContent) return;
    const container = previewContent.querySelector('.pdf-pages') as HTMLElement | null;
    if (!container) return;
    // Calculate the ratio between desired zoom and current rendered zoom.
    // The canvases are rendered at lastRenderedPct, so we scale from there.
    const ratio = customZoomPct / lastRenderedPct;
    container.style.transform = `scale(${ratio})`;
    container.style.transformOrigin = 'top center';
  }

  /** Apply a zoom delta from keyboard/scroll shortcuts. */
  function applyZoomDelta(delta: number) {
    zoomMode = 'custom';
    customZoomPct = Math.max(25, Math.min(300, customZoomPct + delta));
    localStorage.setItem('preview-zoom', 'custom');
    localStorage.setItem('preview-zoom-custom', String(customZoomPct));
    updateZoomTitle();
    applyCssZoom();
    scheduleRender();
  }

  void initTemplateSettings(paperToken);

  async function renderPdf(data: ArrayBuffer) {
    if (!previewContent || !docPreviewPanel) return;

    const pdfjsLib = await getPdfJsLib();
    const pdf = await pdfjsLib.getDocument({ data }).promise;
    const numPages = pdf.numPages;

    previewContent.innerHTML = '';

    const container = document.createElement('div');
    container.className = 'pdf-pages';
    previewContent.appendChild(container);

    const dpr = window.devicePixelRatio || 1;
    const panelWidth = docPreviewPanel.clientWidth;
    const panelHeight = docPreviewPanel.clientHeight;

    // Get first page dimensions to calculate scale.
    const firstPage = await pdf.getPage(1);
    const baseViewport = firstPage.getViewport({ scale: 1 });

    // Show page size info (PDF points → mm: 1pt = 25.4/72 mm)
    const ptToMm = 25.4 / 72;
    const pageWidthMm = Math.round(baseViewport.width * ptToMm);
    const pageHeightMm = Math.round(baseViewport.height * ptToMm);
    const sizeLabel = document.createElement('div');
    sizeLabel.className = 'pdf-page-size-label';
    sizeLabel.textContent = `${pageWidthMm} × ${pageHeightMm} mm`;
    container.appendChild(sizeLabel);

    let scale: number;
    switch (zoomMode) {
      case 'actual':
        scale = 1;
        break;
      case 'fit-page':
        scale = Math.min(
          (panelWidth - 24) / baseViewport.width,
          (panelHeight - 24) / baseViewport.height,
        );
        break;
      case 'custom':
        scale = customZoomPct / 100;
        break;
      case 'fit-width':
      default:
        scale = (panelWidth - 24) / baseViewport.width;
        break;
    }

    // Sync customZoomPct with the actual scale so the slider reflects reality.
    customZoomPct = Math.round(scale * 100);
    localStorage.setItem('preview-zoom-custom', String(customZoomPct));
    lastRenderedPct = customZoomPct;

    for (let i = 1; i <= numPages; i++) {
      const page = await pdf.getPage(i);
      const viewport = page.getViewport({ scale });

      const canvas = document.createElement('canvas');
      canvas.width = Math.floor(viewport.width * dpr);
      canvas.height = Math.floor(viewport.height * dpr);
      canvas.style.width = Math.floor(viewport.width) + 'px';
      canvas.style.height = Math.floor(viewport.height) + 'px';
      canvas.style.display = 'block';
      canvas.style.background = '#fff';

      const ctx = canvas.getContext('2d')!;
      ctx.scale(dpr, dpr);
      await page.render({ canvas, canvasContext: ctx, viewport }).promise;

      container.appendChild(canvas);
    }
  }

  async function loadPdf() {
    showLoading();
    try {
      const blob = await compilePdf(paperToken);
      if (blob.size === 0) {
        showError('返回的 PDF 为空');
        return;
      }
      currentPdfBlob = blob;
      await renderPdf(await blob.arrayBuffer());
    } catch (e: any) {
      showError(e.message || '未知错误');
    }
  }

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
      // Trigger PDF compilation.
      loadPdf();
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
      if (previewContent) {
        previewContent.style.padding = '';
        previewContent.style.height = '';
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
    btnFullscreen.addEventListener('click', async () => {
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
      // Re-render after layout change.
      if (currentPdfBlob && isPreviewMode) {
        renderPdf(await currentPdfBlob.arrayBuffer());
      }
    });
  }

  // Refresh preview
  if (btnRefresh) {
    btnRefresh.addEventListener('click', () => {
      if (isPreviewMode) loadPdf();
    });
  }

  // Settings modal
  if (btnSettings) {
    btnSettings.addEventListener('click', () => {
      openSettingsModal();
    });
  }

  if (btnSettingsClose) {
    btnSettingsClose.addEventListener('click', () => {
      closeSettingsModal();
    });
  }

  if (docSettingsOverlay) {
    docSettingsOverlay.addEventListener('click', (e) => {
      if (e.target === docSettingsOverlay) {
        closeSettingsModal();
      }
    });

    document.addEventListener('keydown', (e) => {
      if (e.key === 'Escape' && !docSettingsOverlay.hasAttribute('hidden')) {
        closeSettingsModal();
      }
    });
  }

  docSettingsNavItems.forEach((item) => {
    item.addEventListener('click', (e) => {
      const href = item.getAttribute('href');
      if (!href?.startsWith('#')) return;
      e.preventDefault();
      scrollToSettingSection(href.slice(1));
    });
  });

  // Zoom menu
  if (btnZoom) {
    updateZoomTitle();
    btnZoom.addEventListener('click', (e) => {
      e.stopPropagation();
      if (zoomMenu) {
        closeZoomMenu();
      } else {
        openZoomMenu();
      }
    });
  }

  // --- Keyboard & mouse shortcuts for the preview panel ---

  if (docPreviewPanel) {
    // Ctrl/Cmd + scroll to zoom (on previewContent, the scrollable container)
    previewContent?.addEventListener('wheel', (e) => {
      if (!isPreviewMode || !currentPdfBlob) return;
      if (e.ctrlKey || e.metaKey) {
        e.preventDefault();
        const delta = e.deltaY < 0 ? 5 : -5;
        applyZoomDelta(delta);
      }
    }, { passive: false });

    // Keyboard shortcuts
    docPreviewPanel.setAttribute('tabindex', '0');
    docPreviewPanel.addEventListener('keydown', (e) => {
      if (!isPreviewMode || !currentPdfBlob) return;

      // Ctrl/Cmd + =  → zoom in
      // Ctrl/Cmd + -  → zoom out
      // Ctrl/Cmd + 0  → reset to fit-width
      if (e.ctrlKey || e.metaKey) {
        if (e.key === '=' || e.key === '+') {
          e.preventDefault();
          applyZoomDelta(5);
        } else if (e.key === '-') {
          e.preventDefault();
          applyZoomDelta(-5);
        } else if (e.key === '0') {
          e.preventDefault();
          zoomMode = 'fit-width';
          localStorage.setItem('preview-zoom', 'fit-width');
          updateZoomTitle();
          scheduleRender();
        }
        return;
      }

      // Plain + / - keys
      if (e.key === '+' || e.key === '=') {
        e.preventDefault();
        applyZoomDelta(5);
      } else if (e.key === '-') {
        e.preventDefault();
        applyZoomDelta(-5);
      }
    });

    // Middle-click or Space+drag to pan
    let isPanning = false;
    let panStartX = 0;
    let panStartY = 0;
    let scrollStartX = 0;
    let scrollStartY = 0;
    let spaceHeld = false;

    docPreviewPanel.addEventListener('mousedown', (e) => {
      // Middle mouse button, or Space held down + left click
      if (e.button === 1 || (e.button === 0 && spaceHeld)) {
        e.preventDefault();
        isPanning = true;
        panStartX = e.clientX;
        panStartY = e.clientY;
        scrollStartX = previewContent!.scrollLeft;
        scrollStartY = previewContent!.scrollTop;
        docPreviewPanel!.style.cursor = 'grabbing';
      }
    });

    document.addEventListener('mousemove', (e) => {
      if (!isPanning) return;
      const dx = e.clientX - panStartX;
      const dy = e.clientY - panStartY;
      previewContent!.scrollLeft = scrollStartX - dx;
      previewContent!.scrollTop = scrollStartY - dy;
    });

    document.addEventListener('mouseup', () => {
      if (isPanning) {
        isPanning = false;
        if (docPreviewPanel) docPreviewPanel.style.cursor = '';
      }
    });

    // Space key to enable grab cursor
    docPreviewPanel.addEventListener('keydown', (e) => {
      if (e.code === 'Space' && !spaceHeld) {
        spaceHeld = true;
        docPreviewPanel!.style.cursor = 'grab';
      }
    });
    docPreviewPanel.addEventListener('keyup', (e) => {
      if (e.code === 'Space') {
        spaceHeld = false;
        if (!isPanning) docPreviewPanel!.style.cursor = '';
      }
    });
  }
}
