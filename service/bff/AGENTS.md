# BFF Service

Backend For Frontend 服务，为 Web 前端提供 HTTP API，内部通过 gRPC 调用 dal 服务。

## 技术栈

- Axum (Web 框架)
- Tonic (gRPC 客户端)
- config (配置管理)
- tower-http/CorsLayer (跨域)

## 目录结构

```
src/
├── main.rs          # 入口：初始化配置、gRPC 连接、启动 HTTP 服务
├── config.rs        # 配置加载（TOML 文件 + 环境变量）
├── handler/
│   ├── mod.rs       # 路由定义
│   └── auth.rs      # 认证相关 handler（注册、登录）
└── rpcclient/
    ├── mod.rs       # gRPC 客户端导出
    └── dal.rs       # dal 服务 gRPC 客户端
```

## 配置

配置文件位于 `config/default.toml`，支持环境变量覆盖（前缀 `APP_`）：

```toml
[server]
host = "0.0.0.0"
port = 3000

[dal]
addr = "http://127.0.0.1:4000"
```

## Web API

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

**错误响应** `500 Internal Server Error`

```json
"错误信息字符串"
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

| 字段 | 类型 | 说明 |
|------|------|------|
| user_id | i64 | 用户 ID |
| username | string | 用户名 |
| token | string | JWT Token (EdDSA) |

```json
{
  "user_id": 1,
  "username": "张三",
  "token": "eyJhbGciOiJFZERTQSJ9..."
}
```

**错误响应**

- `401 Unauthorized` — 邮箱或密码错误
- `500 Internal Server Error` — 服务内部错误

```json
"错误信息字符串"
```
