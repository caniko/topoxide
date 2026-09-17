{
  description = "topoxide deterministic graph figure renderer";

  inputs = {
    harbor-rs.url = "git+https://github.com/caniko/harbor-rs.git?ref=trunk&rev=f209ddbca3fdbb0dc31fa3886ccc2ff7369c18ac";
    nixpkgs.follows = "harbor-rs/nixpkgs";
    rust-overlay.follows = "harbor-rs/rust-overlay";
    crane.follows = "harbor-rs/crane";
  };

  outputs = { self, nixpkgs, harbor-rs, rust-overlay, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = f:
        nixpkgs.lib.genAttrs systems (system:
          f {
            pkgs = import nixpkgs {
              inherit system;
              overlays = [ (import rust-overlay) ];
            };
            lib = nixpkgs.lib;
          });
    in
    {
      packages = forAllSystems ({ pkgs, lib }:
        let
          toolchain = harbor-rs.lib.mkToolchain { inherit pkgs; };
          buildCache = harbor-rs.lib.mkBuildCachePolicy {
            inherit pkgs;
            sccachePackage = harbor-rs.packages.${pkgs.system}.sccache;
            cacheRoot = null;
            namespaceScope = "canix-rust";
            namespaceGeneration = 5;
          };
          topoxideSrc = lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              ./Cargo.toml
              ./Cargo.lock
              ./src
              ./tests
            ];
          };
          topoxide = buildCache.withRustCache { package = toolchain.craneLib.buildPackage {
            pname = "topoxide";
            version = "0.1.0";
            src = topoxideSrc;
            cargoExtraArgs = "--bin topoxide";
          }; };

          website = pkgs.stdenv.mkDerivation {
            pname = "topoxide-website";
            version = "0.1.0";
            src = lib.fileset.toSource {
              root = ./.;
              fileset = lib.fileset.maybeMissing ./website;
            };
            nativeBuildInputs = [ pkgs.zola ];
            phases = [ "buildPhase" "installPhase" ];
            buildPhase = ''
              cp -r --no-preserve=mode $src/website site
              cd site
              zola build
            '';
            installPhase = ''
              cp -r public $out
            '';
          };

          docs = pkgs.stdenv.mkDerivation {
            pname = "topoxide-docs";
            version = "0.1.0";
            src = lib.fileset.toSource {
              root = ./.;
              fileset = lib.fileset.maybeMissing ./docs;
            };
            nativeBuildInputs = [ pkgs.mdbook ];
            buildPhase = ''
              mdbook build docs
            '';
            installPhase = ''
              cp -r docs/book $out
            '';
          };

          site = pkgs.runCommand "topoxide-site" { } ''
            mkdir -p $out
            cp -r ${website}/* $out/
            mkdir -p $out/docs
            cp -r ${docs}/* $out/docs/
          '';
        in
        {
          default = topoxide;
          inherit topoxide website docs site;
        });

      devShells = forAllSystems ({ pkgs, ... }:
        let
          toolchain = harbor-rs.lib.mkToolchain { inherit pkgs; };
        in {
        default = pkgs.mkShell {
          packages = [
            toolchain.rustToolchain
            pkgs.mdbook
            pkgs.zola
          ];
          shellHook = ''
            echo "topoxide development shell"
            echo "Website: cd website && zola serve"
            echo "Documentation: cd docs && mdbook serve"
          '';
        };
      });
    };
}
