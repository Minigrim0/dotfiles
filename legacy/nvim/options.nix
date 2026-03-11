# Options configuration ported from your LazyVim options.lua
{
  programs.nixvim.opts = {
    # From your options.lua
    relativenumber = true;
    number = true;
    expandtab = true;
    shiftwidth = 2;
    tabstop = 2;
    smartindent = true;
    wrap = false;
    
    # Enhanced diagnostic display for inline errors
    signcolumn = "yes";           # Always show sign column
    updatetime = 250;             # Faster diagnostic updates
    
    # Better completion display
    completeopt = "menu,menuone,noselect";
    
    # Code folding settings (prevent auto-folding on file open)
    foldmethod = "expr";
    foldexpr = "nvim_treesitter#foldexpr()";
    foldenable = false;        # Don't fold by default when opening files
    foldlevel = 99;            # Open all folds by default
    foldlevelstart = 99;       # Start with all folds open
    # Highlight current line
    cursorline = true;
  };
}
