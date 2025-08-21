# Rust configuration ported from your rust.lua
{
  programs.nixvim.extraConfigLua = ''
    -- Configure rustaceanvim to use system rust-analyzer
    vim.g.rustaceanvim = {
      -- Use system rust-analyzer from rustup/nix
      tools = {
        -- Configuration for rust tools
      },
    }
  '';
}