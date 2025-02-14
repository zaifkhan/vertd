{
  description = "vertd";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };
  outputs =
    { self, nixpkgs }:
    let
      pkgs = nixpkgs.legacyPackages.x86_64-linux;
    in
    {
      packages.x86_64-linux = {
        default = self.packages.x86_64-linux.vertd;

        vertd = pkgs.rustPlatform.buildRustPackage {
          pname = "vertd";
          version = "0.1.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs = with pkgs; [
            libclang
            ffmpeg
          ];
          propagatedBuildInputs = with pkgs; [ ffmpeg ];
        };
      };
    };
}
