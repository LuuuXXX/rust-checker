# rust-checker 项目规划 v1

> 文档版本：v1  
> 创建日期：2026-05-15  
> 状态：草稿

---

## 一、项目简介

`rust-checker` 是一个面向 Rust 项目的本地质量检查工具，通过统一的配置文件和命令行界面，聚合多种 Cargo 工具的输出，生成结构化报告，帮助开发者快速掌握项目的构建质量、测试覆盖率、依赖安全等关键指标。

---

## 二、核心设计原则

- **配置驱动**：所有行为由 `.localcheck/config.toml` 统一管理，保持可重现性。
- **工具解耦**：每个检查工具独立执行，依赖缺失时可跳过，不影响其他工具。
- **跨平台兼容**：明确区分 Windows 与 Linux/macOS 的运行差异。
- **输出可读性**：支持 Markdown（默认）、HTML、JSON 三种报告格式。
- **完整审计日志**：所有执行过程写入日志文件，便于问题追溯。

---

## 三、目录结构规范

项目执行后，在工程根目录生成如下结构：

```
.localcheck/
├── config.toml         # 主配置文件
├── logs/
│   └── <timestamp>.log # 执行日志
└── report/
    ├── build.md        # 构建报告
    ├── test.md         # 测试报告
    ├── coverage.md     # 覆盖率报告
    ├── deps.md         # 依赖分析报告
    ├── audit.md        # 依赖安全报告
    ├── flamegraph.md   # 火焰图报告
    ├── metrics.md      # 代码度量报告
    ├── binary.md       # 二进制一致性报告
    └── customs_<name>.md  # 用户自定义工具报告（customs 前缀）
```

---

## 四、配置文件格式

### `.localcheck/config.toml`

```toml
[rust]
version = "1.xx.x"        # Rust 工具链版本
rustflags = ""            # 附加编译标志

# 内置工具配置示例（output_path 由 run 命令自动生成）
[tools.build]
desc = "构建项目，分析产物与编译选项"
active = "true"
input_command = "cargo build"
output_path = "report/build.md"   # 由 run 命令首次执行时自动填充

[tools.test]
desc = "运行单元测试与集成测试"
active = "true"
input_command = "cargo test"
output_path = "report/test.md"

# 用户自定义工具示例（无 output_path，由工具自动分配 customs_ 前缀）
[tools.my_custom_tool]
desc = "自定义检查脚本"
active = "true"
input_command = "bash scripts/my_check.sh"
```

**字段说明：**

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `desc` | string | 是 | 工具的一句话功能描述 |
| `active` | `"true"` \| `"false"` | 是 | 是否激活该工具 |
| `input_command` | string | 是 | 实际执行的命令（相对于项目根目录） |
| `output_path` | string | 否 | 报告输出路径，内置工具自动生成，自定义工具无此字段 |

---

## 五、CLI 命令设计

### 5.1 `rust-checker init`

**功能**：初始化 `.localcheck/config.toml`。

**行为逻辑：**

```
执行 init
  └── 是否已存在 .localcheck/config.toml？
        ├── 否 → 询问用户：使用默认配置 / 自定义配置
        │         ├── 默认 → 生成预置所有内置工具的配置文件
        │         └── 自定义 → 引导用户逐项输入工具配置（无 output_path）
        └── 是 → 提示文件已存在，询问是否覆盖
```

### 5.2 `rust-checker run`

**功能**：读取配置并顺序执行所有激活的工具，输出报告与日志。

**行为逻辑：**

```
执行 run
  ├── 前置检查
  │     ├── 检查运行环境（OS、Rust 版本等），Windows/Linux 输出有所区别
  │     └── 逐一检查各工具的依赖是否满足
  │           ├── 依赖缺失 → 询问用户：自动安装 / 手动安装 / 跳过该工具
  │           └── 跳过时在终端和日志中明确标注
  ├── 创建 .localcheck/report/ 与 .localcheck/logs/ 目录
  ├── 按配置顺序执行激活的工具
  │     ├── 内置工具 → 按标准模板解析输出，生成结构化报告
  │     └── 自定义工具 → 捕获原始输出，存为 customs_<name>.<ext>
  └── 汇总日志写入 .localcheck/logs/<timestamp>.log
```

**报告格式支持：**

| 格式 | 参数 | 说明 |
|------|------|------|
| Markdown | 默认 / `--format md` | 可读性强，适合 Git 归档 |
| HTML | `--format html` | 适合浏览器查看 |
| JSON | `--format json` | 适合 CI/CD 集成与二次处理 |

---

## 六、内置工具与报告模板

### 6.1 构建检查（`cargo build`）

**表格 1 — 构建基本信息**

| 构建命令 | 构建 Feature | 可支持 Feature |
|----------|-------------|---------------|
| `cargo build` | default | feat-a, feat-b |

**表格 2 — 构建产物信息**

| 产物位置 | 产物类型 | 产物大小 | 构建总时间 |
|----------|----------|----------|-----------|
| `target/debug/foo` | 可执行二进制 | 5.2 MB | 12.3s |

**表格 3 — 编译选项分析**

| 选项 | 当前值 | 说明 |
|------|--------|------|
| opt-level | 0 | 调试模式，未优化 |
| debug | true | 包含调试信息 |

**表格 4 — 优化建议**

| 类型 | 建议 |
|------|------|
| 二进制大小 | 启用 `strip = true`，使用 LTO |
| 构建时间 | 启用增量编译，使用 `sccache` |

---

### 6.2 测试（`cargo test`）

**表格 1 — 测试基本信息**

| 测试命令 | 测试 Feature | 可支持 Feature |
|----------|-------------|---------------|
| `cargo test` | default | feat-a |

**表格 2 — 测试结果**

| 产物位置 | 测试用例 | 用例耗时 | 测试总时长 |
|----------|----------|----------|-----------|
| `target/debug/deps/foo-xxx` | `test_add` | 0.01s | 1.23s |

---

### 6.3 测试覆盖率（`cargo llvm-cov`）

**表格 — 覆盖率详情**

| 测试文件 | 条件覆盖率 | 分支覆盖率 | 总覆盖率 |
|----------|-----------|-----------|---------|
| `src/lib.rs` | 85.0% | 78.0% | 82.0% |

---

### 6.4 依赖分析

> 待补充：依赖树可视化、重复依赖检测、依赖版本分析。

---

### 6.5 依赖安全检查（`cargo audit`）

> 待补充：CVE 漏洞列表、受影响路径、修复建议。

---

### 6.6 火焰图（`cargo flamegraph`）

> 待补充：火焰图生成路径、热点函数摘要。

---

### 6.7 代码度量

**表格 — 代码质量指标**

| 指标 | 值 | 说明 |
|------|-----|------|
| 最大圈复杂度 | 12 | 建议 ≤ 10 |
| 代码行数 | 3200 | 含注释与空行 |
| unsafe 代码比例 | 2.1% | unsafe 块占总代码比 |

> 其余指标待补充。

---

### 6.8 二进制一致性检查

> 待补充：相同源码在不同环境下产物 Hash 对比、可重现构建验证。

---

## 七、日志规范

- 日志文件路径：`.localcheck/logs/<YYYYMMDD-HHMMSS>.log`
- 内容包括：
  - 工具链环境信息
  - 每个工具的开始/结束时间
  - 被跳过工具及原因
  - 标准输出与标准错误的完整记录
  - 报告文件的生成路径

---

## 八、待定与后续规划

- [ ] 支持 `rust-checker diff` 命令，比较两次报告的差异
- [ ] CI/CD 模式（`--ci` 参数），错误时返回非零退出码
- [ ] 插件机制，允许社区贡献标准化工具模板
- [ ] 补充依赖分析、火焰图、二进制一致性的完整报告模板
- [ ] `init --preset minimal/full` 预设配置级别

---

*本文档为 v1 版本，后续修改请创建新版本文件（如 `plan-v2.md`）以保留历史记录。*
