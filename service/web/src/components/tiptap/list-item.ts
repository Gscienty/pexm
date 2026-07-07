import { ListItem } from '@tiptap/extension-list-item';
import { Plugin, PluginKey } from '@tiptap/pm/state';

const prefixSyncKey = new PluginKey('listItemPrefixSync');

/** 序号前缀类型，传递到后端用于区分渲染样式：
 *  - "number" → 1. 2. 3.   (orderedList)
 *  - "letter" → A. B. C.   (uppercaseLetterList)
 *  - "bullet"  → 无序号     (bulletList) */
type ListPrefix = 'number' | 'letter' | 'bullet';

const PREFIX_BY_PARENT: Record<string, ListPrefix> = {
  orderedList: 'number',
  uppercaseLetterList: 'letter',
  bulletList: 'bullet',
};

function expectedPrefix(parentTypeName: string): ListPrefix {
  return PREFIX_BY_PARENT[parentTypeName] ?? 'number';
}

export const PrefixedListItem = ListItem.extend({
  addAttributes() {
    return {
      ...this.parent?.(),
      prefix: {
        default: 'number' as ListPrefix,
        // 仅用于后端序列化，不渲染到 HTML
        renderHTML: () => null,
        parseHTML: () => 'number' as ListPrefix,
      },
    };
  },

  addProseMirrorPlugins() {
    const nodeName = this.name;
    return [
      new Plugin({
        key: prefixSyncKey,
        appendTransaction(transactions, _oldState, newState) {
          if (!transactions.some((t) => t.docChanged)) return null;

          const tr = newState.tr;
          let modified = false;

          newState.doc.descendants((node, pos) => {
            if (node.type.name !== nodeName) return;

            const $pos = newState.doc.resolve(pos + 1);
            const parent = $pos.parent;
            const want = expectedPrefix(parent.type.name);
            if (node.attrs.prefix !== want) {
              tr.setNodeMarkup(pos, undefined, {
                ...node.attrs,
                prefix: want,
              });
              modified = true;
            }
          });

          return modified ? tr : null;
        },
      }),
    ];
  },
});
