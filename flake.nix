{
  description = "lz, a web-based tagged bookmark manager";

  outputs = inputs @ {
    self,
    flake-parts,
    nixpkgs,
    fenix,
    crane,
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
        inputs.pre-commit-hooks-nix.flakeModule
        inputs.proc-flake.flakeModule
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
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          # Set the build targets supported by the toolchain,
          # wasm32-unknown-unknown is required for trunk.
          targets = ["wasm32-unknown-unknown"];
        };
        craneLib = ((crane.mkLib pkgs).overrideToolchain rustToolchain).overrideScope (_final: _prev: {
          inherit (import nixpkgs {inherit system;}) wasm-bindgen-cli;
        });

        lib = pkgs.lib;
      in {
        _module.args.pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [
            inputs.rust-overlay.overlays.default
          ];
        };
        formatter = pkgs.alejandra;

        packages.default = config.packages.lz;
        packages.lz = let
          nativeBuildInputs =
            (builtins.map (l: pkgs.lib.getDev l) cIncludes)
            ++ cIncludes
            ++ cLibs
            ++ [pkgs.pkg-config];
          node-modules = pkgs.mkYarnPackage {
            name = "node-modules";
            src = ./src/lz-ui;
          };
          src = lib.cleanSourceWith {
            src = ./.; # The original, unfiltered source
            filter = path: type:
              (lib.hasSuffix "\.html" path)
              || (lib.hasSuffix "\.scss" path)
              || (lib.hasInfix "/assets/" path)
              || (lib.hasSuffix "\.json" path)
              || (lib.hasSuffix "\.js" path)
              || (lib.hasSuffix "\.lock" path)
              || (lib.hasSuffix "\.sql" path)
              # Default filter from crane (allow .rs files)
              || (craneLib.filterCargoSources path type);
          };
          commonArgs = {
            inherit src;
            strictDeps = true;

            buildInputs =
              [
              ]
              ++ lib.optionals pkgs.stdenv.isDarwin [
                pkgs.libiconv
              ];
          };
          wasmArgs =
            commonArgs
            // {
              pname = "lz-workspace-wasm";
              cargoExtraArgs = "--package=lz-ui";
              trunkIndexPath = "src/lz-ui/index.html";
              CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
              prePatch = ''
                ln -s ${node-modules}/libexec/lz-ui-js/node_modules src/lz-ui/node_modules
              '';
            };
          cargoArtifactsWasm = craneLib.buildDepsOnly (wasmArgs
            // {
              doCheck = false;
            });
          ui =
            craneLib.buildTrunkPackage wasmArgs
            // {
              pname = "lz-ui-wasm";
              cargoArtifacts = cargoArtifactsWasm;
              wasm-bindgen-cli = pkgs.wasm-bindgen-cli;
              stripPhase = "";
            };
        in
          rustPlatform.buildRustPackage {
            pname = "lz";
            version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).workspace.package.version;
            inherit nativeBuildInputs;
            buildInputs = nativeBuildInputs;
            inherit src;
            cargoLock.lockFile = ./Cargo.lock;

            UI_DIST = ui;
            preBuild = ''
              ln -s ${ui} src/lz-cli/dist
            '';

            postFixup = "mv $out/bin/lz-cli $out/bin/lz";
            meta.mainProgram = "lz";
          };
        packages.yew-fmt = rustPlatform.buildRustPackage {
          pname = "yew-fmt";
          version = (builtins.fromTOML (builtins.readFile "${inputs.yew-fmt}/Cargo.toml")).package.version;
          src = inputs.yew-fmt;
          cargoLock.lockFile = "${inputs.yew-fmt}/Cargo.lock";
          nativeBuildInputs = [pkgs.rustfmt];
          meta.mainProgram = "yew-fmt";
        };
        packages.trunk = rustPlatform.buildRustPackage {
          pname = "trunk";
          version = (builtins.fromTOML (builtins.readFile "${inputs.trunk}/Cargo.toml")).package.version;
          src = inputs.trunk;
          nativeBuildInputs = [pkgs.pkg-config (pkgs.lib.getDev pkgs.openssl) pkgs.openssl];
          buildInputs =
            [(pkgs.lib.getDev pkgs.openssl) pkgs.openssl]
            ++ (
              if pkgs.stdenv.isDarwin
              then
                with pkgs.darwin.apple_sdk.frameworks; [
                  SystemConfiguration
                  CoreServices
                ]
              else []
            );
          checkFlags = ["--skip=tools::tests::download_and_install_binaries"];
          cargoLock.lockFile = "${inputs.trunk}/Cargo.lock";
          meta.mainProgram = "trunk";
        };

        apps = {
          default = config.apps.lz-web;
          lz-web.program = config.packages.lz-web;
        };

        devshells = {
          default = {
            packages = [
              pkgs.sqlx-cli
              pkgs.sqlite
              pkgs.cargo-watch
              pkgs.cargo-nextest
              pkgs.cargo-udeps
              pkgs.tailwindcss
              config.packages.yew-fmt
              pkgs.yarn
              pkgs.nodejs
              pkgs.nodePackages.typescript-language-server
              fenix.packages.${system}.targets.wasm32-unknown-unknown.stable.rust-std
              (pkgs.writeShellApplication {
                name = "trunk";
                text = ''
                  unset RUSTFLAGS
                  CARGO_TARGET_DIR=target/trunk-wasm
                  export CARGO_TARGET_DIR
                  ${pkgs.lib.getExe config.packages.trunk} "$@"
                '';
              })
            ];
            imports = [
              "${inputs.devshell}/extra/language/rust.nix"
              "${inputs.devshell}/extra/language/c.nix"
            ];
            commands = [
              {
                category = "development";
                help = "run all servers for development";
                name = "dev-server";
                package =
                  config.proc.groups.dev-server.package;
              }
              {
                category = "development";
                help = "Run the sqlite-web DB browser on dev-db.sqlite";
                name = "sqlite-web";
                command = "${pkgs.sqlite-web}/bin/sqlite_web -p 8081 -r $PRJ_ROOT/dev-db.sqlite";
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
                  text = ''
                    cargo run --features dev -- generate-openapi-spec rust-client src/lz-openapi
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
              {
                name = "RUSTFMT";
                value = pkgs.lib.getExe config.packages.yew-fmt;
              }
            ];

            language.c.includes = cIncludes;
            language.c.libraries = cLibs;
          };
        };

        proc.groups."dev-server" = {
          processes = {
            backend = {
              command = pkgs.lib.getExe (pkgs.writeShellApplication {
                name = "backend";
                runtimeInputs = [pkgs.cargo-watch];
                text = ''
                  cargo watch --why -L info -i src/lz-ui -i src/lz-openapi -i flake.nix -i flake.lock -- \
                     cargo run --features dev -- \
                     --db dev-db.sqlite web --authentication-header-name X-Tailscale-User-LoginName --default-user-name=developer --listen-on=127.0.0.1:3000
                '';
              });
            };
            frontend = {
              command = pkgs.lib.getExe (pkgs.writeShellApplication {
                name = "frontend";
                text = ''
                  cd src/lz-ui
                  yarn install
                  trunk serve --open
                '';
              });
            };
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
    progenitor = {
      url = "github:oxidecomputer/progenitor/v0.6.0";
      flake = false;
    };
    yew-fmt = {
      url = "github:schvv31n/yew-fmt";
      flake = false;
    };
    trunk = {
      url = "github:trunk-rs/trunk/v0.21.0-rc.3";
      flake = false;
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    crane = {
      url = "github:ipetkov/crane";
    };
  };
}
