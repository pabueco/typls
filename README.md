# typls (typeless)

typls is a cross-platform app that automatically expands user defined abbreviations into any text.

The app is built with Tauri (v2 alpha), Rust, Nuxt (Typescript) and Tailwind.

## Features

- Cross-platform: Works on MacOS, Windows and Linux (X11)
- Customizable: Adjust many settings to make it work for you
- Nice and easy to use UI
- Variables: Pass values to expanded text via placeholders (`{}` or `{name}`)

## Usage

You define the word `hi` should expand to `Hi there, how are you?`. The trigger character is `'`.

So you type `'hi` and confirm it with a space, which would cause the typed sequence to be replaced with `Hi there, how are you?`.

### Variables

The customize expansions, you can either use anonymous (`{}`) or named (`{name}`) variables.
To pass values to the expansions to append them to your abbreviation with a `|` (pipe).

Like this: `'hi|bar`. To pass multiple values just separate them with a `|` again: `'hi|bar|fizz`.

**Anonymous** variables are replaced one by one:

1. `Hi {}, it was really nice to {} you?`
2. `'hi|Peter|meet`
3. `Hi Peter, it was really nice to meet you?`

**Named** variables are replaced globally: 

1. `Hi {name}, I like the name {name}! It was nice to {kind} you.`
2. `'hi|name=Peter|kind=meet`
3. `Hi Peter, I like the name Peter! It was nice to meet you.`

## Installation

Download the file for your platform from the [latest release](https://github.com/pabueco/typls/releases/latest) and install it.

> [!NOTE]
> On MacOS the app needs the `Input Monitoring` and `Accessibility` permissions. This is needed, because it needs to read your key strokes and simulate key presses to expand the text.

## Roadmap & Ideas

- [ ] Default/fallback values for variables
- [ ] Enable/disable expansions for different applications (?)

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
