return {
	-- Dashboard
	{
		"nvimdev/dashboard-nvim",
		event = "VimEnter",
		opts = {
			theme = "hyper",
			config = {
				week_header = {
					enable = true,
				},
				shortcut = {
					{
						desc = " Find Files",
						group = "Label", 
						action = "LazyVim telescope files",
						key = "f",
					},
					{
						desc = " Recent Files",
						group = "Number",
						action = "Telescope oldfiles",
						key = "r", 
					},
					{
						desc = " Find Text",
						group = "DiagnosticHint",
						action = "LazyVim telescope live_grep",
						key = "g",
					},
					{
						desc = " Restore Session", 
						group = "String",
						action = function()
							require("auto-session").RestoreSession()
						end,
						key = "s",
					},
					{
						desc = " Terminal",
						group = "Function", 
						action = "ToggleTerm direction=float",
						key = "t",
					},
					{
						desc = " Config",
						group = "Constant",
						action = function()
							vim.cmd("edit " .. vim.fn.stdpath("config"))
						end,
						key = "c",
					},
					{
						desc = "󰒲 Lazy",
						group = "Statement",
						action = "Lazy",
						key = "l",
					},
					{
						desc = " Git Status",
						group = "Special",
						action = "LazyGit", 
						key = "G",
					},
					{
						desc = " Quit",
						group = "Error",
						action = "qa",
						key = "q",
					},
				},
				footer = function()
					local stats = require("lazy").stats()
					local ms = (math.floor(stats.startuptime * 100 + 0.5) / 100)
					return {
						"⚡ Neovim loaded " .. stats.loaded .. "/" .. stats.count .. " plugins in " .. ms .. "ms",
						"🎨 " .. vim.g.colors_name .. " colorscheme",
						"📁 " .. vim.fn.getcwd(),
					}
				end,
			},
		},
	},
}

