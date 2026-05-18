# rust-checker 项目规划 v3

> 文档版本：v3  
> 创建日期：2026-05-18  
> 状态：草稿

---

## 一、v3 目标定位

v2 完成了工具的生产可用性；v3 聚焦三个方向：**提升执行效率**、**开放插件生态**、**增强趋势与可观测性**，使工具从单次检查演进为持续质量治理平台。

---

## 二、并行执行优化

### 2.1 工具并行调度

当前 v2 按顺序执行所有工具，v3 引入并行调度：

- 无依赖关系的工具并发执行（如 clippy、fmt、doc 可同时运行）
- 有依赖关系的工具保持顺序（如 coverage 依赖 test 产物）
- `config.toml` 支持声明工具间依赖（`depends_on` 字段）

**配置示例**：

```toml
[tools.coverage]
depends_on = ["test"]   # coverage 等待 test 完成后才执行
```

**执行计划输出示例**：

```
[scheduler] Execution plan:
  Stage 1 (parallel): build, fmt, clippy, doc
  Stage 2 (parallel): test, audit, deny, geiger
  Stage 3 (parallel): coverage, bench, bloat, flamegraph
  Stage 4 (parallel): msrv, semver, udeps, deps
```

---

## 三、插件机制

### 3.1 插件接口规范

允许社区或团队贡献自定义工具模板，以标准化方式集成进 rust-checker。

**插件描述文件（`plugin.toml`）**：

```toml
[plugin]
name = "cargo-deny"
version = "1.0.0"
author = "community"
command = "cargo deny check"
report_parser = "deny_parser"   # 指向解析器实现

[plugin.dependencies]
cargo = ["cargo-deny"]
system = []
```

**表格 — 插件接口约定**

| 接口 | 说明 |
|------|------|
| `command` | 执行命令，支持变量替换（如 `${project_root}`） |
| `report_parser` | 解析器名称，对应 Rust trait 实现或内置解析器 |
| `output_schema` | 报告字段的 JSON Schema，用于 summary 聚合 |
| `dependencies` | 声明所需的 cargo 工具和系统依赖 |

### 3.2 插件安装与管理

新增 `rust-checker plugin` 子命令：

```
rust-checker plugin list              # 查看已安装插件
rust-checker plugin add <name>        # 从插件仓库安装
rust-checker plugin remove <name>     # 卸载插件
rust-checker plugin update            # 更新所有插件
```

### 3.3 官方插件仓库

- 维护 `rust-checker-plugins` 仓库，收录社区贡献的标准插件
- 插件通过 `rust-checker plugin add <name>` 一键安装
- 提供插件贡献指南（贡献规范 + CI 验证）

---

## 四、历史趋势追踪

### 4.1 报告历史存储

- 每次 `run` 的结果持久化到 `.localcheck/history/<timestamp>/`
- 保留最近 N 次记录（可配置，默认 10 次）

**配置示例**：

```toml
[history]
max_entries = 10    # 最多保留 10 次历史
```

### 4.2 `diff` 命令增强

在 v2 基础上增强 `diff` 命令：

**基本用法**：

```
rust-checker diff                    # 当前 vs 上一次
rust-checker diff --from 20260510 --to 20260518
rust-checker diff --last 5           # 展示最近 5 次趋势
```

**表格 — diff 输出示例**

| 工具 | 上次 | 本次 | 变化 |
|------|------|------|------|
| cargo test | 40/40 通过 | 42/42 通过 | ↑ +2 |
| cargo llvm-cov | 75.2% | 72.1% | ↓ -3.1% ⚠️ |
| cargo audit | 0 CVE | 1 高危 CVE | ❌ 新增 |
| cargo bloat | 2.8 MB | 3.2 MB | ↑ +400 KB |

### 4.3 趋势图（HTML 模式）

- HTML 报告中为覆盖率、测试通过数、产物大小等数值指标提供折线趋势图
- 数据来源于历史存储，无需外部数据库

---

## 五、Workspace / Monorepo 支持

### 5.1 多 crate 聚合报告

- 自动检测 Cargo workspace，对每个 member 单独运行工具
- 生成 per-crate 报告 + workspace 级别汇总

**目录结构示例**：

```
.localcheck/
└── report/
    ├── summary.md              # workspace 整体汇总
    ├── crate-a/                # 各 member 独立报告
    │   ├── quality/
    │   └── security/
    └── crate-b/
        ├── quality/
        └── security/
```

### 5.2 选择性执行

```
rust-checker run --crate crate-a          # 仅检查指定 crate
rust-checker run --changed                # 仅检查本次 git diff 涉及的 crate
```

---

## 六、配置版本管理

### 6.1 `schema_version` 字段

`config.toml` 加入版本标识，确保配置与工具版本兼容：

```toml
schema_version = "2"
```

### 6.2 `rust-checker upgrade` 命令

自动将旧版配置迁移到新 schema：

```
rust-checker upgrade
  → Detected config schema v1
  → Migrating to schema v2...
  → Backup saved: .localcheck/config.toml.bak
  → Migration complete.
```

**表格 — 迁移规则示例（v1 → v2）**

| v1 字段 | v2 字段 | 说明 |
|--------|--------|------|
| `tools.build.output_path = "report/build.md"` | `tools.build.output_path = "report/quality/build.md"` | 路径结构调整 |
| （无） | `schema_version = "2"` | 自动补充 |

---

## 七、Watch 模式

### 7.1 命令设计

```
rust-checker watch                    # 监听所有 src/ 变更
rust-checker watch --tools build,test # 仅重跑指定工具
```

### 7.2 触发规则配置

```toml
[watch]
paths = ["src/", "tests/"]          # 监听目录
debounce_ms = 500                   # 防抖延迟（ms）

[watch.rules]
"src/**/*.rs" = ["build", "test", "clippy"]
"Cargo.toml"  = ["build", "audit", "deny"]
```

### 7.3 输出方式

- 终端实时刷新，显示当前运行工具与状态
- 仅重新生成受影响工具的报告，summary 自动更新

---

## 八、v3 目录结构更新

```
.localcheck/
├── config.toml           # 含 schema_version
├── history/              # 新增：历史记录
│   ├── 20260510-143200/
│   └── 20260518-093000/
├── logs/
│   └── <timestamp>.log
├── plugins/              # 新增：插件目录
│   └── <plugin-name>/
└── report/
    ├── summary.md
    ├── quality/
    ├── security/
    ├── deps/
    ├── perf/
    └── compat/
```

---

## 九、实现优先级

| 优先级 | 内容 | 说明 |
|--------|------|------|
| P0 | 并行调度（无依赖工具并发执行） | 显著缩短整体耗时 |
| P0 | 历史存储 + `diff` 命令增强 | 趋势感知，治理核心 |
| P1 | Workspace / Monorepo 支持 | 大型项目必要能力 |
| P1 | 配置版本管理（`schema_version` + `upgrade`） | 长期维护可用性 |
| P2 | 插件机制（接口规范 + 安装命令） | 生态开放基础 |
| P2 | Watch 模式 | 开发体验提升 |
| P2 | HTML 趋势图 | 报告可视化增强 |
| P3 | 官方插件仓库 | 生态建设，需社区协作 |
| P3 | 选择性执行（`--changed`） | 依赖 git 集成完善 |

---

## 十、与 v2 的兼容性

- `config.toml` 向前兼容：无 `schema_version` 字段时视为 v1，自动兼容
- v2 报告目录结构保持不变，history 为新增目录
- 插件机制完全可选，不影响现有内置工具行为

---

*本文档为 v3 版本，后续修改请创建新版本文件（如 `plan-v4.md`）以保留历史记录。*
