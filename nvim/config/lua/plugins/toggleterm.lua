return {
	{
		"akinsho/toggleterm.nvim",
		opts = {
			hidden = true,
			start_in_insert = true,
			insert_mappings = true,
			terminal_mappings = true,
			on_open = function(term)
				vim.cmd("startinsert!")
			end,
		},
	},
}
