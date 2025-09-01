{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    naersk.url  = "github:nix-community/naersk";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, naersk, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs { inherit system; };
      naerskLib = pkgs.callPackage naersk {};
      rustSrc = pkgs.rust.packages.stable.rustPlatform.rustLibSrc;

      hyprviz = naerskLib.buildPackage {
        src = ./.;
        buildInputs = with pkgs; [ gtk4 cairo pango graphene ];
        nativeBuildInputs = with pkgs; [ pkg-config wrapGAppsHook4 ];
        postInstall = ''
          install -Dm644 hyprviz.desktop -t $out/share/applications
        '';
      };
    in {
      packages.default = hyprviz;

      defaultPackage = hyprviz;

      devShells.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          fish
          cargo rustc rustfmt clippy rust-analyzer
          gtk4 cairo pango graphene
        ];
        nativeBuildInputs = [ pkgs.pkg-config ];

        shellHook = ''
          if [ -z "$FISH_VERSION" ] && [ -z "$NO_AUTO_FISH" ]; then
            exec ${pkgs.fish}/bin/fish
          fi
        '';

        env.RUST_SRC_PATH = "${rustSrc}";
      };
    });
}
