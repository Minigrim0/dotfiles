{
  description = "Home Manager configuration with NixVim";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    home-manager = {
      url = "github:nix-community/home-manager/release-25.05";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixvim = {
      url = "github:nix-community/nixvim";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, home-manager, nixvim, ... }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      homeConfigurations."minigrim0" = home-manager.lib.homeManagerConfiguration {
        inherit pkgs;
        
        modules = [
          # Import the nixvim home-manager module
          nixvim.homeManagerModules.nixvim
          
          # Your existing home configuration
          ./home.nix
        ];
      };
    };
}