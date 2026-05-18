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

报告包含：
- 依赖树可视化（层级缩进文本）
- 重复依赖检测（同包多版本列表）
- 直接 vs 间接依赖统计
- 未使用依赖提示（via `cargo machete`）

### 2.2 依赖安全检查（`cargo audit`）

报告包含：
- CVE 漏洞列表（编号、等级、受影响 crate 及版本）
- 完整依赖受影响路径
- 修复建议（升级到安全版本 / 无可用修复标注）

### 2.3 火焰图（`cargo flamegraph`）

报告包含：
- SVG 火焰图输出路径
- Top-N 热点函数摘要表（函数名、占比、来源 crate）
- 生成所需的系统权限说明（Linux `perf` / macOS `DTrace`）

### 2.4 二进制一致性检查

报告包含：
- 相同源码在不同环境产物的 SHA-256 Hash 对比
- 可重现构建验证结果（是否启用 `CARGO_REPRODUCIBLE`）
- 影响一致性的环境差异列表

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

## 五、v2 目录结构更新

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

## 六、实现优先级

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

## 七、与 v1 的兼容性

- `config.toml` 向前兼容，新字段均为可选
- 内置工具报告格式保持向后兼容，仅在新字段上扩展

---

*本文档为 v2 版本，后续修改请创建新版本文件（如 `plan-v3.md`）以保留历史记录。*
