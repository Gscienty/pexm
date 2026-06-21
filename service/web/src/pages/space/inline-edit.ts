import { updateExamSpace } from './api';

export function setupInlineEdit(getSpace: () => any) {
	// Double-click to edit title
	const nameEl = document.getElementById('spaceName')!;
	nameEl.title = '双击编辑';
	nameEl.style.cursor = 'text';
	nameEl.addEventListener('dblclick', () => {
		const current = nameEl.textContent || '';
		const input = document.createElement('input');
		input.type = 'text';
		input.value = current;
		input.className = 'inline-edit-input';
		nameEl.replaceWith(input);
		input.focus();
		input.select();

		const save = async () => {
			const val = input.value.trim();
			const space = getSpace();
			if (val && val !== current) {
				try {
					await updateExamSpace(space.id, val, space.description || '');
					space.name = val;
					nameEl.textContent = val;
					document.getElementById('breadcrumbName')!.textContent = val;
					document.title = `${val} - 笔阵`;
					(document.getElementById('editName') as HTMLInputElement).value = val;
				} catch (e) {
					alert('修改失败: ' + (e as Error).message);
					nameEl.textContent = current;
				}
			} else {
				nameEl.textContent = current;
			}
			input.replaceWith(nameEl);
		};
		input.addEventListener('blur', save);
		input.addEventListener('keydown', (e) => {
			if (e.key === 'Enter') input.blur();
			if (e.key === 'Escape') { input.value = current; input.blur(); }
		});
	});

	// Double-click to edit description
	const descEl = document.getElementById('spaceDesc')!;
	descEl.title = '双击编辑';
	descEl.style.cursor = 'text';
	descEl.addEventListener('dblclick', () => {
		const current = descEl.textContent || '';
		const input = document.createElement('input');
		input.type = 'text';
		input.value = current;
		input.className = 'inline-edit-input inline-edit-input-desc';
		descEl.replaceWith(input);
		input.focus();
		input.select();

		const save = async () => {
			const val = input.value.trim();
			const space = getSpace();
			if (val !== current) {
				try {
					await updateExamSpace(space.id, space.name, val);
					space.description = val;
					descEl.textContent = val;
					(document.getElementById('editDesc') as HTMLTextAreaElement).value = val;
				} catch (e) {
					alert('修改失败: ' + (e as Error).message);
					descEl.textContent = current;
				}
			} else {
				descEl.textContent = current;
			}
			input.replaceWith(descEl);
		};
		input.addEventListener('blur', save);
		input.addEventListener('keydown', (e) => {
			if (e.key === 'Enter') input.blur();
			if (e.key === 'Escape') { input.value = current; input.blur(); }
		});
	});
}
