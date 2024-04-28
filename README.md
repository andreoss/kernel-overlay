# Kernel overlay

Builds of vanilla Linux kernel for Nix.

## Available releases

<!--START-->
|Version|Package|Date|
|---|---|---|
|6.9.0-rc6|<b>mainline</b>|2024-04-28|
|6.8.8|<b>stable</b>|2024-04-27|
|6.7.12|<b>6_7</b>|2024-04-03|
|6.6.29|<b>6_6</b>|2024-04-27|
|6.1.88|<b>6_1</b>|2024-04-27|
|5.4.274|<b>5_4</b>|2024-04-13|
|5.15.157|<b>5_15</b>|2024-04-27|
|5.10.215|<b>5_10</b>|2024-04-13|
|4.19.312|<b>4_19</b>|2024-04-13|
<!--END-->

The currently available releases are regularly scrapped from https://kernel.org.
Run `nix flake show github:andreoss/kernel-overlay` to see exact versions.

- `linuxPackages` is an alias for the latest `stable` release.
- `linuxPackages_testing` is an alias for the latest `mainline` release.

## Installation

(Optional) Enable cachix substitutions in `nix.settings`.
Note: this change will be applied only `nix-daemon` restart

```
  nix.settings = {
    experimental-features = [ "nix-command" "flakes" ];
    substituters = [ "https://kernel-overlay.cachix.org" ];
    trusted-public-keys = [
      "kernel-overlay.cachix.org-1:rUvSa2sHn0a7RmwJDqZvijlzZHKeGvmTQfOUr2kaxr4="
    ];
  };
```

Add as an input to a flake

```
{
  description = "OS configuration";
  inputs = {
    ...
    kernel-overlay.url = "github:andreoss/kernel-overlay";
    ...
 ```

 Enable overlay
 ```
  outputs = inputs@{ self, nixpkgs, home-manager, ... }:
    let
      systems = lib.systems.flakeExposed;
      lib = nixpkgs.lib;
      eachSystem = lib.genAttrs systems;
    in rec {
      legacyPackages = eachSystem (system:
        import nixpkgs {
          inherit system;
          overlays = [
            inputs.kernel-overlay.overlays.default
          ];
        });
      ...

```

By default this overlays replaces the default kernel package. In order to use a specific one, specify it
in`configuration.nix`. For example

```
  boot.kernelPackages = pkgs.linuxPackages_4_14;
```
