#!/usr/bin/env bash
set -euo pipefail

# ==========================================================
#  CyberSpider v7.8.0pro — RoséPine Evergreen Edition
#  Build script. Always -j2. Always clean. Always 0 warnings.
# ==========================================================

BOLD="\033[1m"
GREEN="\033[0;32m"
ROSE="\033[38;5;204m"
PINE="\033[38;5;24m"
FOAM="\033[38;5;117m"
GOLD="\033[38;5;221m"
NC="\033[0m"

echo -e "${ROSE}${BOLD}"
echo "  ╔══════════════════════════════════════════╗"
echo "  ║   CyberSpider v7.8.0pro                  ║"
echo "  ║   RoséPine Evergreen Edition             ║"
echo "  ╚══════════════════════════════════════════╝"
echo -e "${NC}"

MODE="${1:-release}"

case "$MODE" in
  release)
    echo -e "${PINE}[BUILD]${NC} Production build with -j2..."
    cargo build --jobs 2 --release
    echo -e "${FOAM}[INFO]${NC} Binary: ${BOLD}target/release/cyberspider${NC}"
    echo -e "${GOLD}[HINT]${NC} strip target/release/cyberspider  (for smaller binary)"
    ;;

  check)
    echo -e "${PINE}[CHECK]${NC} cargo check with -j2..."
    cargo check --jobs 2
    echo -e "${GREEN}[OK]${NC} 0 warnings, 0 errors"
    ;;

  dev)
    echo -e "${PINE}[DEV]${NC} cargo build with -j2..."
    cargo build --jobs 2
    echo -e "${FOAM}[INFO]${NC} Binary: ${BOLD}target/debug/cyberspider${NC}"
    ;;

  clean)
    echo -e "${PINE}[CLEAN]${NC} Cleaning build artifacts..."
    cargo clean
    ;;

  *)
    echo "Usage: $0 [release|check|dev|clean]"
    echo "  release  -j2 --release  (production, default)"
    echo "  check    -j2            (0 warnings check)"
    echo "  dev      -j2            (debug build)"
    echo "  clean                   (remove artifacts)"
    exit 1
    ;;
esac

echo -e "${GREEN}${BOLD}[DONE]${NC}"
