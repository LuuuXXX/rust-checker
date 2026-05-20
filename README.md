# rust-checker

[![CI](https://github.com/LuuuXXX/rust-checker/actions/workflows/ci.yml/badge.svg)](https://github.com/LuuuXXX/rust-checker/actions/workflows/ci.yml)

`rust-checker` 是一个面向 Rust 项目的本地质量检查工具。它通过统一的配置文件调度多种 Cargo 工具，生成结构化报告，帮助开发者在发布前快速掌握构建质量、测试覆盖率、依赖安全等关键指标。

## 特性

- 🔧 **配置驱动**：所有行为由 `.rust-checker/config.toml` 统一管理
- 🔁 **串行执行**：按声明顺序依次运行，结果确定可重现
- 🔗 **依赖感知**：通过 `depends_on` 声明工具间执行顺序
- 🩺 **依赖预检**：启动前逐一检查工具依赖，自动提示安装
- 📄 **多格式报告**：Markdown（默认）、HTML、JSON
- 📊 **汇总总览**：自动生成 `summary.md`，状态一目了然
- 🤖 **CI 友好**：`--ci` 模式输出机器可读 JSON，工具始终以零退出码退出
- 🪵 **完整日志**：时间戳日志文件，记录工具链环境与执行细节
- 📈 **历史趋势**：每次 `run` 持久化快照，`diff` 命令可视化趋势变化
- 🔌 **插件系统**：从 [rust-checker-plugins](https://github.com/LuuuXXX/rust-checker-plugins) 注册表一键安装第三方工具
- 🏗️ **Workspace 支持**：自动识别 Cargo workspace，支持按 crate 或变更集运行
- 👀 **Watch 模式**：监听文件变更后自动重跑检查

---

## 安装

### 从源码编译

```bash
git clone https://github.com/LuuuXXX/rust-checker.git
cd rust-checker
cargo install --path .
```

### 验证安装

```bash
rust-checker --version
```

---

## 快速上手

```bash
# 1. 进入你的 Rust 项目目录
cd /path/to/your/project

# 2. 初始化配置（选择预设）
rust-checker init --preset quality

# 3. 运行检查
rust-checker run

# 4. 查看汇总报告
cat .rust-checker/reports/summary.md

# 5. 对比上次与本次结果（Phase 3）
rust-checker diff
```

---

## CLI 命令参考

### `rust-checker init`

初始化 `.rust-checker/config.toml` 配置文件。

```
rust-checker init [OPTIONS]

Options:
  -d, --dir <DIR>        项目目录（默认当前目录）
  -p, --preset <PRESET>  预设配置 [默认: quality]
                           minimal   - build, test, clippy, fmt
                           quality   - build, test, coverage, clippy, fmt, doc
                           security  - build, test, audit, deny, geiger
                           full      - 全部 18 个内置工具
      --force            强制重新生成（覆盖已有配置）
  -h, --help             显示帮助信息
```

**示例：**

```bash
# 生成最小配置（4个工具）
rust-checker init --preset minimal

# 生成完整配置（18个工具），覆盖现有配置
rust-checker init --preset full --force

# 为其他目录初始化
rust-checker init --dir /path/to/project --preset quality
```

---

### `rust-checker run`

读取配置并执行所有激活的工具，生成报告与日志。每次运行结果自动保存到历史记录。

```
rust-checker run [OPTIONS]

Options:
  -d, --dir <DIR>              项目目录（默认当前目录）
  -f, --format <FORMAT>        报告格式 [默认: markdown]
                                 markdown | html | json
      --ci                     CI 模式：跳过交互提示，生成 ci_result.json
      --only <TOOLS>           只运行指定工具（逗号分隔）
      --crate <CRATE>          只检查指定 crate（Workspace 模式）
      --changed                检测本次 git diff 是否有 crate 变更；无变更时跳过检查（Workspace 模式）
      --set-cmd <TOOL=CMD>     为特定工具覆盖执行命令（可重复）
  -h, --help                   显示帮助信息
```

**示例：**

```bash
# 运行所有激活的工具
rust-checker run

# 只运行 fmt 和 clippy
rust-checker run --only fmt,clippy

# CI 模式（同时生成 ci_result.json）
rust-checker run --ci

# 生成 HTML 格式报告
rust-checker run --format html

# Workspace：只检查指定 crate
rust-checker run --crate my-lib

# Workspace：有 crate 变更时运行检查，无变更时跳过
rust-checker run --changed

# 为 clippy 使用更严格的参数
rust-checker run --set-cmd clippy="cargo clippy -- -D warnings -W clippy::all"

# 同时为多个工具覆盖命令
rust-checker run --set-cmd build="cargo build --release" --set-cmd test="cargo test -- --nocapture"
```

---

### `rust-checker diff`

对比两次运行结果，或展示历史趋势表格。

```
rust-checker diff [OPTIONS]

Options:
  -d, --dir <DIR>        项目目录（默认当前目录）
      --last <N>         展示最近 N 次运行的趋势表格
      --from <YYYYMMDD>  时间范围起始（与 --to 配合使用）
      --to   <YYYYMMDD>  时间范围结束
  -h, --help             显示帮助信息
```

**示例：**

```bash
# 对比最新两次运行
rust-checker diff

# 展示最近 5 次趋势
rust-checker diff --last 5

# 查看某个日期范围内的变化
rust-checker diff --from 20260101 --to 20260131
```

---

### `rust-checker plugin`

管理来自 [rust-checker-plugins](https://github.com/LuuuXXX/rust-checker-plugins) 注册表的插件。

```
rust-checker plugin <SUBCOMMAND>

Subcommands:
  list              列出已安装的插件
  add <NAME>        从注册表安装插件
  remove <NAME>     卸载插件
  update            更新所有已安装的插件
```

**示例：**

```bash
# 查看已安装插件
rust-checker plugin list

# 安装插件
rust-checker plugin add cargo-expand

# 卸载插件
rust-checker plugin remove cargo-expand

# 更新全部插件
rust-checker plugin update
```

---

### `rust-checker watch`

监听文件变更，自动重跑检查。

```
rust-checker watch [OPTIONS]

Options:
  -d, --dir <DIR>        项目目录（默认当前目录）
      --tools <TOOLS>    只重跑指定工具（逗号分隔，覆盖 config 设置）
  -h, --help             显示帮助信息
```

**示例：**

```bash
# 监听变更，重跑所有工具
rust-checker watch

# 只重跑 clippy 和 fmt
rust-checker watch --tools clippy,fmt
```

在 `config.toml` 中可以预设 watch 行为（见[配置文件格式](#配置文件格式)）。

---

### `rust-checker upgrade`

将 `.rust-checker/config.toml` 从旧 schema 迁移到最新版本，迁移前自动备份原文件。

```
rust-checker upgrade [OPTIONS]

Options:
  -d, --dir <DIR>   项目目录（默认当前目录）
  -h, --help        显示帮助信息
```

**示例：**

```bash
# 将旧版配置（schema_version = "1"）迁移至最新版
rust-checker upgrade
```

---

## 配置文件格式

配置文件位于 `.rust-checker/config.toml`，当前 schema 版本为 `"2"`。

```toml
# 配置 schema 版本（当前为 "2"，init 自动生成；upgrade 命令可迁移旧版）
schema_version = "2"

# Rust 工具链配置（可选）
[rust]
version = "1.75.0"
rustflags = ""

# 历史记录配置（Phase 3 新增）
# 每次 run 自动保存快照到 .rust-checker/history/，超出 max_entries 后自动清理
[history]
max_entries = 10

# Watch 模式配置（Phase 3 新增，可选）
# rust-checker watch 使用此配置
[watch]
paths       = ["src"]     # 监听目录列表
debounce_ms = 500         # 防抖延迟（毫秒）
tools       = ["clippy", "fmt", "test"]  # 变更后只重跑这些工具

# 内置工具配置
[tools.build]
desc = "构建项目"
active = true                      # 是否启用
input_command = "cargo build"      # 执行命令
output_path = "quality/build.md"   # 报告路径（可选，内置工具自动填充）
# depends_on = []                  # 依赖工具列表（可选）

[tools.test]
desc = "运行单元测试"
active = true
input_command = "cargo test"
depends_on = ["build"]             # test 在 build 成功后执行

# 自定义工具示例
[tools.my_script]
desc = "自定义检查脚本"
active = true
input_command = "bash scripts/check.sh"
# 报告自动保存至 customs/my_script.md
```

### 字段说明

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `desc` | string | ✅ | 工具的一句话功能描述 |
| `active` | bool | ✅ | 是否激活该工具 |
| `input_command` | string | ✅ | 执行命令（在项目根目录运行） |
| `output_path` | string | ❌ | 报告路径，内置工具自动填充，自定义工具为 `customs/<name>.md` |
| `depends_on` | array | ❌ | 声明前置依赖工具，被依赖工具失败时自动跳过 |

---

## 内置工具列表

下表列出全部 18 个内置工具，并附有功能简介。

### 代码质量（quality/）

| 工具 | 命令 | 简介 | 报告路径 | 需要安装 |
|------|------|------|---------|---------|
| `build` | `cargo build` | 编译项目，验证代码可成功构建，输出产物信息和编译选项分析 | `quality/build.md` | — |
| `test` | `cargo test` | 运行单元测试和集成测试，统计通过 / 失败 / 忽略用例数及总耗时 | `quality/test.md` | — |
| `coverage` | `cargo llvm-cov` | 基于 LLVM 插桩统计代码覆盖率，输出文件级行覆盖率和总覆盖率 | `quality/coverage.md` | `cargo install cargo-llvm-cov` |
| `clippy` | `cargo clippy -- -D warnings` | 运行官方 lint 工具，列出警告/错误详情（规则名 / 位置 / 建议）| `quality/clippy.md` | `rustup component add clippy` |
| `fmt` | `cargo fmt --check` | 检查代码格式是否符合 `rustfmt` 规范，逐文件列出差异行数 | `quality/fmt.md` | `rustup component add rustfmt` |
| `doc` | `cargo doc --no-deps` | 构建文档，统计警告/错误及公开 API 文档覆盖率 | `quality/doc.md` | — |

### 安全（security/）

| 工具 | 命令 | 简介 | 报告路径 | 需要安装 |
|------|------|------|---------|---------|
| `audit` | `cargo audit` | 扫描依赖树中的已知安全漏洞（对照 RustSec Advisory DB），按严重等级汇总 | `security/audit.md` | `cargo install cargo-audit` |
| `deny` | `cargo deny check` | 检查许可证合规性、依赖白名单/黑名单策略，列出违规项 | `security/deny.md` | `cargo install cargo-deny` |
| `geiger` | `cargo geiger` | 统计项目及所有依赖中的 `unsafe` 代码（fn / block / impl / trait），定位风险来源 | `security/geiger.md` | `cargo install cargo-geiger` |

### 依赖分析（deps/）

| 工具 | 命令 | 简介 | 报告路径 | 需要安装 |
|------|------|------|---------|---------|
| `deps` | `cargo tree` | 展示完整依赖树，统计总依赖数和重复依赖 | `deps/deps.md` | — |
| `udeps` | `cargo +nightly udeps` | 检测 `Cargo.toml` 中声明但实际未使用的依赖，建议移除 | `deps/udeps.md` | `cargo install cargo-udeps` |

### 性能（perf/）

| 工具 | 命令 | 简介 | 报告路径 | 需要安装 |
|------|------|------|---------|---------|
| `metrics` | `cargo geiger --output-format Ratio` | 统计 unsafe 代码占比（按 Ratio 输出），用作代码质量度量指标 | `perf/metrics.md` | `cargo install cargo-geiger` |
| `bench` | `cargo bench` | 运行基准测试，输出各基准项的平均耗时、标准差和吞吐量 | `perf/bench.md` | — |
| `bloat` | `cargo bloat --release` | 分析 Release 二进制体积，列出贡献最大的 crate 和函数（Top-N） | `perf/bloat.md` | `cargo install cargo-bloat` |
| `flamegraph` | `cargo flamegraph` | 生成 CPU 性能火焰图（Linux 需 `perf`，macOS 需 `DTrace`） | `perf/flamegraph.md` | `cargo install flamegraph` |

### 兼容性（compat/）

| 工具 | 命令 | 简介 | 报告路径 | 需要安装 |
|------|------|------|---------|---------|
| `msrv` | `cargo msrv` | 验证项目真实最低支持 Rust 版本（MSRV），与 `Cargo.toml` 中声明值比对 | `compat/msrv.md` | `cargo install cargo-msrv` |
| `semver` | `cargo semver-checks` | 检测公开 API 的破坏性变更（SemVer 违规），辅助版本号决策 | `compat/semver.md` | `cargo install cargo-semver-checks` |
| `binary` | `cargo build --release` | 构建 Release 二进制，记录产物大小、SHA-256 和构建环境信息 | `compat/binary.md` | — |

---

## 报告目录结构

执行 `rust-checker run` 后生成如下结构：

```
.rust-checker/
├── config.toml
├── logs/
│   └── 20260518-143200.log        # 执行日志（含工具链环境信息）
├── history/                       # 历史快照（Phase 3）
│   └── 20260518-143200/
│       └── result.json            # 该次运行的工具结果
└── reports/
    ├── summary.md                 # 汇总总览
    ├── ci_result.json             # CI 模式下额外生成
    ├── quality/
    │   ├── build.md
    │   ├── test.md
    │   ├── coverage.md
    │   ├── clippy.md
    │   ├── fmt.md
    │   └── doc.md
    ├── security/
    │   ├── audit.md
    │   ├── deny.md
    │   └── geiger.md
    ├── deps/
    │   ├── deps.md
    │   └── udeps.md
    ├── perf/
    │   ├── metrics.md
    │   ├── bench.md
    │   ├── bloat.md
    │   └── flamegraph.md
    └── compat/
        ├── binary.md
        ├── msrv.md
        └── semver.md
```

---

## CI 集成

`rust-checker` 始终以**零退出码**退出，不干涉 CI/CD 流水线。CI 脚本通过读取 `ci_result.json` 自行决定是否阻断流水线。

### GitHub Actions 示例

```yaml
name: Quality Check
on: [push, pull_request]

jobs:
  rust-checker:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt

      - name: Install rust-checker
        run: cargo install --path .

      - name: Init config
        run: rust-checker init --preset quality

      - name: Run checks (CI mode)
        run: rust-checker run --ci

      - name: Check for errors
        run: |
          ERROR_COUNT=$(jq '.summary.error' .rust-checker/reports/ci_result.json)
          if [ "$ERROR_COUNT" -gt 0 ]; then
            echo "❌ $ERROR_COUNT tool(s) failed"
            exit 1
          fi

      - name: Upload reports
        uses: actions/upload-artifact@v4
        if: always()
        with:
          name: quality-reports
          path: .rust-checker/reports/
```

### ci_result.json 格式

```json
{
  "timestamp": "2026-05-18T14:32:00",
  "summary": {
    "total": 6,
    "ok": 4,
    "warn": 1,
    "error": 0,
    "skipped": 1
  },
  "tools": [
    { "tool": "build",    "status": "ok",      "summary": "构建成功",       "output_path": "quality/build.md" },
    { "tool": "test",     "status": "ok",      "summary": "通过: 42，失败: 0，忽略: 0", "output_path": "quality/test.md" },
    { "tool": "clippy",   "status": "warn",    "summary": "3 个警告",        "output_path": "quality/clippy.md" },
    { "tool": "fmt",      "status": "ok",      "summary": "无问题",          "output_path": "quality/fmt.md" },
    { "tool": "doc",      "status": "ok",      "summary": "构建成功",        "output_path": "quality/doc.md" },
    { "tool": "coverage", "status": "skipped", "summary": "缺少依赖: cargo-llvm-cov", "output_path": "quality/coverage.md" }
  ]
}
```

---

## 示例配置

- [`examples/minimal.toml`](examples/minimal.toml) — 4 个工具（build、test、clippy、fmt）
- [`examples/standard.toml`](examples/standard.toml) — 6 个工具（代码质量全套）
- [`examples/full.toml`](examples/full.toml) — 18 个工具（完整检查，含 watch 配置）
- [`examples/workspace.toml`](examples/workspace.toml) — Workspace / Monorepo 配置示例

使用示例配置：

```bash
cp examples/standard.toml .rust-checker/config.toml
rust-checker run
```

---

## 开发计划

详见 [`docs/development-plan.md`](docs/development-plan.md)。

| 阶段 | 状态 | 内容 |
|------|------|------|
| Phase 1 | ✅ 完成 | CLI 骨架、配置解析、串行执行引擎、基础报告 |
| Phase 2 | ✅ 完成 | 全部内置工具、汇总报告、CI 友好输出 |
| Phase 3 | ✅ 完成 | 历史趋势追踪、Workspace 支持、插件生态、Watch 模式、配置升级 |

---

## 贡献

欢迎提交 PR 和 Issue。

```bash
# 克隆仓库
git clone https://github.com/LuuuXXX/rust-checker.git
cd rust-checker

# 构建
cargo build

# 运行全部测试（单元测试 + 集成测试）
cargo test

# 格式化代码
cargo fmt

# 运行 Clippy
cargo clippy -- -D warnings
```

---

## 许可证

MIT
