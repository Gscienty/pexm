import { Node, mergeAttributes, InputRule } from '@tiptap/core';
import { TextSelection } from '@tiptap/pm/state';
import katex from 'katex';

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    mathBlock: {
      insertMathBlock: (latex?: string) => ReturnType;
    };
  }
}

function renderBlock(latex: string, target: HTMLElement) {
  target.innerHTML = '';
  if (!latex.trim()) {
    const ph = document.createElement('span');
    ph.className = 'math-block-placeholder';
    ph.textContent = '点击编辑 LaTeX 公式';
    target.appendChild(ph);
    return;
  }
  try {
    katex.render(latex, target, {
      throwOnError: false,
      displayMode: true,
      output: 'html',
    });
  } catch {
    target.textContent = latex;
  }
}

export const MathBlock = Node.create({
  name: 'mathBlock',
  group: 'block',
  atom: true,

  addAttributes() {
    return {
      latex: {
        default: '',
      },
    };
  },

  parseHTML() {
    return [{ tag: 'div[data-type="math-block"]' }];
  },

  renderHTML({ HTMLAttributes }) {
    return [
      'div',
      mergeAttributes(HTMLAttributes, {
        'data-type': 'math-block',
        class: 'math-block',
      }),
    ];
  },

  renderText({ node }) {
    return `$$${node.attrs.latex}$$`;
  },

  addNodeView() {
    return ({ node, view, getPos }) => {
      const dom = document.createElement('div');
      dom.className = 'math-block';
      dom.setAttribute('data-type', 'math-block');
      dom.contentEditable = 'false';

      const display = document.createElement('div');
      display.className = 'math-block-display';
      dom.appendChild(display);

      const textarea = document.createElement('textarea');
      textarea.className = 'math-block-textarea';
      textarea.rows = 3;
      textarea.style.display = 'none';
      dom.appendChild(textarea);

      const render = () => renderBlock(node.attrs.latex, display);

      const enterEdit = () => {
        textarea.value = node.attrs.latex;
        display.style.display = 'none';
        textarea.style.display = 'block';
        textarea.focus();
        textarea.select();
      };

      const commit = () => {
        const next = textarea.value;
        textarea.style.display = 'none';
        display.style.display = '';
        if (next !== node.attrs.latex) {
          const pos = getPos();
          if (typeof pos === 'number') {
            const tr = view.state.tr.setNodeMarkup(pos, undefined, { latex: next });
            view.dispatch(tr);
          }
        } else {
          render();
        }
      };

      const cancel = () => {
        textarea.style.display = 'none';
        display.style.display = '';
        render();
      };

      render();

      dom.addEventListener('click', (e) => {
        if (textarea.style.display !== 'none') return;
        e.preventDefault();
        e.stopPropagation();
        enterEdit();
      });

      textarea.addEventListener('blur', commit);
      textarea.addEventListener('keydown', (e) => {
        if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
          e.preventDefault();
          commit();
        } else if (e.key === 'Escape') {
          e.preventDefault();
          cancel();
        }
      });

      return {
        dom,
        update(updatedNode) {
          if (updatedNode.type !== node.type) return false;
          if (updatedNode.attrs.latex !== node.attrs.latex) {
            node = updatedNode;
            if (textarea.style.display === 'none') render();
          }
          return true;
        },
        selectNode() {
          dom.classList.add('ProseMirror-selectednode');
        },
        deselectNode() {
          dom.classList.remove('ProseMirror-selectednode');
        },
      };
    };
  },

  addCommands() {
    return {
      insertMathBlock:
        (latex = '') =>
        ({ commands }) =>
          commands.insertContent({ type: 'mathBlock', attrs: { latex } }),
    };
  },

  addInputRules() {
    return [
      // $$...$$ on a single line -> block math
      new InputRule({
        find: /\$\$([^$\n]+)\$\$$/,
        handler: ({ state, range, match }) => {
          const latex = match[1];
          const node = state.schema.nodes.mathBlock.create({ latex });
          const tr = state.tr;
          tr.delete(range.from, range.to);
          tr.insert(range.from, node);
        },
      }),
      // $$<space> at line start -> empty block math ready for editing
      new InputRule({
        find: /^\$\$\s$/,
        handler: ({ state, range }) => {
          const node = state.schema.nodes.mathBlock.create({ latex: '' });
          const tr = state.tr;
          tr.delete(range.from, range.to);
          tr.insert(range.from, node);
          tr.setSelection(TextSelection.near(tr.doc.resolve(range.from)));
        },
      }),
    ];
  },
});
