# Payment Smart contract

Application Design: [Ecommerce Payment user flow](https://drive.google.com/file/d/1ilBGG7hfkx7r6KzQy_6cEiHJqQlcSf6w/view?usp=sharing)

Prerequires
- NodeJS
- Near CLI
- Rust/Rustup and Wasm

Actions

1. Create new account in testnet
```
export CONTRACT_ID=near-ecommerce-payment-contract.ngocthach2020.testnet
export ACCOUNT_ID=ngocthach2020.testnet
near create-account $CONTRACT_ID --masterAccount $ACCOUNT_ID --initialBalance 5

near view ft.vbidev.testnet ft_balance_of '{"account_id":"ngocthach2020.testnet"}'
```

2. Build contract
```
cargo test & build.sh
```

3. Deploy and init contract
```
near deploy --wasmFile out/contract.wasm near-ecommerce-payment-contract.ngocthach2020.testnet --initFunction new --initArgs '{"owner_id": "ngocthach2020.testnet", "ft_contract_id": "ft.vbidev.testnet"}' --accountId ngocthach2020.testnet 
```

4. Pay order
```
near call ft.vbidev.testnet storage_deposit '{"account_id": "near-ecommerce-payment-contract.ngocthach2020.testnet"}' --accountId ngocthach2020.testnet --deposit 0.01

near call ft.vbidev.testnet ft_transfer_call '{"receiver_id": "near-ecommerce-payment-contract.ngocthach2020.testnet", "amount": "10000000000000000000000000000", "msg": "{\"order_id\": \"order_1\", \"order_amount\": \"1000000000000000000000000000\"}"}' --accountId ngocthach2020.testnet --depositYocto 1 --gas 50000000000000

near call $CONTRACT_ID pay_order '{"order_id": "order_1", "order_amount": "1000000000000000000000000"}' --accountId $ACCOUNT_ID --deposit 1
```

5. Get order

```
near view $CONTRACT_ID get_order '{"order_id": "order_1"}'
```

Ex response:
```
{
  order_id: 'order_1',
  payer_id: 'ngocthach2020.testnet',
  payment_method: 'FungibleToken',
  amount: 1e+27,
  received_amount: 1e+28,
  is_completed: true,
  is_refund: false,
  created_at: 1661876654547708200
}
```

6. Refund
```
near call $CONTRACT_ID refund '{"order_id": "order_1"}' --accountId $ACCOUNT_ID --gas 50000000000000

```

# Install cargo-watch to debug
- cargo install cargo-watch
- cargo watch -x check -x test -x run

# FT token
https://vbi-ui.vercel.app/faucet
https://github.com/nearvndev/vbi-ft