#!/bin/bash

# FOR LINUX!!!
# Assumes that a ~/.bin/ dir exists and is included in $PATH

cargo build --relase
cp ../target/release/typed_graph ~/.bin/typed_graph
