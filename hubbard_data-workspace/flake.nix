{
  description = "A flake to build `hubbard_data` on nixOS";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, fenix, flake-utils, naersk, nixpkgs, ... }:
    flake-utils.lib.eachDefaultSystem (system: {
      packages.default =
        let
          pkgs = nixpkgs.legacyPackages.${system};
          target = "x86_64-unknown-linux-gnu";
          toolchain = with fenix.packages.${system}; combine [
            stable.cargo
            stable.rustc
            targets.${target}.stable.rust-std
          ];
        in
        (naersk.lib.${system}.override {
          cargo = toolchain;
          rustc = toolchain;
        }).buildPackage {
          src = ./.;
          CARGO_BUILD_TARGET = target;
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.fontconfig ];
          CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER =
            let
              inherit (pkgs.pkgsCross.gnu64.stdenv) cc;
            in
            "${cc}/bin/${cc.targetPrefix}cc";
          postInstall = ''
            patchelf --set-interpreter /usr/lib64/ld-linux-x86-64.so.2 $out/bin/hubbard_data
          '';
        };
    });
}
