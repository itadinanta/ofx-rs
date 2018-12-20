#!/bin/sh
mkdir -p examples/simple/resources/simple_plugin.ofx.bundle/Contents/Linux-x86-64/
cargo build && cp target/debug/libsimple_plugin.so examples/simple/resources/simple_plugin.ofx.bundle/Contents/Linux-x86-64/simple_plugin.ofx && Natron -t $PWD/test.py
