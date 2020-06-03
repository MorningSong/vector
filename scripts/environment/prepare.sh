#! /usr/bin/env bash
set -e

rustup default $(cat rust-toolchain)
cd scripts
bundle update --bundler
bundle install
cd ..

cd website
yarn
cd ..
