# Dal Service — 数据访问层 gRPC 服务

## 概述

Dal 是项目的数据访问层服务，通过 gRPC 对外提供用户注册、登录等数据操作接口。当前使用 SQLite 存储，数据库层通过 trait 抽象，便于未来切换到其他数据库。

## 技术栈

- **gRPC**: tonic 0.13 + prost 0.13
- **数据库**: SQLite (rusqlite 0.34, bundled)
- **密码哈希**: Argon2
- **JWT**: EdDSA (auth-lib)
- **异步**: tokio + async-trait

## 目录结构

```
proto/dal/                  ← Proto 定义（项目级共享）
├── service.proto           ← Service + RPC 定义，import 业务 message
└── auth.proto              ← Auth 相关 Request/Response message

service/dal/
├── Cargo.toml
├── build.rs                ← tonic-build 编译 proto/dal/service.proto
└── src/
    ├── main.rs             ← 初始化 DB/Jot，组装 handler，启动 gRPC Server (0.0.0.0:4000)
    ├── db/
    │   ├── mod.rs          ← UserRepo trait + UserRow + DbPool 类型别名
    │   └── sqlite.rs       ← SqliteDb 实现 UserRepo
    └── handler/
        ├── mod.rs          ← DalServiceHandler（实现 DalService trait，委托给子 handler）
        └── auth/
            ├── mod.rs      ← 导出 AuthHandler
            └── handler.rs  ← register / login_by_email 业务逻辑 + JWT 签发
```

## 架构分层

```
main.rs (组装)
  │
  ├─ db::SqliteDb ──实现──▶ db::UserRepo trait
  │                              │
  └─ handler::DalServiceHandler  │
       │                         │
       └─ handler::auth::AuthHandler
            ├─ 依赖 DbPool (Arc<dyn UserRepo>)
            ├─ 依赖 Jot   (JWT 签发/验证)
            └─ 密码哈希/验证 (Argon2)
```

## 数据库抽象

`db::UserRepo` trait 定义了用户数据操作接口，`DbPool = Arc<dyn UserRepo>`：

| 方法 | 说明 |
|------|------|
| `create_user(username, email, password_hash)` | 创建用户，返回 user_id |
| `find_user_by_email(email)` | 按 email 查找用户，返回 `Option<UserRow>` |

`UserRow` 结构：`{ id: i64, username: String, password_hash: String }`

密码哈希/验证逻辑在 handler 层，数据库层只存取已哈希的密码。

## gRPC 接口

| RPC | Request | Response | 说明 |
|-----|---------|----------|------|
| `Register` | `{ username, email, password }` | `{ user_id }` | 注册，email 唯一 |
| `LoginByEmail` | `{ email, password }` | `{ user_id, username, token }` | 邮箱密码登录，返回 JWT |

## 用户模型

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | INTEGER | PK, AUTOINCREMENT | 用户 ID |
| username | TEXT | NOT NULL | 用户名（显示名） |
| email | TEXT | NOT NULL, UNIQUE | 邮箱（账户名，用于登录） |
| password | TEXT | NOT NULL | Argon2 哈希后的密码 |

## 密钥管理

- Ed25519 密钥对存放在项目根 `keys/` 目录（已加入 .gitignore）
- 通过 `scripts/gen_keys.sh` 生成
- `main.rs` 使用 `include_bytes!` 编译时嵌入

## 扩展指南

### 新增业务模块

1. 在 `proto/dal/` 下新建 `{业务}.proto`，定义 message
2. 在 `service.proto` 中 import 并添加 rpc
3. 在 `src/handler/{业务}/handler.rs` 实现业务逻辑
4. 在 `handler/mod.rs` 的 `DalServiceHandler` 中添加字段并委托

### 新增登录方式

1. 在 `auth.proto` 中添加 `LoginByXxxRequest/Response`
2. 在 `service.proto` 中添加 `rpc LoginByXxx`
3. 在 `src/handler/auth/handler.rs` 中实现 `login_by_xxx` 方法
4. 在 `handler/mod.rs` 的 `DalServiceHandler` 中委托

### 切换数据库

1. 在 `src/db/` 下新建 `{db_type}.rs`，实现 `UserRepo` trait
2. 在 `main.rs` 中替换 `Arc::new(SqliteDb::new(...))` 为新实现
3. handler 层代码零改动

## 构建 & 运行

```bash
# 生成密钥（首次）
./scripts/gen_keys.sh

# 构建
cd service/dal && cargo build

# 运行
cargo run
# 监听 0.0.0.0:4000
```