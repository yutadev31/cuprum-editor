#!/bin/sh

plugins=$(ls plugins)

copy_plugin() {
  name=$1
  cp target/debug/$name ~/.cuprum/debug/plugins
}

cargo build --all

mkdir -p ~/.cuprum/debug/plugins
for plugin in "$plugins"; do
  copy_plugin $plugin
done
