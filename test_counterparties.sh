#!/bin/bash

cd ton_client
export TON_USE_SE=false

export TON_NETWORK_ADDRESS=of1.net.validators.tonlabs.io
cargo test counterparties -- --nocapture || exit

# export TON_NETWORK_ADDRESS=of5.net.validators.tonlabs.io
# cargo test counterparties -- --nocapture || exit

export TON_NETWORK_ADDRESS=of2.main.validators.tonlabs.io
cargo test counterparties -- --nocapture || exit

export TON_NETWORK_ADDRESS=of3.main.validators.tonlabs.io
cargo test counterparties -- --nocapture || exit

export TON_NETWORK_ADDRESS=of4.main.validators.tonlabs.io
cargo test counterparties -- --nocapture || exit
