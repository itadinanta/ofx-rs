#!/bin/sh
mkdir -p examples/basic/resources/ofx_rs_basic.ofx.bundle/Contents/Linux-x86-64/
cargo build && cp target/debug/libofx_rs_basic.so examples/basic/resources/ofx_rs_basic.ofx.bundle/Contents/Linux-x86-64/ofx_rs_basic.ofx && Natron -t $PWD/test.py
