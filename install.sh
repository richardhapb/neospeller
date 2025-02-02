#!/bin/bash

if [ ! -f Cargo.toml ]; then
    echo "This script must be run from the root of the project"
    exit 1
fi

if ! command -v cargo &> /dev/null
then
    echo "cargo could not be found"
    exit 1
fi

cargo build --release

sudo ln -s $(pwd)/target/release/$(basename $(pwd)) /usr/local/bin/$(basename $(pwd))
