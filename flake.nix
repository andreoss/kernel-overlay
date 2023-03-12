{
  description = "Kernel overlay";
  inputs = { nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable"; };
  outputs = { self, nixpkgs }:
    let
      json = f: builtins.fromJSON (builtins.readFile f);
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
      lib = pkgs.lib;
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
      trimVersion = x:
        with builtins;
        concatStringsSep "_" (match "([0-9]+)[.]([0-9]+).*" x);
      mkName = n: p:
        if (p.category == "longterm") then
          "${n}_${trimVersion p.version}"
        else
          "${n}_${p.category}";
      mkLinuxPackages = p:
        let name = mkName "linux" p;
        in {
          ${name} = pkgs.linuxKernel.manualConfig {
            inherit lib;
            inherit (pkgs) stdenv;
            version = p.version;
            configfile = ./huge-config;
            kernelPatches = map fetchPatch (patches.${name} or [ ]);
            allowImportFromDerivation = true;
            src = pkgs.fetchurl {
              url = p.url;
              sha256 = p.checksum;
            };
          };
          ${mkName "linuxPackages" p} = pkgs.recurseIntoAttrs
            (pkgs.linuxPackagesFor self.outputs.packages.x86_64-linux.${name});
        };
      kpkgs = merge (map (x: mkLinuxPackages x) sources);
      mkNixos = p: {
        ${mkName "vm" p} = nixpkgs.lib.nixosSystem {
          system = "x86_64-linux";
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
      packages.x86_64-linux = kpkgs;
      nixosConfigurations = merge (map mkNixos sources);
      overlays.default = final: prev: {
        linux = kpkgs.linux_stable;
        linux_testing = kpkgs.linux_mainline;
        linuxPackages = kpkgs.linuxPackages_stable;
        linuxPackages_testing = kpkgs.linuxPackages_mainline;
      };
    };
}
