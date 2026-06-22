# BFF Service

Backend For Frontend 服务，为 Web 前端提供 HTTP API，内部通过 gRPC 调用 dal 服务。对外暴露 REST 和 GraphQL 两种查询接口。

## 技术栈

- Axum (Web 框架)
- axum-extra/cookie (Cookie 处理)
- Tonic (gRPC 客户端)
- auth-lib (JWT 编解码 + AuthClaim extractor)
- async-graphql (GraphQL 端点)
- config (配置管理)

## 目录结构

```
src/
├── main.rs              # 入口：初始化配置、加载密钥、gRPC 连接、启动 HTTP 服务
├── config.rs            # 配置加载（TOML 文件 + 环境变量）
├── graphql/
│   ├── mod.rs           # AppSchema 类型别名
│   ├── query.rs         # Query + Mutation 对象 + GraphQL resolver
│   └── types.rs         # GraphQL 实体类型（SimpleObject / Union）
├── handler/
│   ├── mod.rs           # 路由定义
│   ├── auth.rs          # 认证 handler（注册、登录、获取当前用户、登出）
│   ├── exam_space.rs    # 试卷空间 REST handler（创建）
│   └── graphql.rs       # GraphQL 请求 handler（构建 Schema，注入 context）
└── rpcclient/
    ├── mod.rs           # gRPC 客户端导出
    └── dal.rs           # dal 服务 gRPC 客户端
```

## 配置

配置文件位于 `config/default.toml`：

```toml
[server]
host = "0.0.0.0"
port = 3000

[dal]
addr = "http://127.0.0.1:4000"
```

## 认证机制

- JWT Token 通过 HttpOnly Cookie 传递，Cookie 名为 `token`
- 通过 `auth-lib::AuthClaim` extractor 统一验证，各服务只需在 `AppState` 上实现 `FromRequestParts` 即可
- `AppJotProvider` 是 Jot 的 local newtype，用来从 AppState 中提取 `Arc<Jot>`
- Cookie 属性：HttpOnly、SameSite=Lax、Path=/、Max-Age=24h

## REST 接口

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| GET | `/api/health` | 健康检查 | 否 |
| POST | `/api/auth/register` | 用户注册 | 否 |
| POST | `/api/auth/login` | 邮箱密码登录 | 否 |
| GET | `/api/auth/me` | 获取当前用户 | 是 |
| POST | `/api/auth/logout` | 登出（清除 Cookie） | 是 |
| POST | `/api/exam-spaces` | 创建试卷空间 | 是 |
| POST | `/api/graphql` | GraphQL 查询端点 | 是 |

### 注册

```
POST /api/auth/register
Content-Type: application/json
```

**请求体**

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| username | string | 是 | 用户名 |
| email | string | 是 | 邮箱地址（作为账户名） |
| password | string | 是 | 密码（至少6位） |

**成功响应** `201 Created`

```json
{ "user_id": 1 }
```

### 登录

```
POST /api/auth/login
Content-Type: application/json
```

**请求体**

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| email | string | 是 | 邮箱地址 |
| password | string | 是 | 密码 |

**成功响应** `200 OK`，响应头 `Set-Cookie` 中包含 HttpOnly 的 `token` Cookie。

```json
{ "user_id": 1, "username": "张三" }
```

### 获取当前用户

```
GET /api/auth/me
Cookie: token=<jwt_token>
```

**成功响应** `200 OK`

```json
{ "user_id": 1, "username": "张三" }
```

### 创建试卷空间

```
POST /api/exam-spaces
Content-Type: application/json
Cookie: token=<jwt_token>
```

**请求体**

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| name | string | 是 | 空间名称 |
| description | string | 否 | 空间描述 |

**成功响应** `201 Created`

```json
{ "id": 1 }
```

## GraphQL API

端点：`POST /api/graphql`（需要登录）

### Query

#### exam_spaces — 查询试卷空间

- `token` 参数提供时：返回单个空间（GqlExamSpacesResult::Payload）
- `token` 参数省略时：返回分页列表（GqlExamSpacesResult::Connection）

```graphql
query {
  examSpaces(first: 20) {
    ... on GqlExamSpaceConnection {
      edges { node { id name description paperCount } cursor }
      pageInfo { hasNextPage endCursor }
    }
    ... on GqlExamSpacePayload {
      space { id name description paperCount }
    }
  }
}
```

#### exam_space_members — 查询空间成员

```graphql
query {
  examSpaceMembers(examSpaceId: "1") {
    id userId examSpaceId role username
  }
}
```

#### directory_children — 列出目录子项

```graphql
query {
  directoryChildren(examSpaceId: 1, parentId: 0) {
    directories { id name parentId }
    papers { id title directoryId }
  }
}
```

#### directory_breadcrumb — 获取目录面包屑

```graphql
query {
  directoryBreadcrumb(directoryId: 1) {
    id name
  }
}
```

#### paper_breadcrumb — 获取试卷面包屑

```graphql
query {
  paperBreadcrumb(paperId: 1) {
    directoryItems { id name }
    paper { id title }
  }
}
```

### Mutation

| Mutation | 参数 | 返回 | 说明 |
|----------|------|------|------|
| `createDirectory` | `examSpaceId, parentId?, name` | `GqlDirectory` | 创建目录 |
| `updateDirectory` | `id, name` | `Boolean` | 更新目录名称 |
| `deleteDirectory` | `id` | `Boolean` | 删除目录 |
| `moveDirectory` | `id, newParentId?` | `Boolean` | 移动目录 |
| `createPaper` | `examSpaceId, directoryId?, title` | `GqlPaper` | 创建试卷 |
| `updatePaper` | `id, title` | `Boolean` | 更新试卷标题 |
| `deletePaper` | `id` | `Boolean` | 删除试卷 |
| `movePaper` | `id, newDirectoryId?` | `Boolean` | 移动试卷 |
| `updateExamSpace` | `id, name, description` | `Boolean` | 更新试卷空间 |
| `deleteExamSpace` | `id` | `Boolean` | 删除试卷空间 |

### GraphQL 类型

**GqlExamSpace**

| 字段 | 类型 | 说明 |
|------|------|------|
| id | Int | 空间 ID |
| token | String | 空间唯一 token |
| name | String | 空间名称 |
| description | String | 描述 |
| lastUpdatedAt | String | 最后更新时间 |
| memberCount | Int | 成员数量 |
| paperCount | Int | 试卷数量 |

**GqlDirectory**

| 字段 | 类型 | 说明 |
|------|------|------|
| id | Int | 目录 ID |
| examSpaceId | Int | 所属空间 |
| parentId | Int | 父目录 ID（0 为根） |
| name | String | 目录名称 |
| createdAt | String | 创建时间 |
| updatedAt | String | 更新时间 |

**GqlPaper**

| 字段 | 类型 | 说明 |
|------|------|------|
| id | Int | 试卷 ID |
| examSpaceId | Int | 所属空间 |
| directoryId | Int | 所属目录（0 为根） |
| title | String | 试卷标题 |
| createdAt | String | 创建时间 |
| updatedAt | String | 更新时间 |

## 扩展指南

### 添加新的 GraphQL 实体

1. 在 `src/graphql/types.rs` 中定义 SimpleObject
2. 在 `src/graphql/query.rs` 中添加对应的 Query resolver 或 Mutation 方法
3. 通过 `ctx.data::<DalServiceClient>()` 获取 gRPC 客户端调用 dal 服务
4. 通过 `ctx.data::<Claim>()` 获取当前用户信息

### 添加新的 REST 接口

1. 在 `src/handler/` 下新建或扩展 handler 模块
2. 在 `src/handler/mod.rs` 的 `routes()` 中注册路由
