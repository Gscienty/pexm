import { Extension } from '@tiptap/core';

const DRAG_ICON = `<svg width="10" height="16" viewBox="0 0 10 16"><circle cx="2.5" cy="2.5" r="1.5" fill="currentColor"/><circle cx="7.5" cy="2.5" r="1.5" fill="currentColor"/><circle cx="2.5" cy="8" r="1.5" fill="currentColor"/><circle cx="7.5" cy="8" r="1.5" fill="currentColor"/><circle cx="2.5" cy="13.5" r="1.5" fill="currentColor"/><circle cx="7.5" cy="13.5" r="1.5" fill="currentColor"/></svg>`;

export const DragHandle = Extension.create({
  name: 'dragHandle',

  addStorage() {
    return {
      handle: null as HTMLDivElement | null,
      ghost: null as HTMLElement | null,
      placeholder: null as HTMLElement | null,
      dropIndicator: null as HTMLElement | null,
      blockPos: -1,
      isDragging: false,
      sourcePos: -1,
      targetPos: -1,
      insertBefore: true,
      sourceEl: null as HTMLElement | null,
      hideTimer: null as ReturnType<typeof setTimeout> | null,
      cleanupFns: [] as Array<() => void>,
    };
  },

  onCreate() {
    const editor = this.editor;
    const storage = this.storage;
    const editorEl = editor.view.dom as HTMLElement;

    const handle = document.createElement('div');
    handle.innerHTML = DRAG_ICON;
    Object.assign(handle.style, {
      position: 'fixed',
      display: 'none',
      alignItems: 'center',
      justifyContent: 'center',
      width: '24px',
      height: '24px',
      cursor: 'grab',
      zIndex: '100',
      userSelect: 'none',
      color: '#9b9b9b',
      borderRadius: '4px',
      transition: 'opacity 0.15s ease',
    });
    document.body.appendChild(handle);
    storage.handle = handle;

    const dropIndicator = document.createElement('div');
    Object.assign(dropIndicator.style, {
      position: 'fixed',
      display: 'none',
      height: '2px',
      background: '#4b8ce8',
      borderRadius: '1px',
      pointerEvents: 'none',
      zIndex: '999',
      opacity: '0',
      transition: 'opacity 0.15s ease, top 0.15s ease, left 0.15s ease',
    });
    document.body.appendChild(dropIndicator);
    storage.dropIndicator = dropIndicator;

    const findTopLevelBlock = (el: HTMLElement): HTMLElement | null => {
      let current = el;
      while (current && current.parentElement && current.parentElement !== editorEl) {
        current = current.parentElement;
      }
      return current && current.parentElement === editorEl ? current : null;
    };

    const getBlockPos = (blockEl: HTMLElement): number => {
      try {
        const pos = editor.view.posAtDOM(blockEl, 0);
        return pos - 1;
      } catch {
        return -1;
      }
    };

    const positionHandle = (blockEl: HTMLElement) => {
      const rect = blockEl.getBoundingClientRect();
      const style = window.getComputedStyle(blockEl);
      const lineHeight = parseFloat(style.lineHeight) || parseFloat(style.fontSize) * 1.5;
      handle.style.display = 'flex';
      handle.style.opacity = '1';
      handle.style.left = `${rect.left - 28}px`;
      handle.style.top = `${rect.top + (lineHeight - 24) / 2}px`;
    };

    const hideHandle = () => {
      handle.style.opacity = '0';
      if (storage.hideTimer) clearTimeout(storage.hideTimer);
      storage.hideTimer = setTimeout(() => {
        if (handle.style.opacity === '0') {
          handle.style.display = 'none';
        }
      }, 150);
    };

    const findDropTarget = (clientY: number): { el: HTMLElement; pos: number; insertBefore: boolean } | null => {
      const children = Array.from(editorEl.children) as HTMLElement[];
      const validChildren = children.filter((c) => c !== storage.placeholder);
      if (validChildren.length === 0) return null;

      for (const child of validChildren) {
        const rect = child.getBoundingClientRect();
        const pos = getBlockPos(child);
        if (pos === -1) continue;

        if (clientY < rect.top) {
          return { el: child, pos, insertBefore: true };
        }

        if (clientY >= rect.top && clientY <= rect.bottom) {
          return { el: child, pos, insertBefore: clientY < rect.top + rect.height / 2 };
        }
      }

      const last = validChildren[validChildren.length - 1];
      const lastPos = getBlockPos(last);
      if (lastPos !== -1) {
        return { el: last, pos: lastPos, insertBefore: false };
      }

      return null;
    };

    const createGhost = (sourceEl: HTMLElement) => {
      const clone = sourceEl.cloneNode(true) as HTMLElement;
      const sourceRect = sourceEl.getBoundingClientRect();

      Object.assign(clone.style, {
        position: 'fixed',
        left: `${sourceRect.left}px`,
        top: `${sourceRect.top}px`,
        width: `${sourceRect.width}px`,
        opacity: '0.7',
        pointerEvents: 'none',
        zIndex: '1000',
        background: '#ffffff',
        boxShadow: '0 4px 12px rgba(0,0,0,0.12)',
        borderRadius: '4px',
        transition: 'top 0.15s ease, left 0.15s ease',
        margin: '0',
      });

      document.body.appendChild(clone);
      storage.ghost = clone;
    };

    const snapGhostToTarget = (targetEl: HTMLElement, insertBefore: boolean) => {
      if (!storage.ghost) return;
      const phRect = storage.placeholder?.getBoundingClientRect();
      if (!phRect) return;
      storage.ghost.style.top = `${phRect.top}px`;
    };

    const createPlaceholder = (sourceEl: HTMLElement) => {
      const ph = document.createElement('div');
      const sourceRect = sourceEl.getBoundingClientRect();

      Object.assign(ph.style, {
        height: `${sourceRect.height}px`,
        background: 'rgba(55, 53, 47, 0.06)',
        borderRadius: '4px',
        margin: sourceEl.tagName === 'H1' || sourceEl.tagName === 'H2' || sourceEl.tagName === 'H3'
          ? '0.5em 0'
          : '0.25em 0',
      });

      return ph;
    };

    const onEditorMove = (e: MouseEvent) => {
      if (storage.isDragging || !editor.isEditable) return;

      const target = e.target as HTMLElement;
      const blockEl = findTopLevelBlock(target);
      if (!blockEl) {
        hideHandle();
        return;
      }

      const pos = getBlockPos(blockEl);
      if (pos === -1) {
        hideHandle();
        return;
      }

      storage.blockPos = pos;
      positionHandle(blockEl);
    };

    const onEditorLeave = () => {
      if (!storage.isDragging) {
        hideHandle();
      }
    };

    const onHandleEnter = () => {
      if (storage.hideTimer) clearTimeout(storage.hideTimer);
      handle.style.opacity = '1';
    };

    const onHandleLeave = () => {
      if (!storage.isDragging) {
        hideHandle();
      }
    };

    editorEl.addEventListener('mousemove', onEditorMove);
    editorEl.addEventListener('mouseleave', onEditorLeave);
    handle.addEventListener('mouseenter', onHandleEnter);
    handle.addEventListener('mouseleave', onHandleLeave);

    storage.cleanupFns.push(() => {
      editorEl.removeEventListener('mousemove', onEditorMove);
      editorEl.removeEventListener('mouseleave', onEditorLeave);
      handle.removeEventListener('mouseenter', onHandleEnter);
      handle.removeEventListener('mouseleave', onHandleLeave);
    });

    const onHandleMouseDown = (e: MouseEvent) => {
      e.preventDefault();
      e.stopPropagation();

      if (storage.blockPos === -1) return;

      const sourceBlock = findTopLevelBlockByPos(storage.blockPos);
      if (!sourceBlock) return;

      storage.isDragging = true;
      storage.sourcePos = storage.blockPos;
      storage.sourceEl = sourceBlock;
      storage.targetPos = -1;
      handle.style.cursor = 'grabbing';
      handle.style.display = 'none';

      sourceBlock.style.opacity = '0.3';

      createGhost(sourceBlock);

      const onMouseMove = (e: MouseEvent) => {
        const target = findDropTarget(e.clientY);
        if (!target || target.pos === storage.sourcePos) {
          removePlaceholder();
          hideDropIndicator();
          storage.targetPos = -1;
          if (storage.ghost) {
            storage.ghost.style.transition = 'top 0.15s ease, left 0.15s ease';
            storage.ghost.style.opacity = '0.7';
          }
          return;
        }

        storage.targetPos = target.pos;
        storage.insertBefore = target.insertBefore;

        showDropIndicator(target.el, target.insertBefore);

        const ph = createPlaceholder(sourceBlock);
        removePlaceholder();
        storage.placeholder = ph;

        if (target.insertBefore) {
          editorEl.insertBefore(ph, target.el);
        } else {
          const next = target.el.nextElementSibling;
          if (next) {
            editorEl.insertBefore(ph, next);
          } else {
            editorEl.appendChild(ph);
          }
        }

        snapGhostToTarget(target.el, target.insertBefore);
      };

      const onMouseUp = () => {
        document.removeEventListener('mousemove', onMouseMove);
        document.removeEventListener('mouseup', onMouseUp);

        hideDropIndicator();

        if (storage.sourceEl) {
          storage.sourceEl.style.opacity = '';
        }

        if (storage.ghost) {
          storage.ghost.remove();
          storage.ghost = null;
        }

        removePlaceholder();

        storage.isDragging = false;
        handle.style.cursor = 'grab';

        if (storage.targetPos !== -1 && storage.sourcePos !== -1) {
          moveBlock(editor, storage.sourcePos, storage.targetPos, storage.insertBefore);
        }

        storage.sourcePos = -1;
        storage.targetPos = -1;
        storage.sourceEl = null;
        handle.style.display = 'none';
      };

      document.addEventListener('mousemove', onMouseMove);
      document.addEventListener('mouseup', onMouseUp);
    };

    const findTopLevelBlockByPos = (pos: number): HTMLElement | null => {
      const children = Array.from(editorEl.children) as HTMLElement[];
      for (const child of children) {
        const childPos = getBlockPos(child);
        if (childPos === pos) return child;
      }
      return null;
    };

    const removePlaceholder = () => {
      if (storage.placeholder && storage.placeholder.parentElement) {
        storage.placeholder.remove();
      }
      storage.placeholder = null;
    };

    const showDropIndicator = (targetEl: HTMLElement, insertBefore: boolean) => {
      const rect = targetEl.getBoundingClientRect();
      const editorRect = editorEl.getBoundingClientRect();
      const y = insertBefore ? rect.top : rect.bottom;
      dropIndicator.style.display = 'block';
      dropIndicator.style.left = `${editorRect.left}px`;
      dropIndicator.style.width = `${editorRect.width}px`;
      dropIndicator.style.top = `${y}px`;
      dropIndicator.style.opacity = '1';
    };

    const hideDropIndicator = () => {
      dropIndicator.style.opacity = '0';
      setTimeout(() => {
        if (dropIndicator.style.opacity === '0') {
          dropIndicator.style.display = 'none';
        }
      }, 150);
    };

    handle.addEventListener('mousedown', onHandleMouseDown);
    storage.cleanupFns.push(() => {
      handle.removeEventListener('mousedown', onHandleMouseDown);
    });
  },

  onDestroy() {
    const storage = this.storage;
    if (storage.hideTimer) clearTimeout(storage.hideTimer);
    storage.cleanupFns.forEach((fn: () => void) => fn());
    storage.handle?.remove();
    storage.ghost?.remove();
    storage.placeholder?.remove();
    storage.dropIndicator?.remove();
  },
});

function moveBlock(
  editor: { state: any; view: any },
  sourcePos: number,
  targetPos: number,
  insertBefore: boolean,
) {
  const { state, view } = editor;
  const { tr } = state;

  const sourceNode = tr.doc.nodeAt(sourcePos);
  if (!sourceNode) return;

  const sourceEnd = sourcePos + sourceNode.nodeSize;

  let insertPos: number;
  if (insertBefore) {
    insertPos = targetPos;
  } else {
    const targetNode = tr.doc.nodeAt(targetPos);
    if (!targetNode) return;
    insertPos = targetPos + targetNode.nodeSize;
  }

  if (sourcePos === insertPos || sourceEnd === insertPos) return;

  const nodeCopy = sourceNode.copy(sourceNode.content);

  if (sourcePos < insertPos) {
    tr.insert(insertPos, nodeCopy);
    tr.delete(sourcePos, sourceEnd);
  } else {
    tr.delete(sourcePos, sourceEnd);
    const mappedInsertPos = tr.mapping.map(insertPos);
    tr.insert(mappedInsertPos, nodeCopy);
  }

  view.dispatch(tr);
}
