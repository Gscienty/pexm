export function initOutline() {
  const outlineList = document.getElementById('outlineList');
  const docMain = document.querySelector('.doc-main');

  function renderOutline(headings: { id: string; level: number; text: string }[]) {
    if (!outlineList) return;
    if (headings.length === 0) {
      outlineList.innerHTML = '<li class="outline-empty">暂无标题</li>';
      return;
    }
    outlineList.innerHTML = headings.map(h => {
      const indent = h.level - 1;
      return `<li class="outline-item" data-id="${h.id}" style="padding-left: ${indent * 12 + 8}px">
        <a href="#${h.id}" class="outline-link">${h.text || '未命名'}</a>
      </li>`;
    }).join('');

    outlineList.querySelectorAll('.outline-link').forEach(link => {
      link.addEventListener('click', (e) => {
        e.preventDefault();
        const id = (link.closest('.outline-item') as HTMLElement)?.dataset.id;
        if (!id) return;
        const target = document.getElementById(id);
        if (target) target.scrollIntoView({ behavior: 'smooth', block: 'start' });
      });
    });
  }

  document.addEventListener('headings-change', ((e: CustomEvent) => {
    renderOutline(e.detail.headings);
  }) as EventListener);

  let currentActiveId = '';
  function updateActiveHeading() {
    if (!outlineList) return;
    const items = outlineList.querySelectorAll('.outline-item');
    if (items.length === 0) return;
    let activeId = '';
    items.forEach(item => {
      const id = (item as HTMLElement).dataset.id;
      if (!id) return;
      const el = document.getElementById(id);
      if (el) {
        const rect = el.getBoundingClientRect();
        if (rect.top <= 120) activeId = id;
      }
    });
    if (activeId !== currentActiveId) {
      currentActiveId = activeId;
      items.forEach(item => {
        item.classList.toggle('active', (item as HTMLElement).dataset.id === activeId);
      });
    }
  }

  if (docMain) {
    docMain.addEventListener('scroll', updateActiveHeading, { passive: true });
  }
}
