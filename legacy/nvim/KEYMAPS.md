# NixVim Key Mappings Guide

## Essential Mappings (Most Useful)

### Leader Key
- **Leader**: `<Space>` (Space bar)

### File Operations
- `<leader>ff` - Find files (Telescope)
- `<leader>fg` - Live grep (search in files)
- `<leader>fr` - Recent files
- `<leader>fb` - Buffers
- `<leader>e` - Toggle Snacks file explorer (tree view)

### Git Integration
- `<leader>gg` - Open LazyGit
- Git signs show in gutter automatically

### Code Navigation & Editing
- `s{char}{char}` - Leap to any location (2 characters)
- `gcc` - Comment/uncomment current line
- `gbc` - Block comment
- `<A-v>` - Vertical split and move cursor right

### Window Management
- `<C-h/j/k/l>` - Navigate between windows
- `<C-arrows>` - Navigate between windows (arrow keys)
- `<A-h>` / `<A-l>` - Previous/Next buffer
- `<A-left>` / `<A-right>` - Previous/Next buffer (arrow keys)
- `<A-w>` - Delete current buffer

### Code Folding (nvim-ufo) - Manual Only
- `zc` - Close fold under cursor
- `zo` - Open fold under cursor
- `za` - Toggle fold under cursor
- `zM` - Close all folds
- `zR` - Open all folds

**Note**: Folding is now manual only - files open with all folds expanded by default.

## Rust-Specific Mappings

### Rustaceanvim Commands
- `:RustLsp hover` - Show hover information
- `:RustLsp codeAction` - Show code actions
- `:RustLsp flyCheck` - Run clippy check
- `:RustLsp explainError` - Explain error under cursor
- `:RustLsp openDocs` - Open docs.rs for symbol
- `:RustLsp runnables` - Show runnable commands
- `:RustLsp debuggables` - Show debuggable targets

### Crates.nvim (Cargo.toml)
When in Cargo.toml files:
- Hover over crate versions for information
- Auto-completion for crate names and versions
- LSP integration for crate management

## Terminal
- `<A-t>` - Toggle floating terminal
- `<Esc>` - Exit terminal mode (when in terminal)

## Text Manipulation

### Vim-Surround
- `cs"'` - Change surrounding quotes from " to '
- `cs"(` - Change surrounding quotes to parentheses
- `ds"` - Delete surrounding quotes
- `ysiw"` - Surround inner word with quotes
- `yss"` - Surround entire line with quotes

### Auto-pairs
- Automatically closes brackets, quotes, etc.
- `<CR>` in brackets creates proper indentation

### Indentation
- Visual indentation guides shown automatically
- `>` / `<` - Indent/unindent in visual mode
- `>>` / `<<` - Indent/unindent current line

## Search & Replace
- `/` - Search forward
- `?` - Search backward
- `n` / `N` - Next/previous search result
- `*` - Search for word under cursor
- `:%s/old/new/g` - Replace all occurrences

## LSP Features & Error Display
- `gd` - Go to definition
- `gr` - Go to references
- `gi` - Go to implementation
- `K` - Show hover information
- `<leader>ca` - Code actions
- `<leader>rn` - Rename symbol
- `]d` / `[d` - Next/previous diagnostic
- `<leader>d` - Open diagnostic float (changed from `<leader>e`)
- `<leader>q` - Add diagnostics to location list

**Inline Errors**: Errors and warnings are automatically shown:
- As virtual text inline with your code
- In the sign column (left gutter)
- Underlined in the code
- Floating window on cursor hold

## Telescope Advanced
- `<leader>fh` - Help tags
- In Telescope:
  - `<C-u>` - Scroll up preview
  - `<C-d>` - Scroll down preview
  - `<C-x>` - Open in horizontal split
  - `<C-v>` - Open in vertical split

## Completion (nvim-cmp)
- `<Tab>` - Accept completion or trigger
- `<C-n>` / `<C-p>` - Next/previous completion
- `<C-Space>` - Trigger completion manually
- `<CR>` - Confirm selection

## Session Management
- Sessions auto-saved and restored
- Suppressed in home directory and common folders

## Dashboard
- Shown on startup with recent files and projects
- Week header enabled

## Notes

### Tree Climber Rust (Planned)
The `tree_climber_rust.nvim` plugin is planned for addition but requires proper hash configuration. When added, it will provide:
- `<C-n>` / `<C-p>` - Navigate Rust AST nodes
- Incremental selection in visual mode

### Dependencies
All tools are managed by Nix:
- `rust-analyzer` (via rustup)
- `lua-language-server`
- `stylua` (Lua formatter)
- `ripgrep` & `fd` (for Telescope)
- `lazygit` (Git TUI)

### Configuration Location
- Main config: `/home/minigrim0/.config/home-manager/nvim/`
- Apply changes: `hms` (home-manager switch)