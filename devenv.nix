{ pkgs, lib, config, inputs, ... }:

{
  # https://devenv.sh/packages/
  packages = with pkgs; [ git libyaml openssl zlib ];

  languages.rust.enable = true;
  languages.ruby.enable = true;


  enterShell = ''

  '';

}
