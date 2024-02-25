{
  description = "lz, a web-based tagged bookmark manager";

  outputs = inputs @ {
    self,
    flake-parts,
    nixpkgs,
    fenix,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];

      imports = [
        inputs.devshell.flakeModule
        inputs.flake-parts.flakeModules.easyOverlay
      ];

      perSystem = {
        config,
        pkgs,
        final,
        system,
        ...
      }: let
        cIncludes = [pkgs.sqlite];
        cLibs =
          if pkgs.stdenv.isDarwin
          then [pkgs.libiconv]
          else [];
      in {
        formatter = pkgs.alejandra;

        packages.default = config.packages.disk-spinner;
        packages.lz-web = let
          rustPlatform = pkgs.makeRustPlatform {
            inherit (fenix.packages.${system}.stable) rustc cargo;
          };
          nativeBuildInputs =
            (builtins.map (l: pkgs.lib.getDev l) cIncludes)
            ++ cIncludes
            ++ cLibs
            ++ [pkgs.pkg-config];
        in
          rustPlatform.buildRustPackage {
            pname = "lz-web";
            version = (builtins.fromTOML (builtins.readFile ./src/lz-web/Cargo.toml)).package.version;
            inherit nativeBuildInputs;
            buildInputs = nativeBuildInputs;
            src = let
              fs = pkgs.lib.fileset;
            in
              fs.toSource {
                root = ./.;
                fileset = fs.unions [
                  ./Cargo.toml
                  ./Cargo.lock
                  ./src
                ];
              };
            cargoLock.lockFile = ./Cargo.lock;
            meta.mainProgram = "lz-web";
          };

        apps = {
          default = config.apps.lz-web;
          lz-web.program = config.packages.lz-web;
        };

        devshells = {
          default = {
            imports = [
              "${inputs.devshell}/extra/language/rust.nix"
              "${inputs.devshell}/extra/language/c.nix"
            ];
            packages = [
              fenix.packages.${system}.stable.rust-analyzer
              pkgs.sqlx-cli
              pkgs.sqlite
            ];
            language.rust = {
              enableDefaultToolchain = false;
              packageSet = fenix.packages.${system}.stable;
              tools = ["rust-analyzer" "cargo" "clippy" "rustfmt" "rustc"];
            };
            env = [
              {
                name = "RUST_LOG";
                value = "info";
              }
            ];

            language.c.includes = cIncludes;
            language.c.libraries = cLibs;
          };
        };
      };
    };

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    devshell.url = "github:numtide/devshell";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
}
