#!/bin/sh

cargo build --release --target wasm32-unknown-unknown && \
	cp target/wasm32-unknown-unknown/release/lifegate.wasm html/gate_app.wasm && \
	echo Starting httpd... && \
	busybox httpd -fvv -p 8000 -h html
