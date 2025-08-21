return {
  -- Configure LSP to use system packages instead of mason
  {
    "neovim/nvim-lspconfig",
    opts = function(_, opts)
      -- Disable mason for all servers
      for server, server_opts in pairs(opts.servers or {}) do
        if type(server_opts) == "table" then
          server_opts.mason = false
        end
      end
      return opts
    end,
  },
  
  -- Configure conform to use system formatters
  {
    "stevearc/conform.nvim",
    opts = function(_, opts)
      -- Disable mason installation for formatters
      opts.formatters_by_ft = opts.formatters_by_ft or {}
      
      -- Use system stylua for Lua
      if opts.formatters_by_ft.lua then
        opts.formatters_by_ft.lua = { "stylua" }
      end
      
      -- Disable mason for all formatters
      opts.formatters = opts.formatters or {}
      for name, config in pairs(opts.formatters) do
        if type(config) == "table" then
          config.mason = false
        end
      end
      
      return opts
    end,
  },
}