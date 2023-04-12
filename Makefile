# =================================================== Develop chain ===================================================

.PHONY: format
format:
	cargo fmt

.PHONY: check
check:
	cargo check

.PHONY: build-diora # Build diora runtime
build-diora:
	cargo build --release

# =================================================== Debug chain ===================================================

.PHONY: start-diora # Start diora chain
start-diora:
	./script/generate_config.sh $(chain)
	sleep 5
	./script/start_diora.sh $(chain)


.PHONY: start-diora-rococo # Start diora rococo chain
start-diora-rococo:
	./script/generate_key.sh diora_rococo
	sleep 2
	./script/generate_config.sh diora_rococo
	sleep 5
	./script/start_diora_rococo.sh diora_rococo

.PHONY: stop-diora # Stop the running chain
stop-diora:
	pkill polkadot
	pkill diora

.PHONY: restart-diora # Restart the running chain
restart-diora:
	pkill polkadot
	pkill diora
	sleep 3
	./script/start_diora.sh $(chain)

.PHONY: restart-diora-rococo # Restart the running diora_rococo chain
restart-diora-rococo:
	pkill polkadot
	pkill diora
	sleep 3
	./script/start_diora_rococo.sh diora_rococo

.PHONY: remove-chain # Stop the running chain and remove chain data
remove-chain:
	pkill polkadot
	pkill diora
	rm -rf script/config
	rm -rf script/data
	rm -rf script/keystores

# =================================================== Generate config ===============================================

.PHONY: build-chain-specs # build chain specs
build-chain-specs:
	target/release/diora build-spec --disable-default-bootnode --chain $(chain) > $(chain).json

.PHONY: generate-genesis-and-wasm # generate-genesis-and-wasm
generate-genesis-and-wasm:
	target/release/diora export-genesis-state --chain $(chain) > $(chain).genesis
	target/release/diora export-genesis-wasm --chain $(chain) > $(chain).wasm

.PHONY: help # generate list of targets with descriptions
help:
	@grep '^.PHONY: .* #' Makefile | sort | sed 's/\.PHONY: \(.*\) # \(.*\)/\1	\2/' | expand -t35