#!/bin/sh
cargo build && cp target/debug/libsimple_plugin.so examples/simple/resources/simple_plugin.ofx.bundle/Contents/Linux-x86-64/simple_plugin.ofx
