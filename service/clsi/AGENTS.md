# Clsi Service — 编译服务 gRPC 服务

## 概述

Clsi（Compile Service）是项目的编译服务，通过 gRPC 对外提供将 doc json 结构编译为 PDF 文件的接口。当前为框架搭建阶段，核心编译逻辑尚未实现。

## 技术栈

- **gRPC**: tonic 0.13 + prost 0.13
- **异步**: tokio
- **日志**: tracing + tracing-subscriber

## 目录结构

```
proto/clsi/
└── service.proto           ← Service + RPC 定义 + message

service/clsi/
├── Cargo.toml
├── build.rs                ← tonic-build 编译 proto/clsi/service.proto
└── src/
    ├── main.rs             ← 初始化 handler，启动 gRPC Server (0.0.0.0:5000)
    └── handler/
        ├── mod.rs          ← ClsiServiceHandler（实现 ClsiService trait，委托给子 handler）
        └── compile.rs      ← CompileHandler（编译逻辑，当前返回 not implemented）
```

## 架构分层

```
main.rs (组装)
  │
  └─ handler::ClsiServiceHandler
       │
       └─ handler::compile::CompileHandler
            └─ compile_pdf() → TODO: 实现 doc json → PDF 编译
```

## gRPC 接口

| RPC | Request | Response | 说明 |
|-----|---------|----------|------|
| `CompilePdf` | `{ doc_json }` | `{ pdf_data, success, error }` | 将 doc json 编译为 PDF |

### CompilePdfRequest

| 字段 | 类型 | 说明 |
|------|------|------|
| doc_json | string | doc json 内容 |

### CompilePdfResponse

| 字段 | 类型 | 说明 |
|------|------|------|
| pdf_data | bytes | 编译后的 PDF 文件内容（二进制） |
| success | bool | 是否成功 |
| error | string | 错误信息（如果失败） |

## 构建 & 运行

```bash
cd service/clsi && cargo build
cargo run
# 监听 0.0.0.0:5000
```

## 待实现

- [ ] doc json 结构定义
- [ ] doc json → PDF 编译逻辑
- [ ] 编译缓存（相同输入不重复编译）
- [ ] 编译超时和错误处理
