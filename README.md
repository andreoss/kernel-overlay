# Kernel overlay

Builds of vanilla Linux kernel for Nix.

## Available releases

<!--START-->
|Version|Package|Date|
|---|---|---|
|6.9.3|<b>stable</b>|2024-05-30|
|6.8.12|<b>6_8</b>|2024-05-30|
|6.6.32|<b>6_6</b>|2024-05-25|
|6.10.0-rc3|<b>mainline</b>|2024-06-09|
|6.1.92|<b>6_1</b>|2024-05-25|
|5.4.277|<b>5_4</b>|2024-05-25|
|5.15.160|<b>5_15</b>|2024-05-25|
|5.10.218|<b>5_10</b>|2024-05-25|
|4.19.315|<b>4_19</b>|2024-05-25|
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
