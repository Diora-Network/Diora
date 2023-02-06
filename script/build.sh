#!/bin/bash

cp ../target/release/diora .
cp ../../polkadot/target/release/polkadot .

bash ./generate_config.sh