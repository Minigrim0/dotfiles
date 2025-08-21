-- Keymaps are automatically loaded on the VeryLazy event
-- Default keymaps that are always set: https://github.com/LazyVim/LazyVim/blob/main/lua/lazyvim/config/keymaps.lua
-- Add any additional keymaps here

vim.keymap.set("n", "<A-left>", "<cmd>bprevious<cr>", {desc = "Prev Buffer"})
vim.keymap.set("n", "<A-right>", "<cmd>bnext<cr>", {desc = "Next Buffer"})


vim.keymap.set("n", "<A-h>", "<cmd>bprevious<cr>", {desc = "Prev Buffer"})
vim.keymap.set("n", "<A-l>", "<cmd>bnext<cr>", {desc = "Next Buffer"})

vim.keymap.set("n", "<A-w>", function()
  require("snacks").bufdelete()
end, {desc = "Close Buffer"})

vim.keymap.set("n", "<A-t>", "<cmd>ToggleTerm direction=float<cr>", { desc = "Toggle Floating Terminal"})
vim.keymap.set("t", "<Esc>", [[<C-\><C-N>]], { desc = "Unfocus terminal" })
vim.keymap.set("t", "<A-t>", "<cmd>ToggleTerm direction=float<cr>", { desc = "Toggle terminal from terminal mode"})
