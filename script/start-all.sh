#!/bin/bash
mkdir data > /dev/null 2>&1 &

# relaychain

nohup ./polkadot --alice -d data/node1 --chain config/rococo-local-raw.json --validator  --ws-port 9955 --rpc-port 10025 --port 30033  --rpc-cors all  -lapproval_voting=trace,sync=debug,staking=trace,babe=trace --pruning archive  > data/log.alice 2>&1 &
nohup ./polkadot --bob   -d data/node2 --chain config/rococo-local-raw.json --validator  --ws-port 9946 --rpc-port 10026 --port 30034  --rpc-cors all -lapproval_voting=trace > data/log.bob 2>&1 &

# parachain
nohup ./diora -d ./data/diora1 --alice --force-authoring --collator --discover-local --rpc-cors=all --ws-port 9944 --rpc-port 9933 --port 40041 --chain ./config/diora_local.json -llog=info -lruntime=debug,evm=trace --  --chain ./config/rococo-local-raw.json --discover-local --port 40042 > data/log.2022 2>&1 &
nohup ./diora -d ./data/diora2 --bob --force-authoring --collator --discover-local --rpc-cors=all --ws-port 9945 --rpc-port 9934 --port 40042 --chain ./config/diora_local.json -llog=info -lruntime=debug,evm=trace --  --chain ./config/rococo-local-raw.json --discover-local --port 40043 > data/log.2023 2>&1 &

