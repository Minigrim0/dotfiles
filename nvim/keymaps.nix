# Keymaps configuration ported from your LazyVim setup
{
  programs.nixvim.keymaps = [
    # Buffer navigation
    {
      mode = "n";
      key = "<A-left>";
      action = "<cmd>bprevious<cr>";
      options.desc = "Prev Buffer";
    }
    {
      mode = "n";
      key = "<A-right>";
      action = "<cmd>bnext<cr>";
      options.desc = "Next Buffer";
    }
    {
      mode = "n";
      key = "<A-h>";
      action = "<cmd>bprevious<cr>";
      options.desc = "Prev Buffer";
    }
    {
      mode = "n";
      key = "<A-l>";
      action = "<cmd>bnext<cr>";
      options.desc = "Next Buffer";
    }
    
    # Buffer deletion (snacks)
    {
      mode = "n";
      key = "<A-w>";
      action.__raw = "function() Snacks.bufdelete() end";
      options.desc = "Delete Buffer";
    }
    
    # File explorer toggle (snacks)
    {
      mode = "n";
      key = "<leader>e";
      action.__raw = "function() Snacks.explorer() end";
      options.desc = "Explorer (snacks)";
    }
    
    # Window navigation with Ctrl + hjkl
    {
      mode = "n";
      key = "<C-h>";
      action = "<C-w>h";
      options.desc = "Go to left window";
    }
    {
      mode = "n";
      key = "<C-j>";
      action = "<C-w>j";
      options.desc = "Go to lower window";
    }
    {
      mode = "n";
      key = "<C-k>";
      action = "<C-w>k";
      options.desc = "Go to upper window";
    }
    {
      mode = "n";
      key = "<C-l>";
      action = "<C-w>l";
      options.desc = "Go to right window";
    }

    # Window navigation with Ctrl + arrow keys
    {
      mode = "n";
      key = "<C-Left>";
      action = "<C-w>h";
      options.desc = "Go to left window";
    }
    {
      mode = "n";
      key = "<C-Down>";
      action = "<C-w>j";
      options.desc = "Go to lower window";
    }
    {
      mode = "n";
      key = "<C-Up>";
      action = "<C-w>k";
      options.desc = "Go to upper window";
    }
    {
      mode = "n";
      key = "<C-Right>";
      action = "<C-w>l";
      options.desc = "Go to right window";
    }

    # Window split with Alt-v (vertical split and move current buffer to right)
    {
      mode = "n";
      key = "<A-v>";
      action = "<cmd>vsplit<cr><C-w>l";
      options.desc = "Vertical split and move right";
    }

    # Terminal toggle (from your toggleterm config)
    {
      mode = "n";
      key = "<A-t>";
      action = "<cmd>ToggleTerm direction=float<cr>";
      options.desc = "Toggle Floating Terminal";
    }
    {
      mode = "t";
      key = "<Esc>";
      action = ''<C-\><C-N>'';
      options.desc = "Unfocus terminal";
    }
    {
      mode = "t";
      key = "<A-t>";
      action = "<cmd>ToggleTerm direction=float<cr>";
      options.desc = "Toggle terminal from terminal mode";
    }
  ];
}