# rust-checker 插件开发指南

本文档说明如何为 `rust-checker` 开发第三方插件，并将其贡献至官方插件注册表 [rust-checker-plugins](https://github.com/LuuuXXX/rust-checker-plugins)。

---

## 一、插件概述

`rust-checker` 插件是一个轻量级扩展单元，由一个 `plugin.toml` 描述文件组成。安装后，插件的工具会与内置工具一起被调度运行，并在 `.rust-checker/reports/` 下生成报告。

插件不需要编译，无需发布到 crates.io——只需一个符合规范的 `plugin.toml` 文件。

---

## 二、`plugin.toml` 规范

```toml
# ── 插件元数据 ────────────────────────────────────────────────────────────────
[plugin]
name        = "cargo-deny"          # 插件唯一名称（kebab-case，与目录名一致）
version     = "0.1.0"              # 插件版本号（SemVer）
description = "依赖策略与许可证检查"   # 一句话功能描述
author      = "你的名字 <email>"    # 可选：作者信息
category    = "security"           # 分类：quality | security | deps | perf | compat | custom
tags        = ["license", "deny"]  # 可选：标签列表

# ── 执行命令 ──────────────────────────────────────────────────────────────────
[command]
program = "cargo"           # 可执行程序名
args    = ["deny", "check"] # 参数列表

# 可选：环境变量（暂未实现，当前版本运行时不会应用）
# [command.env]
# RUSTFLAGS = "-D warnings"

# ── 报告配置 ──────────────────────────────────────────────────────────────────
[report]
parser      = "builtin::deny"       # 报告解析器（见下方说明）
output_path = "security/deny.md"    # 报告输出路径（相对于 .rust-checker/reports/）
```

### 字段说明

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `plugin.name` | string | ✅ | 插件唯一标识符，使用 kebab-case |
| `plugin.version` | string | ✅ | SemVer 版本号 |
| `plugin.description` | string | ✅ | 简短功能描述 |
| `plugin.author` | string | ❌ | 作者信息 |
| `plugin.category` | string | ✅ | 分类（见下方枚举） |
| `plugin.tags` | array | ❌ | 辅助搜索的标签 |
| `command.program` | string | ✅ | 可执行文件名（PATH 中可找到的命令） |
| `command.args` | array | ✅ | 命令参数列表 |
| `command.env` | table | ❌ | 额外的环境变量（**暂未实现**：字段已解析保存但当前运行时不会应用到进程，留作未来扩展） |
| `report.parser` | string | ✅ | 输出解析器（见下方说明；**暂未实现**：当前版本所有插件统一使用通用解析器，此字段仅保存于 plugin.toml，供未来版本使用） |
| `report.output_path` | string | ✅ | 报告保存路径（相对于 reports/） |

### category 枚举

| 值 | 说明 |
|----|------|
| `quality` | 代码质量（lint、格式、测试等） |
| `security` | 安全相关（漏洞、许可证、unsafe 等） |
| `deps` | 依赖分析 |
| `perf` | 性能分析 |
| `compat` | 兼容性检查 |
| `custom` | 其他自定义工具 |

### report.parser 说明

> **注意**：`parser` 字段在当前版本中尚未实现路由功能。所有插件均使用通用解析器（等效于 `builtin::generic`）：stdout/stderr 原始输出以文本形式保存到报告。此字段已写入 `plugin.toml` 以便未来版本激活对应解析器，填写时请参照下方表格选择最接近的值。

`parser` 字段声明该工具输出应如何解析。未来支持的值：

| parser 值 | 适用场景 |
|-----------|---------|
| `builtin::build` | 解析 `cargo build` 输出 |
| `builtin::test` | 解析 `cargo test` 输出 |
| `builtin::clippy` | 解析 `cargo clippy` 输出 |
| `builtin::fmt` | 解析 `cargo fmt --check` 输出 |
| `builtin::audit` | 解析 `cargo audit` 输出 |
| `builtin::deny` | 解析 `cargo deny check` 输出 |
| `builtin::generic` | 通用解析器：捕获 stdout/stderr 原始输出（当前版本所有插件实际行为） |

---

## 三、快速上手：创建第一个插件

以封装 `cargo-outdated`（检查过时依赖）为例：

### 1. 创建目录结构

```
my-plugin/
└── plugin.toml
```

### 2. 编写 `plugin.toml`

```toml
[plugin]
name        = "outdated"
version     = "0.1.0"
description = "检查过时的依赖版本"
author      = "Your Name"
category    = "deps"
tags        = ["outdated", "update"]

[command]
program = "cargo"
args    = ["outdated", "--root-deps-only"]

[report]
parser      = "builtin::generic"
output_path = "deps/outdated.md"
```

### 3. 本地测试

```bash
# 进入你的 Rust 项目
cd /path/to/your-project

# 手动安装（模拟 plugin add）
mkdir -p .rust-checker/plugins/outdated
cp my-plugin/plugin.toml .rust-checker/plugins/outdated/

# 运行
rust-checker run --only outdated
```

### 4. 查看报告

```bash
cat .rust-checker/reports/deps/outdated.md
```

---

## 四、贡献到官方注册表

官方插件注册表位于 [github.com/LuuuXXX/rust-checker-plugins](https://github.com/LuuuXXX/rust-checker-plugins)，目录结构如下：

```
rust-checker-plugins/
└── plugins/
    ├── cargo-deny/
    │   └── plugin.toml
    ├── cargo-outdated/
    │   └── plugin.toml
    └── <your-plugin>/
        └── plugin.toml
```

### 贡献流程

1. **Fork** [rust-checker-plugins](https://github.com/LuuuXXX/rust-checker-plugins) 仓库

2. **创建目录**：在 `plugins/` 下新建以插件名命名的目录（与 `plugin.name` 一致）

3. **添加 `plugin.toml`**：按照规范填写所有必填字段

4. **自测**：本地按上述步骤验证插件正常运行

5. **提交 Pull Request**，标题格式：`feat(plugin): add <plugin-name>`

### PR 检查项

提交前确认：

- [ ] `plugin.name` 为 kebab-case，与目录名完全一致
- [ ] `plugin.version` 遵循 SemVer
- [ ] `plugin.description` 清晰描述功能（不超过 80 字符）
- [ ] `plugin.category` 使用规定枚举值
- [ ] `command.program` 对应 crates.io 上可安装的工具
- [ ] `report.output_path` 路径与 `plugin.category` 匹配（如 `security/` 对应 `security` 分类）
- [ ] 本地已用 `rust-checker run --only <name>` 验证插件能正常运行
- [ ] `plugin.toml` 可被 `toml::from_str::<PluginToml>()` 无错误解析

### CI 验证

注册表 CI 会自动对每个 `plugin.toml` 运行 TOML 格式校验。确保你的文件语法正确。

---

## 五、最佳实践

- **命名**：`plugin.name` 与工具名保持一致（如 `cargo-outdated` 对应工具 `cargo outdated`）
- **描述**：简洁说明工具"做什么"，不需要说明"如何安装"
- **路径**：`output_path` 应反映工具分类，便于在汇总报告中归类展示
- **幂等性**：工具应在项目根目录以非交互方式运行（避免需要用户输入）
- **退出码**：若工具以非零码退出，报告状态将标记为 `error`；若输出包含 warning 字样，状态标记为 `warn`

---

## 六、示例插件

参考注册表中的现有插件：

- [`plugins/cargo-deny/plugin.toml`](https://github.com/LuuuXXX/rust-checker-plugins/tree/main/plugins/cargo-deny) — 许可证与依赖策略
- [`plugins/cargo-audit/plugin.toml`](https://github.com/LuuuXXX/rust-checker-plugins/tree/main/plugins/cargo-audit) — 安全漏洞扫描

---

## 七、常见问题

**Q: 插件可以依赖其他工具吗？**

暂不支持插件间的 `depends_on`。如需顺序保证，请在 `.rust-checker/config.toml` 中为已安装插件手动添加 `depends_on`。

**Q: 插件可以自定义报告格式吗？**

当前版本中，报告格式（Markdown/HTML/JSON）由主程序统一处理。`parser` 字段决定内容解析方式，不影响输出格式选择。

**Q: 如何更新已安装的插件？**

```bash
rust-checker plugin update
```

此命令会重新从注册表下载所有已安装插件的最新 `plugin.toml`。
