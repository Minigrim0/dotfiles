return {
  -- Rust support
  {
    "mrcjkb/rustaceanvim",
    version = "^5", -- Recommended
    lazy = false,
    ft = { "rust" },
    opts = {
      -- Let rustaceanvim handle its own configuration
      -- It will work with system rust-analyzer from rustup
    },
  },
}