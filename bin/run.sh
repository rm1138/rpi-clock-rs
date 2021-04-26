#!/bin/sh

cross +nightly build --target arm-unknown-linux-gnueabihf --features vendored-openssl --release && \
  ssh -t pi@"$TARGET" 'sudo systemctl stop rpi-clock' && \
  scp ./target/arm-unknown-linux-gnueabihf/release/rpi-awtrix pi@"$TARGET":~/ && \
  ssh -t pi@"$TARGET" 'sudo systemctl start rpi-clock'
