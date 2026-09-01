{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    playwright.url = "github:pietdevries94/playwright-web-flake";
    playwright.inputs.nixpkgs.follows = "nixpkgs";
    crane.url = "github:ipetkov/crane";
    process-compose-flake.url = "github:Platonic-Systems/process-compose-flake";
    services-flake.url = "github:juspay/services-flake";
  };

  outputs =
    {
      self,
      nixpkgs,
      utils,
      rust-overlay,
      playwright,
      crane,
      services-flake,
      process-compose-flake,
    }:
    utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [
          
          (import rust-overlay)
          (final: prev: {
            inherit (playwright.packages.${system}) playwright-test playwright-driver;
          })
          (final: prev: {
            wasm-bindgen-cli = final.rustPlatform.buildRustPackage {
              pname = "wasm-bindgen-cli";
              version = "0.2.122";
              OPENSSL_NO_VENDOR = 1;
              useFetchCargoVendor = true;
              cargoHash = "sha256-Inup6vvJSG5ghNyeDPyZbfZo4d0LsMG2OJfStoaeDBs=";
              doCheck = false;

              nativeCheckInputs = [ final.nodejs_latest ];

              buildInputs = [ final.openssl ];
              src = final.fetchCrate {
                pname = "wasm-bindgen-cli";
                version = "0.2.122";
                hash = "sha256-vO4RSxi/sMWxmsEs3GuljdMfIRSu75A+Q+c5wgYToRU=";
              };

            };
          })

        ];

        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
          config.allowUnsupportedSystem = true;
        };

        rust_toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        cat_scripts = pkgs.stdenv.mkDerivation {
            pname = "cat_scripts";
            version = "0.1.0";
            src = pkgs.lib.fileset.toSource {
              root = ./.;
              fileset = pkgs.lib.fileset.unions [
                ./scripts
              ];
            };
            installPhase = ''
              mkdir -p $out/lib
              ls -lha .
              ls -lha ./scripts/
              cp ./scripts/postgres_init.sql $out/lib/postgres_init.sql
            '';
        };

        # pgunit = pkgs.postgresqlPackages.callPackage ./scripts/pgunit.nix {};
        # pgu128 = pkgs.postgresqlPackages.callPackage ./scripts/pgu128.nix {};
        # pgdev = pkgs.callPackage "${nixpkgs}/pkgs/servers/sql/postgresql/buildPostgresqlExtension.nix";
        # pgunit = pgdev.postgresqlBuildExtension (finalAttrs: {
        #   pname = "pguint";
        #   version = "1.20260630";

        #   src = pkgs.fetchFromGitHub {
        #     owner = "petere";
        #     repo = "pguint";
        #     tag = "v${finalAttrs.version}";
        #     hash = "sha256-4PVr0dW6CL3ov1W5BPJU1CAphwOyXwqUoYgWCPXjto8=";
        #   };

        #   meta = {
        #     description = "unsigned integer types extension for PostgreSQL";
        #     homepage = "https://github.com/petere/pguint";
        #     license = pkgs.licenses.postgresql;
        #     platforms = pkgs.postgresql.meta.platforms;
        #     maintainers = [ ];
        #   };
        # });

        servicesMod = (import process-compose-flake.lib { inherit pkgs; }).evalModules {
          modules = [
            services-flake.processComposeModules.default
            {
              
              services.postgres."pg1".package = pkgs.postgresql_18;
              services.postgres."pg1".dataDir = "./target/tmp/data_db_pg1";
              services.postgres."pg1".enable = true;
              services.postgres."pg1".socketDir = "./target";
              services.postgres."pg1".extensions = exts: [
                exts.system_stats
                # exts.pgvector
                # exts.vectorchord
                # (exts.callPackage ./scripts/pgu128.nix { })
              ];
              # services.postgres."pg1".settings.shared_preload_libraries = "vectors.so";
              # services.postgres."pg1".settings.shared_preload_libraries = "${pkgs.postgresqlPackages.pgvector}/lib/vectors.so";
              # services.postgres."pg1".settings.shared_preload_libraries = "vector.so, vchord.so";
              # services.postgres."pg1".settings.shared_preload_libraries = "vector.so, vchord.so, uint128.so";
              # services.postgres."pg1".settings.shared_preload_libraries = "vector.so, vchord.so, uint.so";
              services.postgres."pg1".initialDatabases = [
                {
                  name = "catsquad";
                  # schemas = ["${cat_scripts}/lib/postgres_init.sql"];
                }
              ];
              # services.postgres."pg1".initialScript.before = ''
              #   CREATE EXTENSION system_stats;
              #   CREATE EXTENSION vchord CASCADE;
              # '';
                  # schemas = [ "${inputs.northwind}/northwind.sql" ];
                # CREATE EXTENSION IF NOT EXISTS vchord CASCADE;
              # services.redis."r1".port = 0;
              # services.redis."r1".unixSocket = "./redis.sock";
            }
          ];
        };

        src = ./.;
        catsquad_version = "0.1.0";
        craneLib = (crane.mkLib pkgs).overrideToolchain rust_toolchain;
        cargoArtifactsWasmDebug = craneLib.buildDepsOnly {
          pname = "catsquad-web";
          version = "0.1.0";
          src = src;
          strictDeps = true;
          doCheck = false;
          cargoExtraArgs = "--package=catsquad-web";
          CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
          CARGO_PROFILE = "wasm_debug";
          nativeBuildInputs = [
            pkgs.clang
          ];
          wasm-bindgen-cli = pkgs.wasm-bindgen-cli;
        };
        cargoArtifactsApiDebug = craneLib.buildDepsOnly {
          pname = "catsquad-api";
          version = "0.1.0";
          src = src;
          strictDeps = true;
          doCheck = false;
          cargoExtraArgs = "--package=catsquad-api";
          CARGO_PROFILE = "dev";
          nativeBuildInputs = [
            pkgs.clang
            pkgs.wild
          ];
        };

        catsquad-web-dev = craneLib.buildPackage {
          pname = "catsquad-web";
          version = catsquad_version;
          src = src;
          strictDeps = true;
          doCheck = false;
          cargoExtraArgs = "--package=catsquad-web";
          CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
          CARGO_PROFILE = "wasm_debug";

          cargoArtifacts = cargoArtifactsWasmDebug;

          nativeBuildInputs = [
            pkgs.wasm-bindgen-cli
            pkgs.wild
            pkgs.clang
          ];
          wasm-bindgen-cli = pkgs.wasm-bindgen-cli;

          postBuild = ''
            mkdir ./target/dist
            wasm-bindgen ./target/wasm32-unknown-unknown/wasm_debug/catsquad_web.wasm --no-typescript --target no-modules --out-dir ./target/dist --out-name catsquad
            ls -lha ./target/dist
          '';

          installPhase = ''
            mkdir -p $out/lib
            cp ./target/dist/catsquad.js $out/lib
            cp ./target/dist/catsquad_bg.wasm $out/lib
            cp ./assets/* $out/lib
          '';
        };

        catsquad-api-dev-unwrapped = craneLib.buildPackage {
          pname = "catsquad-api-unwrapped";
          version = "0.1.0";
          src = src;
          strictDeps = true;
          doCheck = false;
          cargoExtraArgs = "--package=catsquad-api";
          CARGO_PROFILE = "dev";
          CATSQUAD_WEB_LIB = "${catsquad-web-dev}/lib";

          cargoArtifacts = cargoArtifactsApiDebug;

          nativeBuildInputs = [
            pkgs.wild
            pkgs.clang
            pkgs.pkg-config
          ];

          installPhase = ''
            mkdir -p $out/bin
            mkdir -p $out/lib
            cp ./target/debug/catsquad-api $out/bin
            cp ./target/debug/libcatsquad_db.so $out/lib
            find ./target/debug/deps/ -name 'libsurrealdb-*.so' -exec cp "{}" $out/lib \;
          '';

        };

        catsquad-api-dev = pkgs.stdenv.mkDerivation {
            pname = "catsquad-api-dev";
            version = catsquad_version;

            dontUnpack = true;
            dontBuild = true;

            installPhase = ''
              mkdir -p $out/bin
              echo '#!/usr/bin/env sh' > $out/bin/catsquad-api-dev
              echo 'LD_LIBRARY_PATH="''${LD_LIBRARY_PATH}:${catsquad-api-dev-unwrapped}/lib:${rust_toolchain}/lib/rustlib/x86_64-unknown-linux-gnu/lib" ${catsquad-api-dev-unwrapped}/bin/catsquad-api' >> $out/bin/catsquad-api-dev
              chmod 0755 "$out/bin/catsquad-api-dev"
            '';
          
        };
        
      in
      {
        packages = {
          inherit catsquad-api-dev;
          services = servicesMod.config.outputs.package;
          cat_scripts = cat_scripts;
        };

        devShell =
          with pkgs;
          mkShell {
            inputsFrom = [
              servicesMod.config.services.outputs.devShell
            ];
            packages = [
              # parallel
              # postgresql
              ffmpeg-full
              cargo-expand
              # surrealdb
              rust_toolchain
              wild
              clang
              openssl
              taplo
              vtsls
              emmet-language-server
              tailwindcss-language-server
              prettier
              eslint
              wasm-pack
              wasm-bindgen-cli
              tailwindcss_4
              nodejs_latest
              pnpm
              pkg-config
              ripgrep
              playwright-test
            ];
            RUST_BACKTRACE = 1;
            RUST_SRC_PATH = "${rust_toolchain}/lib/rustlib/src/rust/library";
            PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD = 1;
            PLAYWRIGHT_BROWSERS_PATH = "${pkgs.playwright-driver.browsers}";
            shellHook = ''
              alias debug=./scripts/debug.sh
              alias release=./scripts/release.sh
            '';
          };
      }
    );
}
