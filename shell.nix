{ pkgs ? import <nixpkgs> { } }:

with pkgs;

mkShell {
buildInputs = [ gdb rustc cargo ]; # your dependencies here
}
