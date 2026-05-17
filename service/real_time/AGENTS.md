# real_time 服务

## 概述

基于 axum + WebSocket 的实时协作文档编辑后端服务。使用 y-octo（Yjs CRDT 的 Rust 实现）管理文档状态，支持多人实时同步编辑。

## 技术栈

- **框架**: axum 0.8（WebSocket + HTTP）
- **CRDT**: y-octo（Yjs Rust 实现）
- **异步运行时**: tokio
- **序列化**: serde / serde_json
- **认证**: auth-lib（本地 lib）

## 项目结构

```
src/
├── main.rs                          # 服务入口，路由注册
├── handler/
│   ├── mod.rs                       # handler 模块入口
│   └── websocket.rs                 # WebSocket 连接处理器
└── biz/
    ├── mod.rs                       # biz 模块入口
    ├── handler.rs                   # 核心同步流处理（handle_sync_stream）
    ├── principal.rs                 # 用户主体（Principal），包含身份和 claim
    ├── room_set.rs                  # 房间集合管理（RoomSet）
    ├── doc/
    │   ├── mod.rs
    │   ├── document.rs              # 文档封装（Doc + Awareness + 连接管理）
    │   └── context.rs               # DocumentContext（实现 protocol Context trait）
    ├── protocol/
    │   ├── mod.rs
    │   ├── context.rs               # Context trait 定义
    │   ├── handler.rs               # 协议消息处理（handle_message）
    │   ├── message_type.rs          # 消息类型枚举
    │   ├── sync.rs                  # 同步消息读写
    │   └── awareness.rs             # 感知消息处理
    └── room/
        ├── mod.rs
        ├── cmd.rs                   # DocRoomCommand（房间命令通道）
        └── doc_room.rs              # DocRoom（房间 Actor，管理文档生命周期）
```

## 架构详解

### 1. 服务入口（main.rs）

- 创建 `AppState`（目前为空，后续会加入 RoomSet 等共享状态）
- 注册路由 `GET /api/websocket/doc/{token}` → `handler::websocket::websocket_handler`
- 监听 `0.0.0.0:3000`

### 2. WebSocket Handler（handler/websocket.rs）

WebSocket 连接处理流程：
1. 从 URL path 中提取 token
2. 解析 token，创建 mock Principal（含 mock Claim）
3. 升级 HTTP 连接为 WebSocket
4. 从 RoomSet 获取或创建 DocRoom
5. 调用 `biz::handler::handle_sync_stream` 进入同步循环

### 3. 核心同步流（biz/handler.rs）

`handle_sync_stream` 函数：
1. 将 WebSocket 拆分为 outgoing（写）和 incoming（读）
2. 调用 `docroom_cmd.join(principal, outgoing)` 加入房间
3. 循环读取 incoming 消息，调用 `docroom_cmd.sync(principal, data)` 同步
4. 连接断开后调用 `docroom_cmd.leave(principal)` 离开房间

### 4. Principal（biz/principal.rs）

用户主体，包含：
- `id: u64` - 文档/房间 ID
- `name: String` - 文档名称
- `claim: claim::Claim` - JWT Claim（来自 auth-lib）
- `connection_id: u64` - 连接唯一标识

### 5. Claim（auth-lib）

JWT Claim 结构：
- `sub: u64` - 用户 ID
- `name: String` - 用户名
- `jti: u64` - JWT ID
- `iss`, `exp`, `aud`, `nbf`, `iat` - 标准 JWT 字段

### 6. RoomSet（biz/room_set.rs）

房间集合，管理所有活跃的文档房间：
- 泛型 `M`：与房间关联的业务模型
- `build_docroom(principal, model, doc_binary)` - 创建或获取房间
- `get_docroom(principal)` - 获取已有房间
- `destory_room(id)` - 销毁房间
- `recv_callback()` - 接收房间回调（存储、销毁通知）

### 7. DocRoom（biz/room/doc_room.rs）

房间 Actor，通过 tokio spawn 运行：
- 接收 `DocCommandMessage`（Join/Sync/Leave）
- 每 10 秒检查是否需要持久化（`callback_store`）
- 所有连接断开后自动销毁

### 8. Document（biz/doc/document.rs）

文档封装：
- `doc: Doc` - y-octo CRDT 文档
- `awareness: Awareness` - 用户感知状态
- `connections: HashMap<u64, Arc<Mutex<SplitSink>>>` - 活跃连接
- `version / stored_version` - 版本追踪，用于增量持久化

### 9. 协议层（biz/protocol/）

实现 yjs 同步协议：
- **Context trait**：抽象文档操作（unicast/broadcast/close）
- **handle_message**：解析并处理同步消息（Step1/Step2/Update/Awareness）
- **消息格式**：`[var_string: doc_name] [var_uint: MessageType] [var_uint: DocMessageType?] [var_buffer: payload?]`

### 10. 同步流程

```
Client                    Server
  |                          |
  |--- Step1 (StateVector) ->|  客户端发送本地状态向量
  |                          |
  |<- Step1 (StateVector) ---|  服务端返回服务端状态向量
  |<- Step2 (MissingUpdate) -|  服务端发送客户端缺失的更新
  |                          |
  |--- Step2 (Update) ------>|  客户端发送服务端缺失的更新
  |                          |
  |<-- SyncStatus (true) ----|  同步完成确认
  |                          |
  |--- Update --------------->|  实时编辑同步
  |<-- Update ----------------|  其他客户端的编辑广播
```

## 待实现

- [ ] WebSocket handler 中 token 的真实解析（目前 mock）
- [ ] RoomSet 的泛型 M 确定具体类型
- [ ] 文档持久化回调的实际存储逻辑
- [ ] 错误日志完善（目前多处 TODO）