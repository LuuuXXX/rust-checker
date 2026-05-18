# rust-checker 项目规划 v2

> 文档版本：v2  
> 创建日期：2026-05-18  
> 状态：草稿

---

## 一、v2 目标定位

v1 奠定了工具框架；v2 聚焦三个方向：**补全 v1 未完成项**、**落地 v1 后续规划**、**引入新能力**，使工具具备生产可用性。

---

## 二、补全 v1 未完成的报告模板

### 2.1 依赖分析（`cargo tree` / `cargo machete`）

**表格 1 — 依赖统计概览**

| 直接依赖 | 间接依赖 | 重复依赖（包名） | 未使用依赖 |
|---------|---------|----------------|-----------|
| 12 | 87 | 3 | 2 |

**依赖树（节选）**

```
myproject v0.1.0
├── serde v1.0.193
│   └── serde_derive v1.0.193
└── tokio v1.35.0
    ├── tokio-macros v2.2.0
    └── ...
```

**表格 2 — 重复依赖列表**

| 包名 | 版本列表 | 被引入路径数 |
|------|---------|------------|
| serde | 1.0.160, 1.0.193 | 2 |
| syn | 1.0.109, 2.0.48 | 5 |

**表格 3 — 未使用依赖**

| 包名 | 版本 | 类型 | 建议 |
|------|------|------|------|
| lazy_static | 1.4.0 | 正式依赖 | 删除或确认使用 |
| env_logger | 0.10.0 | 正式依赖 | 移至 dev-dependencies 或删除 |

---

### 2.2 依赖安全检查（`cargo audit`）

**表格 1 — 安全概览**

| 严重 | 高危 | 中危 | 低危 | 已忽略 |
|------|------|------|------|--------|
| 0 | 1 | 2 | 0 | 1 |

**表格 2 — 漏洞详情**

| 编号 | 受影响 crate | 当前版本 | 安全版本 | 等级 | 说明 |
|------|-------------|---------|---------|------|------|
| RUSTSEC-2023-0071 | rsa | 0.9.2 | ≥ 0.9.6 | 高危 | Marvin timing attack |
| RUSTSEC-2022-0093 | ed25519-dalek | 1.0.1 | ≥ 2.0.0 | 中危 | Double public key signing fault attack |

**表格 3 — 受影响依赖路径**

| 漏洞编号 | 引入路径 |
|---------|---------|
| RUSTSEC-2023-0071 | myproject → rustls → rsa |

---

### 2.3 火焰图（`cargo flamegraph`）

**表格 1 — 生成信息**

| 执行命令 | SVG 路径 | Profiling 后端 | 采样总数 |
|---------|---------|---------------|---------|
| cargo flamegraph | `.localcheck/report/perf/flamegraph.svg` | perf (Linux) | 12450 |

> ⚠️ Linux 需要 `perf` 权限（`sudo sysctl kernel.perf_event_paranoid=1`）；macOS 需要 DTrace（`sudo cargo flamegraph`）。

**表格 2 — Top-5 热点函数**

| 排名 | 函数名 | 来源 crate | 占比 |
|------|--------|-----------|------|
| 1 | `myproject::parser::parse_token` | myproject | 18.3% |
| 2 | `serde_json::de::Deserializer::parse_value` | serde_json | 11.2% |
| 3 | `core::str::pattern::TwoWaySearcher::next` | std | 8.7% |
| 4 | `alloc::collections::BTreeMap::insert` | std | 6.1% |
| 5 | `myproject::resolver::resolve` | myproject | 5.4% |

---

### 2.4 二进制一致性检查

**表格 1 — 构建环境信息**

| 环境 | OS | Rust 版本 | Target |
|------|-----|---------|--------|
| 环境 A | Ubuntu 22.04 | 1.75.0 | x86_64-unknown-linux-gnu |
| 环境 B | Ubuntu 22.04 | 1.75.0 | x86_64-unknown-linux-gnu |

**表格 2 — 产物 Hash 对比**

| 产物名称 | 环境 A SHA-256 | 环境 B SHA-256 | 一致性 |
|---------|--------------|--------------|--------|
| target/release/myproject | `a3f1c2d4...` | `a3f1c2d4...` | ✅ 一致 |

**表格 3 — 影响一致性的环境因素**

| 因素 | 说明 | 是否已规避 |
|------|------|----------|
| 构建时间戳嵌入 | 源码中含 `env!("BUILD_TIME")` | ❌ 建议移除 |
| `CARGO_REPRODUCIBLE` | 是否设置该环境变量 | ✅ 已设置 |
| 绝对路径嵌入 | 调试信息含本机路径 | ⚠️ 建议加 `--remap-path-prefix` |

---

## 三、落地 v1 后续规划

### 3.1 新增 `rust-checker diff` 命令

- 接受两个报告目录（或时间戳）进行比较
- 输出每个工具维度的变化摘要（如：测试通过数 ↑3，覆盖率 ↓2.1%，新增 CVE ×1）
- 支持与 `run` 相同的三种输出格式

### 3.2 CI 友好输出

工具**只负责输出报告和给出建议**，不干涉 CI/CD 流水线流程（始终以零退出码退出）。
CI 集成由使用者自行决策：读取报告文件中的结构化数据，按项目标准判断是否阻断流水线。

- `run --ci` 参数：以机器可读格式（JSON / plain）输出摘要，方便 CI 脚本解析
- 每个工具的报告包含明确的状态字段（`status: ok / warn / error`）供外部脚本判断
- 提供示例 CI 脚本片段（GitHub Actions / GitLab CI），展示如何读取报告并自行决定是否失败

### 3.3 `init --preset` 分级预设

- `--preset minimal`：仅启用 build + test
- `--preset standard`：build + test + coverage + audit（推荐默认）
- `--preset full`：启用所有内置工具

---

## 四、v2 新增能力

### 4.1 新增内置工具

| 工具 | 命令 | 报告文件 | 优先级 |
|------|------|---------|--------|
| Clippy 代码规范 | `cargo clippy` | `report/quality/clippy.md` | P1 |
| 格式检查 | `cargo fmt --check` | `report/quality/fmt.md` | P1 |
| 文档构建检查 | `cargo doc --no-deps` | `report/quality/doc.md` | P1 |
| 许可证与依赖策略 | `cargo deny check` | `report/security/deny.md` | P1 |
| unsafe 溯源统计 | `cargo geiger` | `report/security/geiger.md` | P1 |
| MSRV 检查 | `cargo msrv` | `report/compat/msrv.md` | P1 |
| 语义化版本验证 | `cargo semver-checks` | `report/compat/semver.md` | P2 |
| 未使用依赖检测 | `cargo +nightly udeps` | `report/deps/udeps.md` | P2 |
| 性能基准 | `cargo bench` | `report/perf/bench.md` | P2 |
| 二进制体积分析 | `cargo bloat --release` | `report/perf/bloat.md` | P2 |

**各工具价值说明：**

- **cargo doc**：以零额外成本验证公开 API 的文档完整性，文档构建失败直接暴露悬空链接和缺失注释。
- **cargo deny**：一站式许可证合规 + 依赖黑名单策略；对商业项目或开源合规场景是刚需。
- **cargo geiger**：统计整个依赖树中所有 `unsafe` 代码的来源 crate 和行数，粒度远超 metrics 中的全局占比。
- **cargo semver-checks**：在发布前静态检测 API 变更是否破坏语义化版本承诺，库作者必备。
- **cargo udeps**：基于编译器实际使用数据（而非文本扫描）检测真正未被使用的依赖，比 `cargo machete` 更准确，需 nightly。
- **cargo bloat**：逐函数/逐 crate 分析 release 产物的体积贡献，与 flamegraph 互补（一个看时间，一个看空间）。

### 4.2 聚合汇总报告

- 新增 `report/summary.md`（或 `summary.html`）
- 所有工具结果以一页总览呈现，包含状态图标（✅ / ⚠️ / ❌）
- 为 HTML 格式提供带样式的 Dashboard 页面

### 4.3 运行依赖预检（新增）

在工具执行前自动检查其所需的外部命令是否存在，避免因环境缺失导致工具静默失败。

**检查策略**：
- 启动时对本次 `run` 涉及的所有工具逐一检查依赖
- 发现缺失后**不直接退出**，而是分级处理：
  - **有明确安装方案**：打印命令（如 `cargo install cargo-tarpaulin`）并询问是否自动安装
  - **需手动操作**（如系统包）：打印平台对应的安装命令（apt / brew / winget）并提示用户执行
  - **无已知方案**：输出建议性提示，告知用户该工具为可选并跳过

每个内置工具的依赖关系在代码中统一维护，无需用户配置。

**输出示例**：
```
[dependency check] cargo-tarpaulin ... missing
  → Install via cargo: cargo install cargo-tarpaulin
  → Auto-install? [y/N]

[dependency check] perf (flamegraph) ... missing
  → Linux: sudo apt install linux-perf
  → macOS: brew install gperftools
  → Please install manually and re-run.

[dependency check] cargo-machete ... missing
  → No known install method. Tool 'deps' will be skipped.
```

### 4.4 Watch 模式

- `rust-checker watch`：监听文件变更，自动重跑受影响的工具
- 可配置触发规则（如仅 `src/` 变更触发 build + test）

### 4.5 配置版本管理

- `config.toml` 加入 `schema_version` 字段
- `rust-checker upgrade` 命令自动将旧版配置迁移到新 schema

---

## 五、新增工具报告模板

### 5.1 Clippy 代码规范（`cargo clippy`）

**表格 1 — 检查概览**

| 执行命令 | Warnings | Errors | 涉及文件数 |
|---------|---------|--------|-----------|
| cargo clippy | 5 | 0 | 3 |

**表格 2 — 问题详情**

| Lint 规则 | 等级 | 文件 | 行号 | 说明 |
|-----------|------|------|------|------|
| needless_pass_by_value | warning | src/lib.rs | 42 | 参数可改为引用传递 |
| unwrap_used | warning | src/main.rs | 18 | 建议使用 `?` 或 `expect` |

---

### 5.2 格式检查（`cargo fmt --check`）

**表格 — 格式检查结果**

| 文件 | 状态 | 差异行数 |
|------|------|---------|
| src/main.rs | ✅ 通过 | 0 |
| src/lib.rs | ❌ 不通过 | 8 |
| src/config.rs | ❌ 不通过 | 3 |

> 修复命令：`cargo fmt`

---

### 5.3 文档构建检查（`cargo doc --no-deps`）

**表格 1 — 构建概览**

| 执行命令 | Warnings | Errors | 公开 API 文档覆盖率 |
|---------|---------|--------|-------------------|
| cargo doc --no-deps | 3 | 0 | 87.5% |

**表格 2 — 未文档化的公开项（节选）**

| 条目名称 | 类型 | 文件 | 行号 |
|---------|------|------|------|
| `pub fn process` | 函数 | src/lib.rs | 25 |
| `pub struct Config` | 结构体 | src/config.rs | 10 |

---

### 5.4 许可证与依赖策略（`cargo deny check`）

**表格 1 — 许可证概览**

| 许可证 | crate 数量 | 状态 |
|-------|----------|------|
| MIT | 54 | ✅ 允许 |
| Apache-2.0 | 30 | ✅ 允许 |
| GPL-3.0 | 1 | ❌ 违规 |

**表格 2 — 违规与警告项**

| 类型 | crate | 版本 | 原因 | 建议 |
|------|-------|------|------|------|
| 许可证违规 | some-crate | 0.1.0 | GPL-3.0 不被允许 | 替换为兼容版本或添加豁免 |

---

### 5.5 unsafe 溯源统计（`cargo geiger`）

**表格 1 — 项目 unsafe 汇总**

| unsafe fn | unsafe block | unsafe impl | unsafe trait | 行数合计 |
|----------|-------------|-------------|-------------|---------|
| 0 | 2 | 0 | 0 | 5 |

**表格 2 — 各依赖 unsafe 分布（Top-5）**

| crate | 版本 | unsafe fn | unsafe block | unsafe impl | 行数合计 |
|-------|------|----------|-------------|-------------|---------|
| ring | 0.17.7 | 48 | 103 | 12 | 320 |
| libc | 0.2.151 | 210 | 0 | 0 | 210 |
| myproject | 0.1.0 | 0 | 2 | 0 | 5 |

---

### 5.6 MSRV 检查（`cargo msrv`）

**表格 1 — MSRV 检查结果**

| 声明的 MSRV | 验证结果 | 实际最低支持版本 |
|-----------|---------|---------------|
| 1.65.0 | ✅ 通过 | 1.65.0 |

**表格 2 — 影响 MSRV 的依赖**

| 依赖 | 要求最低 Rust 版本 | 与声明 MSRV 兼容 |
|------|-----------------|----------------|
| tokio 1.35.0 | 1.63.0 | ✅ 兼容 |
| clap 4.4.0 | 1.70.0 | ❌ 不兼容，需升级声明 MSRV 或降级依赖版本 |

---

### 5.7 语义化版本验证（`cargo semver-checks`）

**表格 — API 破坏性变更**

| 条目 | 变更类型 | 严重性 | 说明 |
|------|---------|--------|------|
| `pub fn parse` 参数类型变更 | Breaking change | 高 | 入参 `&str` → `String`，调用方需修改 |
| `pub enum Status` 新增变体 | Non-breaking | 低 | 已加 `#[non_exhaustive]`，安全 |

---

### 5.8 未使用依赖检测（`cargo +nightly udeps`）

**表格 — 未使用依赖**

| 包名 | 版本 | 类型 | 建议 |
|------|------|------|------|
| regex | 1.10.2 | 正式依赖 | 移除或确认用途 |
| pretty_env_logger | 0.5.0 | dev-dependencies | 移除 |

---

### 5.9 性能基准（`cargo bench`）

**表格 1 — 基准测试概览**

| 执行命令 | 基准数量 | 最慢基准 |
|---------|---------|---------|
| cargo bench | 5 | `bench_parse_large` |

**表格 2 — 基准结果**

| 基准名称 | 平均耗时 | 标准差 | 吞吐量 |
|---------|---------|--------|--------|
| bench_parse_small | 12.3 µs | ±0.2 µs | 81 MB/s |
| bench_parse_large | 148.0 ms | ±2.1 ms | 67 MB/s |
| bench_resolve | 55.4 µs | ±0.8 µs | — |

---

### 5.10 二进制体积分析（`cargo bloat --release`）

**表格 1 — 体积概览**

| 执行命令 | 总大小 | .text 段 | 最大贡献 crate |
|---------|-------|---------|--------------|
| cargo bloat --release | 3.2 MB | 2.1 MB | std (16.0%) |

**表格 2 — 体积贡献 Top-5（crate 维度）**

| crate | 大小 | 占比 |
|-------|------|------|
| std | 512 KB | 16.0% |
| myproject | 320 KB | 10.0% |
| serde_json | 210 KB | 6.6% |
| tokio | 195 KB | 6.1% |
| clap | 88 KB | 2.8% |

---

## 六、聚合汇总报告模板（`summary.md`）

**表格 — 检查总览**

| 工具 | 状态 | 关键指标 | 报告链接 |
|------|------|---------|---------|
| cargo build | ✅ 通过 | 5.2 MB，12.3s | quality/build.md |
| cargo test | ✅ 通过 | 42/42 通过 | quality/test.md |
| cargo llvm-cov | ⚠️ 警告 | 覆盖率 72.1% < 80% | quality/coverage.md |
| cargo clippy | ⚠️ 警告 | 5 warnings | quality/clippy.md |
| cargo fmt | ❌ 失败 | 2 文件格式不一致 | quality/fmt.md |
| cargo doc | ✅ 通过 | 3 warnings | quality/doc.md |
| cargo audit | ❌ 失败 | 1 高危 CVE | security/audit.md |
| cargo deny | ✅ 通过 | 无违规 | security/deny.md |
| cargo geiger | ⚠️ 警告 | 325 行 unsafe（含依赖） | security/geiger.md |
| cargo tree | ✅ 通过 | 3 重复依赖 | deps/deps.md |
| cargo udeps | ⚠️ 警告 | 1 未使用依赖 | deps/udeps.md |
| cargo flamegraph | ✅ 通过 | 热点: parser 18.3% | perf/flamegraph.md |
| cargo bench | ✅ 通过 | 5 基准 | perf/bench.md |
| cargo bloat | ⚠️ 警告 | release 产物 3.2 MB | perf/bloat.md |
| cargo msrv | ❌ 失败 | 声明 MSRV 与依赖不兼容 | compat/msrv.md |
| cargo semver-checks | ✅ 通过 | 无破坏性变更 | compat/semver.md |

---

## 七、v2 目录结构更新

报告文件按职责归入子目录，避免平铺过多导致混乱：

```
.localcheck/
├── config.toml
├── logs/
│   └── <timestamp>.log
└── report/
    ├── summary.md              # 聚合总览（新增）
    ├── quality/                # 代码质量类
    │   ├── build.md
    │   ├── test.md
    │   ├── coverage.md
    │   ├── clippy.md           # 新增
    │   ├── fmt.md              # 新增
    │   └── doc.md              # 新增
    ├── security/               # 安全类
    │   ├── audit.md            # 补全
    │   ├── deny.md             # 新增（许可证 + 依赖策略）
    │   └── geiger.md           # 新增（unsafe 溯源）
    ├── deps/                   # 依赖分析类
    │   ├── deps.md             # 补全
    │   └── udeps.md            # 新增（未使用依赖）
    ├── perf/                   # 性能类
    │   ├── flamegraph.md       # 补全
    │   ├── bench.md            # 新增
    │   ├── bloat.md            # 新增（二进制体积分析）
    │   └── metrics.md
    └── compat/                 # 兼容性类
        ├── binary.md           # 补全
        ├── msrv.md             # 新增
        └── semver.md           # 新增（语义化版本验证）
```

---

## 八、实现优先级

| 优先级 | 内容 | 说明 |
|--------|------|------|
| P0 | 补全 v1 未完成报告模板（deps / audit / flamegraph / binary） | 消除 v1 欠债 |
| P0 | `--ci` 友好输出（机器可读摘要 + 状态字段） | 生产可用的核心需求 |
| P1 | Clippy / fmt / doc 内置工具 | 常见质量检查补全 |
| P1 | cargo deny（许可证合规 + 依赖策略） | 商业/开源合规刚需 |
| P1 | cargo geiger（unsafe 溯源统计） | 安全审计的细粒度补充 |
| P1 | msrv 内置工具 | 兼容性检查 |
| P1 | `init --preset` 分级预设 | 降低上手门槛 |
| P1 | 聚合汇总报告（summary） | 提升报告可读性 |
| P1 | 运行依赖预检（分级提示 + 可选自动安装） | 避免环境缺失静默失败 |
| P2 | cargo semver-checks（语义化版本验证） | 发库场景必备 |
| P2 | cargo udeps（未使用依赖检测，nightly） | 比 machete 更精确 |
| P2 | cargo bench / cargo bloat（性能基准 + 体积分析） | 性能优化场景 |
| P2 | `diff` 命令 | 趋势追踪 |
| P3 | Watch 模式 | 开发体验提升 |
| P3 | 配置版本迁移（`upgrade`） | 长期维护性 |

---

## 九、与 v1 的兼容性

- `config.toml` 向前兼容，新字段均为可选
- 内置工具报告格式保持向后兼容，仅在新字段上扩展

---

*本文档为 v2 版本，后续修改请创建新版本文件（如 `plan-v3.md`）以保留历史记录。*
