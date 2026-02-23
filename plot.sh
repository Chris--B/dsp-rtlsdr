#!/bin/bash
set -ex

cargo build --bin from_rtlsdr
cargo run   --bin from_rtlsdr --  -s 2400000 | cast -o cf32 | pvv | psd_raster >/dev/null
