# Dal Service — 数据访问层 gRPC 服务

## 概述

Dal 是项目的数据访问层服务，通过 gRPC 对外提供用户注册、登录、试卷空间管理等数据操作接口。当前使用 SQLite 存储，数据库层通过 trait 抽象，便于未来切换到其他数据库。

## 技术栈

- **gRPC**: tonic 0.13 + prost 0.13
- **数据库**: SQLite (rusqlite 0.34, bundled)
- **密码哈希**: Argon2
- **JWT**: EdDSA (auth-lib)
- **异步**: tokio + async-trait

## 目录结构

```
proto/dal/                  ← Proto 定义（项目级共享）
├── service.proto           ← Service + RPC 定义
├── auth.proto              ← Auth 相关 Request/Response message
└── exam_space.proto        ← ExamSpace 相关 Request/Response message

service-ce/sql/
└── dal_init.sql            ← DDL SQL（建表语句，编译时嵌入）

service/dal/
├── Cargo.toml
├── build.rs                ← tonic-build 编译 proto/dal/service.proto
└── src/
    ├── main.rs             ← 初始化 DB/Jot，组装 handler，启动 gRPC Server (0.0.0.0:4000)
    ├── db/
    │   ├── mod.rs          ← UserRepo / ExamSpaceRepo / ExamSpaceMemberRepo trait + Row 结构体
    │   └── sqlite.rs       ← SqliteDb 实现所有 Repo trait
    └── handler/
        ├── mod.rs          ← DalServiceHandler（实现 DalService trait，委托给子 handler）
        ├── auth/
        │   ├── mod.rs      ← 导出 AuthHandler
        │   └── handler.rs  ← register / login_by_email 业务逻辑 + JWT 签发
        └── exam_space/
            ├── mod.rs      ← 导出 ExamSpaceHandler
            └── handler.rs  ← exam_space / member CRUD 业务逻辑
```

## 架构分层

```
main.rs (组装)
  │
  ├─ db::SqliteDb ──实现──▶ db::UserRepo trait
  │                         db::ExamSpaceRepo trait
  │                         db::ExamSpaceMemberRepo trait
  │
  └─ handler::DalServiceHandler
       │
       ├─ handler::auth::AuthHandler
       │    ├─ 依赖 DbPool (Arc<dyn UserRepo>)
       │    ├─ 依赖 Jot (JWT 签发/验证)
       │    └─ 密码哈希/验证 (Argon2)
       │
       └─ handler::exam_space::ExamSpaceHandler
            ├─ 依赖 Arc<dyn ExamSpaceRepo>
            └─ 依赖 Arc<dyn ExamSpaceMemberRepo>
```

## 数据库抽象

### UserRepo

| 方法 | 说明 |
|------|------|
| `create_user(username, email, password_hash)` | 创建用户，返回 user_id |
| `find_user_by_email(email)` | 按 email 查找用户 |

### ExamSpaceRepo

| 方法 | 说明 |
|------|------|
| `create_exam_space(name, description)` | 创建试卷空间，返回 id |
| `update_exam_space(id, name, description)` | 更新试卷空间，自动更新 last_updated_at |
| `delete_exam_space(id)` | 删除空间，同时删除所有成员记录 |
| `find_exam_spaces_by_user(user_id)` | 查询用户有权限的所有空间（JOIN exam_space_members） |
| `find_exam_space_by_id(id)` | 按 ID 查找空间 |

### ExamSpaceMemberRepo

| 方法 | 说明 |
|------|------|
| `add_member(user_id, exam_space_id, role)` | 添加成员（user_id + exam_space_id 唯一） |
| `update_member_role(id, role)` | 更新成员角色 |
| `remove_member(id)` | 移除成员 |
| `find_members_by_space(exam_space_id)` | 查询空间的所有成员 |
| `find_member(user_id, exam_space_id)` | 查找特定成员 |

## 数据库表结构

### users

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | INTEGER | PK, AUTOINCREMENT | 用户 ID |
| username | TEXT | NOT NULL | 用户名（显示名） |
| email | TEXT | NOT NULL, UNIQUE | 邮箱（账户名，用于登录） |
| password | TEXT | NOT NULL | Argon2 哈希后的密码 |

### exam_spaces

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | INTEGER | PK, AUTOINCREMENT | 空间 ID |
| name | TEXT | NOT NULL | 空间名称 |
| description | TEXT | NOT NULL DEFAULT '' | 描述 |
| last_updated_at | DATETIME | NOT NULL DEFAULT now | 最后更新时间 |

### exam_space_members

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | INTEGER | PK, AUTOINCREMENT | 记录 ID |
| user_id | INTEGER | NOT NULL | 用户 ID |
| exam_space_id | INTEGER | NOT NULL | 试卷空间 ID |
| role | TEXT | NOT NULL DEFAULT 'member' | 角色 |
| last_accessed_at | DATETIME | | 最后访问时间 |
| | | UNIQUE (user_id, exam_space_id) | 联合唯一约束 |

DDL 定义在 `service-ce/sql/dal_init.sql`，通过 `include_str!` 编译时嵌入。

## gRPC 接口

| RPC | Request | Response | 说明 |
|-----|---------|----------|------|
| `Register` | `{ username, email, password }` | `{ user_id }` | 注册，email 唯一 |
| `LoginByEmail` | `{ email, password }` | `{ user_id, username, token }` | 邮箱密码登录，返回 JWT |
| `CreateExamSpace` | `{ name, description }` | `{ id }` | 创建试卷空间 |
| `UpdateExamSpace` | `{ id, name, description }` | `Empty` | 更新试卷空间 |
| `DeleteExamSpace` | `{ id }` | `Empty` | 删除试卷空间（级联删除成员） |
| `ListExamSpacesByUser` | `{ user_id }` | `{ []ExamSpace }` | 查询用户的所有空间 |
| `AddExamSpaceMember` | `{ user_id, exam_space_id, role }` | `{ id }` | 添加空间成员 |
| `UpdateExamSpaceMember` | `{ id, role }` | `Empty` | 更新成员角色 |
| `RemoveExamSpaceMember` | `{ id }` | `Empty` | 移除成员 |
| `ListExamSpaceMembers` | `{ exam_space_id }` | `{ []ExamSpaceMember }` | 查询空间成员 |

## 密钥管理

- Ed25519 密钥对存放在项目根 `keys/` 目录（已加入 .gitignore）
- 通过 `service-ce/scripts/gen_keys.sh` 生成
- `main.rs` 使用 `include_bytes!` 编译时嵌入

## 扩展指南

### 新增业务模块

1. 在 `proto/dal/` 下新建 `{业务}.proto`，定义 message
2. 在 `service.proto` 中 import 并添加 rpc
3. 在 `src/handler/{业务}/handler.rs` 实现业务逻辑
4. 在 `handler/mod.rs` 的 `DalServiceHandler` 中添加字段并委托

### 新增数据库表

1. 在 `service-ce/sql/dal_init.sql` 中追加 `CREATE TABLE IF NOT EXISTS`
2. 在 `db/mod.rs` 中定义对应 trait 和 Row 结构体
3. 在 `db/sqlite.rs` 中实现 trait

### 切换数据库

1. 在 `src/db/` 下新建 `{db_type}.rs`，实现所有 `*Repo` trait
2. 在 `main.rs` 中替换 `Arc::new(SqliteDb::new(...))` 为新实现
3. handler 层代码零改动

## 构建 & 运行

```bash
# 生成密钥（首次）
cd service-ce && ./scripts/gen_keys.sh

# 构建
cd service/dal && cargo build

# 运行
cargo run
# 监听 0.0.0.0:4000
```