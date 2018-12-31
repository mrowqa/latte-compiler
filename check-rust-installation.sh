#!/bin/bash

if [ -d ~/.cargo ]; then
    rustc --version
else
    echo "Rust installation not detected. Please consult README."
    exit 1
fi

