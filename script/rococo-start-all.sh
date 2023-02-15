#!/bin/bash
mkdir data > /dev/null 2>&1 &

# relaychain

nohup ./polkadot --alice -d data/node1 --chain config/rococo-local-raw.json --validator  --ws-port 9955 --rpc-port 10025 --port 30033  --rpc-cors all  -lapproval_voting=trace,sync=debug,staking=trace,babe=trace --pruning archive  > data/log.alice 2>&1 &
nohup ./polkadot --bob -d data/node2 --chain config/rococo-local-raw.json --validator  --ws-port 9956 --rpc-port 10026 --port 30034  --rpc-cors all -lapproval_voting=trace > data/log.bob 2>&1 &
nohup ./polkadot --charlie -d data/node3 --chain config/rococo-local-raw.json --validator  --ws-port 9957 --rpc-port 10027 --port 30035  --rpc-cors all -lapproval_voting=trace > data/log.charlie 2>&1 &
nohup ./polkadot --dave -d data/node4 --chain config/rococo-local-raw.json --validator  --ws-port 9958 --rpc-port 10028 --port 300346 --rpc-cors all -lapproval_voting=trace > data/log.dave 2>&1 &

# parachain
nohup ./diora -d ./data/diora1 --keystore-path=./keystore1 --force-authoring --collator --discover-local --rpc-cors=all --ws-port 9944 --rpc-port 9933 --port 40041 --chain ./config/diora_rococo.json -llog=info -lruntime=debug,evm=trace --  --chain ./config/rococo-local-raw.json --discover-local --port 40042 > data/log.2022 2>&1 &
nohup ./diora -d ./data/diora2 --keystore-path=./keystore2 --force-authoring --collator --discover-local --rpc-cors=all --ws-port 9945 --rpc-port 9934 --port 40042 --chain ./config/diora_rococo.json -llog=info -lruntime=debug,evm=trace --  --chain ./config/rococo-local-raw.json --discover-local --port 40043 > data/log.2023 2>&1 &
nohup ./diora -d ./data/diora3 --keystore-path=./keystore3 --force-authoring --collator --discover-local --rpc-cors=all --ws-port 9946 --rpc-port 9935 --port 40043 --chain ./config/diora_rococo.json -llog=info -lruntime=debug,evm=trace --  --chain ./config/rococo-local-raw.json --discover-local --port 40044 > data/log.2024 2>&1 &

