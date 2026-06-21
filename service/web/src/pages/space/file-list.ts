import { deleteDirectory, deletePaper, renamePaper } from './api';
import { showRenameModal, showRenamePaperModal } from './modals';

const spaceToken = () => window.location.pathname.split('/').pop() || '';

function formatDate(str: string) {
	if (!str) return '';
	const d = new Date(str);
	return `${d.getFullYear()}-${String(d.getMonth()+1).padStart(2,'0')}-${String(d.getDate()).padStart(2,'0')}`;
}

function escapeHtml(str: string) {
	const div = document.createElement('div');
	div.textContent = str;
	return div.innerHTML;
}

let onRefresh: (() => void) | null = null;
export function setOnRefresh(fn: () => void) { onRefresh = fn; }

export function renderFileList(directories: any[], papers: any[]) {
	const fileList = document.getElementById('fileList')!;
	const hasContent = directories.length > 0 || papers.length > 0;

	if (!hasContent) {
		fileList.innerHTML = `
			<div class="empty-tab">
				<div class="empty-icon">📂</div>
				<p class="empty-title">此目录为空</p>
				<p class="empty-desc">新建目录或试卷开始组织内容</p>
			</div>`;
		return;
	}

	let html = '<div class="file-table">';
	for (const dir of directories) {
		html += `
			<div class="file-row dir-row" data-dir-id="${dir.id}">
				<div class="file-icon">
					<svg width="16" height="16" viewBox="0 0 24 24" fill="#8CA5C8" stroke="none"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
				</div>
				<div class="file-name">
					<a href="#" class="dir-link" data-dir-id="${dir.id}" data-dir-name="${escapeHtml(dir.name)}">${escapeHtml(dir.name)}</a>
				</div>
				<div class="file-date">${formatDate(dir.updatedAt)}</div>
				<div class="file-actions">
					<button class="file-action-btn" data-action="rename-dir" data-id="${dir.id}" data-name="${escapeHtml(dir.name)}" title="重命名">
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
					</button>
					<button class="file-action-btn" data-action="delete-dir" data-id="${dir.id}" title="删除目录">
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
					</button>
				</div>
			</div>`;
	}
	for (const paper of papers) {
		html += `
			<div class="file-row paper-row" data-paper-id="${paper.id}">
				<div class="file-icon">
					<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#5B8EDE" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
				</div>
				<div class="file-name">
					<a href="/doc/${paper.id}?space=${encodeURIComponent(spaceToken())}" class="paper-link">${escapeHtml(paper.title)}</a>
				</div>
				<div class="file-date">${formatDate(paper.updatedAt)}</div>
				<div class="file-actions">
					<button class="file-action-btn" data-action="rename-paper" data-id="${paper.id}" data-name="${escapeHtml(paper.title)}" title="重命名">
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
					</button>
					<button class="file-action-btn" data-action="delete-paper" data-id="${paper.id}" title="删除试卷">
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
					</button>
				</div>
			</div>`;
	}
	html += '</div>';
	fileList.innerHTML = html;

	// Bind directory click events
	fileList.querySelectorAll('.dir-link').forEach(el => {
		el.addEventListener('click', (e) => {
			e.preventDefault();
			const dirId = parseInt((el as HTMLElement).dataset.dirId || '0');
			const dirName = (el as HTMLElement).dataset.dirName || '';
			// Dispatch custom event for main module to handle
			fileList.dispatchEvent(new CustomEvent('navigate-dir', { detail: { dirId, dirName } }));
		});
	});

	fileList.querySelectorAll('[data-action="rename-dir"]').forEach(btn => {
		btn.addEventListener('click', () => {
			const id = parseInt((btn as HTMLElement).dataset.id || '0');
			const currentName = (btn as HTMLElement).dataset.name || '';
			showRenameModal(id, currentName);
		});
	});

	fileList.querySelectorAll('[data-action="delete-dir"]').forEach(btn => {
		btn.addEventListener('click', async () => {
			const id = parseInt((btn as HTMLElement).dataset.id || '0');
			if (confirm('确定删除此目录？目录下的试卷将移至上级目录。')) {
				try {
					await deleteDirectory(id);
					onRefresh?.();
				} catch (e) {
					alert('删除失败: ' + (e as Error).message);
				}
			}
		});
	});

	fileList.querySelectorAll('[data-action="rename-paper"]').forEach(btn => {
		btn.addEventListener('click', () => {
			const id = parseInt((btn as HTMLElement).dataset.id || '0');
			const currentName = (btn as HTMLElement).dataset.name || '';
			showRenamePaperModal(id, currentName);
		});
	});

	fileList.querySelectorAll('[data-action="delete-paper"]').forEach(btn => {
		btn.addEventListener('click', async () => {
			const id = parseInt((btn as HTMLElement).dataset.id || '0');
			if (confirm('确定删除此试卷？')) {
				try {
					await deletePaper(id);
					onRefresh?.();
				} catch (e) {
					alert('删除失败: ' + (e as Error).message);
				}
			}
		});
	});
}
