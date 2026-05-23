import { Extension } from '@tiptap/core';

function slugify(text: string): string {
  return text
    .trim()
    .toLowerCase()
    .replace(/\s+/g, '-')
    .replace(/[^\w\u4e00-\u9fff-]/g, '')
    .replace(/-+/g, '-')
    .replace(/^-|-$/g, '');
}

export interface HeadingInfo {
  id: string;
  text: string;
  level: number;
}

export const Anchor = Extension.create({
  name: 'anchor',

  onCreate(editor) {
    updateHeadingIds(editor.editor);
  },

  onUpdate(editor) {
    updateHeadingIds(editor.editor);
  },

  addGlobalAttributes() {
    return [
      {
        types: ['heading'],
        attributes: {
          id: {
            default: null,
            parseHTML: (element: HTMLElement) => element.getAttribute('id'),
            renderHTML: (attributes: Record<string, string>) => {
              if (!attributes.id) return {};
              return { id: attributes.id };
            },
          },
        },
      },
    ];
  },
});

function updateHeadingIds(editor: ReturnType<typeof Extension.create>['editor']) {
  const { state, view } = editor;
  const { tr } = state;
  let modified = false;
  const headings: HeadingInfo[] = [];
  const usedIds = new Set<string>();

  state.doc.descendants((node: { type: { name: string }; textContent: string; attrs: Record<string, unknown> }, pos: number) => {
    if (node.type.name === 'heading') {
      const text = node.textContent;
      let baseId = slugify(text) || `heading-${pos}`;
      let id = baseId;
      let counter = 1;
      while (usedIds.has(id)) {
        id = `${baseId}-${counter}`;
        counter++;
      }
      usedIds.add(id);

      if (node.attrs.id !== id) {
        tr.setNodeMarkup(pos, undefined, { ...node.attrs, id });
        modified = true;
      }

      headings.push({ id, text, level: node.attrs.level as number });
    }
  });

  if (modified && !view.composing) {
    view.dispatch(tr);
  }

  const wrapper = view.dom.closest('.tiptap-wrapper');
  if (wrapper) {
    wrapper.dispatchEvent(
      new CustomEvent('headings-change', {
        detail: { headings },
        bubbles: true,
      })
    );
  }
}
