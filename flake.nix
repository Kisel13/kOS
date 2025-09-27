{
  description = "Dev-shell flake";

  inputs = {
    nixpkgs-old.url = "github:nixos/nixpkgs?rev=029dea9aaacf920ce8f7d89e4cf09da31a38d8e1";
    nixpkgs.url = "github:nixos/nixpkgs";
  };

  outputs = { nixpkgs, nixpkgs-old, ... }@inputs: 
  let
    pkgs = nixpkgs.legacyPackages."x86_64-linux";
    pkgs-old = nixpkgs-old.legacyPackages."x86_64-linux";
    python = pkgs.python3;
  in {
    devShells.x86_64-linux.default = (
      let
        ovmf-arch = pkgs.stdenv.mkDerivation {
          pname = "ovmf-arch";
          version = "202508";

          src = pkgs.fetchurl {
            url = "https://archive.archlinux.org/packages/e/edk2-ovmf/edk2-ovmf-202508-1-any.pkg.tar.zst";
            sha256 = "0gigald65nyvyjs1jxm8mld3fpbf49llgccjlzakbglhpxks4zqx";
          };

          nativeBuildInputs = [ pkgs.zstd pkgs.xz ];

          unpackPhase = ''
            mkdir src
            cd src
            zstd -d < $src | tar -xv
          '';

          installPhase = ''
            mkdir -p $out
            cp -r ./usr/* $out/
          '';
        };
      in
      (pkgs.buildFHSEnv {
        name = "dev-env";
        targetPkgs = pkgs: (with pkgs; [
          binutils
          gcc
          gnumake
          gnugrep
          gnused
          diffutils
          python3
          nasm
          pkgs-old.pkgsCross.i686-embedded.buildPackages.gcc
          libisoburn
          qemu
          rustup
        ] ++ [ ovmf-arch ]);
        runScript = "fish";
      })
    ).env;
  };
}
