# typls (typeless)

typls is a cross-platform app that automatically expands user defined abbreviations into any text.

Example: type `'hi`, get `Hi there, it was really nice to meet you!`

The app is built with Tauri (v2), Rust, Nuxt (Typescript) and Tailwind.

## Features

- Cross-platform: Works on MacOS, Windows and Linux (X11)
- Customizable: Adjust many settings to make it work for you
- Variables: Pass values to expanded text via placeholders (`{}` or `{name}`) and define default values to make them optional (`{=bar}` or `{foo=bar}`)
- Groups: Scope abbreviations to applications and automatically use the one matching the active/focused application
- Works in any app: Because typls listens directly to and simulates keyboard input, it can expand text in any application you interact with: Websites, native apps, terminals to remote servers, etc.
- Clean and easy to use interface

## Usage

You define the word `hi` should expand to `Hi there, how are you?`. The trigger character is `'`.

So you type `'hi` and confirm it with a space, which would cause the typed sequence to be replaced with `Hi there, how are you?`.

You can also enable the option to auto expand abbreviations as soon as one match is found, when they don't contain any variables.

### Variables

The customize expansions, you can either use anonymous (`{}`) or named (`{name}`) variables.
To pass values to the expansions to append them to your abbreviation with a `|` (pipe).

Like this: `'hi|bar`. To pass multiple values just separate them with a `|` again: `'hi|bar|fizz`.
Named variables are quite similar: `'hi|foo=bar` and `'hi|foo=bar|bazz=fizz`.

**Anonymous** variables are replaced one by one:

1. `Hi {}, it was really nice to {} you?`
2. `'hi|Peter|meet`
3. `Hi Peter, it was really nice to meet you?`

**Named** variables are replaced globally:

1. `Hi {name}, I like the name {name}! It was nice to {kind} you.`
2. `'hi|name=Peter|kind=meet`
3. `Hi Peter, I like the name Peter! It was nice to meet you.`

#### Default values

Sometimes you want to be able to customize the expansion text, but not have to pass a value every time you use it. That's where you can use default values for variables.

To assign a default value simply add it after the variable name separated by an equal sign: `Hi {name}` -> `Hi {name=there}`. This also works for unnamed variables: `Hi {}` -> `Hi {=there}`.

Let's take a example from before and add some default values:

`Hi {=there}, it was really nice to {=meet} you.`

Now, if you just type `'hi`, you get `Hi there, it was really nice to meet you.`.

But you can also pass values to customize some (or all variables): `'hi|Peter` -> `Hi Peter, it was really nice to meet you.`.

### Groups

Groups can be used to define multiple expansions with the same abbreviation. Only expansions in the active group (or without any group) are considered when expanding an abbreviation.

By default the active group is determined automatically based on the currently focused application. You can assign apps to groups to only make expansions in these groups available within the matching applications.

You can also set the active group manually (and permanently) via the menu in the top right of the app.

## Installation

Download the file for your platform from the [latest release](https://github.com/pabueco/typls/releases/latest) and install it.

> [!NOTE]
> Currently the application is not signed, so the operating systems don't trust it. You might have to jump through some hoops to actually use it.
>
> On MacOS, after attempting opening the installer for the first time you'll get a popup that the app cannot be opened because it isn't trusted. To continue, simply go to the "Privacy & Security" settings scroll down past the permission until you see "typls" mentioned and click "Open anyway".
>
> Alternatively you can run `xattr -dr com.apple.quarantine /Applications/typls.app/` in your terminal to do the same thing automatically and allow the app to be opened.

> [!IMPORTANT]
> MacOS Permissions: On MacOS the app needs the `Input Monitoring` and `Accessibility` permissions. This is needed, because it needs to read your key strokes and simulate key presses to expand the text.

## Roadmap & Ideas

- [x] Default/fallback values for variables (-> `{variable=default})
- [x] Enable/disable expansions for different applications (-> Groups)

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
