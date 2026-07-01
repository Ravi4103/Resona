# Resona

A modern, feature-rich offline music player built with Tauri v2, Vue 3, and Rust.

## Features

- **Glass Morphism UI**: Beautiful glass-styled interface with backdrop blur effects
- **WebGL Living Backgrounds**: Animated gradient backgrounds that adapt to album artwork colors
- **Fullscreen Player**: Immersive fullscreen mode with lyrics and queue panels
- **Floating Miniplayer**: Always-on-top miniplayer for background listening
- **Advanced Lyrics**: Support for synced (LRC) and plain lyrics with auto-scroll
- **Equalizer**: 10-band equalizer with preset profiles
- **Theme System**: Dynamic theming with light/dark/black modes
- **Internationalization**: Multi-language support (vue-i18n)
- **Discord RPC**: Show current track on Discord
- **Sleep Timer**: Automatic playback stop after set time
- **ReplayGain**: Volume normalization support
- **Remote Control**: Browser-based remote control server

## Prerequisites

- **Node.js** 18+ and **pnpm** (recommended) or npm
- **Rust** and **Cargo** (for Tauri backend)
- **System dependencies** for Tauri:
  - Windows: Visual Studio C++ Build Tools
  - macOS: Xcode Command Line Tools
  - Linux: webkit2gtk, libayatana-appindicator, librsvg2

## Installation

```bash
# Install dependencies
pnpm install
```

## Running the Application

### Development Mode

```bash
# Run the Tauri development server
pnpm tauri dev
```

This will:
1. Start the Vite dev server (frontend)
2. Build and run the Tauri backend
3. Open the application window

### Building for Production

```bash
# Build the application
pnpm tauri build
```

The built application will be in the `src-tauri/target/release/bundle/` directory.

## Available Scripts

```bash
# Development
pnpm dev          # Run Vite dev server only (frontend only)
pnpm tauri dev    # Run full Tauri development environment

# Building
pnpm build        # Build frontend only
pnpm tauri build  # Build full application

# Testing
pnpm test         # Run Vitest tests
pnpm test:ui      # Run Vitest with UI

# Linting
pnpm lint         # Run TypeScript type checking
```

## Project Structure

```
├── src/                    # Vue 3 frontend
│   ├── assets/            # CSS and static assets
│   ├── components/        # Vue components
│   │   ├── player/       # Player-specific components
│   │   └── ...
│   ├── locales/          # i18n translation files
│   ├── stores/           # Pinia state management
│   ├── views/            # Page views
│   └── ...
├── src-tauri/            # Tauri v2 Rust backend
│   ├── src/              # Rust source code
│   └── ...
├── JUCE/                 # Audio engine (JUCE framework)
└── ...
```

## Tech Stack

### Frontend
- **Vue 3** - Progressive JavaScript framework
- **Pinia** - State management
- **Vue Router** - Routing
- **Tailwind CSS v4** - Styling
- **Radix Vue** - UI components
- **vue-i18n** - Internationalization
- **OGL** - WebGL for animated backgrounds
- **Lucide Vue** - Icons

### Backend
- **Tauri v2** - Desktop framework
- **Rust** - Backend language
- **JUCE** - Audio processing engine

## Development Notes

### Glass Styling
The application uses a glass morphism design system with CSS variables:
- `--bg-glass` - Glass surface background
- `--bg-glass-elevated` - Elevated glass surfaces
- `--border-glass` - Glass borders
- `--dynamic-primary` - Dynamic primary color from artwork
- `--dynamic-surface` - Dynamic surface color

### Theme System
Themes are applied via CSS classes on the root element:
- `.light` - Light mode
- `.dark` - Dark mode (default)
- `.black` - Pure black for OLED displays

Dynamic colors transition smoothly (1.5s ease-in-out) when tracks change.

### Testing
Component tests use Vitest with @vue/test-utils:
```bash
pnpm test
```

## License

MIT
