#!/bin/bash

# default runner builds and flashes the board
cargo run --release --bin=rp_qtpy --target=thumbv6m-none-eabi
