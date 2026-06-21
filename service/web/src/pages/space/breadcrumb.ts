import { renameDirectory } from './api';

export interface BreadcrumbItem {
	id: number;
	name: string;
}

let breadcrumbStack: BreadcrumbItem[] = [{ id: 0, name: '根目录' }];
let onNavigate: ((parentId: number) => void) | null = null;

export function getBreadcrumbStack() { return breadcrumbStack; }
export function setBreadcrumbStack(stack: BreadcrumbItem[]) { breadcrumbStack = stack; }
export function setOnNavigate(fn: (parentId: number) => void) { onNavigate = fn; }

function escapeHtml(str: string) {
	const div = document.createElement('div');
	div.textContent = str;
	return div.innerHTML;
}

export function renderBreadcrumb() {
	const container = document.getElementById('dirBreadcrumb')!;
	if (breadcrumbStack.length <= 1) {
		container.style.display = 'none';
		return;
	}
	container.style.display = 'flex';

	const maxVisible = 5;
	let items = breadcrumbStack;
	let collapsed = false;
	let collapseStart = -1;
	let collapseEnd = -1;

	if (items.length > maxVisible + 1) {
		collapsed = true;
		collapseStart = 1;
		collapseEnd = items.length - 2;
	}

	let html = '';
	items.forEach((item, idx) => {
		const isLast = idx === items.length - 1;

		if (collapsed && idx >= collapseStart && idx < collapseEnd) {
			if (idx === collapseStart) {
				html += `<span class="dir-ellipsis" title="${items.slice(collapseStart, collapseEnd).map(i => escapeHtml(i.name)).join(' / ')}">...</span><span class="dir-sep">/</span>`;
			}
			return;
		}

		if (isLast) {
			html += `<span class="dir-crumb current">${escapeHtml(item.name)}</span>`;
			if (item.id !== 0) {
				html += `<button class="dir-rename-btn" data-dir-id="${item.id}" data-name="${escapeHtml(item.name)}" title="重命名目录">
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
				</button>`;
			}
		} else {
			html += `<a href="#" class="dir-crumb" data-dir-id="${item.id}" data-crumb-idx="${idx}">${escapeHtml(item.name)}</a><span class="dir-sep">/</span>`;
		}
	});

	container.innerHTML = html;

	container.querySelectorAll('.dir-crumb[data-dir-id]').forEach(el => {
		el.addEventListener('click', (e) => {
			e.preventDefault();
			const crumbIdx = parseInt((el as HTMLElement).dataset.crumbIdx || '0');
			const dirId = parseInt((el as HTMLElement).dataset.dirId || '0');
			breadcrumbStack = breadcrumbStack.slice(0, crumbIdx + 1);
			onNavigate?.(dirId);
		});
	});

	container.querySelectorAll('.dir-ellipsis').forEach(el => {
		el.addEventListener('click', () => {
			collapsed = false;
			renderBreadcrumb();
		});
	});

	container.querySelectorAll('.dir-rename-btn').forEach(btn => {
		btn.addEventListener('click', () => {
			const dirId = parseInt((btn as HTMLElement).dataset.dirId || '0');
			const currentName = (btn as HTMLElement).dataset.name || '';
			showRenameModal(dirId, currentName, true);
		});
	});
}

// Inline rename modal (imported from modals to avoid circular dep)
let showRenameModal: (dirId: number, currentName: string, fromBreadcrumb?: boolean) => void = () => {};
export function setRenameModalFn(fn: typeof showRenameModal) { showRenameModal = fn; }
