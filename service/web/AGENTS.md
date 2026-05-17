# web 服务

## 概述

基于 Astro 的前端应用，提供试卷编辑功能。核心是 Tiptap 富文本编辑器，通过 y-prosemirror + yjs 实现多人实时协作。

## 技术栈

- **框架**: Astro 4.16
- **编辑器**: Tiptap 3.23（基于 ProseMirror）
- **协作**: yjs 13.6 + y-prosemirror 1.3
- **包管理**: pnpm 9

## 项目结构

```
src/
├── pages/
│   └── doc/
│       └── [id].astro              # 文档编辑页面（核心页面）
├── components/
│   ├── TiptapEditor.astro          # Tiptap 编辑器组件（入口）
│   ├── console/
│   │   └── Sidebar.astro           # 控制台侧边栏
│   └── tiptap/
│       ├── collaboration.ts        # 协作扩展（集成 yjs + WebSocket）
│       ├── question.ts             # 题目节点（自定义 block 节点）
│       ├── anchor.ts               # 标题锚点（自动生成 heading id）
│       └── drag-handle.ts          # 拖拽手柄
└── lib/
    ├── protocol.ts                 # 消息协议编解码（与后端对应）
    └── ws-provider.ts              # WebSocket 连接管理（DocWebSocketProvider）
```

## 架构详解

### 1. 文档编辑页面（doc/[id].astro）

页面结构：
```
┌──────────────────────────────────────────────┐
│ Sidebar                                       │
├──────────┬────────────────────┬──────────────┤
│ 大纲     │ doc-title          │ 圈点留言     │
│ (outline)│ (contenteditable)  │ (annotations)│
│          │ TiptapEditor       │              │
│          │ (协作编辑器)       │              │
└──────────┴────────────────────┴──────────────┘
```

**重要**: `doc-title` 是一个独立的 `contenteditable` div，**不在 Tiptap 编辑器内**，因此**不参与实时同步**。它只更新页面标题和面包屑导航。

### 2. TiptapEditor 组件（TiptapEditor.astro）

编辑器初始化流程：
1. 创建 Tiptap Editor 实例
2. 加载扩展：`StarterKit`（基础功能）、`Anchor`（标题锚点）、`Question`（题目节点）、`DragHandle`（拖拽）
3. 如果有 `docName`，加载 `Collaboration` 扩展（启用实时协作）
4. 协作模式下 `content` 传 `undefined`（内容由 yjs 同步），非协作模式使用静态 HTML

### 3. 协作扩展（collaboration.ts）

```typescript
export const Collaboration = Extension.create({
  addStorage() {
    const doc = new Y.Doc();
    const yXmlFragment = doc.getXmlFragment('prosemirror');  // 根类型名固定为 'prosemirror'
    return { doc, provider: null, yXmlFragment };
  },
  onCreate() {
    // 创建 WebSocket 连接
    const provider = new DocWebSocketProvider(doc, docName, { baseUrl: wsUrl });
  },
  addProseMirrorPlugins() {
    return [ySyncPlugin(yXmlFragment), yUndoPlugin()];
  },
});
```

**关键点**:
- YDoc 的根 XmlFragment 名称固定为 `'prosemirror'`，对应后端 `Parent::String("prosemirror")`
- `ySyncPlugin` 绑定 ProseMirror 与 YXmlFragment，实现双向同步
- `yUndoPlugin` 提供基于 yjs 的撤销/重做

### 4. WebSocket Provider（ws-provider.ts）

`DocWebSocketProvider` 管理 WebSocket 连接和同步生命周期：

**连接流程**:
1. `connect()` 建立 WebSocket 连接到 `ws://host/api/websocket/doc/{roomName}`
2. `onopen` 时发送 Step1（携带本地 StateVector）
3. 收到 Step2 后 `Y.applyUpdate(doc, data, this)` 应用远程更新
4. 首次同步完成后注册 `updateHandler`：`doc.on('update', this.updateHandler)`

**updateHandler 逻辑**:
```typescript
this.updateHandler = (update, origin) => {
    if (origin === this) return;  // 跳过自己 apply 的更新（避免回环）
    this.sendUpdate(update);      // 将本地编辑发送到服务端
};
```

**重要**: `updateHandler` 只在首次收到 Step2 后才注册。在此之前本地编辑不会发送到服务端（但实际中首次同步很快，通常不会丢失）。

**重连**: 连接断开后 3 秒自动重连。

### 5. 消息协议（protocol.ts）

与后端 `real_time` 完全对应的 TypeScript 实现：

**编码函数**:
- `writeVarUint` / `writeVarString` / `writeVarBuffer`: varint 编码
- `encodeSyncStep1Message(name, stateVector)`: 编码 Step1 消息
- `encodeSyncStep2Message(name, update)`: 编码 Step2 消息

**解码函数**:
- `readVarUint` / `readVarString` / `readVarBuffer`: varint 解码
- `decodeMessage(data)`: 解码完整消息，返回 `{ name, messageType, docMessageType?, data?, syncStatus? }`

**消息格式**（与后端一致）:
```
[var_string: room_name] [var_uint: MessageType] [var_uint: DocMessageType?] [var_buffer: payload?]
```

### 6. 题目节点（question.ts）

自定义 Tiptap block 节点，用于试卷中的题目：

```typescript
export const Question = Node.create({
  name: 'question',
  group: 'block',
  content: '(paragraph|heading|bulletList|orderedList|blockquote|codeBlock|question)+',
  defining: true,    // 在 ProseMirror 中作为独立块
  isolating: true,   // 隔离选区边界
});
```

**特性**:
- 支持嵌套（content 中包含 question）
- 自动编号（通过 ProseMirror Plugin + Decoration 实现 "第 1 题"、"第 1.1 题"）
- 快捷键：`Mod-Shift-Q` 切换题目，`Enter` 在空段落时退出题目，`Backspace` 在空题目时删除
- 输入规则：输入 `. ` 或 `。 ` 自动创建题目
- 渲染为 `<div data-type="question" class="question-block">`

**在 y-prosemirror 中的表示**: Question 节点被序列化为 `Y.XmlElement('question')`，其子节点（paragraph 等）作为 Y.XmlElement 的 children。这与普通 block 节点（如 blockquote）的处理方式相同。

### 7. 标题锚点（anchor.ts）

自动为 heading 节点生成唯一 id，用于大纲导航：
- 基于标题文本生成 slug（支持中文）
- 去重处理（同名标题加后缀 `-1`, `-2`...）
- 通过 CustomEvent `headings-change` 通知大纲组件更新

### 8. y-prosemirror 同步机制

**ProseMirror → Yjs**（本地编辑）:
1. 用户在 Tiptap 中编辑
2. ProseMirror 触发 transaction
3. `ySyncPlugin` 的 `apply` 检测到非 yjs 来源的变更
4. 调用 `binding._prosemirrorChanged(doc)` → `updateYFragment(doc, type, doc, meta)`
5. `updateYFragment` 递归遍历 ProseMirror 文档树，通过 `createTypeFromElementNode` 将每个节点转换为 Y.XmlElement/Y.XmlText
6. Yjs 生成 update 事件 → `updateHandler` 发送到服务端

**Yjs → ProseMirror**（远程更新）:
1. 收到服务端 Step2 消息
2. `Y.applyUpdate(doc, data, this)` 应用更新
3. Yjs 触发 observe 回调 → `binding._typeChanged(events, transaction)`
4. `_typeChanged` 通过 `createNodeIfNotExists` 将 Y.XmlElement 转换回 ProseMirror Node
5. 使用 `tr.replace` 替换整个文档内容

**关键函数**:
- `createTypeFromElementNode(node, meta)`: ProseMirror Node → Y.XmlElement
- `createNodeFromYElement(el, schema, meta)`: Y.XmlElement → ProseMirror Node
- `updateYFragment(y, yDomFragment, pNode, meta)`: 递归 diff 并更新 YXmlFragment
- `normalizePNodeContent(pnode)`: 将 ProseMirror 的 inline text nodes 合并为数组

### 9. 已知问题

- **标题不同步**: `doc-title` 是独立 contenteditable div，不在 Tiptap 编辑器内，不参与 yjs 同步。多人协作时标题不会实时更新。
- **Question 节点同步**: Question 作为自定义 Y.XmlElement('question') 参与同步，理论上与普通节点无异。但如果 y-octo 后端处理 XMLElement 时出现问题（如 `repair` 中 `swap_take` 导致的 `InvalidParent` 错误），会导致同步失败。
- **updateHandler 注册时机**: 只在首次收到 Step2 后才注册，如果初始同步失败则永远不会发送本地编辑。
- **全量替换**: `_typeChanged` 使用 `tr.replace(0, docSize, newSlice)` 全量替换文档，而非增量 patch。大文档时可能有性能问题。