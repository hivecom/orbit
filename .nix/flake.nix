{
  description = "Orbit dev shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {inherit system;};
      in {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            nodejs

            cargo-tauri
            cargo-watch
            rust-analyzer
            wasm-pack
            bacon
            tmux

            pkg-config
            llvmPackages.bintools
          ];

          buildInputs = with pkgs; [
            webkitgtk_4_1
            dbus
          ];

          shellHook = ''
            # Needed on Wayland to report the correct display scale according to wiki.nixos.org
            export XDG_DATA_DIRS="$GSETTINGS_SCHEMAS_PATH"

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

            # vite-plus runs from this global prefix, and its test runner
            # resolves the jsdom test environment relative to its own install -
            # not the workspace - so jsdom must live here too. Without it,
            # `vp run -r test` fails for any package using `environment: jsdom`
            # (Vue component mounts, DOM-touching platform adapters).
            if [ ! -d "$NPM_PREFIX/lib/node_modules/jsdom" ]; then
              echo "nix-direnv: installing jsdom (test environment)..."
              npm install -g jsdom --silent
            fi
          '';
        };
      }
    );
}
