#!/bin/sh

cross +nightly build --target arm-unknown-linux-gnueabihf --features vendored-openssl --release && \
  ssh -t pi@192.168.1.105 'killall -q rpi-awtrix' || \
  scp ./target/arm-unknown-linux-gnueabihf/release/rpi-awtrix pi@192.168.1.105:~/ && \
  ssh -t pi@192.168.1.105 'sudo setcap cap_sys_nice+ep ./rpi-awtrix && ./rpi-awtrix'
