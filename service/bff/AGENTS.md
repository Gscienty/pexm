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
│   ├── query.rs         # Query 对象 + GraphQL resolver
│   └── types.rs         # GraphQL 实体类型（SimpleObject）
├── handler/
│   ├── mod.rs           # 路由定义
│   ├── auth.rs          # 认证 handler（注册、登录、获取当前用户）
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

## 接口总览

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| GET | `/api/health` | 健康检查 | 否 |
| POST | `/api/auth/register` | 用户注册 | 否 |
| POST | `/api/auth/login` | 邮箱密码登录 | 否 |
| GET | `/api/auth/me` | 获取当前用户 | 是 |
| POST | `/api/exam-spaces` | 创建试卷空间 | 是 |
| POST | `/api/graphql` | GraphQL 查询端点 | 是 |

---

## Web API 请求/响应格式

### 健康检查

```
GET /api/health
```

**响应** `200 OK`

```
ok
```

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

```json
{
  "username": "张三",
  "email": "zhangsan@example.com",
  "password": "123456"
}
```

**成功响应** `201 Created`

| 字段 | 类型 | 说明 |
|------|------|------|
| user_id | i64 | 用户 ID |

```json
{
  "user_id": 1
}
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

```json
{
  "email": "zhangsan@example.com",
  "password": "123456"
}
```

**成功响应** `200 OK`

响应头 `Set-Cookie` 中包含 HttpOnly 的 `token` Cookie。

| 字段 | 类型 | 说明 |
|------|------|------|
| user_id | i64 | 用户 ID |
| username | string | 用户名 |

```json
{
  "user_id": 1,
  "username": "张三"
}
```

### 获取当前用户

```
GET /api/auth/me
Cookie: token=<jwt_token>
```

**成功响应** `200 OK`

| 字段 | 类型 | 说明 |
|------|------|------|
| user_id | u64 | 用户 ID |
| username | string | 用户名 |

```json
{
  "user_id": 1,
  "username": "张三"
}
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

```json
{
  "name": "数学期中试卷",
  "description": "三年级数学期中考试试卷"
}
```

**成功响应** `201 Created`

| 字段 | 类型 | 说明 |
|------|------|------|
| id | i64 | 新创建的空间 ID |

```json
{
  "id": 1
}
```

**说明**: 创建试卷空间后，DAL 会自动将创建者作为 `owner` 角色加入 `exam_space_members` 表。

---

## GraphQL API

端点：`POST /api/graphql`（需要登录）

### 查询当前用户的试卷空间

```graphql
query {
  myExamSpaces {
    id
    name
    description
    lastUpdatedAt
  }
}
```

### 查询试卷空间成员

```graphql
query {
  examSpaceMembers(examSpaceId: 1) {
    id
    userId
    examSpaceId
    role
    lastAccessedAt
  }
}
```

### GraphQL Schema 类型

**ExamSpace**

| 字段 | 类型 | 说明 |
|------|------|------|
| id | Int | 空间 ID |
| name | String | 空间名称 |
| description | String | 描述 |
| lastUpdatedAt | String | 最后更新时间 |

**ExamSpaceMember**

| 字段 | 类型 | 说明 |
|------|------|------|
| id | Int | 成员记录 ID |
| userId | Int | 用户 ID |
| examSpaceId | Int | 试卷空间 ID |
| role | String | 角色（如 member、owner） |
| lastAccessedAt | String | 最后访问时间 |

---

## 扩展指南

### 添加新的 GraphQL 实体

1. 在 `src/graphql/types.rs` 中定义 SimpleObject
2. 在 `src/graphql/query.rs` 中添加对应的 resolver 方法
3. 通过 `ctx.data::<DalServiceClient>()` 获取 gRPC 客户端调用 dal 服务
4. 通过 `ctx.data::<Claim>()` 获取当前用户信息