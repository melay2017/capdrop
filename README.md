# CapDrop (截落)

Lightweight screenshot tool built with Tauri 2 + Rust.

## Features (MVP)

- Global hotkey screenshot capture
- Smart region selection
- Lightweight annotation (arrows, text, rectangles)
- Auto-save (clipboard / local file / Markdown embed)

## Tech Stack

- **Runtime:** Tauri 2 (Rust + WebView2)
- **Frontend:** Vanilla JS / TypeScript
- **Build:** Single executable, 3-10MB, no runtime dependency

## Development

```bash
# Prerequisites: Rust, Node.js 18+, pnpm
cd src-tauri
cargo tauri dev
```

## License

MIT
