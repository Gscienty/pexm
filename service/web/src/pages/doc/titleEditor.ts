import { graphqlRequest } from './api';

const siteName = '笔阵';

export function initTitleEditor(paperId: string, initialTitle: string) {
  const docTitle = document.querySelector('.doc-title') as HTMLElement | null;
  if (!docTitle) return;

  let isComposing = false;
  let lastSavedTitle = initialTitle;

  const updateTitles = () => {
    const text = docTitle.textContent?.trim() || '';
    if (!text) {
      docTitle.innerHTML = '';
    }
    const title = text || '未命名试卷';
    document.title = `${title} - ${siteName}`;
    const breadcrumbCurrent = document.querySelector('.breadcrumb-current');
    if (breadcrumbCurrent) {
      breadcrumbCurrent.textContent = title;
    }
  };

  docTitle.addEventListener('input', updateTitles);
  updateTitles();

  docTitle.addEventListener('compositionstart', () => { isComposing = true; });
  docTitle.addEventListener('compositionend', () => { isComposing = false; });

  docTitle.addEventListener('keydown', (e) => {
    if (e.key === 'Enter' && !isComposing) {
      e.preventDefault();
      const editor = document.querySelector('.ProseMirror');
      if (editor instanceof HTMLElement) {
        editor.focus();
      }
    }
  });

  // Save title on blur
  docTitle.addEventListener('blur', async () => {
    const newTitle = docTitle.textContent?.trim() || '';
    if (!newTitle || newTitle === lastSavedTitle || paperId === 'new') return;
    try {
      await graphqlRequest(
        `mutation($id: Int!, $title: String!) { updatePaper(id: $id, title: $title) }`,
        { id: parseInt(paperId), title: newTitle },
      );
      lastSavedTitle = newTitle;
    } catch (e) {
      console.error('Failed to save title:', e);
    }
  });
}
