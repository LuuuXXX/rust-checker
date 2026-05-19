# rust-checker 开发计划

> 状态：Phase 1 ✅ 完成 · Phase 2 ✅ 完成 · Phase 3 ✅ 完成

---

## 一、总体目标

将 `rust-checker` 从框架原型演进为一个**生产可用、生态开放的 Rust 项目持续质量治理平台**，分三个阶段落地：

| 阶段 | 目标定位 | 关键里程碑 | 状态 |
|------|---------|-----------|------|
| Phase 1 | 核心框架可用 | CLI 骨架 + 基础工具 + 结构化报告 | ✅ 完成 |
| Phase 2 | 生产可用 | 工具全覆盖 + CI 集成 + 汇总报告 | ✅ 完成 |
| Phase 3 | 平台化 | 插件生态 + 趋势追踪 + Workspace 支持 | ✅ 完成 |

---

## 二、Phase 1 — 核心框架

### 目标
建立可运行的 CLI 骨架，支持配置驱动的工具调度，输出基础报告与日志。

---

### 2.1 目录与配置规范

- [x] 确定 `.localcheck/` 目录结构（config / logs / reports）
- [x] 实现 `config.toml` 解析（工具列表、激活状态、命令、输出路径）
- [x] 支持内置工具自动填充 `output_path`，自定义工具自动分配 `customs/<name>.md` 路径

### 2.2 `rust-checker init` 命令

- [x] 检测 `.localcheck/config.toml` 是否已存在，存在则 `--force` 才覆盖
- [x] 实现 `--preset minimal / quality / security / full` 四档预设
  - `minimal`：build + test + clippy + fmt
  - `quality`：build + test + coverage + clippy + fmt + doc
  - `security`：build + test + audit + deny + geiger
  - `full`：所有 18 个内置工具

### 2.3 `rust-checker run` 命令（核心执行引擎）

- [x] 串行执行模型：按配置声明顺序依次运行，输出清晰进度日志
- [x] 支持 `depends_on` 字段声明工具间执行依赖（拓扑排序）
- [x] 运行依赖预检：启动前逐一检查工具依赖是否满足
  - [x] 有 cargo 安装方案 → 打印命令并询问是否自动安装
  - [x] 无已知方案 → 提示工具为可选并跳过
- [x] 创建 `.localcheck/reports/` 与 `.localcheck/logs/` 目录
- [x] 内置工具按标准模板解析输出，生成结构化报告
- [x] 自定义工具捕获原始输出，存为 `customs/<name>.md`
- [x] 汇总日志写入 `.localcheck/logs/<YYYYMMDD-HHMMSS>.log`

### 2.4 报告格式支持

- [x] Markdown（默认，适合 Git 归档）
- [x] HTML（适合浏览器查看）
- [x] JSON（适合 CI/CD 集成与二次处理）

### 2.5 内置工具 — 报告模板（Phase 1 交付）

| 工具 | 命令 | 报告路径 |
|------|------|---------|
| 构建检查 | `cargo build` | `quality/build.md` |
| 测试 | `cargo test` | `quality/test.md` |
| 测试覆盖率 | `cargo llvm-cov` | `quality/coverage.md` |
| 代码度量 | `cargo geiger`（基础指标） | `perf/metrics.md` |

- [x] build 报告：构建基本信息、产物信息、编译选项分析、优化建议
- [x] test 报告：测试基本信息、测试结果（用例 / 耗时 / 总时长）
- [x] coverage 报告：文件级条件覆盖率 / 分支覆盖率 / 总覆盖率
- [x] metrics 报告：圈复杂度、代码行数、unsafe 代码比例

### 2.6 日志规范

- [x] 记录工具链环境信息（OS、Rust 版本）
- [x] 记录每个工具的开始 / 结束时间
- [x] 记录被跳过工具及原因
- [x] 记录标准输出与标准错误完整内容
- [x] 记录报告文件生成路径

---

## 三、Phase 2 — 生产可用

### 目标
补全所有内置工具，完善报告体系，支持 CI 集成，提升上手体验。

---

### 3.1 补全 Phase 1 遗留报告模板

- [x] **依赖分析**（`cargo tree`）：依赖统计概览、依赖树节选、重复依赖列表
- [x] **依赖安全检查**（`cargo audit`）：安全概览（严重 / 高危 / 中危 / 低危）、漏洞详情、受影响依赖路径
- [x] **火焰图**（`cargo flamegraph`）：生成信息、Top-5 热点函数；平台差异说明（Linux perf / macOS DTrace）
- [x] **二进制信息**（`cargo build --release`）：构建环境信息、产物 SHA-256 对比

### 3.2 新增内置工具（P1）

| 工具 | 命令 | 报告路径 |
|------|------|---------|
| Clippy 代码规范 | `cargo clippy` | `quality/clippy.md` |
| 格式检查 | `cargo fmt --check` | `quality/fmt.md` |
| 文档构建检查 | `cargo doc --no-deps` | `quality/doc.md` |
| 许可证与依赖策略 | `cargo deny check` | `security/deny.md` |
| unsafe 溯源统计 | `cargo geiger` | `security/geiger.md` |
| MSRV 检查 | `cargo msrv` | `compat/msrv.md` |

- [x] clippy 报告：检查概览（warnings / errors / 涉及文件数）、问题详情（lint 规则 / 等级 / 文件 / 行号）
- [x] fmt 报告：逐文件格式检查状态、差异行数、修复命令提示
- [x] doc 报告：构建概览（warnings / errors / 公开 API 文档覆盖率）、未文档化的公开项列表
- [x] deny 报告：许可证概览（各证书 crate 数量 / 状态）、违规与警告项
- [x] geiger 报告：项目 unsafe 汇总（fn / block / impl / trait / 行数）、各依赖 unsafe 分布 Top-N
- [x] msrv 报告：声明 MSRV vs 验证结果 vs 实际最低版本、影响 MSRV 的依赖列表

### 3.3 新增内置工具（P2）

| 工具 | 命令 | 报告路径 |
|------|------|---------|
| 语义化版本验证 | `cargo semver-checks` | `compat/semver.md` |
| 未使用依赖检测 | `cargo +nightly udeps` | `deps/udeps.md` |
| 性能基准 | `cargo bench` | `perf/bench.md` |
| 二进制体积分析 | `cargo bloat --release` | `perf/bloat.md` |

- [x] semver 报告：API 破坏性变更列表（条目 / 变更类型 / 严重性 / 说明）
- [x] udeps 报告：未使用依赖列表（包名 / 版本 / 类型 / 建议）
- [x] bench 报告：基准测试概览（数量 / 最慢项）、各基准结果（平均耗时 / 标准差 / 吞吐量）
- [x] bloat 报告：体积概览（总大小 / .text 段 / 最大贡献 crate）、体积贡献 Top-N

### 3.4 报告目录结构重组

```
.localcheck/
├── config.toml
├── logs/
│   └── <timestamp>.log
└── reports/
    ├── summary.md              # 聚合总览
    ├── quality/                # 代码质量
    │   ├── build.md
    │   ├── test.md
    │   ├── coverage.md
    │   ├── clippy.md
    │   ├── fmt.md
    │   └── doc.md
    ├── security/               # 安全
    │   ├── audit.md
    │   ├── deny.md
    │   └── geiger.md
    ├── deps/                   # 依赖分析
    │   ├── deps.md
    │   └── udeps.md
    ├── perf/                   # 性能
    │   ├── flamegraph.md
    │   ├── bench.md
    │   ├── bloat.md
    │   └── metrics.md
    └── compat/                 # 兼容性
        ├── binary.md
        ├── msrv.md
        └── semver.md
```

- [x] 重组报告子目录（quality / security / deps / perf / compat）

### 3.5 聚合汇总报告（`summary.md`）

- [x] 所有工具以一页总览呈现，含状态图标（✅ / ⚠️ / ❌）
- [x] 每个工具包含：状态、关键指标摘要、报告链接
- [ ] HTML 格式提供带样式的 Dashboard 页面（留待 Phase 3）

### 3.6 CI 友好输出

- [x] `run --ci` 参数：以机器可读 JSON 格式输出摘要并写入 `ci_result.json`
- [x] 每个工具报告包含明确的 `status: ok / warn / error / skipped` 状态字段
- [x] 提供 CI 脚本示例（GitHub Actions），展示如何读取报告并自行决定是否失败
- [x] 工具本身始终以零退出码退出，不干涉 CI/CD 流水线

### 3.7 测试、示例与文档（本轮补充）

- [x] 单元测试覆盖：所有 18 个工具解析器（happy path + edge case）
- [x] 新增单元测试：config 加载、report 写入、topo sort、dependency_check、runner
- [x] 集成测试：`init` / `run` 端到端测试（使用编译后二进制 + tempdir）
- [x] 示例配置：`examples/minimal.toml`、`examples/standard.toml`、`examples/full.toml`
- [x] README：完整说明（安装、快速上手、命令参考、配置格式、报告结构、CI 集成）
- [x] GitHub Actions CI：`.github/workflows/ci.yml`（lint + 多平台 test + E2E self-check）

---

## 四、Phase 3 — 平台化

### 目标
引入插件生态、历史趋势追踪和 Workspace 支持，使工具演进为持续质量治理平台。

---

### 4.1 历史趋势追踪（P0）

- [x] 每次 `run` 结果持久化到 `.localcheck/history/<timestamp>/`
- [x] `config.toml` 新增 `[history]` 段，支持 `max_entries` 配置（默认 10）
- [x] 增强 `rust-checker diff` 命令：
  - `diff`：当前 vs 上一次
  - `diff --from <date> --to <date>`：指定时间段对比
  - `diff --last <n>`：展示最近 N 次趋势
- [x] diff 输出：每个工具的关键指标变化（↑ / ↓ / ❌ 新增）
- [ ] HTML 报告中为数值指标（覆盖率、测试通过数、产物大小等）提供折线趋势图

### 4.2 Workspace / Monorepo 支持（P1）

- [x] 自动检测 Cargo workspace，识别所有 member crate
- [x] `rust-checker run --crate <name>`：仅检查指定 crate
- [x] `rust-checker run --changed`：仅检查本次 git diff 涉及的 crate

### 4.3 配置版本管理（P1）

- [x] 实现 `rust-checker upgrade` 命令：自动迁移旧版配置到新 schema，迁移前备份原文件
- [x] 无 `schema_version` 字段时视为旧版配置，自动兼容

### 4.4 插件机制（P2）

**接口规范**

- [x] 定义并稳定 `plugin.toml` schema（command / report_parser / output_schema / dependencies）
- [x] 执行引擎支持动态加载插件的 `command` 与 `report_parser`

**安装与管理**

- [x] 实现 `rust-checker plugin` 子命令：
  - `plugin list`：查看已安装插件
  - `plugin add <name>`：从插件仓库安装（拉取 `plugin.toml` 到 `.localcheck/plugins/<name>/`）
  - `plugin remove <name>`：卸载插件
  - `plugin update`：更新所有插件

**官方插件仓库**

- [x] 创建 `rust-checker-plugins` GitHub 仓库作为插件注册表
- [x] 初始收录已有内置工具的 `plugin.toml` 描述文件作为示例
- [ ] 提供插件贡献指南（贡献规范 + CI 验证）

### 4.5 Watch 模式（P2）

- [x] 实现 `rust-checker watch` 命令，监听文件变更后自动重跑受影响工具
- [x] 支持 `--tools` 参数指定只重跑哪些工具
- [x] `config.toml` 新增 `[watch]` 段：
  - `paths`：监听目录列表
  - `debounce_ms`：防抖延迟
  - `tools`：文件变更时触发的工具列表

---

## 五、跨阶段通用事项

### 5.1 跨平台兼容

- [x] 跨平台 CI 矩阵（ubuntu-latest / macos-latest / windows-latest）
- [ ] 明确区分 Windows / Linux / macOS 的系统依赖安装提示（apt / brew / winget / choco）

### 5.2 测试策略

- [x] 每个内置工具的报告解析器有对应单元测试（含 happy path 和 edge case）—— 99 个单元测试
- [x] `init` / `run` 命令有集成测试（使用编译后二进制 + tempdir）—— 11 个集成测试
- [x] 跨平台 CI 矩阵（ubuntu-latest / macos-latest / windows-latest）
- [ ] `diff` 命令集成测试（Phase 3）

### 5.3 向后兼容原则

- [x] `config.toml` 新字段均为可选，无新字段时行为等同旧版
- [x] 报告格式只在新字段上扩展，已有字段不重命名或删除
- [x] `schema_version` 未设置时按最低版本逻辑处理

### 5.4 文档

- [x] 每个内置工具的报告字段说明（字段含义、来源、示例值）
- [x] CLI 命令参考文档（所有子命令、参数、示例）
- [x] 快速上手指南（安装 → init → run → 查看报告）
- [x] CI 集成指南（GitHub Actions 示例）
- [ ] 插件开发指南（plugin.toml 规范 + 贡献流程）（Phase 3）

---

## 六、实现优先级汇总

| 优先级 | 阶段 | 内容 | 状态 |
|--------|------|------|------|
| P0 | Phase 1 | CLI 骨架：`init` / `run`，配置解析，串行执行引擎 | ✅ |
| P0 | Phase 1 | 基础报告：build / test / coverage / metrics | ✅ |
| P0 | Phase 1 | 运行依赖预检（分级提示 + 可选自动安装） | ✅ |
| P0 | Phase 1 | 日志系统（结构化写入 + 时间戳文件名） | ✅ |
| P0 | Phase 2 | 补全 Phase 1 遗留报告：deps / audit / flamegraph / binary | ✅ |
| P0 | Phase 2 | CI 友好输出（`--ci` 参数 + 状态字段） | ✅ |
| P1 | Phase 2 | Clippy / fmt / doc 内置工具 | ✅ |
| P1 | Phase 2 | cargo deny（许可证合规 + 依赖策略） | ✅ |
| P1 | Phase 2 | cargo geiger（unsafe 溯源统计） | ✅ |
| P1 | Phase 2 | cargo msrv（MSRV 检查） | ✅ |
| P1 | Phase 2 | `init --preset` 分级预设 | ✅ |
| P1 | Phase 2 | 聚合汇总报告（summary.md） | ✅ |
| P1 | Phase 2 | 测试补全（单元测试 99 个 + 集成测试 11 个） | ✅ |
| P1 | Phase 2 | 示例配置（minimal / standard / full） | ✅ |
| P1 | Phase 2 | 文档（README + development-plan.md 更新） | ✅ |
| P1 | Phase 2 | GitHub Actions CI（lint + 多平台 test + E2E） | ✅ |
| P1 | Phase 3 | 历史存储 + `diff` 命令增强 | 🔜 |
| P1 | Phase 3 | Workspace / Monorepo 支持 | 🔜 |
| P1 | Phase 3 | 配置版本管理（`schema_version` + `upgrade`） | 🔜 |
| P2 | Phase 2 | cargo semver-checks（语义化版本验证） | ✅ |
| P2 | Phase 2 | cargo udeps（未使用依赖检测，nightly） | ✅ |
| P2 | Phase 2 | cargo bench / cargo bloat（性能基准 + 体积分析） | ✅ |
| P2 | Phase 3 | 插件机制（接口规范 + 安装命令） | 🔜 |
| P2 | Phase 3 | Watch 模式 | 🔜 |
| P2 | Phase 3 | HTML 趋势图 | 🔜 |
| P3 | Phase 3 | 官方插件仓库（rust-checker-plugins） | 🔜 |
| P3 | Phase 3 | 选择性执行（`--changed`，依赖 git 集成） | 🔜 |

---

*各阶段完成后请在对应 checklist 打勾。*

---

## 一、总体目标

将 `rust-checker` 从框架原型演进为一个**生产可用、生态开放的 Rust 项目持续质量治理平台**，分三个阶段落地：

| 阶段 | 目标定位 | 关键里程碑 |
|------|---------|-----------|
| Phase 1 | 核心框架可用 | CLI 骨架 + 基础工具 + 结构化报告 |
| Phase 2 | 生产可用 | 工具全覆盖 + CI 集成 + 汇总报告 |
| Phase 3 | 平台化 | 插件生态 + 趋势追踪 + Workspace 支持 |

---

## 二、Phase 1 — 核心框架

### 目标
建立可运行的 CLI 骨架，支持配置驱动的工具调度，输出基础报告与日志。

---

### 2.1 目录与配置规范

- [ ] 确定 `.localcheck/` 目录结构（config / logs / report / history / plugins）
- [ ] 实现 `config.toml` 解析（工具列表、激活状态、命令、输出路径）
- [ ] 支持内置工具自动填充 `output_path`，自定义工具自动分配 `customs_<name>` 前缀

### 2.2 `rust-checker init` 命令

- [ ] 检测 `.localcheck/config.toml` 是否已存在，存在则询问是否覆盖
- [ ] 提供交互式引导：默认配置 vs 自定义配置
- [ ] 实现 `--preset minimal / standard / full` 三档预设
  - `minimal`：build + test
  - `standard`：build + test + coverage + audit（默认推荐）
  - `full`：所有内置工具

### 2.3 `rust-checker run` 命令（核心执行引擎）

- [ ] 串行执行模型：按配置声明顺序依次运行，输出清晰进度日志
- [ ] 支持 `depends_on` 字段声明工具间执行依赖
- [ ] 运行依赖预检：启动前逐一检查工具依赖是否满足
  - 有 cargo 安装方案 → 打印命令并询问是否自动安装
  - 需手动操作（系统包）→ 打印平台对应安装命令（apt / brew / winget）
  - 无已知方案 → 提示工具为可选并跳过
- [ ] 创建 `.localcheck/report/` 与 `.localcheck/logs/` 目录
- [ ] 内置工具按标准模板解析输出，生成结构化报告
- [ ] 自定义工具捕获原始输出，存为 `customs_<name>.<ext>`
- [ ] 汇总日志写入 `.localcheck/logs/<YYYYMMDD-HHMMSS>.log`

### 2.4 报告格式支持

- [ ] Markdown（默认，适合 Git 归档）
- [ ] HTML（适合浏览器查看）
- [ ] JSON（适合 CI/CD 集成与二次处理）

### 2.5 内置工具 — 报告模板（Phase 1 交付）

| 工具 | 命令 | 报告路径 |
|------|------|---------|
| 构建检查 | `cargo build` | `quality/build.md` |
| 测试 | `cargo test` | `quality/test.md` |
| 测试覆盖率 | `cargo llvm-cov` | `quality/coverage.md` |
| 代码度量 | `cargo geiger`（基础指标） | `perf/metrics.md` |

- [ ] build 报告：构建基本信息、产物信息、编译选项分析、优化建议
- [ ] test 报告：测试基本信息、测试结果（用例 / 耗时 / 总时长）
- [ ] coverage 报告：文件级条件覆盖率 / 分支覆盖率 / 总覆盖率
- [ ] metrics 报告：圈复杂度、代码行数、unsafe 代码比例

### 2.6 日志规范

- [ ] 记录工具链环境信息（OS、Rust 版本）
- [ ] 记录每个工具的开始 / 结束时间
- [ ] 记录被跳过工具及原因
- [ ] 记录标准输出与标准错误完整内容
- [ ] 记录报告文件生成路径

---

## 三、Phase 2 — 生产可用

### 目标
补全所有内置工具，完善报告体系，支持 CI 集成，提升上手体验。

---

### 3.1 补全 Phase 1 遗留报告模板

- [ ] **依赖分析**（`cargo tree` + `cargo machete`）：依赖统计概览、依赖树节选、重复依赖列表、未使用依赖
- [ ] **依赖安全检查**（`cargo audit`）：安全概览（严重 / 高危 / 中危 / 低危）、漏洞详情、受影响依赖路径
- [ ] **火焰图**（`cargo flamegraph`）：生成信息、Top-5 热点函数；平台差异说明（Linux perf / macOS DTrace）
- [ ] **二进制一致性检查**：构建环境信息、产物 SHA-256 对比、影响一致性的环境因素清单

### 3.2 新增内置工具（P1）

| 工具 | 命令 | 报告路径 |
|------|------|---------|
| Clippy 代码规范 | `cargo clippy` | `quality/clippy.md` |
| 格式检查 | `cargo fmt --check` | `quality/fmt.md` |
| 文档构建检查 | `cargo doc --no-deps` | `quality/doc.md` |
| 许可证与依赖策略 | `cargo deny check` | `security/deny.md` |
| unsafe 溯源统计 | `cargo geiger` | `security/geiger.md` |
| MSRV 检查 | `cargo msrv` | `compat/msrv.md` |

- [ ] clippy 报告：检查概览（warnings / errors / 涉及文件数）、问题详情（lint 规则 / 等级 / 文件 / 行号）
- [ ] fmt 报告：逐文件格式检查状态、差异行数、修复命令提示
- [ ] doc 报告：构建概览（warnings / errors / 公开 API 文档覆盖率）、未文档化的公开项列表
- [ ] deny 报告：许可证概览（各证书 crate 数量 / 状态）、违规与警告项
- [ ] geiger 报告：项目 unsafe 汇总（fn / block / impl / trait / 行数）、各依赖 unsafe 分布 Top-N
- [ ] msrv 报告：声明 MSRV vs 验证结果 vs 实际最低版本、影响 MSRV 的依赖列表

### 3.3 新增内置工具（P2）

| 工具 | 命令 | 报告路径 |
|------|------|---------|
| 语义化版本验证 | `cargo semver-checks` | `compat/semver.md` |
| 未使用依赖检测 | `cargo +nightly udeps` | `deps/udeps.md` |
| 性能基准 | `cargo bench` | `perf/bench.md` |
| 二进制体积分析 | `cargo bloat --release` | `perf/bloat.md` |

- [ ] semver 报告：API 破坏性变更列表（条目 / 变更类型 / 严重性 / 说明）
- [ ] udeps 报告：未使用依赖列表（包名 / 版本 / 类型 / 建议）
- [ ] bench 报告：基准测试概览（数量 / 最慢项）、各基准结果（平均耗时 / 标准差 / 吞吐量）
- [ ] bloat 报告：体积概览（总大小 / .text 段 / 最大贡献 crate）、体积贡献 Top-N

### 3.4 报告目录结构重组

```
.localcheck/
├── config.toml
├── logs/
│   └── <timestamp>.log
└── report/
    ├── summary.md              # 聚合总览
    ├── quality/                # 代码质量
    │   ├── build.md
    │   ├── test.md
    │   ├── coverage.md
    │   ├── clippy.md
    │   ├── fmt.md
    │   └── doc.md
    ├── security/               # 安全
    │   ├── audit.md
    │   ├── deny.md
    │   └── geiger.md
    ├── deps/                   # 依赖分析
    │   ├── deps.md
    │   └── udeps.md
    ├── perf/                   # 性能
    │   ├── flamegraph.md
    │   ├── bench.md
    │   ├── bloat.md
    │   └── metrics.md
    └── compat/                 # 兼容性
        ├── binary.md
        ├── msrv.md
        └── semver.md
```

- [ ] 重组报告子目录（quality / security / deps / perf / compat）
- [ ] 提供旧版扁平路径 → 子目录路径的向后兼容适配层

### 3.5 聚合汇总报告（`summary.md`）

- [ ] 所有工具以一页总览呈现，含状态图标（✅ / ⚠️ / ❌）
- [ ] 每个工具包含：状态、关键指标摘要、报告链接
- [ ] HTML 格式提供带样式的 Dashboard 页面

### 3.6 CI 友好输出

- [ ] `run --ci` 参数：以机器可读格式（JSON / plain）输出摘要
- [ ] 每个工具报告包含明确的 `status: ok / warn / error` 状态字段
- [ ] 提供 CI 脚本示例（GitHub Actions / GitLab CI），展示如何读取报告并自行决定是否失败
- [ ] 工具本身始终以零退出码退出，不干涉 CI/CD 流水线

---

## 四、Phase 3 — 平台化

### 目标
引入插件生态、历史趋势追踪和 Workspace 支持，使工具演进为持续质量治理平台。

---

### 4.1 历史趋势追踪（P0）

- [ ] 每次 `run` 结果持久化到 `.localcheck/history/<timestamp>/`
- [ ] `config.toml` 新增 `[history]` 段，支持 `max_entries` 配置（默认 10）
- [ ] 增强 `rust-checker diff` 命令：
  - `diff`：当前 vs 上一次
  - `diff --from <date> --to <date>`：指定时间段对比
  - `diff --last <n>`：展示最近 N 次趋势
- [ ] diff 输出：每个工具的关键指标变化（↑ / ↓ / ❌ 新增）
- [ ] HTML 报告中为数值指标（覆盖率、测试通过数、产物大小等）提供折线趋势图

### 4.2 Workspace / Monorepo 支持（P1）

- [ ] 自动检测 Cargo workspace，对每个 member 单独运行工具
- [ ] 生成 per-crate 报告 + workspace 级别汇总（`summary.md` 展示所有 member 状态）
- [ ] `rust-checker run --crate <name>`：仅检查指定 crate
- [ ] `rust-checker run --changed`：仅检查本次 git diff 涉及的 crate

### 4.3 配置版本管理（P1）

- [ ] `config.toml` 新增 `schema_version` 字段
- [ ] 实现 `rust-checker upgrade` 命令：自动迁移旧版配置到新 schema，迁移前备份原文件
- [ ] 无 `schema_version` 字段时视为旧版配置，自动兼容

### 4.4 插件机制（P2）

**接口规范**

- [ ] 定义并稳定 `plugin.toml` schema（command / report_parser / output_schema / dependencies）
- [ ] 执行引擎支持动态加载插件的 `command` 与 `report_parser`

**安装与管理**

- [ ] 实现 `rust-checker plugin` 子命令：
  - `plugin list`：查看已安装插件
  - `plugin add <name>`：从插件仓库安装（拉取 `plugin.toml` 到 `.localcheck/plugins/<name>/`）
  - `plugin remove <name>`：卸载插件
  - `plugin update`：更新所有插件

**官方插件仓库**

- [ ] 创建 `rust-checker-plugins` GitHub 仓库作为插件注册表
- [ ] 初始收录已有内置工具的 `plugin.toml` 描述文件作为示例
- [ ] 提供插件贡献指南（贡献规范 + CI 验证）

### 4.5 Watch 模式（P2）

- [ ] 实现 `rust-checker watch` 命令，监听文件变更后自动重跑受影响工具
- [ ] 支持 `--tools` 参数指定只重跑哪些工具
- [ ] `config.toml` 新增 `[watch]` 段：
  - `paths`：监听目录列表
  - `debounce_ms`：防抖延迟
  - `[watch.rules]`：文件模式 → 触发工具映射
- [ ] 终端实时刷新状态，仅重新生成受影响工具的报告，summary 自动更新

---

## 五、跨阶段通用事项

### 5.1 跨平台兼容

- [ ] 明确区分 Windows / Linux / macOS 的运行差异（路径分隔符、权限要求、工具可用性）
- [ ] 系统依赖安装提示覆盖三个平台（apt / brew / winget / choco）

### 5.2 测试策略

- [ ] 每个内置工具的报告解析器有对应单元测试（含 happy path 和 edge case）
- [ ] `init` / `run` / `diff` 命令有集成测试（使用 fixture 项目）
- [ ] 跨平台 CI 矩阵（ubuntu-latest / macos-latest / windows-latest）

### 5.3 向后兼容原则

- [ ] `config.toml` 新字段均为可选，无新字段时行为等同旧版
- [ ] 报告格式只在新字段上扩展，已有字段不重命名或删除
- [ ] `schema_version` 未设置时按最低版本逻辑处理

### 5.4 文档

- [ ] 每个内置工具的报告字段说明（字段含义、来源、示例值）
- [ ] CLI 命令参考文档（所有子命令、参数、示例）
- [ ] 快速上手指南（安装 → init → run → 查看报告）
- [ ] CI 集成指南（GitHub Actions / GitLab CI 示例）
- [ ] 插件开发指南（plugin.toml 规范 + 贡献流程）

---

## 六、实现优先级汇总

| 优先级 | 阶段 | 内容 |
|--------|------|------|
| P0 | Phase 1 | CLI 骨架：`init` / `run`，配置解析，串行执行引擎 |
| P0 | Phase 1 | 基础报告：build / test / coverage / metrics |
| P0 | Phase 1 | 运行依赖预检（分级提示 + 可选自动安装） |
| P0 | Phase 1 | 日志系统（结构化写入 + 时间戳文件名） |
| P0 | Phase 2 | 补全 Phase 1 遗留报告：deps / audit / flamegraph / binary |
| P0 | Phase 2 | CI 友好输出（`--ci` 参数 + 状态字段） |
| P1 | Phase 2 | Clippy / fmt / doc 内置工具 |
| P1 | Phase 2 | cargo deny（许可证合规 + 依赖策略） |
| P1 | Phase 2 | cargo geiger（unsafe 溯源统计） |
| P1 | Phase 2 | cargo msrv（MSRV 检查） |
| P1 | Phase 2 | `init --preset` 分级预设 |
| P1 | Phase 2 | 聚合汇总报告（summary.md + HTML Dashboard） |
| P1 | Phase 3 | 历史存储 + `diff` 命令增强 |
| P1 | Phase 3 | Workspace / Monorepo 支持 |
| P1 | Phase 3 | 配置版本管理（`schema_version` + `upgrade`） |
| P2 | Phase 2 | cargo semver-checks（语义化版本验证） |
| P2 | Phase 2 | cargo udeps（未使用依赖检测，nightly） |
| P2 | Phase 2 | cargo bench / cargo bloat（性能基准 + 体积分析） |
| P2 | Phase 3 | 插件机制（接口规范 + 安装命令） |
| P2 | Phase 3 | Watch 模式 |
| P2 | Phase 3 | HTML 趋势图 |
| P3 | Phase 3 | 官方插件仓库（rust-checker-plugins） |
| P3 | Phase 3 | 选择性执行（`--changed`，依赖 git 集成） |

---

*各阶段完成后请在对应 checklist 打勾。*
