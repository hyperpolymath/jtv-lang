# Julia the Viper VSCode Extension

Language support for Julia the Viper - a reversible systems programming language with purity guarantees.

## Features

- Syntax highlighting for `.jtv` files
- Language Server Protocol (LSP) integration with `jtv-lsp`
- Commands for running, debugging, and formatting
- IntelliSense support
- Error diagnostics with type checking and purity analysis
- Code completion
- Hover information

## Requirements

- Julia the Viper toolchain installed (`jtv-cli`, `jtv-lsp`, `jtv-debug`)
- VSCode 1.80.0 or higher

## Extension Settings

This extension contributes the following settings:

* `jtv.lsp.path`: Path to jtv-lsp executable (default: "jtv-lsp")
* `jtv.trace.server`: Trace LSP communication (off/messages/verbose)

## Commands

* `JtV: Run File` - Run the current `.jtv` file
* `JtV: Debug File` - Launch interactive debugger
* `JtV: Format File` - Format the current file

## Language Features

Julia the Viper provides:
- **Reversibility**: All computations are reversible
- **Purity tracking**: `@total` and `@pure` annotations with verification
- **Formal verification**: Hindley-Milner type system with extensions
- **WASM backend**: Compile to WebAssembly

## Installation

1. Install Julia the Viper toolchain
2. Install this extension from VSCode marketplace or `.vsix`
3. Open a `.jtv` file to activate the extension

## Building from Source

```bash
cd vscode-extension
npm install
npm run compile
npm run package
code --install-extension julia-the-viper-0.1.0.vsix
```

## License

MPL-2.0

## Author

Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
