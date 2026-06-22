# web 服务

## 概述

基于 Astro 的前端应用，提供试卷编辑功能。核心是 Tiptap 富文本编辑器，通过 y-prosemirror + yjs 实现多人实时协作。包含用户认证（注册/登录）、Console 控制台页面、试卷空间管理、试卷编辑与预览等。

## 技术栈

- **框架**: Astro 4.16
- **编辑器**: Tiptap 3.23（基于 ProseMirror）
- **协作**: yjs 13.6 + y-prosemirror 1.3
- **包管理**: pnpm 9

## 项目结构

```
src/
├── pages/
│   ├── index.astro                  # 首页
│   ├── login.astro                  # 登录页面
│   ├── register.astro               # 注册页面
│   ├── console.astro                # 控制台（需登录）
│   ├── console/
│   │   └── papers.astro             # 试卷空间管理页面
│   ├── space/
│   │   ├── [token].astro            # 试卷空间详情页（Tab: 文件 / 设置）
│   │   ├── api.ts                   # GraphQL 请求封装
│   │   ├── breadcrumb.ts            # 面包屑加载
│   │   ├── file-list.ts             # 文件列表（目录 + 试卷）渲染与交互
│   │   ├── inline-edit.ts           # 内联编辑（标题、描述双击修改）
│   │   ├── modals.ts                # 弹窗（新建目录、确认删除等）
│   │   └── space.css                # 空间页面样式
│   ├── doc/
│   │   ├── [id].astro               # 试卷编辑页面（核心页面）
│   │   ├── api.ts                   # GraphQL 请求与认证
│   │   ├── breadcrumb.ts            # 面包屑加载
│   │   ├── titleEditor.ts           # 标题编辑与保存
│   │   ├── outline.ts               # 大纲渲染与滚动高亮
│   │   ├── preview.ts               # 预览模式与全屏逻辑
│   │   └── styles.css               # 编辑页面样式
│   └── 404.astro                    # 404 页面
├── components/
│   ├── TiptapEditor.astro          # Tiptap 编辑器组件（入口）
│   ├── console/
│   │   └── Sidebar.astro           # 控制台侧边栏（可折叠，状态持久化）
│   └── tiptap/
│       ├── collaboration.ts        # 协作扩展（集成 yjs + WebSocket）
│       ├── question.ts             # 题目节点（自定义 block 节点）
│       ├── anchor.ts               # 标题锚点（自动生成 heading id）
│       ├── drag-handle.ts          # 拖拽手柄
│       └── uppercase-letter-list.ts # 大写字母有序列表
└── lib/
    ├── protocol.ts                 # 消息协议编解码（与后端对应）
    └── ws-provider.ts              # WebSocket 连接管理（DocWebSocketProvider）
```

## 部署架构

用户通过 Nginx 统一访问，无需关注后端端口：

```
Browser (:4321 dev / :80 prod)
  │
  ├── /api/auth/* ──▶ Nginx ──▶ BFF :3000 ── gRPC ──▶ DAL :4000
  ├── /api/websocket/* ──▶ Nginx ──▶ real_time :3001
  └── /* (静态资源) ──▶ Nginx ──▶ web :4321
```

前端所有 API 请求使用**相对路径**（如 `/api/auth/login`），无需写死 localhost:3000。

## 认证机制

- JWT Token 存储在 **HttpOnly Cookie**（`token`）中，JavaScript 无法访问
- 所有 API 请求通过 `fetch(..., { credentials: 'include' })` 自动携带 Cookie
- **Sidebar** 组件在页面加载时检查登录状态
- **Console 页面**：`client:load` 时检查登录状态，未登录自动跳转 `/login`
- **登录成功后**自动跳转 `/console`

## 页面路由

| 路径 | 说明 | 认证 |
|------|------|------|
| `/` | 首页 | 否 |
| `/login` | 登录页 | 否 |
| `/register` | 注册页 | 否 |
| `/console` | 控制台 | 是（未登录跳转 /login） |
| `/console/papers` | 试卷空间管理 | 是 |
| `/space/[token]` | 试卷空间详情 | 是 |
| `/doc/[id]` | 文档编辑页 | 是 |

## 架构详解

### 1. 试卷空间详情页（space/[token].astro）

页面结构：
```
┌──────────────────────────────────────────────┐
│ Sidebar                                       │
├──────────────────────────────────────────────┤
│ 标题（双击编辑）  描述（双击编辑）             │
│ Tab: 文件 | 设置                              │
│ ───────────────────────────────────────────── │
│ 文件 Tab:                                     │
│   新建目录 / 新建试卷                          │
│   目录列表（可展开/折叠/改名/删除）             │
│   试卷列表（可改名/删除）                       │
│ 设置 Tab:                                     │
│   空间名称 / 描述 / 保存修改 / 删除空间        │
│   成员列表                                     │
└──────────────────────────────────────────────┘
```

功能拆分为多个 TS 模块：`api.ts`（GraphQL）、`breadcrumb.ts`、`file-list.ts`、`inline-edit.ts`、`modals.ts`。

### 2. 文档编辑页面（doc/[id].astro）

页面结构：
```
┌──────────────────────────────────────────────┐
│ Sidebar                                       │
├──────────────────────────────────────────────┤
│ Header: 面包屑              [预览/关闭预览]   │
├──────────┬────────────────────┬──────────────┤
│ 大纲     │ doc-title          │ 圈点留言     │
│ (outline)│ (contenteditable)  │ (annotations)│
│          │ TiptapEditor       │              │
│          │ (协作编辑器)       │              │
└──────────┴────────────────────┴──────────────┘
```

**预览模式**：点击「预览」后页面分左右两栏：
- 左栏：编辑区（大纲和圈点留言隐藏），保持原始宽度
- 右栏：PDF 预览区域（含全屏按钮）
- 预览/全屏状态通过 localStorage 持久化

功能拆分为多个 TS 模块：`api.ts`、`breadcrumb.ts`、`titleEditor.ts`、`outline.ts`、`preview.ts`。

**重要**: `doc-title` 是一个独立的 `contenteditable` div，**不在 Tiptap 编辑器内**，因此**不参与实时同步**。它只更新页面标题和面包屑导航，失焦时自动保存到后端。

### 3. 侧边栏（Sidebar.astro）

- 可折叠/展开，折叠时只显示 icon
- 折叠状态通过 localStorage 持久化
- 折叠时通过 `html.classList` 在页面加载前应用，避免闪烁
- 导航项：试卷空间、题库

### 4. TiptapEditor 组件（TiptapEditor.astro）

编辑器初始化流程：
1. 创建 Tiptap Editor 实例
2. 加载扩展：`StarterKit`（基础功能）、`Anchor`（标题锚点）、`Question`（题目节点）、`DragHandle`（拖拽）
3. 如果有 `docName`，加载 `Collaboration` 扩展（启用实时协作）
4. 协作模式下 `content` 传 `undefined`（内容由 yjs 同步），非协作模式使用静态 HTML

### 5. 协作扩展（collaboration.ts）

```typescript
export const Collaboration = Extension.create({
  addStorage() {
    const doc = new Y.Doc();
    const yXmlFragment = doc.getXmlFragment('prosemirror');
    return { doc, provider: null, yXmlFragment };
  },
  onCreate() {
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

### 6. WebSocket Provider（ws-provider.ts）

`DocWebSocketProvider` 管理 WebSocket 连接和同步生命周期：

**连接流程**:
1. `connect()` 建立 WebSocket 连接到 `wss://域名/api/websocket/doc/{roomName}`
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

**重连**: 连接断开后 3 秒自动重连。

### 7. 消息协议（protocol.ts）

与后端 `real_time` 完全对应的 TypeScript 实现：

**编码函数**:
- `writeVarUint` / `writeVarString` / `writeVarBuffer`: varint 编码
- `encodeSyncStep1Message(name, stateVector)`: 编码 Step1 消息
- `encodeSyncStep2Message(name, update)`: 编码 Step2 消息

**解码函数**:
- `readVarUint` / `readVarString` / `readVarBuffer`: varint 解码
- `decodeMessage(data)`: 解码完整消息

**消息格式**（与后端一致）:
```
[var_string: room_name] [var_uint: MessageType] [var_uint: DocMessageType?] [var_buffer: payload?]
```

### 8. 题目节点（question.ts）

自定义 Tiptap block 节点，用于试卷中的题目：

```typescript
export const Question = Node.create({
  name: 'question',
  group: 'block',
  content: '(paragraph|heading|bulletList|orderedList|blockquote|codeBlock|question)+',
  defining: true,
  isolating: true,
});
```

**特性**:
- 自动编号（通过 ProseMirror Plugin + Decoration 实现 "第 1 题"、"第 1.1 题"）
- 快捷输入：`A. ` / `a. ` / `A.。` / `a.。` 触发大写字母有序列表
- 回车在空段落时退出题目，Backspace 在空题目时删除

### 9. 标题锚点（anchor.ts）

自动为 heading 节点生成唯一 id，用于大纲导航：
- 基于标题文本生成 slug（支持中文）
- 去重处理（同名标题加后缀 `-1`, `-2`...）
- 通过 CustomEvent `headings-change` 通知大纲组件更新

### 10. y-prosemirror 同步机制

**ProseMirror → Yjs**（本地编辑）:
1. 用户在 Tiptap 中编辑 → ProseMirror transaction
2. `ySyncPlugin#apply` 检测非 yjs 来源变更
3. `updateYFragment` 递归遍历 ProseMirror 文档树，转换为 Y.XmlElement/Y.XmlText
4. Yjs 生成 update 事件 → `updateHandler` 发送到服务端

**Yjs → ProseMirror**（远程更新）:
1. 收到服务端 Step2 → `Y.applyUpdate(doc, data, this)`
2. Yjs observe 回调 → `_typeChanged` 将 Y.XmlElement 转为 ProseMirror Node
3. 使用 `tr.replace(0, docSize, newSlice)` 全量替换文档

## 已知问题

- **标题不同步**: `doc-title` 是独立 contenteditable div，不参与 yjs 同步
- **全量替换**: `_typeChanged` 使用 `tr.replace` 全量替换，大文档时可能有性能问题

## 构建 & 运行

```bash
cd service/web
pnpm install
pnpm dev
# 监听 0.0.0.0:4321
```
