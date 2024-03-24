{ pkgs ? import <nixpkgs> { } }:

with pkgs;

mkShell {
buildInputs = [ rustc cargo ]; # your dependencies here
}
