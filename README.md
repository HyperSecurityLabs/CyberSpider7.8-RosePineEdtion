# CyberSpider7.8-RosePineEdtion
A multi-threaded web reconnaissance spider, Written in Rust From HyperSecurityOffensiveLabs .
 🕷️ CyberSpider v7.8.0pro — RoséPine Evergreen Edition

> **Version 7.0 → 7.8.0pro — The "Stop Playing, Start Dominating" Release**

<br>

<div align="center">

![Version](https://img.shields.io/badge/version-7.8.0pro-eb6f92?style=for-the-badge&labelColor=191724)
![Rust](https://img.shields.io/badge/rust-1.70%2B-31748f?style=for-the-badge&logo=rust&logoColor=white&labelColor=191724)
![License](https://img.shields.io/badge/license-MIT-9ccfd8?style=for-the-badge&labelColor=191724)
![Status](https://img.shields.io/badge/status-offensive%20security-c4a7e7?style=for-the-badge&labelColor=191724)
![Warnings](https://img.shields.io/badge/warnings-0-50fa7b?style=for-the-badge&labelColor=191724)
![Build](https://img.shields.io/badge/build-passing-f6c177?style=for-the-badge&labelColor=191724)

</div>

<br>

---

## 🏆 CHANGELOG — From v7.0 to v7.8.0pro

### v7.8.0pro — "RoséPine Evergreen" (Current)

```
Release Date:  May 2026
Codename:      RoséPine Evergreen
Motto:         "No warnings. No mercy."
```

| Area | Change | Details |
|------|--------|---------|
| 🎨 **Theme** | RoséPine default | Full truecolor palette: rose `#eb6f92`, pine `#31748f`, foam `#9ccfd8`, iris `#c4a7e7`, gold `#f6c177` |
| 🌀 **Spinner** | Async braille spinner | Tokio task ticks at 160ms with `yield_now()` — zero CPU waste, smooth animation |
| 🔴 **URL Color** | All URLs in green | Every discovered URL displayed in vibrant green `#50fa7b` |
| 📡 **Live Output** | Real-time crawl feed | `spinner.log()` prints discovered URLs above the spinning animation in real-time |
| ⏱ **Throttled Logs** | 200ms min interval | Prevents terminal flooding during high-throughput crawls |
| 🛡️ **Media Corruption** | Attack module added | `MediaCorruptionAttacker` with 5 real attack vectors: PUT overwrite, path traversal, ImageTragick (CVE-2016-3714), SVG XXE, upload endpoint exploitation |
| 🔎 **Media Detection** | Magic byte + MIME scanner | Detects corrupted media across 20+ formats (jpg, png, gif, mp4, pdf, etc.) |
| 🏷️ **--tag mode** | Specialized campaigns | `--tag media-corruption` triggers full aggressive media attack campaign |
| 📂 **Endpoint Discovery** | Auto-probe upload paths | Scans 20+ common upload endpoints per domain (`/upload`, `/api/media`, `/wp-admin/*`) |
| 🕵️ **Admin Scanner** | Path discovery | Probes 15+ admin paths (`/admin`, `/wp-admin`, `/manager`) for accessible pages |
| ✅ **Verification** | SHA-256 hash compare | After attack, re-fetches media and compares hash to confirm corruption |
| 🔧 **CLI** | New flags | `--tag`, `--deep-scan`, `--media-check`, `--show-modules`, `--no-banner` |
| 🧹 **Warnings** | Zero tolerance | All 12 compiler warnings fixed — 0 warnings, 0 errors |
| 🔌 **Dependencies** | `pnet` + `socket2` added | Network-level packet crafting and raw socket capabilities |
| 📖 **Config** | Updated defaults | Threads=2, theme=rosepine, all new fields in `cyberspider.toml` |
| 🧪 **Verification** | Corruption confirmed | Re-downloads attacked media and verifies hash change |

---

### v7.0.0 — "The Great Cleanup"

| Change | Description |
|--------|-------------|
| 🗑️ Removed `#![allow(dead_code)]` | From all 55 `.rs` files — zero dead code tolerance |
| 🔢 Threads default 2 | Changed from 1 to 2 (`-j2`) for balanced performance |
| 🚫 Removed `indicatif` | All progress bars removed, replaced with braille spinner |
| 🎨 RoséPine branding | "CyberSpider v7.8.0pro - OFFENSIVE SECURITY" with "RoséPine Evergreen Edition" |
| 🎭 Removed `--theme` arg | Always uses RoséPine braille spinner — no theming needed |
| 📦 Version bump | 7.0.0 → 7.8.0pro across all files (Cargo.toml, banner, configs, sources) |

---

### v7.0.0 — "Original Release"

```
Release Date:  Early 2026
Motto:         "Let's see what's out there"
```

- First public release
- Multi-threaded crawling engine
- URL extraction with regex
- External source integration (Wayback Machine, Common Crawl, VirusTotal)
- Distributed architecture (coordinator/worker)
- Plugin system with libloading
- SQLite and Redis database support
- DOT graph visualization
- Webhook notifications (Discord/Slack)
- Proxy rotation support
- 4 themes: CyberWave, Matrix, Neon, Terminal
- Basic progress bars with `indicatif`

---

<br>

## 🚀 QUICK COMPARISON: v7.0 vs v7.8.0pro

| Feature | v7.0 | v7.8.0pro |
|---------|------|-----------|
| Compiler Warnings | 12 | **0** 🎯 |
| Theme Options | 4 (cyberwave/matrix/neon/terminal) | RoséPine (always on) |
| Spinner Animation | Static `println!` lines | **Async tokio task** 160ms tick |
| URL Highlighting | None | **Vibrant green** `#50fa7b` |
| Live Crawl Feed | Only with verbose flag | **Always via spinner.log()** |
| Media Detection | ❌ | ✅ Magic byte + MIME scanner |
| Media Corruption | ❌ | ✅ **5 real attack vectors** |
| --tag Campaigns | ❌ | ✅ `media-corruption` mode |
| Upload Endpoint Discovery | ❌ | ✅ 20+ paths auto-probed |
| Admin Path Scanner | ❌ | ✅ 15+ paths checked |
| Corruption Verification | ❌ | ✅ SHA-256 hash comparison |
| Default Threads | 1 | **2** 🚀 |
| Dead Code Allowed | 55 files | **0** 🎯 |
| `--theme` flag | ✅ | Removed (RoséPine only) |
| `indicatif` bars | ✅ | Removed (braille only) |
| `pnet` + `socket2` | ❌ | ✅ Network attack capabilities |

---

## 🧬 ARCHITECTURE OVERVIEW

```
                          ┌──────────────────────┐
                          │   main.rs (CLI)      │
                          │--site --tag --media  │
                          └──────────┬───────────┘
                                     │
                          ┌──────────▼───────────┐
                          │  BrailleSpinner      │
                          │  (async tokio task)  │
                          │  160ms + yield_now() │
                          └──────────┬───────────┘
                                     │
                          ┌──────────▼───────────┐
                          │  Spider::run()       │
                          │  SpiderConfig driven │
                          └──────────┬───────────┘
                                     │
               ┌─────────────────────┼─────────────────────┐
               │                     │                     │
    ┌──────────▼──────────┐ ┌───────▼────────┐ ┌──────────▼──────────┐
    │  Crawler::crawl_    │ │  URL Extractor │ │  External Sources   │
    │  targets()          │ │  (regex magic) │ │  (Wayback/CC/VT)    │
    └──────────┬──────────┘ └────────────────┘ └─────────────────────┘
               │
     ┌─────────┼─────────────┐
     │         │             │
┌────▼────┐ ┌──▼──┐  ┌──────▼──────┐
│ Media   │ │Media│  │  Upload     │
│ Detect  │ │Attack│ │  Endpoint   │
│(passive)│ │(real│  │  Discovery  │
└─────────┘ └─────┘  └─────────────┘
```

---

## 🔥 REAL ATTACK VECTORS DOCUMENTED

### 1. PUT Overwrite with Auth Progression
Tries 8 auth strategies in sequence: no auth → Bearer admin → Bearer root → Basic admin:admin → Basic admin:password → Basic root:root → X-API-Key → X-Auth-Token. If any succeeds, the media file is overwritten with corrupted data.

### 2. Path Traversal Upload
Sends multipart POST requests to 20+ upload endpoints with `../../../target.jpg` filename injection. Tries 6 different form field names (`file`, `upload`, `image`, `media`, `asset`, `qqfile`, `files`).

### 3. ImageTragick (CVE-2016-3714)
Crafts an SVG containing an ImageMagick MSL delegate payload. When processed by a vulnerable ImageMagick instance, the `url()` delegate writes arbitrary files (including overwriting existing media). Targets 15+ image processor endpoints.

### 4. SVG XXE Injection
XML External Entity payload embedded in SVG. Targets XML parsers during image/media processing. Can read internal files and in some configurations write to the filesystem.

### 5. Upload Form Discovery & Exploitation
Probes 20+ common upload paths per domain. When found, sends corrupted media files via multipart upload with proper MIME types per file extension.

---

## 👤 AUTHOR

<div align="center">

# 🧙‍♂️ Khaninkali

### *Security Researcher — Offensive Security Mindset*  
#### *(Not an expert, just really, really spicy)*

<br>

| 📡 **Contact** | 🔗 |
|--------------|-----|
| 🐙 GitHub | [github.com/hypersecurityLabs](https://github.com/hypersecurityLabs) |
| 📱 Telegram | [@hypersecurity_offsec](https://t.me/hypersecurity_offsec) |
| 📢 Channel | [HyperSecurity Offsec](https://t.me/hypersecurity_offsec) |
| 🌐 Website | (https://hypersecurityoffensivelabs.great-site.net) |

</div>

<br>

---

## 🏛️ ORGANIZATION

### **HyperSecurity Offensive Labs**
*Advanced Security Research and Development*

> "We don't just find vulnerabilities. We make them our pets."

<br>

---

## ⚡ SYSTEM REQUIREMENTS

| Component | Requirement |
|-----------|-------------|
| **Rust** | 1.70+ (edition 2021) |
| **OS** | Linux (primary), macOS (secondary), Windows (untested — YMMV) |
| **RAM** | 256MB idle, 1GB+ during deep scans |
| **Storage** | ~500MB for build artifacts |
| **Network** | Outbound HTTP/HTTPS access to targets |
| **Optional** | Root for raw socket features (`--tag media-corruption` packet modes) | Dont Misuse!

---

## 📦 BUILT WITH

| Crate | Version | Why |
|-------|---------|-----|
| [tokio](https://tokio.rs) | 1.x | Async runtime that doesn't make us cry |
| [reqwest](https://docs.rs/reqwest) | 0.11 | HTTP client with multipart + TLS |
| [clap](https://docs.rs/clap) | 4.x | CLI argument parsing that just works |
| [colored](https://docs.rs/colored) | 2.x | Truecolor terminal output |
| [serde](https://docs.rs/serde) | 1.x | Serialization for configs + results |
| [pnet](https://docs.rs/pnet) | 0.35 | Packet crafting for network-level attacks |
| [socket2](https://docs.rs/socket2) | 0.5 | Raw socket control |
| [sha2](https://docs.rs/sha2) | 0.10 | SHA-256 for corruption verification |
| [scraper](https://docs.rs/scraper) | 0.17 | HTML parsing for URL extraction |

---

## 📜 LICENSE

**MIT License** — Do what you want, just don't blame us when you inevitably use this for something illegal and get caught. See the [LICENSE](LICENSE) file for the full text.

---

## ⚠️ FINAL WARNING

```
┌──────────────────────────────────────────────────────────────┐
│                                                              │
│   THIS TOOL IS FOR AUTHORIZED SECURITY TESTING ONLY.         │
│                                                              │
│   * You must have written permission before scanning.        │
│   * Media corruption attacks WILL damage target systems.     │
│   * The authors assume ZERO liability for your actions.      │
│   * If you break it, you bought it.                          │
│                                                              │
│   "With great power comes great responsibility"              │
│   — Uncle Ben, probably                                      │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

<br>

---

<div align="center">

### ⭐ Star this repo if you enjoy legally violating digital property ⭐

### 🌟 **v7.8.0pro — RoséPine Evergreen** 🌟

*"No warnings. No mercy."*

<br>

```
  ╔════════════════════════════════════════════════╗
  ║   Made with ❤️, ☕, and 0 compiler warnings    ║
  ║        by Khaninkali @ HyperSecurity Labs      ║
  ╚════════════════════════════════════════════════╝
```

</div>
