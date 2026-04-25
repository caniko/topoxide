{
  description = "topoxide deterministic graph figure renderer";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
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
            pkgs = import nixpkgs { inherit system; };
            lib = nixpkgs.lib;
          });
    in
    {
      packages = forAllSystems ({ pkgs, lib }:
        let
          topoxide = pkgs.rustPlatform.buildRustPackage {
            pname = "topoxide";
            version = "0.1.0";
            src = lib.fileset.toSource {
              root = ./.;
              fileset = lib.fileset.unions [
                ./Cargo.toml
                ./Cargo.lock
                ./src
                ./tests
              ];
            };
            cargoLock.lockFile = ./Cargo.lock;
          };

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

      devShells = forAllSystems ({ pkgs, ... }: {
        default = pkgs.mkShell {
          packages = [
            pkgs.cargo
            pkgs.clippy
            pkgs.mdbook
            pkgs.rustc
            pkgs.rustfmt
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
