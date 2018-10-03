#!/bin/bash

port=8080
ip=0.0.0.0
export SHAAT_ADDR=$ip:$port
export SHAAT_STATIC=../static
export SHAAT_DB=../db.sqlite

#systemfd --no-pid -s http::$ip:$port -- \
         #cargo watch --ignore-nothing -d 2 -w ../target/debug/shaat -x run
cargo run

