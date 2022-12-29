#!/bin/bash
mkdir data > /dev/null 2>&1 &

# parachain
nohup ./diora -d ./data/diora --bob --force-authoring --collator --discover-local --rpc-cors=all --ws-port 9944 --rpc-port 9933 --port 40041 --chain raw-diora-chainspec.json -llog=info -lruntime=debug,evm=trace --  raw-diora-chainspec.json --discover-local --port 40042 > data/log.2022 2>&1 &
