## Getting Started
```shell
git submodule update --init --recursive
cd deps/protocol-v1
anchor build
cd sdk
yarn
yarn build
cd ../../..
yarn
anchor test
```

## Warning
This code is unaudited. Use at your own risk.