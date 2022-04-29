@echo off

call near delete social-bounty2.cryptosketches.testnet cryptosketches.testnet
call near create-account social-bounty2.cryptosketches.testnet --masterAccount cryptosketches.testnet

call near deploy social-bounty2.cryptosketches.testnet .\res\social_bounty.wasm "new_default_meta" "{""owner_id"": ""social-bounty.cryptosketches.testnet"", ""total_supply"": ""1000000000000000""}"