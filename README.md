# typls (typeless)

typls is a cross-platform app that automatically expands user defined abbreviations into any text.

The app is built with Tauri (v2 alpha), Rust, Nuxt (Typescript) and Tailwind.

## Features

- Cross-platform: Works on MacOS, Windows and Linux (X11)
- Customizable: Adjust many settings to make it work for you
- Nice UI: Modern and easy to use
- Variables: Pass values to expanded text via placeholders (`{}` or `{name}`)
- (Coming soon) Different expansions for different applications

## Installation

Download the file for your platform from [releases](https://github.com/pabueco/typls/releases) and install it.

> [!NOTE]
> On MacOS the app needs the `Input Monitoring` and `Accessibility` permissions. This is needed, because it needs to read your key strokes and simulate key presses to expand the text.

## Development

```sh
# Install dependencies
pnpm install

# Run development server
pnpm tauri dev
```

## Building

```sh
pnpm tauri build
```
