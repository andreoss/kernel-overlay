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
|6.19.14.0|6_19|2026-04-22|
|7.1.0-rc2|mainline|2026-05-03|
|7.0.3|stable|2026-04-30|
|6.18.26|6_18|2026-04-30|
|6.12.85|6_12|2026-04-30|
|6.6.137|6_6|2026-04-30|
|6.1.170|6_1|2026-04-30|
|5.15.204|5_15|2026-04-30|
|5.10.254|5_10|2026-04-30|

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
