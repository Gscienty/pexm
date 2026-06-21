import { renameDirectory, renamePaper, createDirectory, createPaper } from './api';
import { getBreadcrumbStack, renderBreadcrumb } from './breadcrumb';

function escapeHtml(str: string) {
	const div = document.createElement('div');
	div.textContent = str;
	return div.innerHTML;
}

let onRefresh: (() => void) | null = null;
export function setOnRefresh(fn: () => void) { onRefresh = fn; }

export function showRenameModal(dirId: number, currentName: string, fromBreadcrumb = false) {
	document.getElementById('createModal')?.remove();

	const modal = document.createElement('div');
	modal.id = 'createModal';
	modal.className = 'modal-overlay';
	modal.innerHTML = `
		<div class="modal">
			<div class="modal-body">
				<input type="text" id="renameInput" value="${escapeHtml(currentName)}" autofocus />
				<button class="modal-submit" id="modalSubmit">保存</button>
			</div>
		</div>`;
	document.body.appendChild(modal);

	const input = document.getElementById('renameInput') as HTMLInputElement;
	input.focus();
	input.select();

	const closeModal = () => modal.remove();
	modal.addEventListener('click', (e) => { if (e.target === modal) closeModal(); });

	document.getElementById('modalSubmit')?.addEventListener('click', async () => {
		const value = input.value.trim();
		if (!value) return;
		try {
			await renameDirectory(dirId, value);
			if (fromBreadcrumb) {
				const stack = getBreadcrumbStack();
				const item = stack.find(b => b.id === dirId);
				if (item) item.name = value;
				renderBreadcrumb();
			}
			closeModal();
			onRefresh?.();
		} catch (e) {
			alert('重命名失败: ' + (e as Error).message);
		}
	});

	input.addEventListener('keydown', (e) => {
		if (e.key === 'Enter') document.getElementById('modalSubmit')?.click();
		if (e.key === 'Escape') closeModal();
	});
}

export function showRenamePaperModal(paperId: number, currentTitle: string) {
	document.getElementById('createModal')?.remove();

	const modal = document.createElement('div');
	modal.id = 'createModal';
	modal.className = 'modal-overlay';
	modal.innerHTML = `
		<div class="modal">
			<div class="modal-body">
				<input type="text" id="renameInput" value="${escapeHtml(currentTitle)}" autofocus />
				<button class="modal-submit" id="modalSubmit">保存</button>
			</div>
		</div>`;
	document.body.appendChild(modal);

	const input = document.getElementById('renameInput') as HTMLInputElement;
	input.focus();
	input.select();

	const closeModal = () => modal.remove();
	modal.addEventListener('click', (e) => { if (e.target === modal) closeModal(); });

	document.getElementById('modalSubmit')?.addEventListener('click', async () => {
		const value = input.value.trim();
		if (!value) return;
		try {
			await renamePaper(paperId, value);
			closeModal();
			onRefresh?.();
		} catch (e) {
			alert('重命名失败: ' + (e as Error).message);
		}
	});

	input.addEventListener('keydown', (e) => {
		if (e.key === 'Enter') document.getElementById('modalSubmit')?.click();
		if (e.key === 'Escape') closeModal();
	});
}

export function showCreateModal(type: 'directory' | 'paper', currentSpaceId: number, currentParentId: number) {
	document.getElementById('createModal')?.remove();

	const placeholder = type === 'directory' ? '输入目录名称' : '输入试卷标题';
	const submitLabel = type === 'directory' ? '创建目录' : '创建试卷';
	const modal = document.createElement('div');
	modal.id = 'createModal';
	modal.className = 'modal-overlay';
	modal.innerHTML = `
		<div class="modal">
			<div class="modal-body">
				<input type="text" id="createInput" placeholder="${placeholder}" autofocus />
				<button class="modal-submit" id="modalSubmit">${submitLabel}</button>
			</div>
		</div>`;
	document.body.appendChild(modal);

	const input = document.getElementById('createInput') as HTMLInputElement;
	input.focus();

	const closeModal = () => modal.remove();
	modal.addEventListener('click', (e) => { if (e.target === modal) closeModal(); });

	document.getElementById('modalSubmit')?.addEventListener('click', async () => {
		const value = input.value.trim();
		if (!value) return;
		try {
			if (type === 'directory') {
				const parentId = currentParentId === 0 ? null : currentParentId;
				await createDirectory(currentSpaceId, parentId, value);
			} else {
				const directoryId = currentParentId === 0 ? null : currentParentId;
				await createPaper(currentSpaceId, directoryId, value);
			}
			closeModal();
			onRefresh?.();
		} catch (e) {
			alert('创建失败: ' + (e as Error).message);
		}
	});

	input.addEventListener('keydown', (e) => {
		if (e.key === 'Enter') document.getElementById('modalSubmit')?.click();
	});
}
