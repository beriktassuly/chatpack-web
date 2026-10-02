# chatpack-web

**Privacy-first, browser-based chat export processing for LLM and RAG pipelines.**

[![CI](https://github.com/berektassuly/chatpack-web/actions/workflows/ci.yml/badge.svg)](https://github.com/berektassuly/chatpack-web/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

[**🌐 Live Demo**](https://chatpack.berektassuly.com) |
[Core Library](https://github.com/berektassuly/chatpack) |
[Report Bug](https://github.com/berektassuly/chatpack-web/issues)

---

## Overview

**chatpack-web** brings the power of the [chatpack](https://github.com/berektassuly/chatpack) Rust library to the browser using WebAssembly (WASM). It allows you to convert messy chat exports into clean, token-efficient datasets for LLMs (ChatGPT, Claude, Gemini) without your data ever leaving your device.

Raw chat exports waste **80%+** of context window tokens on JSON syntax and metadata. Chatpack compresses this data by **13x**, retaining only what matters.

```
┌─────────────────┐     ┌────────────────┐     ┌─────────────────┐
│ Telegram JSON   │     │                │     │ Clean CSV       │
│ WhatsApp TXT    │ ──▶│  chatpack-web  │ ──▶ │ 13x compression │
│ Instagram JSON  │     │ (Browser WASM) │     │ LLM-ready       │
│ Discord Export  │     │                │     │ RAG-optimized   │
└─────────────────┘     └────────────────┘     └─────────────────┘
```

---

## Features

- **🔒 100% Private:** All processing happens locally in your browser via WebAssembly. Files are **never** uploaded to any server.
- **⚡ High Performance:** Powered by Rust, processing 100,000+ messages per second.
- **📉 Token Efficient:** Reduces token usage by ~92% (CSV output) compared to raw JSON.
- **🧩 Multi-Platform:** Supports Telegram, WhatsApp, Instagram, and Discord.
- **🧠 Smart Processing:**
  - Auto-detects source platform from filenames.
  - Merges consecutive messages from the same sender.
- **🛠️ Configurable:** Toggle timestamps, reply targets (ID, sender, or topic), and output formats (CSV, JSON, JSONL). Enabling replies keeps message IDs and disables merging so targets remain exact.

### Compression Results

| Format  | Input (Telegram) | Output      | Savings          |
| ------- | ---------------- | ----------- | ---------------- |
| **CSV** | 11.2M tokens     | 850K tokens | **92% (13x)** 🔥 |
| JSONL   | 11.2M tokens     | 1.0M tokens | 91% (11x)        |
| JSON    | 11.2M tokens     | 1.3M tokens | 88% (8x)         |

---

## Usage Guide

### 1. Export your chat

Detailed instructions are available inside the application, but here is a quick summary:

| Platform      | Instructions                                                                              |
| ------------- | ----------------------------------------------------------------------------------------- |
| **Telegram**  | Desktop App → Settings → Advanced → Export Telegram data → JSON (Machine-readable)        |
| **WhatsApp**  | Chat Menu (⋮) → More → Export chat → "Without Media" (`.txt`)                             |
| **Instagram** | Settings → Your activity → Download information → JSON → Messages only (`message_1.json`) |
| **Discord**   | Use [DiscordChatExporter](https://github.com/Tyrrrz/DiscordChatExporter) (JSON/TXT/CSV)   |

### 2. Process

1. Go to [chatpack.berektassuly.com](https://chatpack.berektassuly.com).
2. **Drag and drop** your export file (the app will auto-detect the source).
3. Select your desired output format (CSV is recommended for LLMs).
4. Toggle options like **Timestamps** or **Replies** if needed.
5. Click **Convert** and download the result.

---

## Development

This project uses **React** (Vite) for the UI and **Rust** (`wasm-pack`) for the core logic.

### Prerequisites

- **Node.js** v20+ & `pnpm`
- **Rust** (latest stable)
- **wasm-pack**: `curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh`

### Setup

```bash
# 1. Clone the repository
git clone https://github.com/Berektassuly/chatpack-web.git
cd chatpack-web

# 2. Install JS dependencies
pnpm install

# 3. Build the WASM module
pnpm run build:wasm

# 4. Start the development server
pnpm run dev
```

### Build for Production

```bash
pnpm run build
```

This command compiles the Rust code to optimized WASM and builds the React application into the `dist/` folder.

### Testing

```bash
# Run Frontend tests (Vitest)
pnpm test

# Run Rust/WASM tests
cargo test --manifest-path wasm/Cargo.toml
```

---

## Project Structure

```
chatpack-web/
├── .github/
│   └── workflows/
│       ├── ci.yml            # Lint, Format, Test (Frontend & Rust)
│       └── deploy.yml        # Build & Deploy to GitHub Pages
├── public/
│   └── CNAME                 # Custom domain configuration
├── src/
│   ├── components/           # React UI Components
│   │   ├── ConvertButton.tsx
│   │   ├── DropZone.tsx      # File upload area
│   │   ├── ExportGuide.tsx   # "How to export" modal
│   │   ├── FlagsSelector.tsx # Options (Timestamps, Replies)
│   │   ├── FormatDropdown.tsx
│   │   ├── FormatSelector.tsx
│   │   ├── ResultPreview.tsx # Output stats and preview
│   │   ├── SourceDropdown.tsx
│   │   ├── SourceSelector.tsx
│   │   ├── sourceTypes.ts    # Auto-detection logic
│   │   └── index.ts
│   ├── hooks/
│   │   └── useWasm.ts        # WASM loading, error handling, and invocation
│   ├── test/
│   │   ├── setup.ts          # Vitest setup
│   │   └── wasm_mock.ts      # Mock for WASM module in tests
│   ├── wasm/                 # Generated WASM output (gitignored)
│   ├── App.test.tsx
│   ├── App.tsx               # Main application state and layout
│   ├── index.css             # Global styles (Tailwind-like variables)
│   ├── main.tsx              # React entry point
│   └── wasm.d.ts             # TypeScript definitions for WASM module
├── wasm/                     # Rust Source Code
│   ├── src/
│   │   └── lib.rs            # WASM bindings to chatpack crate
│   ├── Cargo.toml            # Rust dependencies
│   └── rustfmt.toml
├── index.html
├── vite.config.ts            # Vite config (WASM plugin)
├── vitest.config.ts          # Test config
├── tailwind.config.js
└── package.json
```

### WASM Interface

The Rust backend exposes the following API to TypeScript via `src/hooks/useWasm.ts`:

```typescript
/**
 * Converts raw chat content into structured output.
 * @param input - The raw file content string
 * @param source - "telegram" | "whatsapp" | "instagram" | "discord"
 * @param format - "csv" | "json" | "jsonl"
 * @param includeTimestamps - Whether to include timestamps in output
 * @param includeReplies - Whether to include reply context ID
 */
export function convert(
  input: string,
  source: string,
  format: string,
  includeTimestamps: boolean,
  includeReplies: boolean,
): string
```

---

## Deployment

The project is automatically deployed to GitHub Pages via GitHub Actions on push to `main`.

**Custom Domain:**
The `public/CNAME` file ensures the site is served at [chatpack.berektassuly.com](https://chatpack.berektassuly.com).

---

## Related Repositories

| Repository                                                   | Description       |
| ------------------------------------------------------------ | ----------------- |
| [chatpack](https://github.com/Berektassuly/chatpack)         | Core Rust library |
| [chatpack-cli](https://github.com/Berektassuly/chatpack-cli) | Command-line tool |
| [chatpack-web](https://github.com/Berektassuly/chatpack-web) | This repository   |

---

## License

MIT License. See [LICENSE](LICENSE) for details.

© [Mukhammedali Berektassuly](https://berektassuly.com)
