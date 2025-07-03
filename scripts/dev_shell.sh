#!/bin/sh

nix develop --extra-experimental-features 'nix-command flakes' -c $SHELL