{
  description = "Build environment for the nibli syllogism explainer video";

  # Both pins match the NixOS config's flake.lock (nixpkgs = nixos-26.05,
  # nixpkgs-apps = master), so most store paths are already present.
  # piper-tts comes from master: 1.8.0 runs the joe voice (needs Piper 1.5+).
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/7dcdd87373d72e7a2173e4db02e20d3d7f495cfd";
    nixpkgs-apps.url = "github:NixOS/nixpkgs/27eb89b729375903ba8e21c79a19e408e432531e";
  };

  outputs = { nixpkgs, nixpkgs-apps, ... }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      apps = nixpkgs-apps.legacyPackages.${system};
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        packages = [
          pkgs.ffmpeg-full
          pkgs.espeak-ng
          (pkgs.python313.withPackages (ps: [ ps.pillow ]))
          apps.piper-tts
          apps.whisper-cpp
        ];
      };
    };
}
