# Kernel overlay

Builds of the vanilla Linux kernel for Nix.

The current releases are regularly pulled from https://kernel.org.
Run `nix flake show github:andreoss/kernel-overlay' to see the exact versions.

- `linuxPackages` is an alias for the latest `stable` release.
- `linuxPackages_testing` is an alias for the latest `mainline` release.


## Available releases

<!--START-->
|Version|Package|Date|
|---|---|---|
|6.12.0-rc3|<b>mainline</b>|2024-10-13|
|6.11.3|<b>stable</b>|2024-10-10|
|6.10.14|<b>6_10</b>|2024-10-10|
|6.6.56|<b>6_6</b>|2024-10-10|
|6.1.112|<b>6_1</b>|2024-09-30|
|5.15.167|<b>5_15</b>|2024-09-12|
|5.10.226|<b>5_10</b>|2024-09-12|
|5.4.284|<b>5_4</b>|2024-09-12|
|4.19.322|<b>4_19</b>|2024-09-12|
<!--END-->

## Installation

(Optional) Enable cachix substitutions in `nix.settings`.

https://app.cachix.org/cache/kernel-overlay

NOTE: This change will only have effect after a `nix-daemon' restart.

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

By default, this overlay replaces the default kernel package. To use a specific one, specify it
in`configuration.nix`. For example

```
  boot.kernelPackages = pkgs.linuxPackages_4_14;
```
