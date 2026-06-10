{
  description = "Orbit dev shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            pkgs.nodejs
          ];

          shellHook = ''
            # Resolve prefix relative to the actual project root at activation time
            NPM_PREFIX="$(git -C "$PWD" rev-parse --show-toplevel 2>/dev/null || echo "$PWD")/.nix/.npm"
            export NPM_CONFIG_PREFIX="$NPM_PREFIX"
            export PATH="$NPM_PREFIX/bin:$PATH"
            mkdir -p "$NPM_PREFIX/bin"

            # pnpm comes from corepack, pinned via the "packageManager" field in package.json
            corepack enable --install-directory "$NPM_PREFIX/bin" pnpm 2>/dev/null || true

            if ! command -v vp &>/dev/null; then
              echo "nix-direnv: installing vite-plus..."
              npm install -g vite-plus --silent
            fi
          '';
        };
      }
    );
}
