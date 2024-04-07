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
          then
            with pkgs.darwin.apple_sdk.frameworks; [
              pkgs.libiconv
              CoreFoundation
              SystemConfiguration
              Security
              CoreServices
            ]
          else [];
        rustPlatform = pkgs.makeRustPlatform {
          inherit (fenix.packages.${system}.stable) rustc cargo;
        };
      in {
        formatter = pkgs.alejandra;

        packages.default = config.packages.lz;
        packages.lz = let
          nativeBuildInputs =
            (builtins.map (l: pkgs.lib.getDev l) cIncludes)
            ++ cIncludes
            ++ cLibs
            ++ [pkgs.pkg-config];
        in
          rustPlatform.buildRustPackage {
            pname = "lz";
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
            postFixup = "mv $out/bin/lz-cli $out/bin/lz";
            meta.mainProgram = "lz";
          };
        packages.dioxus-cli =
          pkgs.dioxus-cli.override
          {
            rustPlatform.buildRustPackage = args: (rustPlatform.buildRustPackage (args
              // {
                version = (builtins.fromTOML (builtins.readFile "${inputs.dioxus}/Cargo.toml")).workspace.package.version;
                src = inputs.dioxus;
                cargoLock.lockFile = "${inputs.dioxus}/Cargo.lock";
                cargoHash = null;
                buildAndTestSubdir = "packages/cli";
                doCheck = false;
                nativeBuildInputs = args.nativeBuildInputs ++ [pkgs.makeBinaryWrapper];

                # Unset RUSTFLAGS to prevent cargo from attempting to
                # link the wasm binary with macOS frameworks, which
                # panics rustc:
                postFixup = ''
                  mv $out/bin/dx $out/bin/.dx-wrapped
                  makeBinaryWrapper $out/bin/.dx-wrapped $out/bin/dx \
                    --unset RUSTFLAGS
                '';
              }));
          };
        packages.cargo-progenitor = rustPlatform.buildRustPackage {
          pname = "cargo-progenitor";
          version = "0.6.0";
          src = inputs.progenitor;
          cargoLock = {
            lockFile = "${inputs.progenitor}/Cargo.lock";
            outputHashes = {
              "dropshot-0.9.1-dev" = "sha256-xjOwr9/NJgfAv/5GlqZY1/TsyBc3cNmhF8vlBdYt8Tw=";
            };
          };
          buildAndTestSubdir = "cargo-progenitor";
          doCheck = false;
          buildInputs =
            if pkgs.stdenv.isDarwin
            then
              with pkgs.darwin.apple_sdk.frameworks; [
                Security
                SystemConfiguration
              ]
            else [];
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
                category = "development";
                help = "run all servers for development";
                name = "dev-server";
                package = pkgs.writeShellApplication {
                  name = "dev-server";
                  text = ''
                    cd "$PRJ_ROOT"/src/lz-ui
                    dx serve
                  '';
                };
              }
              {
                category = "development";
                help = "Run the sqlite-web DB browser on dev-db.sqlite";
                name = "sqlite-web";
                command = "${pkgs.sqlite-web}/bin/sqlite_web -r $PRJ_ROOT/dev-db.sqlite";
              }
              {
                category = "development";
                help = "setup the pre-commit hook for this repo";
                name = "setup-pre-commit";
                command = config.pre-commit.installationScript;
              }

              {
                category = "maintenance";
                help = "regenerate the frontend OpenAPI client";
                name = "regenerate-openapi-client";
                package = pkgs.writeShellApplication {
                  name = "regenerate-openapi-client";
                  runtimeInputs = [config.packages.cargo-progenitor];
                  text = ''
                    cargo run -- generate-openapi-spec | \
                       cargo progenitor --input /dev/stdin --output src/lz-openapi --name lz-openapi --version 0.1.0 --interface builder
                    cargo fmt -p lz-openapi
                  '';
                };
              }
              {
                category = "maintenance";
                help = "set up and migrate the dev database";
                name = "setup-db";
                package = pkgs.writeShellApplication {
                  name = "setup-db";
                  text = ''
                    sqlx database reset
                    cargo sqlx prepare --workspace
                  '';
                };
              }
            ];
            packages = [
              config.packages.dioxus-cli
              config.packages.cargo-progenitor
              pkgs.sqlx-cli
              pkgs.sqlite
              pkgs.cargo-watch
              pkgs.nodejs_21
              pkgs.nodePackages.typescript-language-server
              fenix.packages.${system}.targets.wasm32-unknown-unknown.stable.rust-std
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
    dioxus = {
      url = "github:DioxusLabs/dioxus/v0.5.2";
      flake = false;
    };
    progenitor = {
      url = "github:oxidecomputer/progenitor/v0.6.0";
      flake = false;
    };
  };
}
