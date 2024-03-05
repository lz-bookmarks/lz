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
        inputs.flake-root.flakeModule
        inputs.proc-flake.flakeModule
        inputs.pre-commit-hooks-nix.flakeModule
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
          then [pkgs.libiconv pkgs.darwin.apple_sdk.frameworks.CoreFoundation]
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
            commands = [
              {
                help = "Run all servers for development";
                name = "dev-server";
                package = config.proc.groups.dev-server.package;
              }
              {
                help = "Set up the pre-commit hook for this repo";
                name = "setup-pre-commit";
                command = config.pre-commit.installationScript;
              }
            ];
            packages = [
              pkgs.sqlx-cli
              pkgs.sqlite
              pkgs.cargo-watch
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
              {
                name = "DATABASE_URL";
                eval = "sqlite:$PRJ_ROOT/dev-db.sqlite";
              }
            ];

            language.c.includes = cIncludes;
            language.c.libraries = cLibs;
          };
        };

        proc.groups.dev-server.processes = {
          backend.command = ''
            ${pkgs.cargo-watch}/bin/cargo-watch -i *.nix -- cargo run --color always --bin lz-web -- --db dev-db.sqlite --authentication-header-name X-Tailscale-User-LoginName --default-user-name=developer --listen-on=127.0.0.1:3000
          '';
          sqlite-web = {
            command = "${pkgs.sqlite-web}/bin/sqlite_web -x -r dev-db.sqlite";
          };
        };

        pre-commit.settings = {
          hooks = {
            alejandra.enable = true;
            rustfmt.enable = true;
          };
        };
      };
    };

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    devshell.url = "github:numtide/devshell";
    proc-flake.url = "github:srid/proc-flake";
    flake-root.url = "github:srid/flake-root";
    pre-commit-hooks-nix.url = "github:cachix/pre-commit-hooks.nix";
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
