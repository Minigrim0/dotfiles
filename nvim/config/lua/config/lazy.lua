return {
  defaults = {
    lazy = false,
    version = false,
  },
  install = { colorscheme = { "gruvbox-material" } },
  checker = { enabled = true },
  performance = {
    rtp = {
      disabled_plugins = {
        "gzip",
        "tarPlugin",
        "tohtml",
        "tutor",
        "zipPlugin",
      },
    },
  },
}