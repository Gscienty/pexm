import { Node, mergeAttributes, InputRule } from '@tiptap/core';
import katex from 'katex';

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    mathInline: {
      insertMathInline: (latex?: string) => ReturnType;
    };
  }
}

function renderInline(latex: string, target: HTMLElement) {
  try {
    katex.render(latex, target, {
      throwOnError: false,
      displayMode: false,
      output: 'html',
    });
  } catch {
    target.textContent = latex;
  }
}

export const MathInline = Node.create({
  name: 'mathInline',
  group: 'inline',
  inline: true,
  atom: true,

  addAttributes() {
    return {
      latex: {
        default: '',
      },
    };
  },

  parseHTML() {
    return [{ tag: 'span[data-type="math-inline"]' }];
  },

  renderHTML({ HTMLAttributes }) {
    return [
      'span',
      mergeAttributes(HTMLAttributes, {
        'data-type': 'math-inline',
        class: 'math-inline',
      }),
    ];
  },

  renderText({ node }) {
    return `$${node.attrs.latex}$`;
  },

  addNodeView() {
    return ({ node, view, getPos }) => {
      const dom = document.createElement('span');
      dom.className = 'math-inline';
      dom.setAttribute('data-type', 'math-inline');
      dom.contentEditable = 'false';

      const display = document.createElement('span');
      display.className = 'math-inline-display';
      dom.appendChild(display);

      const input = document.createElement('input');
      input.type = 'text';
      input.className = 'math-inline-input';
      input.style.display = 'none';
      dom.appendChild(input);

      // Hidden span used to measure the rendered text width so the input
      // can resize to fit its content.
      const measurer = document.createElement('span');
      measurer.style.cssText = 'visibility:hidden;position:absolute;white-space:pre;';
      dom.appendChild(measurer);

      const render = () => renderInline(node.attrs.latex, display);

      const resize = () => {
        measurer.style.font = window.getComputedStyle(input).font;
        measurer.textContent = input.value || ' ';
        input.style.width = `${measurer.offsetWidth + 8}px`;
      };

      const enterEdit = () => {
        if (input.style.display !== 'none') return;
        input.value = node.attrs.latex;
        display.style.display = 'none';
        input.style.display = 'inline-block';
        resize();
        input.focus();
        input.select();
      };

      const commit = () => {
        if (input.style.display === 'none') return;
        const next = input.value;
        input.style.display = 'none';
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
        if (input.style.display === 'none') return;
        input.style.display = 'none';
        display.style.display = '';
        render();
      };

      render();

      // Single click enters edit mode. We stop propagation so PM doesn't
      // select the atom node (which would trigger selectNode/deselectNode
      // around the focus change and cause a visible flicker).
      dom.addEventListener('click', (e) => {
        if (input.style.display !== 'none') return;
        e.preventDefault();
        e.stopPropagation();
        enterEdit();
      });

      input.addEventListener('blur', commit);
      input.addEventListener('input', resize);
      input.addEventListener('keydown', (e) => {
        // Stop ProseMirror from handling keystrokes while editing the input
        e.stopPropagation();
        if (e.key === 'Enter') {
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
            if (input.style.display === 'none') render();
          }
          return true;
        },
        // Let ProseMirror ignore events originating from the input so that
        // text editing (backspace, arrows, etc.) behaves normally.
        stopEvent: (event) => event.target === input,
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
      insertMathInline:
        (latex = '') =>
        ({ commands }) =>
          commands.insertContent({ type: 'mathInline', attrs: { latex } }),
    };
  },

  addInputRules() {
    return [
      // $...$ -> inline math. Negative lookbehind prevents matching the inner
      // $...$ of a $$...$$ block (which is handled by MathBlock's input rule).
      new InputRule({
        find: /(?<!\$)\$([^$\n]+)\$$/,
        handler: ({ state, range, match }) => {
          const latex = match[1];
          const node = state.schema.nodes.mathInline.create({ latex });
          const tr = state.tr;
          tr.delete(range.from, range.to);
          tr.insert(range.from, node);
        },
      }),
    ];
  },
});
