{
  description = "The More Aggressive Nix Eval Cache";

  inputs = {
    flakeCompat.url = github:edolstra/flake-compat;
    flakeCompat.flake = false;

    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
  };

  outputs = inputs: let
    commit = inputs.self.shortRev or "dirty";
    date = inputs.self.lastModifiedDate or inputs.self.lastModified or "19700101";
    version = "1.2.0+${builtins.substring 0 8 date}.${commit}";

    nixpkgsForHost = host:
      import inputs.nixpkgs {
        overlays = [
          (self: super: {
            nix-eval-cache = self.rustPlatform.buildRustPackage {
              pname = "nix-eval-cache";
              inherit version;
              src = ./.;
              cargoLock.lockFile = ./Cargo.lock;

              passthru.tests = {
                version = self.testVersion {package = super.nix-eval-cache;};
              };

              meta = {
                description = "The More Aggressive Nix Eval Cache.";
                homepage = "https://github.com/numtide/nix-eval-cache";
                license = self.lib.licenses.mit;
                maintainers = [self.lib.maintainers.mic92];
                platforms = self.lib.systems.doubles.all;
              };
            };
          })
        ];
        system = host;
      };

    # nixpkgs."aarch64-darwin" = nixpkgsForHost "aarch64-darwin";
    nixpkgs."aarch64-linux" = nixpkgsForHost "aarch64-linux";
    nixpkgs."i686-linux" = nixpkgsForHost "i686-linux";
    # nixpkgs."x86_64-darwin" = nixpkgsForHost "x86_64-darwin";
    nixpkgs."x86_64-linux" = nixpkgsForHost "x86_64-linux";

    buildBinariesForHost = host: pkgs: let
      binaries = builtins.listToAttrs (
        builtins.map (pkg: {
          name = "nix-eval-cache-${pkg.stdenv.targetPlatform.config}";
          value = pkg;
        })
        pkgs
      );
    in
      binaries
      // {
        "nix-eval-cache-binaries" = nixpkgs.${host}.linkFarm "nix-eval-cache-binaries" (
          nixpkgs.${host}.lib.mapAttrsToList
          (name: binary: {
            inherit name;
            path = "${binary}/bin/nix-eval-cache";
          })
          binaries
        );
        "default" = builtins.elemAt pkgs 0;
      };
  in rec {
    # checks."aarch64-darwin" = packages."aarch64-darwin";
    checks."aarch64-linux" = packages."aarch64-linux";
    checks."i686-linux" = packages."i686-linux";
    # checks."x86_64-darwin" = packages."x86_64-darwin";
    checks."x86_64-linux" = packages."x86_64-linux";

    # defaultPackage."aarch64-darwin" = packages."aarch64-darwin"."nix-eval-cache-aarch64-apple-darwin";
    defaultPackage."aarch64-linux" = packages."aarch64-linux"."nix-eval-cache-aarch64-unknown-linux-gnu";
    defaultPackage."i686-linux" = packages."i686-linux"."nix-eval-cache-i686-unknown-linux-gnu";
    # defaultPackage."x86_64-darwin" = packages."x86_64-darwin"."nix-eval-cache-x86_64-apple-darwin";
    defaultPackage."x86_64-linux" = packages."x86_64-linux"."nix-eval-cache-x86_64-unknown-linux-gnu";

    devShell."x86_64-linux" = with nixpkgs."x86_64-linux";
      mkShell {
        name = "nix-eval-cache";
        packages = [
          cargo
          cargo-bloat
          cargo-license
          cargo-tarpaulin
          cargo-watch
          rust-analyzer
          clippy
          alejandra
          nodejs
          nodePackages.prettier
          nodePackages.prettier-plugin-toml
          rustc
          shfmt
          treefmt
        ];
      };

    # packages."aarch64-darwin" = with nixpkgs."aarch64-darwin";
    #   buildBinariesForHost "aarch64-darwin" [
    #     nix-eval-cache
    #   ];
    packages."aarch64-linux" = with nixpkgs."aarch64-linux";
      buildBinariesForHost "aarch64-linux" [
        nix-eval-cache
        pkgsStatic.nix-eval-cache
      ];
    packages."i686-linux" = with nixpkgs."i686-linux";
      buildBinariesForHost "i686-linux" [
        nix-eval-cache
      ];
    # packages."x86_64-darwin" = with nixpkgs."x86_64-darwin";
    #   buildBinariesForHost "x86_64-darwin" [
    #     nix-eval-cache
    #   ];
    packages."x86_64-linux" = with nixpkgs."x86_64-linux"; (buildBinariesForHost "x86_64-linux" [
      nix-eval-cache
      pkgsStatic.nix-eval-cache

      pkgsCross.aarch64-multiplatform.pkgsStatic.nix-eval-cache

      pkgsCross.armv7l-hf-multiplatform.pkgsStatic.nix-eval-cache

      pkgsCross.gnu32.pkgsStatic.nix-eval-cache

      pkgsCross.raspberryPi.pkgsStatic.nix-eval-cache
    ]);
  };
}
