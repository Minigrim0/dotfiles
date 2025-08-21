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
    
    # Buffer deletion
    {
      mode = "n";
      key = "<A-w>";
      action = "<cmd>bd<cr>";
      options.desc = "Close Buffer";
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