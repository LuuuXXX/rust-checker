# rust-checker

[![CI](https://github.com/LuuuXXX/rust-checker/actions/workflows/ci.yml/badge.svg)](https://github.com/LuuuXXX/rust-checker/actions/workflows/ci.yml)

`rust-checker` 是一个面向 Rust 项目的本地质量检查工具。它通过统一的配置文件调度多种 Cargo 工具，生成结构化报告，帮助开发者在发布前快速掌握构建质量、测试覆盖率、依赖安全等关键指标。

## 特性

- 🔧 **配置驱动**：所有行为由 `.localcheck/config.toml` 统一管理
- 🔁 **串行执行**：按声明顺序依次运行，结果确定可重现
- 🔗 **依赖感知**：通过 `depends_on` 声明工具间执行顺序
- 🩺 **依赖预检**：启动前逐一检查工具依赖，自动提示安装
- 📄 **多格式报告**：Markdown（默认）、HTML、JSON
- 📊 **汇总总览**：自动生成 `summary.md`，状态一目了然
- 🤖 **CI 友好**：`--ci` 模式输出机器可读 JSON，工具始终以零退出码退出
- 🪵 **完整日志**：时间戳日志文件，记录工具链环境与执行细节

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
rust-checker init --preset standard

# 3. 运行检查
rust-checker run

# 4. 查看汇总报告
cat .localcheck/reports/summary.md
```

---

## CLI 命令参考

### `rust-checker init`

初始化 `.localcheck/config.toml` 配置文件。

```
rust-checker init [OPTIONS]

Options:
  -d, --dir <DIR>        项目目录（默认当前目录）
  -p, --preset <PRESET>  预设配置 [默认: minimal]
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

读取配置并执行所有激活的工具，生成报告与日志。

```
rust-checker run [OPTIONS]

Options:
  -d, --dir <DIR>          项目目录（默认当前目录）
  -f, --format <FORMAT>    报告格式 [默认: markdown]
                             markdown | html | json
      --ci                 CI 模式：跳过交互提示，生成 ci_result.json
      --only <TOOLS>       只运行指定工具（逗号分隔）
  -h, --help               显示帮助信息
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
```

---

## 配置文件格式

配置文件位于 `.localcheck/config.toml`。

```toml
# 配置 schema 版本（可选，用于未来迁移）
schema_version = "1"

# Rust 工具链配置（可选）
[rust]
version = "1.75.0"
rustflags = ""

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

| 工具 | 命令 | 报告路径 | 需要安装 |
|------|------|---------|---------|
| `build` | `cargo build` | `quality/build.md` | — |
| `test` | `cargo test` | `quality/test.md` | — |
| `coverage` | `cargo llvm-cov` | `quality/coverage.md` | `cargo install cargo-llvm-cov` |
| `clippy` | `cargo clippy` | `quality/clippy.md` | `rustup component add clippy` |
| `fmt` | `cargo fmt --check` | `quality/fmt.md` | `rustup component add rustfmt` |
| `doc` | `cargo doc --no-deps` | `quality/doc.md` | — |
| `audit` | `cargo audit` | `security/audit.md` | `cargo install cargo-audit` |
| `deny` | `cargo deny check` | `security/deny.md` | `cargo install cargo-deny` |
| `geiger` | `cargo geiger` | `security/geiger.md` | `cargo install cargo-geiger` |
| `metrics` | `cargo geiger --output-format Ratio` | `perf/metrics.md` | `cargo install cargo-geiger` |
| `deps` | `cargo tree` | `deps/deps.md` | — |
| `msrv` | `cargo msrv` | `compat/msrv.md` | `cargo install cargo-msrv` |
| `semver` | `cargo semver-checks` | `compat/semver.md` | `cargo install cargo-semver-checks` |
| `udeps` | `cargo +nightly udeps` | `deps/udeps.md` | `cargo install cargo-udeps` |
| `bench` | `cargo bench` | `perf/bench.md` | — |
| `bloat` | `cargo bloat --release` | `perf/bloat.md` | `cargo install cargo-bloat` |
| `flamegraph` | `cargo flamegraph` | `perf/flamegraph.md` | `cargo install flamegraph` |
| `binary` | `cargo build --release` | `compat/binary.md` | — |

---

## 报告目录结构

执行 `rust-checker run` 后生成如下结构：

```
.localcheck/
├── config.toml
├── logs/
│   └── 20260518-143200.log        # 执行日志（含工具链环境信息）
└── reports/
    ├── summary.md                 # 汇总总览
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
          ERROR_COUNT=$(jq '.summary.error' .localcheck/reports/ci_result.json)
          if [ "$ERROR_COUNT" -gt 0 ]; then
            echo "❌ $ERROR_COUNT tool(s) failed"
            exit 1
          fi

      - name: Upload reports
        uses: actions/upload-artifact@v4
        if: always()
        with:
          name: quality-reports
          path: .localcheck/reports/
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
    { "tool": "coverage", "status": "ok",      "summary": "覆盖率: 87.5%",  "output_path": "quality/coverage.md" },
    { "tool": "doc",      "status": "skipped", "summary": "缺少依赖: cargo", "output_path": "quality/doc.md" }
  ]
}
```

---

## 示例配置

- [`examples/minimal.toml`](examples/minimal.toml) — 4 个工具（build、test、clippy、fmt）
- [`examples/standard.toml`](examples/standard.toml) — 6 个工具（代码质量全套）
- [`examples/full.toml`](examples/full.toml) — 18 个工具（完整检查）

使用示例配置：

```bash
cp examples/standard.toml .localcheck/config.toml
rust-checker run
```

---

## 开发计划

详见 [`docs/development-plan.md`](docs/development-plan.md)。

| 阶段 | 状态 | 内容 |
|------|------|------|
| Phase 1 | ✅ 完成 | CLI 骨架、配置解析、串行执行引擎、基础报告 |
| Phase 2 | ✅ 完成 | 全部内置工具、汇总报告、CI 友好输出 |
| Phase 3 | 🔜 规划中 | 历史趋势追踪、Workspace 支持、插件生态 |

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