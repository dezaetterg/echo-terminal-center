#!/bin/bash
# Echo Terminal Center Installer
# Standalone installer with system detection and build dependency auto-resolution.
set -euo pipefail

# --- Paths ---
INSTALL_DIR="${XDG_BIN_HOME:-$HOME/.local/bin}"
DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}"
DESKTOP_DIR="$DATA_DIR/applications"
ICON_DIR="$DATA_DIR/icons/hicolor/512x512/apps"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# --- Color Definitions (ANSI 256 + standard) ---
C_CYAN='\033[38;5;51m'
C_SKY='\033[38;5;75m'
C_BLUE='\033[38;5;33m'
C_GOLD='\033[38;5;220m'
C_AMBER='\033[38;5;208m'
C_WHITE='\033[1;37m'
C_GREEN='\033[38;5;48m'
C_RED='\033[38;5;196m'
C_YELLOW='\033[38;5;226m'
C_DIM='\033[38;5;245m'
C_BOLD='\033[1m'
C_RESET='\033[0m'

# --- Helpers ---
info()    { echo -e "  ${C_GREEN}[✓]${C_RESET} $*"; }
warn()    { echo -e "  ${C_YELLOW}[!]${C_RESET} $*"; }
err()     { echo -e "  ${C_RED}[✗]${C_RESET} $*" >&2; }
item()    { echo -e "  ${C_SKY}•${C_RESET} $*"; }
section() { echo -e "\n${C_BOLD}${C_SKY}$*${C_RESET}"; }

# --- CLI Options Parsing ---
AUTO_CONFIRM=false
FORCE_BUILD=false
for arg in "$@"; do
    case "$arg" in
        -y|--yes)
            AUTO_CONFIRM=true
            ;;
        -b|--build|--from-source)
            FORCE_BUILD=true
            ;;
        -h|--help)
            echo "Usage: ./install.sh [OPTIONS]"
            echo
            echo "Options:"
            echo "  -y, --yes          Automatic yes to prompts (non-interactive)"
            echo "  -b, --build        Force build from source with cargo"
            echo "  -h, --help         Show this help message"
            exit 0
            ;;
    esac
done

# --- Print Colorful CLI Banner ---
print_banner() {
    echo
    echo -e "     \033[38;2;226;229;233m▄\033[0m\033[38;2;221;221;226m\033[48;2;232;235;237m▀\033[0m\033[38;2;205;209;213m\033[48;2;242;246;248m▀\033[0m\033[38;2;200;204;209m\033[48;2;242;245;248m▀\033[0m\033[38;2;204;210;213m\033[48;2;236;239;243m▀\033[0m\033[38;2;207;210;214m\033[48;2;230;232;238m▀\033[0m\033[38;2;208;215;218m\033[48;2;227;229;235m▀\033[0m\033[38;2;211;216;221m\033[48;2;225;228;234m▀\033[0m\033[38;2;219;221;224m\033[48;2;227;230;235m▀\033[0m\033[38;2;221;221;226m\033[48;2;226;230;235m▀\033[0m\033[38;2;233;237;239m▄\033[0m         \033[1;36mE C H O   T E R M I N A L   C E N T E R\033[0m"
    echo -e "   \033[38;2;200;206;206m\033[48;2;175;183;192m▀\033[0m\033[38;2;200;207;214m\033[48;2;215;221;225m▀\033[0m\033[38;2;244;247;251m\033[48;2;253;253;255m▀\033[0m\033[38;2;253;253;253m\033[48;2;247;249;250m▀\033[0m\033[38;2;247;250;252m\033[48;2;241;246;247m▀\033[0m\033[38;2;240;245;247m\033[48;2;240;244;247m▀\033[0m\033[38;2;238;242;244m\033[48;2;239;243;245m▀\033[0m\033[38;2;236;238;242m\033[48;2;238;242;244m▀\033[0m\033[38;2;234;236;240m\033[48;2;237;242;244m▀\033[0m\033[38;2;232;233;238m\033[48;2;238;241;245m▀\033[0m\033[38;2;229;230;235m\033[48;2;238;240;244m▀\033[0m\033[38;2;226;228;234m\033[48;2;234;237;240m▀\033[0m\033[38;2;255;255;255m\033[48;2;243;245;248m▀\033[0m\033[38;2;234;236;237m\033[48;2;224;228;232m▀\033[0m\033[38;2;175;187;187m▄\033[0m       \033[38;5;245mTerminal Control Center & System Fetch\033[0m"
    echo -e " \033[38;2;195;200;204m▄\033[0m\033[38;2;166;174;183m\033[48;2;182;190;199m▀\033[0m\033[38;2;188;195;203m\033[48;2;226;228;233m▀\033[0m\033[38;2;255;255;255m\033[48;2;244;247;250m▀\033[0m\033[38;2;245;248;250m\033[48;2;238;241;245m▀\033[0m\033[38;2;241;245;247m\033[48;2;240;244;247m▀\033[0m\033[38;2;242;246;247m\033[48;2;240;244;247m▀\033[0m\033[38;2;242;246;248m\033[48;2;241;245;248m▀\033[0m\033[38;2;241;245;247m\033[48;2;242;246;247m▀\033[0m\033[38;2;242;246;247m\033[48;2;242;247;248m▀\033[0m\033[38;2;242;246;248m\033[48;2;245;249;250m▀\033[0m\033[38;2;242;246;248m\033[48;2;248;252;252m▀\033[0m\033[38;2;243;248;249m\033[48;2;244;247;249m▀\033[0m\033[38;2;248;252;253m\033[48;2;227;230;232m▀\033[0m\033[38;2;238;242;244m\033[48;2;193;198;204m▀\033[0m\033[38;2;183;190;197m\033[48;2;144;154;162m▀\033[0m\033[38;2;159;168;175m▀\033[0m\033[38;2;167;171;177m▄\033[0m\033[38;2;191;194;199m▄\033[0m     \033[38;5;75m──────────────────────────────────────────\033[0m"
    echo -e "\033[38;2;213;216;218m▄\033[0m\033[38;2;171;179;184m\033[48;2;183;190;197m▀\033[0m\033[38;2;195;200;208m\033[48;2;205;208;216m▀\033[0m\033[38;2;233;235;240m\033[48;2;224;227;233m▀\033[0m\033[38;2;232;233;238m\033[48;2;231;232;237m▀\033[0m\033[38;2;238;241;245m\033[48;2;238;241;245m▀\033[0m\033[38;2;239;243;246m\033[48;2;240;244;247m▀\033[0m\033[38;2;240;244;247m\033[48;2;241;245;246m▀\033[0m\033[38;2;242;246;248m\033[48;2;245;249;249m▀\033[0m\033[38;2;242;246;247m\033[48;2;250;253;253m▀\033[0m\033[38;2;251;254;255m\033[48;2;210;214;218m▀\033[0m\033[38;2;241;245;246m\033[48;2;184;192;200m▀\033[0m\033[38;2;227;232;234m\033[48;2;214;221;228m▀\033[0m\033[38;2;205;211;214m\033[48;2;192;200;206m▀\033[0m\033[38;2;177;185;191m\033[48;2;173;182;188m▀\033[0m\033[38;2;190;197;203m\033[48;2;127;133;145m▀\033[0m\033[38;2;154;160;170m▀\033[0m\033[38;2;209;214;219m\033[48;2;221;225;227m▀\033[0m\033[38;2;214;218;223m\033[48;2;223;226;232m▀\033[0m\033[38;2;248;252;255m\033[48;2;224;227;234m▀\033[0m\033[38;2;218;221;223m\033[48;2;225;228;230m▀\033[0m    \033[38;5;75mPackage    : \033[1;37mecho-terminal-center\033[0m"
    echo -e "\033[38;2;214;217;220m\033[48;2;207;210;214m▀\033[0m\033[38;2;204;211;217m\033[48;2;206;212;217m▀\033[0m\033[38;2;209;214;221m\033[48;2;211;215;222m▀\033[0m\033[38;2;224;225;231m\033[48;2;228;229;234m▀\033[0m\033[38;2;233;235;240m\033[48;2;236;238;243m▀\033[0m\033[38;2;238;242;246m\033[48;2;238;243;245m▀\033[0m\033[38;2;239;244;246m\033[48;2;249;252;253m▀\033[0m\033[38;2;246;250;251m\033[48;2;216;220;223m▀\033[0m\033[38;2;234;238;240m\033[48;2;141;150;159m▀\033[0m\033[38;2;167;175;182m\033[48;2;148;159;170m▀\033[0m\033[38;2;153;164;175m\033[48;2;164;173;182m▀\033[0m\033[38;2;170;179;190m▀\033[0m\033[38;2;159;168;178m▀\033[0m  \033[38;2;193;198;202m▄\033[0m\033[38;2;217;221;224m\033[48;2;242;246;249m▀\033[0m\033[38;2;254;255;255m\033[48;2;247;249;250m▀\033[0m\033[38;2;214;217;224m\033[48;2;224;225;232m▀\033[0m\033[38;2;211;214;221m\033[48;2;224;226;231m▀\033[0m\033[38;2;222;225;227m\033[48;2;220;224;226m▀\033[0m\033[38;2;222;222;226m\033[48;2;231;233;235m▀\033[0m   \033[38;5;75mBinary     : \033[1;32mecho-terminal\033[0m"
    echo -e "\033[38;2;193;198;204m\033[48;2;186;193;199m▀\033[0m\033[38;2;206;209;215m\033[48;2;207;211;218m▀\033[0m\033[38;2;219;220;227m\033[48;2;226;228;234m▀\033[0m\033[38;2;230;231;237m\033[48;2;232;233;238m▀\033[0m\033[38;2;236;240;243m\033[48;2;244;246;250m▀\033[0m\033[38;2;247;251;251m\033[48;2;206;211;216m▀\033[0m\033[38;2;206;211;214m\033[48;2;134;143;154m▀\033[0m\033[38;2;132;141;152m\033[48;2;168;177;187m▀\033[0m\033[38;2;158;167;176m\033[48;2;143;151;159m▀\033[0m\033[38;2;173;182;188m▀\033[0m    \033[38;2;182;187;193m\033[48;2;190;195;201m▀\033[0m\033[38;2;168;176;184m\033[48;2;200;205;210m▀\033[0m\033[38;2;245;249;250m\033[48;2;251;252;255m▀\033[0m\033[38;2;236;238;242m\033[48;2;233;235;240m▀\033[0m\033[38;2;232;233;238m\033[48;2;237;241;243m▀\033[0m\033[38;2;236;236;241m\033[48;2;244;246;249m▀\033[0m\033[38;2;211;215;218m\033[48;2;205;210;213m▀\033[0m\033[38;2;201;204;208m\033[48;2;164;168;175m▀\033[0m   \033[38;5;75mPlatform   : \033[1;37mLinux (All Distros & DEs)\033[0m"
    echo -e "\033[38;2;192;197;204m\033[48;2;201;205;211m▀\033[0m\033[38;2;216;219;225m\033[48;2;223;226;232m▀\033[0m\033[38;2;233;234;238m\033[48;2;239;241;244m▀\033[0m\033[38;2;236;238;243m\033[48;2;228;230;235m▀\033[0m\033[38;2;218;222;226m\033[48;2;167;174;181m▀\033[0m\033[38;2;144;152;162m\033[48;2;164;174;182m▀\033[0m\033[38;2;180;190;198m\033[48;2;209;217;222m▀\033[0m\033[38;2;159;167;176m\033[48;2;170;179;187m▀\033[0m     \033[38;2;244;244;244m\033[48;2;183;190;195m▀\033[0m\033[38;2;232;238;241m\033[48;2;189;196;202m▀\033[0m\033[38;2;232;236;238m\033[48;2;223;226;231m▀\033[0m\033[38;2;236;238;243m\033[48;2;237;240;244m▀\033[0m\033[38;2;241;245;247m\033[48;2;245;250;251m▀\033[0m\033[38;2;242;247;247m\033[48;2;245;249;250m▀\033[0m\033[38;2;243;245;249m\033[48;2;236;238;243m▀\033[0m\033[38;2;214;219;222m\033[48;2;233;235;237m▀\033[0m\033[38;2;202;206;209m\033[48;2;234;237;238m▀\033[0m   \033[38;5;75mLanguage   : \033[1;37mRust (Native Binary)\033[0m"
    echo -e "\033[38;2;195;199;203m\033[48;2;171;175;182m▀\033[0m\033[38;2;225;228;234m\033[48;2;211;214;220m▀\033[0m\033[38;2;248;250;251m\033[48;2;243;244;247m▀\033[0m\033[38;2;206;210;214m\033[48;2;175;183;188m▀\033[0m\033[38;2;154;163;172m\033[48;2;171;180;189m▀\033[0m\033[38;2;194;200;207m\033[48;2;215;217;224m▀\033[0m\033[38;2;220;223;231m\033[48;2;223;224;230m▀\033[0m\033[38;2;204;209;215m\033[48;2;236;239;243m▀\033[0m\033[38;2;203;206;210m▄\033[0m    \033[38;2;198;203;208m\033[48;2;193;199;204m▀\033[0m\033[38;2;221;223;227m\033[48;2;198;203;209m▀\033[0m\033[38;2;223;224;230m\033[48;2;238;240;243m▀\033[0m\033[38;2;241;245;247m\033[48;2;239;243;246m▀\033[0m\033[38;2;245;249;250m\033[48;2;241;243;246m▀\033[0m\033[38;2;241;244;247m\033[48;2;230;231;237m▀\033[0m\033[38;2;231;233;237m\033[48;2;232;234;237m▀\033[0m\033[38;2;248;251;252m\033[48;2;239;243;247m▀\033[0m\033[38;2;221;223;227m\033[48;2;207;212;216m▀\033[0m   \033[38;5;75mEngine     : \033[1;37mRatatui + Crossterm\033[0m"
    echo -e " \033[38;2;188;192;198m\033[48;2;182;187;194m▀\033[0m\033[38;2;216;219;225m\033[48;2;209;214;220m▀\033[0m\033[38;2;143;152;162m\033[48;2;185;190;197m▀\033[0m\033[38;2;199;205;212m\033[48;2;215;218;224m▀\033[0m\033[38;2;229;230;236m\033[48;2;239;241;247m▀\033[0m\033[38;2;233;233;238m\033[48;2;241;244;247m▀\033[0m\033[38;2;241;243;248m\033[48;2;239;243;244m▀\033[0m\033[38;2;238;241;242m\033[48;2;248;251;253m▀\033[0m\033[38;2;219;223;223m\033[48;2;242;245;246m▀\033[0m\033[38;2;228;231;233m▄\033[0m\033[38;2;207;211;218m▄\033[0m\033[38;2;174;180;187m▄\033[0m\033[38;2;148;156;166m\033[48;2;177;184;191m▀\033[0m\033[38;2;211;217;223m\033[48;2;215;219;225m▀\033[0m\033[38;2;233;235;240m\033[48;2;219;222;229m▀\033[0m\033[38;2;234;236;240m\033[48;2;214;218;225m▀\033[0m\033[38;2;225;227;234m\033[48;2;209;214;221m▀\033[0m\033[38;2;222;224;230m\033[48;2;222;224;230m▀\033[0m\033[38;2;225;228;232m\033[48;2;204;210;216m▀\033[0m\033[38;2;214;219;223m\033[48;2;206;209;214m▀\033[0m\033[38;2;199;202;205m▀\033[0m   \033[38;5;75mModes      : \033[1;37mTUI / fetch / --json / --short\033[0m"
    echo -e "  \033[38;2;194;199;203m\033[48;2;189;195;200m▀\033[0m\033[38;2;213;217;220m\033[48;2;184;191;197m▀\033[0m\033[38;2;223;227;229m\033[48;2;210;214;217m▀\033[0m\033[38;2;242;246;249m\033[48;2;245;249;250m▀\033[0m\033[38;2;250;251;252m\033[48;2;253;253;253m▀\033[0m\033[38;2;249;251;251m\033[48;2;252;252;252m▀\033[0m\033[38;2;243;248;249m\033[48;2;249;252;253m▀\033[0m\033[38;2;245;249;250m\033[48;2;244;249;250m▀\033[0m\033[38;2;253;255;255m\033[48;2;239;242;244m▀\033[0m\033[38;2;239;242;249m\033[48;2;229;230;235m▀\033[0m\033[38;2;210;216;223m\033[48;2;201;208;212m▀\033[0m\033[38;2;195;202;207m\033[48;2;188;197;203m▀\033[0m\033[38;2;199;207;213m\033[48;2;197;205;211m▀\033[0m\033[38;2;202;209;216m\033[48;2;201;207;214m▀\033[0m\033[38;2;199;207;214m\033[48;2;219;224;231m▀\033[0m\033[38;2;217;223;229m\033[48;2;197;204;210m▀\033[0m\033[38;2;188;196;202m\033[48;2;212;219;224m▀\033[0m\033[38;2;173;183;192m\033[48;2;163;170;175m▀\033[0m\033[38;2;177;179;185m▀\033[0m    \033[38;5;75mVersion    : \033[1;37m1.0.0\033[0m"
    echo -e "   \033[38;2;195;198;202m▀\033[0m\033[38;2;193;200;207m\033[48;2;212;216;220m▀\033[0m\033[38;2;214;218;222m\033[48;2;239;242;244m▀\033[0m\033[38;2;252;252;252m\033[48;2;224;229;231m▀\033[0m\033[38;2;254;254;255m\033[48;2;230;232;235m▀\033[0m\033[38;2;251;252;253m\033[48;2;248;251;251m▀\033[0m\033[38;2;244;249;249m\033[48;2;249;252;253m▀\033[0m\033[38;2;235;238;242m\033[48;2;236;240;242m▀\033[0m\033[38;2;216;218;224m\033[48;2;232;235;237m▀\033[0m\033[38;2;203;208;214m\033[48;2;235;237;240m▀\033[0m\033[38;2;209;212;219m\033[48;2;235;236;241m▀\033[0m\033[38;2;214;218;225m\033[48;2;229;231;236m▀\033[0m\033[38;2;226;228;235m\033[48;2;201;206;213m▀\033[0m\033[38;2;184;191;198m\033[48;2;139;151;161m▀\033[0m\033[38;2;162;170;178m\033[48;2;139;149;159m▀\033[0m\033[38;2;215;219;221m▀\033[0m      \033[38;5;244m[echo-terminal-center]\033[0m     \033[38;5;244m[Release v1.0.0]\033[0m"
    echo -e "     \033[38;2;219;222;222m▀\033[0m\033[38;2;195;199;204m▀\033[0m\033[38;2;209;213;218m\033[48;2;193;197;204m▀\033[0m\033[38;2;194;198;204m\033[48;2;145;154;162m▀\033[0m\033[38;2;204;208;213m\033[48;2;159;169;175m▀\033[0m\033[38;2;232;236;238m\033[48;2;209;213;217m▀\033[0m\033[38;2;250;251;251m\033[48;2;240;241;243m▀\033[0m\033[38;2;245;247;248m\033[48;2;223;226;227m▀\033[0m\033[38;2;237;241;244m\033[48;2;208;213;216m▀\033[0m\033[38;2;227;230;233m\033[48;2;202;205;210m▀\033[0m\033[38;2;177;184;191m▀\033[0m\033[38;2;153;162;171m▀\033[0m     "
    echo
}

# --- System Detection ---
detect_system() {
    DISTRO_NAME="Linux"
    DISTRO_ID="unknown"
    if [ -f /etc/os-release ]; then
        # shellcheck disable=SC1091
        . /etc/os-release
        DISTRO_NAME="${PRETTY_NAME:-$NAME}"
        DISTRO_ID="${ID:-unknown}"
    fi

    DESKTOP_ENV="${XDG_CURRENT_DESKTOP:-${DESKTOP_SESSION:-Unknown}}"
    ARCH_NAME="$(uname -m)"

    PKG_MGR="unknown"
    if command -v apt-get > /dev/null 2>&1; then
        PKG_MGR="apt"
    elif command -v dnf > /dev/null 2>&1; then
        PKG_MGR="dnf"
    elif command -v pacman > /dev/null 2>&1; then
        PKG_MGR="pacman"
    elif command -v zypper > /dev/null 2>&1; then
        PKG_MGR="zypper"
    fi
}

print_system_info() {
    section "Host Environment:"
    item "Distribution : ${C_WHITE}${DISTRO_NAME}${C_RESET} (${ARCH_NAME})"
    item "Desktop Env  : ${C_WHITE}${DESKTOP_ENV}${C_RESET}"
    item "Pkg Manager  : ${C_WHITE}${PKG_MGR}${C_RESET}"
    item "Supported DE : ${C_GREEN}All Environments${C_RESET} (GNOME, KDE, Cinnamon, XFCE, MATE, Hyprland, Sway, etc.)"
}

# Map missing build dependencies to distro packages
get_distro_packages() {
    local target_pkg="$1"
    case "$PKG_MGR" in
        apt)
            case "$target_pkg" in
                rust) echo "cargo rustc build-essential" ;;
                cc)   echo "build-essential" ;;
            esac
            ;;
        dnf)
            case "$target_pkg" in
                rust) echo "cargo rust gcc" ;;
                cc)   echo "gcc" ;;
            esac
            ;;
        pacman)
            case "$target_pkg" in
                rust) echo "rust base-devel" ;;
                cc)   echo "base-devel" ;;
            esac
            ;;
        zypper)
            case "$target_pkg" in
                rust) echo "cargo rust gcc" ;;
                cc)   echo "gcc" ;;
            esac
            ;;
        *)
            echo "$target_pkg"
            ;;
    esac
}

suggest_and_install_packages() {
    local missing_items=("$@")
    local install_list=""
    for m in "${missing_items[@]}"; do
        pkgs="$(get_distro_packages "$m")"
        install_list="$install_list $pkgs"
    done
    install_list="$(echo "$install_list" | xargs)"

    echo -e "\n  ${C_BOLD}Recommended install command:${C_RESET}"
    case "$PKG_MGR" in
        apt)    echo "    sudo apt update && sudo apt install -y $install_list" ;;
        dnf)    echo "    sudo dnf install -y $install_list" ;;
        pacman) echo "    sudo pacman -S --needed $install_list" ;;
        zypper) echo "    sudo zypper install -y $install_list" ;;
        *)      echo "    Install: $install_list or run 'curl --proto \'=https\' --tlsv1.2 -sSf https://sh.rustup.rs | sh'" ;;
    esac
    echo

    if [ "$PKG_MGR" != "unknown" ]; then
        if [ "$AUTO_CONFIRM" = true ]; then
            answer="y"
        else
            read -rp "  Install missing dependencies automatically via sudo? [y/N]: " answer
        fi
        case "$answer" in
            [yY][eE][sS]|[yY])
                echo -e "  ${C_SKY}Running package manager...${C_RESET}"
                case "$PKG_MGR" in
                    apt)    sudo apt-get update && sudo apt-get install -y $install_list ;;
                    dnf)    sudo dnf install -y $install_list ;;
                    pacman) sudo pacman -S --needed --noconfirm $install_list ;;
                    zypper) sudo zypper install -y $install_list ;;
                esac
                echo -e "  ${C_GREEN}Dependencies installed. Continuing installation...${C_RESET}"
                return
                ;;
        esac
    fi

    err "Required build dependencies are missing. Please install Rust and re-run install.sh."
    exit 1
}

# --- Download Official Release Binary ---
download_prebuilt_binary() {
    local release_ver="1.0.0"
    local tar_url="https://github.com/dezaetterg/echo-terminal-center/releases/download/v${release_ver}/echo-terminal-${release_ver}-x86_64.tar.gz"
    local temp_dir
    temp_dir="$(mktemp -d -t echo-bin-XXXXXX 2>/dev/null || mktemp -d)"

    info "Fetching official release binary (v${release_ver}) from GitHub..."
    local dl_ok=false
    if command -v curl >/dev/null 2>&1; then
        if curl -fSL --progress-bar "$tar_url" -o "$temp_dir/release.tar.gz" 2>/dev/null; then
            dl_ok=true
        fi
    elif command -v wget >/dev/null 2>&1; then
        if wget -q --show-progress "$tar_url" -O "$temp_dir/release.tar.gz" 2>/dev/null; then
            dl_ok=true
        fi
    fi

    if [ "$dl_ok" = true ] && [ -f "$temp_dir/release.tar.gz" ]; then
        if tar -xzf "$temp_dir/release.tar.gz" -C "$temp_dir" 2>/dev/null; then
            local extracted_bin
            extracted_bin="$(find "$temp_dir" -type f -name "echo-terminal" -perm /111 2>/dev/null | head -n1)"
            if [ -n "$extracted_bin" ] && [ -x "$extracted_bin" ]; then
                cp "$extracted_bin" "$SCRIPT_DIR/echo-terminal"
                chmod +x "$SCRIPT_DIR/echo-terminal"
                rm -rf "$temp_dir"
                PREBUILT_BIN="$SCRIPT_DIR/echo-terminal"
                info "Release binary downloaded and verified successfully."
                return 0
            fi
        fi
    fi

    rm -rf "$temp_dir"
    warn "Could not retrieve pre-compiled binary from GitHub."
    return 1
}

# --- Dependencies Verification ---
check_and_resolve_dependencies() {
    section "Checking Build & Runtime Dependencies:"

    # Check if precompiled binary already exists locally
    PREBUILT_BIN=""
    if [ -f "$SCRIPT_DIR/echo-terminal" ] && [ -x "$SCRIPT_DIR/echo-terminal" ]; then
        PREBUILT_BIN="$SCRIPT_DIR/echo-terminal"
    elif [ -f "$SCRIPT_DIR/target/release/echo-terminal" ] && [ -x "$SCRIPT_DIR/target/release/echo-terminal" ]; then
        PREBUILT_BIN="$SCRIPT_DIR/target/release/echo-terminal"
    fi

    # Try downloading official release binary if not present and not forced to compile
    if [ -z "$PREBUILT_BIN" ] && [ "$FORCE_BUILD" = false ]; then
        if command -v curl >/dev/null 2>&1 || command -v wget >/dev/null 2>&1; then
            if download_prebuilt_binary; then
                PREBUILT_BIN="$SCRIPT_DIR/echo-terminal"
            fi
        fi
    fi

    if [ -n "$PREBUILT_BIN" ]; then
        info "Pre-compiled release binary ready ($PREBUILT_BIN)."
    else
        # If cargo is in ~/.cargo/bin, ensure it is available in this subshell
        if ! command -v cargo >/dev/null 2>&1; then
            if [ -f "$HOME/.cargo/env" ]; then
                # shellcheck disable=SC1091
                source "$HOME/.cargo/env"
            fi
        fi

        MISSING_REQUIRED=()

        # 1. Rust compiler & Cargo
        if command -v cargo > /dev/null 2>&1 && command -v rustc > /dev/null 2>&1; then
            RUSTC_VER="$(rustc --version 2>&1 | awk '{print $2}')"
            CARGO_VER="$(cargo --version 2>&1 | awk '{print $2}')"
            local major minor
            major="$(echo "$RUSTC_VER" | cut -d. -f1)"
            minor="$(echo "$RUSTC_VER" | cut -d. -f2)"
            if [ "$major" -eq 1 ] && [ "$minor" -lt 85 ]; then
                warn "Installed rustc (${RUSTC_VER}) is older than 1.85.0 (Debian/Ubuntu/Mint repository version)."
                warn "Modern TUI dependencies require Rust >= 1.85."
                info "Downloading official pre-compiled release binary instead..."
                if download_prebuilt_binary; then
                    PREBUILT_BIN="$SCRIPT_DIR/echo-terminal"
                else
                    err "Cannot build with rustc ${RUSTC_VER}. Please update Rust via rustup:"
                    err "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
                    exit 1
                fi
            else
                info "Rust toolchain found (rustc ${RUSTC_VER}, cargo ${CARGO_VER})."
            fi
        else
            err "Rust toolchain (cargo / rustc) not found."
            info "Attempting to download official pre-compiled release binary..."
            if download_prebuilt_binary; then
                PREBUILT_BIN="$SCRIPT_DIR/echo-terminal"
            else
                MISSING_REQUIRED+=("rust")
            fi
        fi

        # 2. C Compiler / Linker (only required when building from source)
        if [ -z "$PREBUILT_BIN" ]; then
            if command -v cc > /dev/null 2>&1 || command -v gcc > /dev/null 2>&1 || command -v clang > /dev/null 2>&1; then
                CC_NAME="$(cc --version 2>/dev/null | head -n1 || gcc --version 2>/dev/null | head -n1 || clang --version 2>/dev/null | head -n1 || echo 'C compiler')"
                info "C linker found (${CC_NAME%% *})."
            else
                warn "C linker (gcc/clang) not found in PATH."
                MISSING_REQUIRED+=("cc")
            fi

            # Handle Missing Required Dependencies
            if [ ${#MISSING_REQUIRED[@]} -gt 0 ]; then
                echo
                err "Required build tools are missing."
                suggest_and_install_packages "${MISSING_REQUIRED[@]}"
            fi
        fi
    fi

    # 3. Optional Runtime Tools Status Check
    echo
    echo -e "  ${C_DIM}Optional runtime utilities:${C_RESET}"
    for tool_spec in "pactl:Audio Control" "bluetoothctl:Bluetooth Manager" "sensors:Hardware Temperatures" "flatpak:Flatpak Packages" "systemctl:System Services"; do
        tool_cmd="${tool_spec%%:*}"
        tool_desc="${tool_spec##*:}"
        if command -v "$tool_cmd" > /dev/null 2>&1; then
            item "${C_WHITE}$tool_desc${C_RESET} ($tool_cmd) : ${C_GREEN}Available${C_RESET}"
        else
            item "${C_WHITE}$tool_desc${C_RESET} ($tool_cmd) : ${C_DIM}Not installed (module fallback active)${C_RESET}"
        fi
    done
}

# --- Show Target Paths ---
show_target_plan() {
    local mode_desc="Compile from Source (cargo build --release)"
    if [ -n "${PREBUILT_BIN:-}" ]; then
        mode_desc="Pre-compiled Release Binary (Native ELF)"
    fi

    section "Installation Target Layout (echo-terminal-center):"
    echo -e "  ${C_DIM}┌───────────────────────────────────────────────────────────────────────────┐${C_RESET}"
    echo -e "  ${C_DIM}│${C_RESET}  ${C_BOLD}Application Name${C_RESET}  : ${C_WHITE}Echo Terminal Center${C_RESET}"
    echo -e "  ${C_DIM}│${C_RESET}  ${C_BOLD}Binary Executable${C_RESET} : ${C_WHITE}$INSTALL_DIR/echo-terminal${C_RESET}"
    echo -e "  ${C_DIM}│${C_RESET}  ${C_BOLD}Desktop Entry${C_RESET}     : ${C_WHITE}$DESKTOP_DIR/echo-terminal.desktop${C_RESET}"
    echo -e "  ${C_DIM}│${C_RESET}  ${C_BOLD}Application Icon${C_RESET}  : ${C_WHITE}utilities-terminal (System Icon)${C_RESET}"
    echo -e "  ${C_DIM}│${C_RESET}  ${C_BOLD}Build Mode${C_RESET}        : ${C_WHITE}$mode_desc${C_RESET}"
    echo -e "  ${C_DIM}└───────────────────────────────────────────────────────────────────────────┘${C_RESET}"
    echo

    if [ "$AUTO_CONFIRM" = false ]; then
        read -rp "  Proceed with installation? [Y/n]: " answer
        case "$answer" in
            [nN][oO]|[nN])
                echo "Installation cancelled by user."
                exit 0
                ;;
        esac
    fi
}

# --- Installation Steps ---
perform_installation() {
    section "Compiling & Installing Echo Terminal Center:"

    # 1. Directories
    mkdir -p "$INSTALL_DIR" "$DESKTOP_DIR"
    info "Target directories initialized."

    # 2. Determine or Build Binary
    local source_bin=""
    if [ -n "${PREBUILT_BIN:-}" ] && [ -f "$PREBUILT_BIN" ] && [ -x "$PREBUILT_BIN" ]; then
        source_bin="$PREBUILT_BIN"
        info "Using release binary ($source_bin)."
    elif [ -f "$SCRIPT_DIR/echo-terminal" ] && [ -x "$SCRIPT_DIR/echo-terminal" ]; then
        source_bin="$SCRIPT_DIR/echo-terminal"
        info "Using pre-compiled binary ($source_bin)."
    elif [ -f "$SCRIPT_DIR/target/release/echo-terminal" ] && [ -x "$SCRIPT_DIR/target/release/echo-terminal" ]; then
        source_bin="$SCRIPT_DIR/target/release/echo-terminal"
        info "Using existing release build ($source_bin)."
    else
        cd "$SCRIPT_DIR"
        # Compatibility fix: older Cargo (< 1.78, e.g. 1.75 on Ubuntu/Mint) rejects lockfile version 4
        if [ -f "$SCRIPT_DIR/Cargo.lock" ] && grep -q "version = 4" "$SCRIPT_DIR/Cargo.lock" 2>/dev/null; then
            sed -i 's/version = 4/version = 3/' "$SCRIPT_DIR/Cargo.lock" 2>/dev/null || true
        fi

        info "Compiling optimized release binary (cargo build --release)..."
        if ! cargo build --release; then
            warn "Cargo build failed with current lockfile. Regenerating Cargo.lock..."
            rm -f "$SCRIPT_DIR/Cargo.lock"
            cargo build --release
        fi
        source_bin="$SCRIPT_DIR/target/release/echo-terminal"
    fi

    # 3. Install Executable
    install -m 755 "$source_bin" "$INSTALL_DIR/echo-terminal"
    info "Executable installed to $INSTALL_DIR/echo-terminal"

    # 4. Remove any stale symlinks
    rm -f "$INSTALL_DIR/echo-terminal-center"

    # 5. Desktop Entry
    cat << 'EOF' > "$DESKTOP_DIR/echo-terminal.desktop"
[Desktop Entry]
Type=Application
Name=Echo Terminal Center
GenericName=Terminal Control Center
Comment=Terminal Control Center and System Information
Exec=echo-terminal
Icon=utilities-terminal
Terminal=true
Categories=System;Monitor;Utility;
Keywords=system;info;fetch;monitor;audio;bluetooth;power;terminal;
EOF
    chmod 644 "$DESKTOP_DIR/echo-terminal.desktop"
    info "Desktop entry registered at $DESKTOP_DIR/echo-terminal.desktop"

    # 6. Optional Icon installation if asset exists
    if [ -f "$SCRIPT_DIR/assets/echo_logo.png" ]; then
        mkdir -p "$ICON_DIR"
        cp "$SCRIPT_DIR/assets/echo_logo.png" "$ICON_DIR/echo-terminal.png"
        chmod 644 "$ICON_DIR/echo-terminal.png"
        info "Application icon deployed to $ICON_DIR/echo-terminal.png"
    fi

    # 7. Update Desktop Database & Icon Cache
    if command -v update-desktop-database > /dev/null 2>&1; then
        update-desktop-database "$DESKTOP_DIR" 2>/dev/null && info "Desktop database updated." || true
    fi
    if command -v gtk-update-icon-cache > /dev/null 2>&1 && [ -d "$DATA_DIR/icons/hicolor" ]; then
        gtk-update-icon-cache -f -t "$DATA_DIR/icons/hicolor" 2>/dev/null && info "Icon cache updated." || true
    fi
}

# --- Summary Card ---
print_summary() {
    local rust_v cargo_v bin_size
    rust_v="$(rustc --version 2>&1 | awk '{print $2}' || echo 'installed')"
    cargo_v="$(cargo --version 2>&1 | awk '{print $2}' || echo 'installed')"
    bin_size="$(ls -lh "$INSTALL_DIR/echo-terminal" 2>/dev/null | awk '{print $5}' || echo 'N/A')"

    echo
    echo -e "  ${C_GREEN}${C_BOLD}========================================================================${C_RESET}"
    echo -e "  ${C_WHITE}${C_BOLD}             Echo Terminal Center successfully installed!               ${C_RESET}"
    echo -e "  ${C_GREEN}------------------------------------------------------------------------${C_RESET}"
    echo -e "  ${C_BOLD}Interactive TUI ${C_RESET} : ${C_SKY}echo-terminal${C_RESET}"
    echo -e "  ${C_BOLD}System Fetch    ${C_RESET} : ${C_SKY}echo-terminal fetch${C_RESET}"
    echo -e "  ${C_BOLD}JSON Output     ${C_RESET} : ${C_SKY}echo-terminal --json${C_RESET}"
    echo -e "  ${C_BOLD}Single Line     ${C_RESET} : ${C_SKY}echo-terminal --short${C_RESET}"
    echo -e "  ${C_BOLD}Application Menu${C_RESET} : Search for ${C_WHITE}\"Echo Terminal Center\"${C_RESET} in your launcher"
    echo -e "  ${C_BOLD}Binary Location ${C_RESET} : ${C_WHITE}$INSTALL_DIR/echo-terminal${C_RESET} (${bin_size})"
    echo -e "  ${C_BOLD}Built With      ${C_RESET} : Rust ${rust_v} (Cargo ${cargo_v})"
    echo -e "  ${C_BOLD}Compatibility   ${C_RESET} : ${C_GREEN}Universal${C_RESET} (all Linux distros & terminals)"
    echo -e "  ${C_GREEN}========================================================================${C_RESET}"

    # Verify if ~/.local/bin is in PATH
    case ":$PATH:" in
        *":$INSTALL_DIR:"*) ;;
        *)
            echo
            warn "'$INSTALL_DIR' is not in your current PATH."
            echo -e "      To run '${C_WHITE}echo-terminal${C_RESET}' directly from any terminal, add this to ${C_WHITE}~/.bashrc${C_RESET} or ${C_WHITE}~/.zshrc${C_RESET}:"
            echo -e "      ${C_SKY}export PATH=\"\$HOME/.local/bin:\$PATH\"${C_RESET}"
            ;;
    esac
    echo
}

# --- Main Entrypoint ---
main() {
    print_banner
    detect_system
    print_system_info
    check_and_resolve_dependencies
    show_target_plan
    perform_installation
    print_summary
}

main "$@"
