{ user, repo, ref ? "main", buildScript ? ":" }:

let
  pkgs = import <nixpkgs> { };
in

pkgs.vimUtils.buildVimPlugin {
  pname = "${pkgs.lib.strings.sanitizeDerivationName repo}";
  version = ref;
  src = builtins.fetchGit {
    url = "https://github.com/${user}/${repo}.git";
    inherit ref;
  };
  inherit buildScript;
}
