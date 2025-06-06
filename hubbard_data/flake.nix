{
  description = "A flake to build `hubbard_data` on nixOS";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };
  outputs = inputs@{ self, nixpkgs, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in
    {
      devShells.${system} = {
        default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            pkg-config
            fish
            stdenv
          ];
          buildInputs = with pkgs; [
            fontconfig
          ];
          shellHook = ''
            exec fish
          '';
        };
      };
    };
}
