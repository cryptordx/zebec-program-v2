cargo build-bpf   
solana program deploy $(pwd)target/deploy/the_stream.so
node index.js usdc