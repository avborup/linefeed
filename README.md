# Linefeed

A language specifically tailored for writing clean solutions for Advent of Code.

> [!WARNING]  
> It is currently in a _**very**_ early, exploratory stage.

## Why the name?

The idea of creating Linefeed was born when I did Advent of Code 2024, where I wanted a language specifically tailored for writing clean solutions for Advent of Code:

1. It's a reference to having to read and print lines of text, each separated by a [line feed](https://en.wikipedia.org/wiki/Newline).
2. It's a nod to the Christmassy origin of Advent of Code. The common acronym for line feed is **LF**, which sounds like **elf**.

## Examples

To demonstrate the syntax and features of Linefeed, solutions for Advent of Code 2020 have been implemented in Linefeed. They can be found in [`tests/linefeed/advent_of_code_2020`](https://github.com/avborup/linefeed/tree/main/linefeed/tests/linefeed/advent_of_code_2020).

I've also included a syntax-highlighted sample snippet in the bottom of this README.

## Language Server

Linefeed includes an LSP server that provides semantic token highlighting.

### Installation

From the root of the repository:

```bash
cargo install --path linefeed-lsp
```

### Configuration

Example configuration for Neovim (using `astrolsp`):

```lua
linefeed_lsp = {
  cmd = { "linefeed-lsp" },
  filetypes = { "linefeed" },
  root_dir = require("lspconfig.util").root_pattern "*.lf",
  on_attach = function(client, bufnr)
    -- Enable semantic token highlighting if server supports it
    if client.server_capabilities.semanticTokensProvider then
      vim.lsp.semantic_tokens.start(bufnr, client.id)
    end
  end,
}
```

## Planned features

See https://github.com/avborup/linefeed/issues/1 for a tracking issue.

## Profiling

Linefeed includes a built-in VM profiler to analyze runtime performance. Enable it by compiling with the `profile-vm` feature:

```bash
cargo run --bin linefeed --features profile-vm -- your_program.lf
```

This prints a summary to stderr showing:
- Instruction frequency and timing
- Source code hotspots
- Function call statistics

To export full (non-truncated) profiler data to a file, set the `LINEFEED_PROFILE_OUTPUT` environment variable:

```bash
LINEFEED_PROFILE_OUTPUT=profile.txt cargo run --bin linefeed --features profile-vm -- your_program.lf
```

## Sample snippet

For completeness sake, here's a sample Linefeed snippet with syntax highlight (from the language server):

<p align="center">
  <img width="595" height="719" alt="image" src="https://github.com/user-attachments/assets/7c70fd94-33c8-4384-8aa2-50ccf8c79b9f" />
</p>
