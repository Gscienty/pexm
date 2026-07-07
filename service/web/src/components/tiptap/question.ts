import { Node, wrappingInputRule, mergeAttributes } from '@tiptap/core';
import { Plugin, PluginKey, Selection } from '@tiptap/pm/state';
import { Decoration, DecorationSet } from '@tiptap/pm/view';

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    question: {
      toggleQuestion: () => ReturnType;
    };
  }
}

const questionNumberingKey = new PluginKey('questionNumbering');

function computeQuestionLabels(doc: any): DecorationSet {
  const decorations: Decoration[] = [];
  const counters: number[] = [0];
  const stack: number[] = [];

  doc.descendants((node: any, pos: number) => {
    while (stack.length > 0 && pos >= stack[stack.length - 1]) {
      counters.pop();
      stack.pop();
    }

    if (node.type.name === 'question') {
      counters[counters.length - 1]++;
      const label = counters.join('.');
      decorations.push(
        Decoration.node(pos, pos + node.nodeSize, {
          'data-label': `第 ${label} 题`,
        }),
      );
      counters.push(0);
      stack.push(pos + node.nodeSize);
    }
  });

  return DecorationSet.create(doc, decorations);
}

export const Question = Node.create({
  name: 'question',
  group: 'block',
  content: '(paragraph|heading|bulletList|orderedList|uppercaseLetterList|blockquote|codeBlock|mathBlock|question)+',
  defining: true,
  isolating: true,

  parseHTML() {
    return [{ tag: 'div[data-type="question"]' }];
  },

  renderHTML({ HTMLAttributes }) {
    return [
      'div',
      mergeAttributes(HTMLAttributes, { 'data-type': 'question', class: 'question-block' }),
      0,
    ];
  },

  addCommands() {
    return {
      toggleQuestion:
        () =>
        ({ commands }) => {
          return commands.toggleWrap('question');
        },
    };
  },

  addKeyboardShortcuts() {
    return {
      'Mod-Shift-q': () => this.editor.commands.toggleQuestion(),
      Enter: () => {
        const { state, view } = this.editor;
        const { $from } = state.selection;

        if ($from.parent.type.name !== 'paragraph' || $from.parent.content.size > 0) {
          return false;
        }

        let questionDepth = -1;
        for (let d = $from.depth; d > 0; d--) {
          if ($from.node(d).type.name === 'question') {
            questionDepth = d;
            break;
          }
        }

        if (questionDepth === -1) return false;
        if ($from.depth !== questionDepth + 1) return false;

        const questionNode = $from.node(questionDepth);
        const questionStart = $from.before(questionDepth);
        const questionEnd = $from.after(questionDepth);
        const paraFrom = $from.before($from.depth);
        const paraTo = $from.after($from.depth);
        const paraSize = paraTo - paraFrom;

        const tr = state.tr;
        const paragraph = state.schema.nodes.paragraph.create();

        if (questionNode.childCount <= 1) {
          tr.replaceWith(questionStart, questionEnd, paragraph);
          tr.setSelection(
            Selection.near(tr.doc.resolve(questionStart + 1)),
          );
        } else {
          tr.delete(paraFrom, paraTo);
          const newQuestionEnd = questionEnd - paraSize;
          tr.insert(newQuestionEnd, paragraph);
          tr.setSelection(
            Selection.near(tr.doc.resolve(newQuestionEnd + 1)),
          );
        }

        view.dispatch(tr);
        return true;
      },
      Backspace: () => {
        const { state, view } = this.editor;
        const { $from } = state.selection;

        if (!state.selection.empty) return false;

        let questionDepth = -1;
        for (let d = $from.depth; d > 0; d--) {
          if ($from.node(d).type.name === 'question') {
            questionDepth = d;
            break;
          }
        }

        if (questionDepth === -1) return false;

        const questionNode = $from.node(questionDepth);

        if (questionNode.textContent.trim() !== '') return false;

        const questionStart = $from.before(questionDepth);
        const questionEnd = $from.after(questionDepth);
        const paragraph = state.schema.nodes.paragraph.create();

        const tr = state.tr;
        tr.replaceWith(questionStart, questionEnd, paragraph);
        tr.setSelection(
          Selection.near(tr.doc.resolve(questionStart + 1)),
        );
        view.dispatch(tr);
        return true;
      },
    };
  },

  addInputRules() {
    return [
      wrappingInputRule({
        find: /^(\.|。)\s$/,
        type: this.type,
        joinPredicate: () => false,
      }),
    ];
  },

  addProseMirrorPlugins() {
    return [
      new Plugin({
        key: questionNumberingKey,
        state: {
          init: (_, { doc }) => computeQuestionLabels(doc),
          apply: (tr, value) => {
            if (tr.docChanged) return computeQuestionLabels(tr.doc);
            return value;
          },
        },
        props: {
          decorations: (state) => questionNumberingKey.getState(state),
        },
      }),
    ];
  },
});
