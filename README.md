# nvim-cargo-add

A Neovim plugin for managing Cargo dependencies with auto-completion and floating window preview. This plugin provides a seamless interface for adding, removing, and managing Rust dependencies directly from Neovim.

## Features

- Add dependencies with auto-completion from crates.io
- Add development dependencies
- Remove dependencies with auto-completion from current project
- List current dependencies in a floating window
- Auto-format Cargo.toml after changes
- Floating window preview for crate information
- Tab completion for crate names and versions

## Prerequisites

- Neovim >= 0.9.0
- Rust toolchain
- Cargo
- [lazy.nvim](https://github.com/folke/lazy.nvim)

## Installation

Using [lazy.nvim](https://github.com/folke/lazy.nvim):

```lua
{
    "nwiizo/nvim-cargo-add",
    dependencies = { "saecki/crates.nvim" },
    build = function()
        vim.notify("Building nvim-cargo-add...", vim.log.levels.INFO)
        local result = vim.fn.system("cargo build --release")
        vim.notify("Build result: " .. result, vim.log.levels.INFO)
    end,
    config = function()
        require("nvim_cargo_add").setup({
            auto_format = true,    -- Enable automatic formatting
            float_preview = true,  -- Enable floating window preview
        })
    end,
    ft = { "toml" },  -- Load when editing Cargo.toml
    cmd = {
        "CargoAdd",
        "CargoAddDev",
        "CargoRemove",
        "CargoDeps"
    }
}
```

## Usage

### Commands

- `:CargoAdd <crate_name> [version]` - Add a new dependency
  - Example: `:CargoAdd serde 1.0`
  - Use Tab for auto-completion and preview

- `:CargoAddDev <crate_name> [version]` - Add a new development dependency
  - Example: `:CargoAddDev pretty_assertions`
  - Use Tab for auto-completion and preview

- `:CargoRemove <crate_name>` - Remove a dependency
  - Example: `:CargoRemove serde`
  - Auto-completes with current project dependencies

- `:CargoDeps` - List all current dependencies in a floating window

### Configuration

You can configure the plugin behavior when calling setup:

```lua
require('nvim_cargo_add').setup({
    auto_format = true,    -- Automatically format Cargo.toml after changes
    float_preview = true   -- Show floating window with crate details during completion
})
```

## Auto-completion Features

- Crate name completion from crates.io
- Shows crate versions in completion menu
- Displays download counts and descriptions
- Real-time search as you type
- Floating window preview with detailed information

## Development

For local development:

```lua
{
    "nwiizo/nvim-cargo-add",
    dependencies = { "saecki/crates.nvim" },
    dir = vim.fn.expand("~/path/to/local/nvim-cargo-add"),
    build = function()
        local plugin_dir = vim.fn.expand("~/path/to/local/nvim-cargo-add")
        vim.fn.system(string.format(
            "cd %s && cargo build --release",
            vim.fn.shellescape(plugin_dir)
        ))
    end,
    config = function()
        require("nvim_cargo_add").setup()
    end,
}
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [crates.io](https://crates.io) for providing the package registry API
- [lazy.nvim](https://github.com/folke/lazy.nvim) for the plugin manager
- The Neovim community for inspiration and support

## TODO

- [ ] Add support for features selection
- [ ] Implement workspace dependencies management
- [ ] Add support for custom registries
- [ ] Implement dependency graph visualization
- [ ] Add support for updating dependencies

## Author

nwiizo
