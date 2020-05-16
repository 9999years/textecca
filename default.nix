{ pkgs ? import <nixpkgs> { }, ... }:
let
  inherit (pkgs) stdenv stdenvNoCC lib fetchzip rustPlatform fetchFromGitHub;

  fetchCrate = { crate, version, ... }@attrs_:
    let attrs = removeAttrs attrs_ [ "crate" "version" ];
    in fetchTarball ({
      name = crate;
      url = "https://crates.io/api/v1/crates/${crate}/${version}/download";
    } // attrs);

  ucd = fetchzip {
    url = "https://www.unicode.org/Public/zipped/13.0.0/UCD.zip";
    sha256 = "1jn8617ai6dh22c3zmad5g617lppzzjcrynl70jw1ld47ihbgzp3";
    stripRoot = false;
  };

  ucd-generate = rustPlatform.buildRustPackage rec {
    pname = "ucd-generate";
    version = "0.2.8";

    src = fetchCrate {
      crate = "ucd-generate";
      inherit version;
      sha256 = "1kjzk57s0p37vq8ifqniymlcs5vkbv5aznzhkz2gi0g8r9mpwc38";
    };

    cargoSha256 = "0gmffbcrsa1wlkc55x4rafqyv0lzx9ngsdqlp3l42adizh5rv89g";
  };

  ucd-general-category = { categories, trie ? true }:
    stdenvNoCC.mkDerivation rec {
      name = "ucd_general_category.rs";
      version = "0.0.0";

      nativeBuildInputs = [ ucd-generate ];

      dontUnpack = true;
      dontConfigure = true;
      dontBuild = true;
      installPhase = ''
        ucd-generate general-category \
            --include ${lib.escapeShellArg categories} \
            ${lib.optionalString trie "--trie-set"} \
            ${ucd} \
            > $out
      '';
      dontFixup = true;
    };

in {
  inherit ucd ucd-generate;
  ucd-general-category = ucd-general-category { categories = "N,P,S,Zs"; };
}
