import { Node, mergeAttributes, wrappingInputRule } from '@tiptap/core';
import { Selection } from '@tiptap/pm/state';

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    uppercaseLetterList: {
      toggleUppercaseLetterList: () => ReturnType;
    };
  }
}

export const UppercaseLetterList = Node.create({
  name: 'uppercaseLetterList',

  group: 'block',

  content: 'listItem+',

  priority: 1000,

  addAttributes() {
    return {
      order: {
        default: 1,
        parseHTML: (element) => {
          return element.hasAttribute('start')
            ? parseInt(element.getAttribute('start') || '1', 10)
            : 1;
        },
      },
    };
  },

  parseHTML() {
    return [{ tag: 'ol', priority: 1000 }];
  },

  renderHTML({ HTMLAttributes }) {
    return [
      'ol',
      mergeAttributes(HTMLAttributes, {
        'data-type': 'uppercase-letter-list',
        type: 'A',
      }),
      0,
    ];
  },

  addInputRules() {
    return [
      wrappingInputRule({
        find: /^([A-Za-z])[.。] $/,
        type: this.type,
        getAttributes: (match) => {
          const letter = match[1].toUpperCase();
          const order = letter.charCodeAt(0) - 64;
          return { order };
        },
      }),
    ];
  },

  addCommands() {
    return {
      toggleUppercaseLetterList:
        () =>
        ({ commands }) => {
          return commands.wrapIn(this.name);
        },
    };
  },

  addKeyboardShortcuts() {
    return {
      'Mod-Shift-a': () => this.editor.commands.toggleUppercaseLetterList(),
      Enter: () => {
        const { state, view } = this.editor;
        const { $from } = state.selection;

        if ($from.node($from.depth).type.name !== 'listItem') return false;

        const paragraph = $from.parent;
        if (paragraph.content.size > 0) {
          const isAtEnd = $from.parentOffset === paragraph.content.size;
          if (!isAtEnd) return false;

          const listItemStart = $from.before($from.depth);
          const contentAfterCursor = paragraph.content.cut($from.parentOffset);
          const newParagraph = state.schema.nodes.paragraph.create(null, contentAfterCursor);
          const newListItem = state.schema.nodes.listItem.create(null, newParagraph);

          const tr = state.tr;
          tr.delete($from.parentOffset, listItemStart + paragraph.nodeSize);
          tr.insert(listItemStart, newListItem);
          tr.setSelection(Selection.near(tr.doc.resolve(listItemStart + 1)));
          view.dispatch(tr);
          return true;
        }

        const listItemStart = $from.before($from.depth);
        const listItemEnd = $from.after($from.depth);
        const newParagraph = state.schema.nodes.paragraph.create();

        const tr = state.tr;
        tr.replaceWith(listItemStart, listItemEnd, newParagraph);
        tr.setSelection(Selection.near(tr.doc.resolve(listItemStart + 1)));
        view.dispatch(tr);
        return true;
      },
    };
  },
});