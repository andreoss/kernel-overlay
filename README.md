# Kernel overlay

Builds of the vanilla Linux kernel for Nix.

Current releases are regularly pulled from https://kernel.org.
Run the following command to see the exact versions:

```sh
nix flake show github:andreoss/kernel-overlay
``` 

- `linuxPackages` is an alias for the latest **stable** release.  
- `linuxPackages_testing` is an alias for the latest **mainline** release.

## Available releases

<!--START-->
|Version|Package|Date|
|---|---|---|
|7.0.0-rc5|mainline|2026-03-22|
|6.19.9|stable|2026-03-19|
|6.18.19|6_18|2026-03-19|
|6.12.77|6_12|2026-03-13|
|6.6.129|6_6|2026-03-05|
|6.1.166|6_1|2026-03-05|
|5.15.202|5_15|2026-03-04|
|5.10.252|5_10|2026-03-04|

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

By default, this overlay replaces the default kernel package.

To use a specific kernel, set it in configuration.nix. For example:

```
  boot.kernelPackages = pkgs.linuxPackages_4_14;
```
