#!/usr/bin/env bash

# Generate the various keys for the gensis.
#
# This script will print the info to stdout in a form which is slightly easier to be
# copied and pasted to the Rust source file.

.. key insert --chain diora_rococo --keystore-path ./keystore1 --suri "${SECRET1}" --scheme Sr25519 --key-type nmbs
./diora key insert --chain diora_rococo --keystore-path ./keystore2 --suri "${SECRET2}" --scheme Sr25519 --key-type nmbs
./diora key insert --chain diora_rococo --keystore-path ./keystore3 --suri "${SECRET3}" --scheme Sr25519 --key-type nmbs
