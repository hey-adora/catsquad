{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    playwright.url = "github:pietdevries94/playwright-web-flake";
    playwright.inputs.nixpkgs.follows = "nixpkgs";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    # playwright.inputs.utils.follows = "utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      utils,
      rust-overlay,
      playwright,
      crane,
    }:
    utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [
          (import rust-overlay)
          (final: prev: {
            inherit (playwright.packages.${system}) playwright-test playwright-driver;
          })
          # (final: prev: {
          #   wasm-bindgen-cli = prev.wasm-bindgen-cli.overrideAttrs (
          #     drv: rec {
          #       version = "0.2.122";
                
          #       # src = pkgs.fetchFromGitHub {
          #       #   owner = "wasm-bindgen";
          #       #   repo = "wasm-bindgen";
          #       #   rev = version;
          #       #   sha256 = "sha256-37MalZkgFkEguWCO2T91GuM3p05wjgeNYaLQ0Ay5kaI=";
          #       # };

          #       src = final.fetchCrate {
          #         pname = "wasm-bindgen-cli";
          #         version = version;
          #         hash = "sha256-vO4RSxi/sMWxmsEs3GuljdMfIRSu75A+Q+c5wgYToRU=";
          #       };
          #       cargoHash = "";
          #       #cargoRoot = "wasm-bindgen-cli-0.2.122-vendor";
          #       cargoDeps = drv.cargoDeps.overrideAttrs (final.lib.const {
          #         name = "wasm-bindgen-cli-${version}-vendor";
          #         inherit src;
          #         outputHashMode = "recursive"; 
          #         outputHash = "sha256-nFmlDCixAZi1Zqv+bieuOygJT4OzqtcL1xPI4RRRNac=";
          #       });
          #       # sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=
          #       # src = final.fetchFromGitHub {
          #       #   owner = "luanti-org";
          #       #   repo = "luanti";
          #       #   tag = finalAttrs.version;
          #       #   hash = "sha256-EzLjLkN/3BdcpWJ92QnrdhxKmY6Bz2JkOC0oX0TrUtI=";
          #       # };
          #     }
          #   );
          # })
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
        # pkgsWasm = pkgs.pkgsCross.wasm32-unknown-unknown;
        # pkgsCross = import nixpkgs {
        #   inherit overlays;
        #   config.allowUnfree = true;
        #   localSystem = system;
        #   crossSystem.config = "wasm32-unknown-unknown";
        #   # crossSystem.system = "wasm32-unknown-unknown";
        #   # crossSystem = { config = "wasm32-unknown-unknown"; };
        # };
        rust_toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        # craneLib = crane.mkLib pkgs;
        src = ./.;
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
        # cargoArtiofacts = craneLib.buildDepsOnly {
        #   inherit src;
        #   strictDeps = true;
        #   CARGO_BUILD_TARGET = "wasm32-unknown-unknown"; 
        #   doCheck = false; 
        # };
      in
      {
        packages = {
          catsquad-web-dev = craneLib.buildPackage {
            pname = "catsquad-web";
            version = "0.1.0";
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
              wasm-bindgen ./target/wasm32-unknown-unknown/wasm_debug/catsquad_web.wasm --no-typescript --target no-modules --out-dir ./target/dist --out-name catsquad_1
              ls -lha ./target/dist
            '';

            installPhase = ''
              mkdir -p $out/lib
              mv ./target/dist/catsquad_1.js $out/lib
              mv ./target/dist/catsquad_1_bg.wasm $out/lib
            '';
          };

          catsquad-api-dev = craneLib.buildPackage {
            pname = "catsquad-api";
            version = "0.1.0";
            src = src;
            strictDeps = true;
            doCheck = false;
            cargoExtraArgs = "--package=catsquad-api";
            CARGO_PROFILE = "dev";
            CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER = "clang";

            cargoArtifacts = cargoArtifactsApiDebug;

            nativeBuildInputs = [
              pkgs.wild
              pkgs.clang
            ];
          };
            # inherit src cargoArtiofacts;
            # src = craneLib.cleanCargoSource ./.;
            # buildType = "wasm_debug";
            # cargoExtraArgs = "-p catsquad-web";

              # pwd
              # ls -lha .
              # rustc --version
              # cargo --version
              # wasm-bindgen --version
              # target/wasm32-unknown-unknown/wasm_debug

            # buildPhase = ''
            #   cargo build --package=catsquad-web --lib --target=wasm32-unknown-unknown --profile wasm_debug --offline
            # '';
            # installPhase = ''
            #   mkdir -p $out/lib
            #   cp target/wasm32-unknown-unknown/wasm_debug/catsquad_web.wasm $out/lib/
            # '';
          
            
          # catsquad-web-old = pkgs.rustPlatform.buildRustPackage {
          #   pname = "catsquad-web";
          #   version = "1.0.0";
          #   src = ./.;
          #   cargoHash = "sha256-kFIgeBDJY2V/LAKBdc/qj3bZ0UeO/PR8a2M1aVMb4Ag=";
          #   doCheck = false;

          #   buildType = "wasm_debug";
          #   buildPhase = ''
          #     cargo build --package=catsquad-web --lib --target=wasm32-unknown-unknown --profile wasm_debug --offline
          #   '';
          #   installPhase = ''
          #     mkdir -p $out/lib
          #     cp target/wasm32-unknown-unknown/wasm_debug/catsquad_web.wasm $out/lib/
          #   '';

          #   # cargoBuildType = "custom-optimized";
          #   # cargoBuildFlags = [ "--profile" "wasm-debug" ];
          #   # cargoTestFlags  = [ "--profile" "wasm-debug" ];
          #   # env = {
          #   #   # RUSTFLAGS = ''--cfg=web_sys_unstable_apis -C linker=wasm-ld'';
          #   # };
          #   # CARGO_BUILD_TARGET = "wasm32-unknown-unknown";


          #   nativeBuildInputs = [
          #     rust_toolchain
          #     pkgs.wild
          #     pkgs.clang
          #     pkgs.openssl
          #     pkgs.wabt
          #   ];

          # };
          # catsquad-dev = pkgs.stdenv.mkDerivation {
          #   pname = "catsquad-dev";
          #   version = "0.1.0";

          #   # buildCommand = ''
          #   #   mkdir -p $out/bin
          #   #   echo '#!/bin/bash' > $out/bin/catsquad-dev
          #   #   echo 'echo "Hello, World!"' >> $out/bin/catsquad-dev
          #   #   chmod 0755 "$out/bin/catsquad-dev"
          #   # '';
          #   # src = ./.;
          #   src = pkgs.lib.fileset.toSource {
          #     root = ./.;
          #     fileset = pkgs.lib.fileset.unions [
          #       ./catsquad-api
          #       ./catsquad-client
          #       ./catsquad-db
          #       ./catsquad-log
          #       ./catsquad-web
          #       ./artbounty
          #       ./artbounty-seed
          #       ./Cargo.toml
          #       ./Cargo.lock
          #     ];
          #   };

          #   buildPhase = ''
          #     export HOME=$(pwd)
          #     echo "wtf"
          #     ls -lha .
          #     cargo build
          #     ls -lha .

          #     echo '#!/bin/bash' > catsquad-dev
          #     echo 'echo "Hello, World!"' >> catsquad-dev
          #     chmod 0755 catsquad-dev
          #   '';

          #   installPhase = ''
          #     ls -lha .
          #     mkdir -p $out/bin
          #     mv catsquad-dev $out/bin/catsquad-dev
          #   '';
          #   nativeBuildInputs = with pkgs; [
          #     # rustc
          #     # gcc
          #     rust_toolchain
          #   ];

          # };

        };
        devShell =
          with pkgs;
          mkShell {
            packages = [
              ffmpeg-full
              # perf
              # samply
              surrealdb
              # inotify-tools
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
              # cargo-leptos
              wasm-pack
              wasm-bindgen-cli
              # wasm-bindgen-cli_0_2_121
              tailwindcss_4
              nodejs_latest
              pnpm
              # watchman
              # yarn
              pkg-config
              ripgrep
              # playwright
              # python312Packages.playwright
              # playwright-driver.browsers
              playwright-test
            ];
            RUST_BACKTRACE = 1;
            RUST_SRC_PATH = "${rust_toolchain}/lib/rustlib/src/rust/library";
            # LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
            #   pkgs.openssl
            # ];
            PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD = 1;
            PLAYWRIGHT_BROWSERS_PATH = "${pkgs.playwright-driver.browsers}";
            # PLAYWRIGHT_BROWSERS_PATH = 0;
            # PLAYWRIGHT_BROWSERS_PATH = pkgs.playwright-driver.browsers;
            # PLAYWRIGHT_SKIP_VALIDATE_HOST_REQUIREMENTS = true;
            # PLAYWRIGHT_NODEJS_PATH = "${pkgs.nodejs_25}/bin/node";
            # PLAYWRIGHT_LAUNCH_OPTIONS_EXECUTABLE_PATH = "${pkgs.playwright-driver.browsers}/chromium-1208";
          };
      }
    );
}
