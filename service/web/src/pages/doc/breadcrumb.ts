import { graphqlRequest, escapeHtml } from './api';

const siteName = '笔阵';

async function loadPaperBreadcrumb(paperId: string) {
  const data = await graphqlRequest(
    `query($paperId: Int!) { paperBreadcrumb(paperId: $paperId) { directoryItems { id name } paper { id examSpaceId directoryId title } } }`,
    { paperId: parseInt(paperId) },
  );
  return data?.paperBreadcrumb || null;
}

async function loadSpaceByToken(token: string) {
  const data = await graphqlRequest(
    `query($token: String) { examSpaces(token: $token) { ... on GqlExamSpacePayload { space { id token name } } } }`,
    { token },
  );
  return data?.examSpaces?.space || null;
}

export async function initBreadcrumb(paperId: string, spaceToken: string) {
  const breadcrumb = document.getElementById('docBreadcrumb')!;
  const titleEl = document.getElementById('breadcrumbTitle')!;
  let currentSpaceToken = spaceToken;

  // If new paper with space context
  if (paperId === 'new' && currentSpaceToken) {
    try {
      const space = await loadSpaceByToken(currentSpaceToken);
      if (space) {
        breadcrumb.innerHTML = `
          <a href="/console" class="breadcrumb-link">控制台</a>
          <span class="breadcrumb-sep">/</span>
          <a href="/console/papers" class="breadcrumb-link">试卷空间</a>
          <span class="breadcrumb-sep">/</span>
          <a href="/space/${encodeURIComponent(space.token)}" class="breadcrumb-link">${escapeHtml(space.name)}</a>
          <span class="breadcrumb-sep">/</span>
          <span class="breadcrumb-current">新建试卷</span>`;
        titleEl.textContent = '新建试卷';
        document.title = `新建试卷 - ${siteName}`;
        const docTitle = document.getElementById('docTitle');
        if (docTitle) docTitle.textContent = '新建试卷';
      }
    } catch (e) {
      console.error('load space error:', e);
    }
    return;
  }

  // Existing paper - load breadcrumb
  if (paperId && paperId !== 'new') {
    try {
      const result = await loadPaperBreadcrumb(paperId);
      if (result && result.paper) {
        const paper = result.paper;
        const dirItems = result.directoryItems || [];

        // Load space info
        let spaceLink = '';
        if (currentSpaceToken) {
          const space = await loadSpaceByToken(currentSpaceToken);
          if (space) {
            spaceLink = `<a href="/space/${encodeURIComponent(space.token)}" class="breadcrumb-link">${escapeHtml(space.name)}</a><span class="breadcrumb-sep">/</span>`;
            currentSpaceToken = space.token;
          }
        }

        // Build directory breadcrumb
        let dirHtml = dirItems.map((item: { id: number; name: string }) =>
          `<a href="/space/${encodeURIComponent(currentSpaceToken)}?dir=${item.id}" class="breadcrumb-link">${escapeHtml(item.name)}</a><span class="breadcrumb-sep">/</span>`
        ).join('');

        breadcrumb.innerHTML = `
          <a href="/console" class="breadcrumb-link">控制台</a>
          <span class="breadcrumb-sep">/</span>
          <a href="/console/papers" class="breadcrumb-link">试卷空间</a>
          <span class="breadcrumb-sep">/</span>
          ${spaceLink}${dirHtml}<span class="breadcrumb-current">${escapeHtml(paper.title)}</span>`;

        document.title = `${paper.title} - ${siteName}`;
        const docTitle = document.getElementById('docTitle');
        if (docTitle) {
          docTitle.textContent = paper.title;
        }
        return paper.title as string;
      }
    } catch (e) {
      console.error('load breadcrumb error:', e);
      titleEl.textContent = paperId;
    }
  }
  return '';
}
