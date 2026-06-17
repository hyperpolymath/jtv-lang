<!--
SPDX-License-Identifier: CC-BY-SA-4.0
Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
-->

# JtV Language Server Protocol

The JtV LSP provides IDE integration for editors supporting the Language Server Protocol.

## Features

### Core Features

| Feature | Status | Description |
|---------|--------|-------------|
| Syntax Highlighting | ✓ | Semantic tokens for JtV syntax |
| Diagnostics | ✓ | Real-time error reporting |
| Hover Information | ✓ | Type info and documentation |
| Go to Definition | ✓ | Navigate to symbol definitions |
| Find References | ✓ | Find all symbol usages |
| Completion | ✓ | Context-aware auto-completion |
| Signature Help | ✓ | Function parameter hints |
| Rename | ✓ | Safe symbol renaming |
| Code Actions | ✓ | Quick fixes and refactorings |
| Formatting | ✓ | Document and range formatting |

### Advanced Features

| Feature | Status | Description |
|---------|--------|-------------|
| Inlay Hints | ✓ | Type annotations, parameter names |
| Code Lens | ✓ | Run/debug/test actions |
| Folding | ✓ | Code folding regions |
| Selection Range | ✓ | Smart selection expansion |
| Call Hierarchy | Planned | Incoming/outgoing calls |
| Type Hierarchy | Planned | Type inheritance tree |

## Installation

### VS Code

Install the "Julia the Viper" extension from the marketplace:

```bash
code --install-extension hyperpolymath.jtv-vscode
```

### Neovim (nvim-lspconfig)

```lua
require('lspconfig').jtv.setup{
  cmd = { "jtv", "lsp" },
  filetypes = { "jtv" },
  root_dir = require('lspconfig').util.root_pattern('jtv.toml', '.git'),
}
```

### Vim (vim-lsp)

```vim
if executable('jtv')
  au User lsp_setup call lsp#register_server({
    \ 'name': 'jtv',
    \ 'cmd': {server_info->['jtv', 'lsp']},
    \ 'allowlist': ['jtv'],
    \ })
endif
```

### Emacs (lsp-mode)

```elisp
(require 'lsp-mode)

(add-to-list 'lsp-language-id-configuration '(jtv-mode . "jtv"))

(lsp-register-client
 (make-lsp-client
  :new-connection (lsp-stdio-connection '("jtv" "lsp"))
  :major-modes '(jtv-mode)
  :server-id 'jtv-ls))
```

### Sublime Text (LSP)

In `LSP.sublime-settings`:

```json
{
  "clients": {
    "jtv": {
      "command": ["jtv", "lsp"],
      "selector": "source.jtv"
    }
  }
}
```

## Configuration

### Server Settings

Settings can be configured via the LSP initialization options or `jtv.toml`:

```toml
[lsp]
# Diagnostics
diagnostics.enable = true
diagnostics.purity = true
diagnostics.types = true

# Completion
completion.snippets = true
completion.auto_import = true

# Inlay hints
inlay_hints.type_hints = true
inlay_hints.parameter_names = true

# Formatting
format.on_save = true
format.indent = 4

# Analysis
analysis.strict = false
analysis.max_errors = 100
```

### Per-Editor Configuration

#### VS Code (settings.json)

```json
{
  "jtv.diagnostics.enable": true,
  "jtv.inlayHints.typeHints": true,
  "jtv.format.onSave": true
}
```

## Diagnostics

### Error Types

| Code | Category | Description |
|------|----------|-------------|
| E001 | Parse | Syntax error |
| E002 | Parse | Unexpected token |
| E101 | Type | Type mismatch |
| E102 | Type | Unknown type |
| E103 | Type | Coercion failure |
| E201 | Purity | @pure function contains I/O |
| E202 | Purity | @total function contains loop |
| E203 | Purity | Impure call in Data context |
| E301 | Name | Undefined variable |
| E302 | Name | Duplicate definition |

### Warning Types

| Code | Category | Description |
|------|----------|-------------|
| W001 | Style | Unused variable |
| W002 | Style | Shadowed variable |
| W003 | Style | Unreachable code |
| W101 | Performance | Redundant operation |

## Code Actions

### Quick Fixes

- **Add type annotation**: Insert inferred type
- **Import symbol**: Add missing import
- **Remove unused**: Delete unused variable
- **Fix purity**: Suggest @pure/@total removal

### Refactorings

- **Extract variable**: Extract expression to variable
- **Extract function**: Extract code to function
- **Inline variable**: Inline variable usage
- **Rename symbol**: Rename across project

## Semantic Tokens

The LSP provides semantic tokens for enhanced highlighting:

| Token Type | Modifiers | Example |
|------------|-----------|---------|
| `variable` | `declaration`, `readonly` | `x = 5` |
| `function` | `declaration`, `pure`, `total` | `@pure fn foo` |
| `parameter` | - | `fn f(x: Int)` |
| `type` | - | `Int`, `Float` |
| `number` | `integer`, `float`, `rational`, `complex` | `42`, `3.14`, `1/2`, `3+4i` |
| `keyword` | - | `if`, `while`, `for` |
| `operator` | - | `+`, `=` |
| `comment` | - | `// comment` |

## Commands

The LSP supports these workspace commands:

| Command | Description |
|---------|-------------|
| `jtv.run` | Run current file |
| `jtv.check` | Check current file |
| `jtv.restart` | Restart LSP server |
| `jtv.showType` | Show type at cursor |
| `jtv.showPurity` | Show purity at cursor |

## Troubleshooting

### Server Won't Start

1. Verify `jtv` is in PATH: `which jtv`
2. Check JtV version: `jtv --version`
3. Try manual start: `jtv lsp --log /tmp/jtv-lsp.log`

### Diagnostics Not Showing

1. Check file is saved (LSP may require save)
2. Verify file extension is `.jtv`
3. Check `diagnostics.enable` setting

### High CPU Usage

1. Limit analysis scope in `jtv.toml`
2. Exclude large generated files
3. Set `analysis.max_errors`

### Logging

Enable detailed logging:

```bash
jtv lsp --log /tmp/jtv-lsp.log
```

Or set environment variable:

```bash
JTV_LOG=debug jtv lsp
```

## See Also

- [CLI](./CLI.md)
- [Formatter](./Formatter.md)
- [Editor Setup](../tutorials/Editor-Setup.md)
