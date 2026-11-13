{
  description = "xD: declarative Johnny.Decimal config and the jd CLI in one tree";

  inputs.nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1"; # unstable Nixpkgs

  outputs =
    { self, ... }@inputs:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      forEachSupportedSystem =
        f:
        inputs.nixpkgs.lib.genAttrs supportedSystems (
          system: f { pkgs = import inputs.nixpkgs { inherit system; }; }
        );
    in
    {
      devShells = forEachSupportedSystem (
        { pkgs }:
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              # The xD crates (docs/adr/0001-architecture.md)
              cargo
              rustc
              clippy
              rustfmt
              rust-analyzer
              jujutsu
              nickel
              nushell
              jq
              git
            ];
          };
        }
      );

      formatter = forEachSupportedSystem ({ pkgs }: pkgs.nixfmt-tree);
    };
}
