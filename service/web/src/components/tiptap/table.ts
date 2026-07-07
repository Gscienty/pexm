import { Node, Extension, mergeAttributes, InputRule } from '@tiptap/core';
import { Plugin, PluginKey, TextSelection, NodeSelection } from '@tiptap/pm/state';
import {
  CellSelection,
  TableView,
  columnResizing,
  tableEditing,
  addColumnBefore as pmAddColumnBefore,
  addColumnAfter as pmAddColumnAfter,
  addRowBefore as pmAddRowBefore,
  addRowAfter as pmAddRowAfter,
  deleteColumn as pmDeleteColumn,
  deleteRow as pmDeleteRow,
  deleteTable as pmDeleteTable,
  mergeCells as pmMergeCells,
  splitCell as pmSplitCell,
  toggleHeaderRow as pmToggleHeaderRow,
  toggleHeaderColumn as pmToggleHeaderColumn,
  selectedRect,
  findTable,
  isInTable,
} from '@tiptap/pm/tables';

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    table: {
      insertTable: (options?: { rows?: number; cols?: number; withHeaderRow?: boolean }) => ReturnType;
      addColumnBefore: () => ReturnType;
      addColumnAfter: () => ReturnType;
      addRowBefore: () => ReturnType;
      addRowAfter: () => ReturnType;
      deleteColumn: () => ReturnType;
      deleteRow: () => ReturnType;
      deleteTable: () => ReturnType;
      mergeCells: () => ReturnType;
      splitCell: () => ReturnType;
      toggleHeaderRow: () => ReturnType;
      toggleHeaderColumn: () => ReturnType;
    };
  }
}

// --- helpers ---

function createTableNode(schema: any, rows: number, cols: number, withHeaderRow: boolean) {
  const { table, tableRow, tableCell, tableHeader } = schema.nodes;
  const rowNodes: any[] = [];
  if (withHeaderRow) {
    const cells = [];
    for (let c = 0; c < cols; c++) cells.push(tableHeader.createAndFill());
    rowNodes.push(tableRow.create(null, cells));
  }
  const startBody = withHeaderRow ? 1 : 0;
  for (let r = startBody; r < rows; r++) {
    const cells = [];
    for (let c = 0; c < cols; c++) cells.push(tableCell.createAndFill());
    rowNodes.push(tableRow.create(null, cells));
  }
  return table.create(null, rowNodes);
}

function activeCellNode(state: any): any | null {
  const { selection } = state;
  if (selection instanceof CellSelection) return selection.$anchorCell.nodeAfter;
  for (let d = selection.$from.depth; d > 0; d--) {
    const node = selection.$from.node(d);
    const role = node.type.spec.tableRole;
    if (role === 'cell' || role === 'header_cell') return node;
  }
  return null;
}

function canMerge(state: any): boolean {
  if (!(state.selection instanceof CellSelection)) return false;
  try {
    const rect = selectedRect(state);
    return (rect.right - rect.left) * (rect.bottom - rect.top) > 1;
  } catch {
    return false;
  }
}

function canSplit(state: any): boolean {
  const cell = activeCellNode(state);
  if (!cell) return false;
  return (cell.attrs.colspan || 1) > 1 || (cell.attrs.rowspan || 1) > 1;
}

function getActiveTable(state: any): { node: any; pos: number } | null {
  const { selection } = state;
  if (selection instanceof NodeSelection && selection.node.type.name === 'table') {
    return { node: selection.node, pos: selection.from };
  }
  const found = findTable(selection.$head) || findTable(selection.$from);
  return found ? { node: found.node, pos: found.pos } : null;
}

// Locate the tableRow node (and its DOM <tr>) whose bottom border the user is
// dragging. A cell with rowspan>1 has its visual bottom on a later row; we
// resolve the actual row node at that visual index by looking for a cell that
// *starts* on that row.
function findRowNodeAtRowIndex(view: any, tableEl: HTMLTableElement, rowIndex: number) {
  const row = tableEl.rows[rowIndex];
  if (!row) return null;
  // row.cells only contains <td>/<th> whose starting row is this row, so any of
  // them resolves to the same tableRow node at `rowIndex`.
  for (const cell of Array.from(row.cells)) {
    const cellPos = view.posAtDOM(cell);
    const $cell = view.state.doc.resolve(cellPos);
    for (let d = $cell.depth; d > 0; d--) {
      const node = $cell.node(d);
      if (node.type.name === 'tableRow') {
        return { pos: $cell.before(d), node, rowEl: row };
      }
    }
  }
  return null;
}

// --- node definitions ---

export const Table = Node.create({
  name: 'table',
  group: 'block',
  content: 'tableRow+',
  isolating: true,
  selectable: true,

  parseHTML() {
    return [{ tag: 'table' }];
  },

  renderHTML({ HTMLAttributes }) {
    return [
      'table',
      mergeAttributes(HTMLAttributes, { class: 'tiptap-table' }),
      ['tbody', 0],
    ];
  },
});

export const TableRow = Node.create({
  name: 'tableRow',
  content: '(tableCell | tableHeader)*',

  parseHTML() {
    return [{ tag: 'tr' }];
  },

  renderHTML({ HTMLAttributes }) {
    return ['tr', mergeAttributes(HTMLAttributes), 0];
  },

  addAttributes() {
    return {
      rowHeight: {
        default: null,
        parseHTML: (el: HTMLElement) => {
          const h = el.getAttribute('data-rowheight');
          return h ? Number(h) : null;
        },
        renderHTML: (attrs: any) => {
          if (!attrs.rowHeight) return {};
          return {
            'data-rowheight': attrs.rowHeight,
            style: `height: ${attrs.rowHeight}px;`,
          };
        },
      },
    };
  },
});

export const TableCell = Node.create({
  name: 'tableCell',
  content: 'block+',
  isolating: true,

  parseHTML() {
    return [{ tag: 'td' }];
  },

  renderHTML({ HTMLAttributes }) {
    return ['td', mergeAttributes(HTMLAttributes), 0];
  },

  addAttributes() {
    return {
      colspan: {
        default: 1,
        parseHTML: (el: HTMLElement) => Number(el.getAttribute('colspan') || 1),
      },
      rowspan: {
        default: 1,
        parseHTML: (el: HTMLElement) => Number(el.getAttribute('rowspan') || 1),
      },
      colwidth: {
        default: null,
        parseHTML: (el: HTMLElement) => {
          const w = el.getAttribute('data-colwidth');
          return w ? w.split(',').map((s) => Number(s)) : null;
        },
        renderHTML: (attrs: any) =>
          attrs.colwidth ? { 'data-colwidth': attrs.colwidth.join(',') } : {},
      },
      background: {
        default: null,
        parseHTML: (el: HTMLElement) => el.style.backgroundColor || null,
        renderHTML: (attrs: any) =>
          attrs.background ? { style: `background-color: ${attrs.background};` } : {},
      },
    };
  },
});

export const TableHeader = Node.create({
  name: 'tableHeader',
  content: 'block+',
  isolating: true,

  parseHTML() {
    return [{ tag: 'th' }];
  },

  renderHTML({ HTMLAttributes }) {
    return ['th', mergeAttributes(HTMLAttributes), 0];
  },

  addAttributes() {
    return {
      colspan: {
        default: 1,
        parseHTML: (el: HTMLElement) => Number(el.getAttribute('colspan') || 1),
      },
      rowspan: {
        default: 1,
        parseHTML: (el: HTMLElement) => Number(el.getAttribute('rowspan') || 1),
      },
      colwidth: {
        default: null,
        parseHTML: (el: HTMLElement) => {
          const w = el.getAttribute('data-colwidth');
          return w ? w.split(',').map((s) => Number(s)) : null;
        },
        renderHTML: (attrs: any) =>
          attrs.colwidth ? { 'data-colwidth': attrs.colwidth.join(',') } : {},
      },
      background: {
        default: null,
        parseHTML: (el: HTMLElement) => el.style.backgroundColor || null,
        renderHTML: (attrs: any) =>
          attrs.background ? { style: `background-color: ${attrs.background};` } : {},
      },
    };
  },
});

// --- row height resizing plugin ---

const rowResizeKey = new PluginKey('tableRowResize');
const ROW_HANDLE = 5; // px from bottom edge that triggers a row drag

function rowResizingPlugin() {
  return new Plugin({
    key: rowResizeKey,
    props: {
      handleDOMEvents: {
        mousedown: (view: any, event: MouseEvent) => {
          const target = event.target as HTMLElement | null;
          const cell = target?.closest?.('td, th') as HTMLElement | null;
          if (!cell) return false;
          const tableEl = cell.closest('table') as HTMLElement | null;
          if (!tableEl || !view.dom.contains(tableEl)) return false;

          const rect = cell.getBoundingClientRect();
          const offsetY = event.clientY - rect.bottom;
          const offsetXFromRight = rect.right - event.clientX;
          // Only the bottom edge zone; leave the bottom-right corner to column resize.
          if (offsetY > ROW_HANDLE || offsetY < -ROW_HANDLE) return false;
          if (offsetXFromRight < 6) return false;

          const trEl = cell.parentElement as HTMLTableRowElement | null;
          if (!trEl) return false;
          const targetRowIndex = trEl.rowIndex + ((cell as HTMLTableCellElement).rowSpan || 1) - 1;
          const info = findRowNodeAtRowIndex(view, tableEl as HTMLTableElement, targetRowIndex);
          if (!info) return false;

          event.preventDefault();
          event.stopPropagation();

          const startY = event.clientY;
          const startHeight = info.rowEl.getBoundingClientRect().height;
          const prevBodyCursor = document.body.style.cursor;
          document.body.style.cursor = 'ns-resize';

          const onMove = (e: MouseEvent) => {
            const delta = e.clientY - startY;
            const h = Math.max(20, Math.round(startHeight + delta));
            info.rowEl.style.height = `${h}px`;
          };
          const onUp = (e: MouseEvent) => {
            document.removeEventListener('mousemove', onMove);
            document.removeEventListener('mouseup', onUp);
            document.body.style.cursor = prevBodyCursor;
            const delta = e.clientY - startY;
            const h = Math.max(20, Math.round(startHeight + delta));
            info.rowEl.style.height = '';
            const tr = view.state.tr;
            const newAttrs = { ...info.node.attrs, rowHeight: h };
            tr.setNodeMarkup(info.pos, undefined, newAttrs);
            tr.setMeta('addToHistory', true);
            view.dispatch(tr);
          };
          document.addEventListener('mousemove', onMove);
          document.addEventListener('mouseup', onUp);
          return true;
        },
      },
    },
  });
}

// --- floating toolbar ---

class TableToolbarView {
  private view: any;
  private editor: any;
  private toolbar: HTMLElement;
  private onScroll: () => void;

  constructor(view: any, editor: any) {
    this.view = view;
    this.editor = editor;
    this.toolbar = document.createElement('div');
    this.toolbar.className = 'tiptap-table-toolbar';
    this.toolbar.style.cssText =
      'position:fixed;display:none;z-index:1000;gap:2px;padding:4px;background:#fff;border:1px solid #e7e5e4;border-radius:8px;box-shadow:0 2px 8px rgba(0,0,0,0.12);flex-wrap:wrap;max-width:520px;';
    document.body.appendChild(this.toolbar);
    this.buildButtons();
    this.update(view);

    this.onScroll = () => this.update(this.view);
    window.addEventListener('scroll', this.onScroll, true);
    window.addEventListener('resize', this.onScroll);
  }

  private btn(label: string, title: string, cmd: () => void, enabled: (s: any) => boolean, groupEnd = false) {
    const b = document.createElement('button');
    b.type = 'button';
    b.textContent = label;
    b.title = title;
    b.className = 'tiptap-table-toolbar-btn' + (groupEnd ? ' tiptap-table-toolbar-btn-group' : '');
    b.style.cssText =
      'appearance:none;border:none;background:#f5f5f4;color:#1c1917;padding:3px 7px;border-radius:5px;font-size:12px;cursor:pointer;line-height:1.4;';
    b.addEventListener('mousedown', (e) => e.preventDefault());
    b.addEventListener('click', () => cmd());
    (b as any)._enabled = enabled;
    this.toolbar.appendChild(b);
    return b;
  }

  private buildButtons() {
    const run = (name: string) => () => this.editor.chain().focus()[name]().run();
    this.btn('↑行', '上方插入行', run('addRowBefore'), () => true);
    this.btn('↓行', '下方插入行', run('addRowAfter'), () => true, true);
    this.btn('←列', '左侧插入列', run('addColumnBefore'), () => true);
    this.btn('→列', '右侧插入列', run('addColumnAfter'), () => true, true);
    this.btn('合并', '合并单元格', run('mergeCells'), (s) => canMerge(s));
    this.btn('拆分', '拆分单元格', run('splitCell'), (s) => canSplit(s), true);
    this.btn('表头行', '切换表头行', run('toggleHeaderRow'), () => true);
    this.btn('表头列', '切换表头列', run('toggleHeaderColumn'), () => true, true);
    this.btn('删行', '删除行', run('deleteRow'), () => true);
    this.btn('删列', '删除列', run('deleteColumn'), () => true);
    this.btn('删表', '删除表格', run('deleteTable'), () => true);
  }

  update(view: any) {
    this.view = view;
    const state = view.state;
    const active = getActiveTable(state) || isInTable(state);
    if (!active) {
      this.toolbar.style.display = 'none';
      return;
    }

    // button enablement
    const buttons = this.toolbar.querySelectorAll('button');
    buttons.forEach((b: any) => {
      const en = b._enabled ? b._enabled(state) : true;
      b.disabled = !en;
      b.style.opacity = en ? '1' : '0.4';
      b.style.cursor = en ? 'pointer' : 'not-allowed';
    });

    // position near the table
    const table = getActiveTable(state);
    if (!table) {
      this.toolbar.style.display = 'none';
      return;
    }
    const dom = view.nodeDOM(table.pos) as HTMLElement | null;
    if (!dom || dom.nodeType !== 1) {
      this.toolbar.style.display = 'none';
      return;
    }
    const rect = dom.getBoundingClientRect();
    this.toolbar.style.display = 'flex';
    const tbH = this.toolbar.offsetHeight || 32;
    let top = rect.top - tbH - 6;
    if (top < 4) top = rect.bottom + 6;
    let left = rect.left;
    const maxLeft = window.innerWidth - this.toolbar.offsetWidth - 8;
    if (left > maxLeft) left = Math.max(8, maxLeft);
    this.toolbar.style.top = `${top}px`;
    this.toolbar.style.left = `${left}px`;
  }

  destroy() {
    window.removeEventListener('scroll', this.onScroll, true);
    window.removeEventListener('resize', this.onScroll);
    this.toolbar.remove();
  }
}

function tableToolbarPlugin(editor: any) {
  return new Plugin({
    key: new PluginKey('tableToolbar'),
    view(view: any) {
      return new TableToolbarView(view, editor);
    },
  });
}

// --- styled table node view (adds a class for CSS targeting) ---

class StyledTableView extends TableView {
  constructor(node: any, defaultCellMinWidth: number, _view?: any) {
    super(node, defaultCellMinWidth);
    if (this.table) this.table.classList.add('tiptap-table');
  }
}

// --- main extension: commands + plugins + input rules ---

export const TableExtensions = Extension.create({
  name: 'tableExtensions',

  // extendNodeSchema is invoked for EVERY node in the schema. Each of the 4
  // table node extensions must only contribute its own tableRole, not override
  // every other node's role. Centralizing here and dispatching by extension
  // name avoids the bug where a per-node extendNodeSchema (which always returns
  // a fixed object regardless of which node is being built) clobbers all other
  // nodes with the same role.
  extendNodeSchema(extension) {
    const role: Record<string, string> = {
      table: 'table',
      tableRow: 'row',
      tableCell: 'cell',
      tableHeader: 'header_cell',
    };
    return role[extension.name] ? { tableRole: role[extension.name] } : {};
  },

  addProseMirrorPlugins() {
    // rowResizing must come first so that when a row-drag starts (returns true),
    // tableEditing's cell-selection mousedown handler never runs (someProp stops
    // at the first truthy handler), avoiding a cell selection during row resize.
    return [
      rowResizingPlugin(),
      columnResizing({
        cellMinWidth: 60,
        defaultCellMinWidth: 100,
        View: StyledTableView as any,
      }),
      tableEditing(),
      tableToolbarPlugin(this.editor),
    ];
  },

  addCommands() {
    return {
      insertTable:
        (options) =>
        ({ tr, dispatch, state }) => {
          const { rows = 3, cols = 3, withHeaderRow = true } = options || {};
          const tableNode = createTableNode(state.schema, rows, cols, withHeaderRow);
          const from = state.selection.from;
          tr.replaceSelectionWith(tableNode);
          // cursor into the first cell's first paragraph
          tr.setSelection(TextSelection.near(tr.doc.resolve(from + 3)));
          if (dispatch) dispatch(tr.scrollIntoView());
          return true;
        },
      addColumnBefore: () => ({ state, dispatch }) => pmAddColumnBefore(state, dispatch),
      addColumnAfter: () => ({ state, dispatch }) => pmAddColumnAfter(state, dispatch),
      addRowBefore: () => ({ state, dispatch }) => pmAddRowBefore(state, dispatch),
      addRowAfter: () => ({ state, dispatch }) => pmAddRowAfter(state, dispatch),
      deleteColumn: () => ({ state, dispatch }) => pmDeleteColumn(state, dispatch),
      deleteRow: () => ({ state, dispatch }) => pmDeleteRow(state, dispatch),
      deleteTable: () => ({ state, dispatch }) => pmDeleteTable(state, dispatch),
      mergeCells: () => ({ state, dispatch }) => pmMergeCells(state, dispatch),
      splitCell: () => ({ state, dispatch }) => pmSplitCell(state, dispatch),
      toggleHeaderRow: () => ({ state, dispatch }) => pmToggleHeaderRow(state, dispatch),
      toggleHeaderColumn: () => ({ state, dispatch }) => pmToggleHeaderColumn(state, dispatch),
    };
  },

  addInputRules() {
    return [
      // `||| ` at line start -> 2x2 table (header + body)
      new InputRule({
        find: /^\|\|\|\s$/,
        handler: ({ state, range }) => {
          const tableNode = createTableNode(state.schema, 2, 2, true);
          const tr = state.tr;
          tr.delete(range.from, range.to);
          tr.insert(range.from, tableNode);
          tr.setSelection(TextSelection.near(tr.doc.resolve(range.from + 3)));
        },
      }),
    ];
  },
});

// Bundle all table-related extensions as a single array for easy registration.
export const TableBlock = [Table, TableRow, TableCell, TableHeader, TableExtensions];
