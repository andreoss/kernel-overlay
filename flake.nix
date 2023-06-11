{
  description = "Kernel overlay";
  inputs = { flake-utils.url = "github:numtide/flake-utils"; };
  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        systems = lib.systems.flakeExposed;
        lib = nixpkgs.lib;
        pkgs = import nixpkgs { inherit system; };
        json = f: builtins.fromJSON (builtins.readFile f);
        merge = builtins.foldl' (x: y: x // y) { };
        meta = json ./meta.json;
        sources = json ./sources.json;
        patches = json ./patches.json;
        fetchPatch = p: {
          name = p.name;
          patch = builtins.fetchurl {
            url = p.url;
            sha256 = p.checksum;
          };
          extraConfig = p.config or "";
        };
        mkName = n: p: "${n}_${p.package.name}";
        mkLinuxPackages = p:
          let name = mkName "linux" p;
          in {
            ${name} = pkgs.linuxKernel.manualConfig {
              inherit lib;
              inherit (pkgs) stdenv;
              version = p.version;
              configfile = ./config;
              kernelPatches = map fetchPatch (patches.${name} or [ ]);
              allowImportFromDerivation = true;
              src = pkgs.fetchurl {
                url = p.url;
                sha256 = p.checksum;
              };
            };
            ${mkName "linuxPackages" p} = pkgs.recurseIntoAttrs
              (pkgs.linuxPackagesFor self.outputs.packages.${system}.${name});
          };
        kpkgs = merge (map (x: mkLinuxPackages x) sources);
        mkNixos = p: {
          ${mkName "vm" p} = nixpkgs.lib.nixosSystem {
            inherit system;
            modules = [
              ({ config, pkgs, ... }: {
                system.stateVersion = "23.05";
                boot.extraModulePackages = with config.boot.kernelPackages; [ ];
                boot.kernelPackages = kpkgs.${mkName "linuxPackages" p};
              })
            ];
          };
        };
      in {
        nixosConfigurations = merge (map mkNixos sources);
        packages = kpkgs;
        overlays.default = final: prev: {
          linux = kpkgs.linux_stable;
          linux_testing = kpkgs.linux_mainline;
          linuxPackages = kpkgs.linuxPackages_stable;
          linuxPackages_testing = kpkgs.linuxPackages_mainline;
        };
      });
}
