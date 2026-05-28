#!/usr/bin/env bash
# =============================================================================
# install-tools.sh — rust-checker 依赖工具一键安装脚本
# =============================================================================
# 用法：
#   bash scripts/install-tools.sh [OPTIONS]
#
# 选项：
#   --preset PRESET    指定预设：minimal|quality|security|full（默认：full）
#   --dry-run          仅展示将要执行的操作，不实际安装
#   --check            仅检查当前工具状态并退出
#   --yes / -y         非交互模式（CI 环境推荐）
#   --offline          离线模式：跳过网络下载
#   --no-system        跳过系统包安装（apt/brew 等）
#   --help / -h        显示帮助
#
# 支持平台：Linux（x86_64/aarch64）、macOS（Intel/Apple Silicon）、
#           Windows（Git Bash / WSL）
# =============================================================================
set -euo pipefail

SCRIPT_VERSION="1.0.0"

# ── 颜色（仅 TTY 时启用）─────────────────────────────────────────────────────
if [ -t 1 ]; then
    R='\033[0;31m' G='\033[0;32m' Y='\033[1;33m'
    B='\033[0;34m' C='\033[0;36m' W='\033[1m' N='\033[0m'
else
    R='' G='' Y='' B='' C='' W='' N=''
fi

# ── 默认选项 ──────────────────────────────────────────────────────────────────
OPT_PRESET="full"
OPT_DRY_RUN=false
OPT_CHECK=false
OPT_YES=false
OPT_OFFLINE=false
OPT_NO_SYSTEM=false

# ── 统计 ───────────────────────────────────────────────────────────────────────
CNT_ALREADY=0
CNT_INSTALLED=0
CNT_FAILED=0
CNT_SKIPPED=0

# ── 全局环境变量（由 detect_os 填充） ─────────────────────────────────────────
OS_TYPE=""
ARCH=""
SYS_PKG_MGR=""
HAS_RUSTUP=false
HAS_CARGO=false
RUST_VERSION=""

# =============================================================================
# 工具函数
# =============================================================================

log_info()  { echo -e "${B}[INFO]${N}  $*"; }
log_ok()    { echo -e "${G}[OK]${N}    $*"; }
log_warn()  { echo -e "${Y}[WARN]${N}  $*"; }
log_error() { echo -e "${R}[ERROR]${N} $*" >&2; }
log_step()  { echo -e "${C}[STEP]${N}  $*"; }
log_skip()  { echo -e "${Y}[SKIP]${N}  $*"; }
log_dry()   { echo -e "${C}[DRY]${N}   $*"; }

banner() {
    echo ""
    echo -e "${W}╔══════════════════════════════════════════════════════════════╗${N}"
    echo -e "${W}║     rust-checker 依赖工具一键安装脚本 v${SCRIPT_VERSION}            ║${N}"
    echo -e "${W}╚══════════════════════════════════════════════════════════════╝${N}"
    echo ""
}

section() {
    echo ""
    echo -e "${W}──── $* ────────────────────────────────────────────────────────${N}"
}

cmd_exists() { command -v "$1" &>/dev/null; }

# 执行命令（dry-run 时只打印）
run_cmd() {
    if $OPT_DRY_RUN; then
        log_dry "将执行：$*"
        return 0
    fi
    "$@"
}

# 询问确认（--yes 或非 TTY 时自动处理）
confirm() {
    local prompt="$1"
    if $OPT_YES; then
        log_info "$prompt [自动确认]"
        return 0
    fi
    if [ ! -t 0 ]; then
        log_warn "$prompt [非交互 stdin，跳过]"
        return 1
    fi
    printf "%b? %s [y/N] " "$Y" "$prompt"
    printf "%b" "$N"
    local reply
    read -r reply
    [[ "$reply" =~ ^[Yy]$ ]]
}

# =============================================================================
# 工具注册表（每个工具独立的 case 查询，避免 heredoc 解析问题）
# =============================================================================

# 工具所需的主二进制名
tool_binary() {
    case "$1" in
        build|test|bench|doc|deps|binary) echo "cargo" ;;
        clippy)    echo "cargo-clippy" ;;
        fmt)       echo "cargo-fmt" ;;
        coverage)  echo "cargo-llvm-cov" ;;
        audit)     echo "cargo-audit" ;;
        deny)      echo "cargo-deny" ;;
        geiger|metrics) echo "cargo-geiger" ;;
        msrv)      echo "cargo-msrv" ;;
        semver)    echo "cargo-semver-checks" ;;
        udeps)     echo "cargo-udeps" ;;
        bloat)     echo "cargo-bloat" ;;
        flamegraph) echo "cargo-flamegraph" ;;
        *)         echo "" ;;
    esac
}

# 安装类型：builtin | rustup_component | cargo
tool_install_type() {
    case "$1" in
        build|test|bench|doc|deps|binary) echo "builtin" ;;
        clippy|fmt) echo "rustup_component" ;;
        coverage|audit|deny|geiger|metrics|msrv|semver|udeps|bloat|flamegraph) echo "cargo" ;;
        *) echo "" ;;
    esac
}

# cargo install 的包名 或 rustup component add 的组件名
tool_install_cmd() {
    case "$1" in
        clippy)     echo "clippy" ;;
        fmt)        echo "rustfmt" ;;
        coverage)   echo "cargo-llvm-cov" ;;
        audit)      echo "cargo-audit" ;;
        deny)       echo "cargo-deny" ;;
        geiger|metrics) echo "cargo-geiger" ;;
        msrv)       echo "cargo-msrv" ;;
        semver)     echo "cargo-semver-checks" ;;
        udeps)      echo "cargo-udeps" ;;
        bloat)      echo "cargo-bloat" ;;
        flamegraph) echo "flamegraph" ;;
        *) echo "" ;;
    esac
}

# 工具说明
tool_desc() {
    case "$1" in
        build)      echo "构建项目" ;;
        test)       echo "运行单元测试" ;;
        bench)      echo "基准测试" ;;
        doc)        echo "文档生成检查" ;;
        deps)       echo "依赖树展示" ;;
        binary)     echo "二进制信息" ;;
        clippy)     echo "代码静态分析" ;;
        fmt)        echo "代码格式检查" ;;
        coverage)   echo "测试覆盖率" ;;
        audit)      echo "安全漏洞审计" ;;
        deny)       echo "依赖策略检查" ;;
        geiger)     echo "unsafe 代码检查" ;;
        metrics)    echo "代码指标统计" ;;
        msrv)       echo "最低支持 Rust 版本" ;;
        semver)     echo "语义化版本检查" ;;
        udeps)      echo "未使用依赖检查（需 nightly）" ;;
        bloat)      echo "二进制体积分析" ;;
        flamegraph) echo "性能火焰图" ;;
        *) echo "" ;;
    esac
}

# 系统依赖包（格式：linux=PKG macos=PKG windows=PKG，空格分隔，可缺省）
tool_system_deps() {
    case "$1" in
        coverage)   echo "linux=llvm macos=llvm" ;;
        flamegraph) echo "linux=linux-tools-generic windows=UNSUPPORTED" ;;
        *) echo "" ;;
    esac
}

# 从 tool_system_deps 字符串中提取当前 OS 对应的包名
get_sys_dep_for_current_os() {
    local deps_str="$1"
    [[ -z "$deps_str" ]] && return 0
    local key
    case "$OS_TYPE" in
        linux)   key="linux" ;;
        macos)   key="macos" ;;
        windows) key="windows" ;;
        *)       return 0 ;;
    esac
    # 提取 key=VALUE
    local val
    val="$(echo "$deps_str" | tr ' ' '\n' | grep "^${key}=" | cut -d= -f2)"
    echo "$val"
}

# 工具全集（按展示顺序）
ALL_TOOLS="build test clippy fmt doc coverage audit deny geiger metrics deps msrv semver udeps bench bloat flamegraph binary"

# 各预设工具列表
tools_in_preset() {
    case "$1" in
        minimal)  echo "build test clippy fmt" ;;
        quality)  echo "build test clippy fmt doc coverage" ;;
        security) echo "build test audit deny geiger" ;;
        full)     echo "$ALL_TOOLS" ;;
        *)        echo "build test clippy fmt" ;;
    esac
}

# 工具是否在指定预设中
tool_in_preset() {
    local name="$1" preset="$2"
    local preset_tools
    preset_tools="$(tools_in_preset "$preset")"
    for t in $preset_tools; do
        [[ "$t" == "$name" ]] && return 0
    done
    return 1
}

# =============================================================================
# 环境检测
# =============================================================================

detect_os() {
    case "$(uname -s 2>/dev/null || echo unknown)" in
        Linux*)  OS_TYPE="linux" ;;
        Darwin*) OS_TYPE="macos" ;;
        CYGWIN*|MINGW*|MSYS*) OS_TYPE="windows" ;;
        *)       OS_TYPE="unknown" ;;
    esac

    ARCH="$(uname -m 2>/dev/null || echo unknown)"

    # 包管理器
    SYS_PKG_MGR=""
    if [[ "$OS_TYPE" == "linux" ]]; then
        cmd_exists apt-get  && SYS_PKG_MGR="apt"
        cmd_exists dnf      && SYS_PKG_MGR="dnf"
        cmd_exists yum      && SYS_PKG_MGR="yum"
        cmd_exists pacman   && SYS_PKG_MGR="pacman"
        cmd_exists zypper   && SYS_PKG_MGR="zypper"
        cmd_exists apk      && SYS_PKG_MGR="apk"
    elif [[ "$OS_TYPE" == "macos" ]]; then
        cmd_exists brew && SYS_PKG_MGR="brew"
    elif [[ "$OS_TYPE" == "windows" ]]; then
        cmd_exists winget && SYS_PKG_MGR="winget"
        cmd_exists choco  && SYS_PKG_MGR="choco"
    fi

    cmd_exists rustup && HAS_RUSTUP=true || HAS_RUSTUP=false
    cmd_exists cargo  && HAS_CARGO=true  || HAS_CARGO=false

    if $HAS_CARGO; then
        RUST_VERSION="$(rustc --version 2>/dev/null | awk '{print $2}' || echo 未知)"
    fi
}

print_env_info() {
    section "环境信息"
    echo ""
    printf "  %-22s %s\n" "操作系统："    "$OS_TYPE ($ARCH)"
    printf "  %-22s %s\n" "系统包管理器：" "${SYS_PKG_MGR:-（未检测到）}"

    local rustup_status
    $HAS_RUSTUP \
        && rustup_status="${G}✓ 已安装${N}" \
        || rustup_status="${Y}✗ 未找到（离线/自定义安装场景）${N}"
    printf "  %-22s " "rustup："
    echo -e "$rustup_status"

    local cargo_status
    $HAS_CARGO \
        && cargo_status="${G}✓ 已安装 (rust $RUST_VERSION)${N}" \
        || cargo_status="${R}✗ 未找到${N}"
    printf "  %-22s " "cargo："
    echo -e "$cargo_status"

    printf "  %-22s %s\n" "目标预设：" "$OPT_PRESET"

    $OPT_DRY_RUN && echo -e "  ${Y}模式：预演（dry-run），不实际执行安装${N}"
    $OPT_CHECK   && echo -e "  ${C}模式：仅检查状态${N}"
    $OPT_OFFLINE && echo -e "  ${Y}离线模式：已启用，将跳过网络下载${N}"
    echo ""

    # cargo 是必须前提
    if ! $HAS_CARGO; then
        log_error "未找到 cargo。请先安装 Rust 后再运行本脚本。"
        echo ""
        echo "  在线安装：  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        echo "  离线安装：  https://static.rust-lang.org/dist/"
        echo "              （下载对应平台工具链包，解压后设置 PATH）"
        exit 1
    fi

    # 无 rustup 提示
    if ! $HAS_RUSTUP; then
        log_warn "未找到 rustup。说明当前环境可能通过离线包或系统包安装了 Rust。"
        echo ""
        echo "  说明："
        echo "    • clippy 和 rustfmt 通常随 Rust 工具链包一起分发。"
        echo "      若它们已在 PATH 中，脚本将自动识别并跳过安装。"
        echo "    • 若缺失组件，请手动从工具链包中提取，或访问："
        echo "      https://static.rust-lang.org/dist/"
        echo "    • cargo install 类工具仍需网络（或本地 registry 镜像）。"
        echo ""
    fi
}

# =============================================================================
# 工具状态总览表
# =============================================================================

print_tool_table() {
    section "工具状态总览（预设：$OPT_PRESET）"
    echo ""

    # 表头
    printf "  ${W}%-14s %-11s %-22s %-24s %s${N}\n" \
        "工具" "状态" "所需二进制" "安装方式" "说明"
    printf "  %-14s %-11s %-22s %-24s %s\n" \
        "──────────" "────────" "────────────────" "──────────────────" "──────────"

    for name in $ALL_TOOLS; do
        local binary itype icmd desc in_preset_color status_str install_str
        binary="$(tool_binary "$name")"
        itype="$(tool_install_type "$name")"
        icmd="$(tool_install_cmd "$name")"
        desc="$(tool_desc "$name")"

        # 在预设中？
        local in_current_preset=false
        tool_in_preset "$name" "$OPT_PRESET" && in_current_preset=true || true

        # 是否已安装
        local is_installed=false
        [[ -n "$binary" ]] && cmd_exists "$binary" && is_installed=true || true
        # builtin 类工具只要 cargo 存在就算"已安装"
        [[ "$itype" == "builtin" ]] && $HAS_CARGO && is_installed=true || true

        # 状态字符串
        if $is_installed; then
            status_str="${G}✓ 已安装${N}"
        elif $in_current_preset; then
            status_str="${R}✗ 缺失  ${N}"
        else
            status_str="${Y}○ 不在预设${N}"
        fi

        # 安装方式说明
        case "$itype" in
            builtin)          install_str="cargo 内置" ;;
            rustup_component) install_str="rustup component add $icmd" ;;
            cargo)            install_str="cargo install $icmd" ;;
            *)                install_str="-" ;;
        esac

        # 只打印预设内的工具，或已安装的工具
        if $in_current_preset || $is_installed; then
            printf "  %-14s " "$name"
            printf "%b%-11b" "$status_str" "$N"
            printf "%-22s " "$binary"
            printf "%-24s " "$install_str"
            printf "%s\n" "$desc"
        fi
    done
    echo ""
}

# =============================================================================
# 安装逻辑
# =============================================================================

# 安装系统包
install_system_pkg() {
    local pkg="$1" tool="$2"
    [[ -z "$pkg" || "$pkg" == "UNSUPPORTED" ]] && return 0
    $OPT_NO_SYSTEM && { log_skip "[$tool] --no-system 已跳过系统包：$pkg"; return 0; }
    $OPT_OFFLINE   && { log_skip "[$tool] --offline 已跳过系统包：$pkg"; return 0; }

    log_step "[$tool] 安装系统依赖：$pkg"
    case "$SYS_PKG_MGR" in
        apt)    run_cmd sudo apt-get install -y "$pkg" ;;
        dnf)    run_cmd sudo dnf install -y "$pkg" ;;
        yum)    run_cmd sudo yum install -y "$pkg" ;;
        pacman) run_cmd sudo pacman -S --noconfirm "$pkg" ;;
        zypper) run_cmd sudo zypper install -y "$pkg" ;;
        apk)    run_cmd sudo apk add "$pkg" ;;
        brew)   run_cmd brew install "$pkg" ;;
        *)
            log_warn "[$tool] 未检测到包管理器，请手动安装：$pkg"
            return 1
            ;;
    esac
}

# 安装单个工具
install_tool() {
    local name="$1"
    local binary itype icmd sys_deps_str
    binary="$(tool_binary "$name")"
    itype="$(tool_install_type "$name")"
    icmd="$(tool_install_cmd "$name")"
    sys_deps_str="$(tool_system_deps "$name")"

    # 已安装（builtin 只需 cargo 存在）
    local already=false
    if [[ "$itype" == "builtin" ]] && $HAS_CARGO; then
        already=true
    elif [[ -n "$binary" ]] && cmd_exists "$binary"; then
        already=true
    fi

    if $already; then
        log_ok "[$name] ${binary} 已安装，跳过"
        CNT_ALREADY=$((CNT_ALREADY + 1))
        return 0
    fi

    case "$itype" in

        builtin)
            # 不应到达此处（cargo 缺失时已在 print_env_info 退出）
            log_error "[$name] cargo 未找到"
            CNT_FAILED=$((CNT_FAILED + 1))
            ;;

        rustup_component)
            if ! $HAS_RUSTUP; then
                log_warn "[$name] 未找到 rustup，无法自动安装 rustup component $icmd"
                echo "  → 请检查工具链包中是否包含 $binary，或从以下地址手动下载："
                echo "    https://static.rust-lang.org/dist/"
                CNT_SKIPPED=$((CNT_SKIPPED + 1))
                return 0
            fi
            if $OPT_OFFLINE; then
                log_step "[$name] 尝试离线安装（可能已有本地缓存）：rustup component add $icmd"
            else
                log_step "[$name] 安装：rustup component add $icmd"
            fi
            if run_cmd rustup component add "$icmd"; then
                log_ok "[$name] 安装成功"
                CNT_INSTALLED=$((CNT_INSTALLED + 1))
            else
                log_error "[$name] rustup component add $icmd 失败"
                CNT_FAILED=$((CNT_FAILED + 1))
            fi
            ;;

        cargo)
            # 离线模式下跳过 cargo install
            if $OPT_OFFLINE; then
                log_skip "[$name] 离线模式：跳过 cargo install $icmd"
                echo "  → 离线安装选项："
                echo "    A. 配置本地 registry 镜像后执行：cargo install $icmd"
                echo "    B. 从 GitHub Releases 下载预编译二进制并放入 PATH"
                CNT_SKIPPED=$((CNT_SKIPPED + 1))
                return 0
            fi

            # 检查并安装系统依赖
            local sys_pkg
            sys_pkg="$(get_sys_dep_for_current_os "$sys_deps_str")"
            if [[ "$sys_pkg" == "UNSUPPORTED" ]]; then
                log_warn "[$name] 当前平台（$OS_TYPE）不支持此工具，跳过"
                CNT_SKIPPED=$((CNT_SKIPPED + 1))
                return 0
            fi
            if [[ -n "$sys_pkg" ]] && ! $OPT_DRY_RUN; then
                log_info "[$name] 检测到系统依赖：$sys_pkg（若 cargo install 失败可能需要先安装）"
                install_system_pkg "$sys_pkg" "$name" || true
            elif [[ -n "$sys_pkg" ]] && $OPT_DRY_RUN; then
                log_dry "[$name] 系统依赖：$sys_pkg（dry-run，跳过安装）"
            fi

            log_step "[$name] 安装：cargo install $icmd"
            if run_cmd cargo install "$icmd"; then
                log_ok "[$name] 安装成功"
                CNT_INSTALLED=$((CNT_INSTALLED + 1))
            else
                log_error "[$name] cargo install $icmd 失败"
                echo "  → 请检查网络连接或 crates.io 可用性"
                echo "  → 如使用镜像，请在 ~/.cargo/config.toml 中配置 [source.crates-io]"
                CNT_FAILED=$((CNT_FAILED + 1))
            fi
            ;;

        *)
            log_warn "[$name] 未知安装类型（$itype），跳过"
            CNT_SKIPPED=$((CNT_SKIPPED + 1))
            ;;
    esac
}

# 主安装流程
do_install() {
    local preset_tools
    preset_tools="$(tools_in_preset "$OPT_PRESET")"

    section "安装（预设：$OPT_PRESET）"

    # 统计需要安装的工具数
    local need_install=0
    for name in $preset_tools; do
        local itype binary
        itype="$(tool_install_type "$name")"
        binary="$(tool_binary "$name")"
        if [[ "$itype" == "builtin" ]] && $HAS_CARGO; then
            :  # 已有
        elif [[ -n "$binary" ]] && cmd_exists "$binary"; then
            :  # 已有
        else
            need_install=$((need_install + 1))
        fi
    done

    if [[ $need_install -eq 0 ]]; then
        log_ok "预设 '$OPT_PRESET' 的所有工具均已安装，无需操作"
        return 0
    fi

    log_info "需要安装/处理 $need_install 个工具"
    echo ""

    if ! $OPT_DRY_RUN && ! $OPT_YES; then
        confirm "是否继续安装以上工具？" || { log_warn "用户取消"; exit 0; }
    fi
    echo ""

    for name in $preset_tools; do
        install_tool "$name"
    done
}

# =============================================================================
# 安装后验证
# =============================================================================

post_install_verify() {
    $OPT_DRY_RUN && return 0  # dry-run 无需验证

    local preset_tools
    preset_tools="$(tools_in_preset "$OPT_PRESET")"

    section "安装后验证"
    echo ""

    local ok=0 fail=0
    for name in $preset_tools; do
        local binary itype
        binary="$(tool_binary "$name")"
        itype="$(tool_install_type "$name")"

        if [[ "$itype" == "builtin" ]]; then
            log_ok "[$name] cargo 内置工具 ✓"
            ok=$((ok + 1))
        elif cmd_exists "$binary"; then
            local ver
            ver="$("$binary" --version 2>/dev/null | head -1 || echo '版本未知')"
            log_ok "[$name] $binary ✓  ($ver)"
            ok=$((ok + 1))
        else
            log_error "[$name] $binary 仍未找到 ✗"
            fail=$((fail + 1))
        fi
    done

    echo ""
    echo -e "  验证结果：${G}$ok 通过${N}  ${R}$fail 失败${N}"
    [[ $fail -gt 0 ]] && return 1 || return 0
}

# =============================================================================
# 离线安装指引
# =============================================================================

print_offline_guide() {
    $OPT_OFFLINE || return 0

    section "离线安装指引"
    echo ""
    cat <<'EOF'
  在完全离线或受限网络环境中，可通过以下方式获取工具：

  ① Rust 工具链（rustup/cargo/clippy/rustfmt）：
       https://static.rust-lang.org/dist/
     下载对应平台和版本的工具链压缩包，手动解压并配置 PATH。
     文件名示例：rust-1.XX.0-x86_64-unknown-linux-gnu.tar.gz

  ② cargo install 类工具（cargo-audit、cargo-deny 等）：
     • 方式 A：配置企业内网 crates.io 镜像（sparse 协议）：
         # ~/.cargo/config.toml
         [source.crates-io]
         replace-with = "my-mirror"
         [source.my-mirror]
         registry = "https://your-mirror.example.com/index"
     • 方式 B：在有网环境用 cargo vendor 打包后同步到离线机器。
     • 方式 C：从各工具的 GitHub Releases 下载预编译二进制并放入 PATH：
         cargo-audit  : https://github.com/rustsec/rustsec/releases
         cargo-deny   : https://github.com/EmbarkStudios/cargo-deny/releases
         cargo-geiger : https://github.com/geiger-rs/cargo-geiger/releases
         cargo-llvm-cov: https://github.com/taiki-e/cargo-llvm-cov/releases

  ③ 系统包（llvm、linux-tools-generic 等）：
     使用发行版的离线包仓库（DVD/ISO 安装源），或
     从发行版官网下载 .deb/.rpm/.pkg.tar.zst 等离线包手动安装。

EOF
}

# =============================================================================
# 汇总报告
# =============================================================================

print_summary() {
    section "安装汇总"
    echo ""
    echo -e "  ${G}本次安装成功：  $CNT_INSTALLED 个${N}"
    echo -e "  ${C}之前已安装：    $CNT_ALREADY 个${N}"
    echo -e "  ${Y}跳过：          $CNT_SKIPPED 个${N}"
    echo -e "  ${R}失败：          $CNT_FAILED 个${N}"
    echo ""

    if $OPT_DRY_RUN; then
        log_info "预演完成（--dry-run）。去掉该标志后重新运行以实际安装。"
    elif $OPT_CHECK; then
        log_info "状态检查完成（--check）。"
    elif [[ $CNT_FAILED -gt 0 ]]; then
        log_warn "部分工具安装失败，请查看上方错误信息。"
        log_info "提示：使用 --offline 查看离线安装指引；--no-system 可跳过系统包。"
    else
        log_ok "安装完成！"
        echo ""
        echo -e "  ${W}下一步：${N}"
        echo "    1. 初始化项目配置：  rust-checker init --preset $OPT_PRESET"
        echo "    2. 运行全量检查：     rust-checker run"
    fi
    echo ""
}

# =============================================================================
# 参数解析
# =============================================================================

parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --preset)
                shift
                OPT_PRESET="${1:-full}"
                case "$OPT_PRESET" in
                    minimal|quality|security|full) ;;
                    *) log_error "不支持的预设：$OPT_PRESET（可选：minimal|quality|security|full）"
                       exit 1 ;;
                esac
                ;;
            --dry-run)   OPT_DRY_RUN=true ;;
            --check)     OPT_CHECK=true ;;
            --yes|-y)    OPT_YES=true ;;
            --offline)   OPT_OFFLINE=true ;;
            --no-system) OPT_NO_SYSTEM=true ;;
            --help|-h)   show_help; exit 0 ;;
            *) log_error "未知选项：$1（使用 --help 查看帮助）"; exit 1 ;;
        esac
        shift
    done
}

show_help() {
    banner
    cat <<'HELP'
用法：
  bash scripts/install-tools.sh [OPTIONS]

选项：
  --preset PRESET    安装指定预设的工具（默认：full）
                       minimal   — build, test, clippy, fmt
                       quality   — minimal + doc, coverage
                       security  — build, test, audit, deny, geiger
                       full      — 所有工具
  --dry-run          仅展示将要执行的操作，不实际安装
  --check            仅检查工具状态并输出报告，不安装
  --yes/-y           非交互模式（CI 环境推荐）
  --offline          离线模式：跳过网络下载，显示离线安装指引
  --no-system        跳过系统包安装（apt/brew 等）
  --help/-h          显示此帮助

示例：
  # 检查所有工具的当前安装状态
  bash scripts/install-tools.sh --check

  # 安装 quality 预设所需工具（CI/非交互环境）
  bash scripts/install-tools.sh --preset quality --yes

  # 预演 full 预设安装（不实际执行）
  bash scripts/install-tools.sh --preset full --dry-run

  # 离线环境下查看缺失工具及安装指引
  bash scripts/install-tools.sh --check --offline

  # 安装 security 预设，跳过系统包
  bash scripts/install-tools.sh --preset security --yes --no-system
HELP
}

# =============================================================================
# 主入口
# =============================================================================

main() {
    parse_args "$@"
    banner
    detect_os
    print_env_info
    print_offline_guide
    print_tool_table

    if $OPT_CHECK; then
        print_summary
        # 如有缺失工具则以非零退出
        local preset_tools
        preset_tools="$(tools_in_preset "$OPT_PRESET")"
        for name in $preset_tools; do
            local itype binary
            itype="$(tool_install_type "$name")"
            binary="$(tool_binary "$name")"
            if [[ "$itype" == "builtin" ]] && ! $HAS_CARGO; then
                exit 1
            elif [[ "$itype" != "builtin" ]] && [[ -n "$binary" ]] && ! cmd_exists "$binary"; then
                exit 2
            fi
        done
        exit 0
    fi

    do_install
    post_install_verify || true
    print_summary

    [[ $CNT_FAILED -gt 0 ]] && exit 1 || exit 0
}

main "$@"
