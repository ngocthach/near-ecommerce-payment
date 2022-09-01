use near_units::parse_near;
use serde_json::json;
use workspaces::prelude::*;
use workspaces::{network::Sandbox, testnet, sandbox, Account, Contract, Worker};

const WASM_FILEPATH: &str = "../../out/contract.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let worker = sandbox().await?;
  let wasm = std::fs::read(WASM_FILEPATH)?;
  let contract = worker.dev_deploy(&wasm).await?;

  // create accounts
  let owner = worker.root_account().unwrap();
  let user = owner
    .create_subaccount(&worker, "near-ecommerce-payment-contract")
    .initial_balance(parse_near!("30 N"))
    .transact()
    .await?
    .into_result()?;

  let ft_contract_id = String::from("ft.vbidev.testnet");

  let _create_contract = user
      .call(&worker, contract.id(), "new")
      .args_json(json!({"owner_id": owner.id(), "ft_contract_id": ft_contract_id}))?
      .transact()
      .await?
      .json()?;

  test_pay_order(&owner, &user, &contract, &worker).await?;

  Ok(())
}

async fn test_pay_order(
  caller: &Account,
  user: &Account,
  contract: &Contract,
  worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {

  // before transfer
  let contract_balance = user
      .view_account(&worker)
      .await?
      .balance;

  caller
    .call(&worker, contract.id(), "pay_order")
    .args_json(json!({"order_id": "order_1", "order_amount": "1000000000000000000000000"}))?
    .transact()
    .await?;

  // after transfer
  let new_contract_balance = user
      .view_account(&worker)
      .await?
      .balance;

  assert_eq!(new_contract_balance, contract_balance + 1000000000000000000000000);
  println!("Pay order successfully transferred to contract account {} ✅", contract.id());
  Ok(())
}
