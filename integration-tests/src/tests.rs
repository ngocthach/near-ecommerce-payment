use anyhow::Ok;
use near_units::parse_near;
use serde_json::{json};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{Balance, Timestamp, json_types::U128, AccountId};
use workspaces::prelude::*;
use workspaces::{network::Sandbox, Account, Contract, Worker};

const PAYMENT_CONTRACT_PATH: &str = "../contract/payment/out/contract.wasm";
const FT_TOKEN_PATH: &str = "../contract/ft/out/vbi-ft.wasm";

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct OrderDetail {
  pub order_id: AccountId,
  pub payer_id: AccountId,
  pub amount: Balance,
  pub received_amount: Balance,
  pub is_completed: bool,
  pub is_refund: bool,
  pub created_at: Timestamp
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let worker: Worker<Sandbox> = workspaces::sandbox().await?;
  // dev-deploy payment contract
  let payment_wasm = std::fs::read(PAYMENT_CONTRACT_PATH)?;
  let payment_contract: Contract = worker.dev_deploy(&payment_wasm).await?;
  // dev-deploy ft-token contract
  let ft_token_wasm = std::fs::read(FT_TOKEN_PATH)?;
  let ft_contract: Contract = worker.dev_deploy(&ft_token_wasm).await?;

  // create accounts
  // mainnet -> root account = near ex abc.near, xyz.near
  // testnet -> root account = testnet, ex: payment.testnet
  let owner: Account = worker.root_account().unwrap();
  let user: Account = owner
      .create_subaccount(&worker, "near-ecommerce-payment-contract")
      .initial_balance(parse_near!("30 N"))
      .transact()
      .await?
      .into_result()?;

  // Init contract
  ft_contract
      .call(&worker, "new_default_meta")
      .args_json(serde_json::json!({
            "owner_id": owner.id(),
            "total_supply": parse_near!("1,000,000,000 N").to_string(),
        }))?
      .transact()
      .await?;

  // Init contract
  payment_contract
      .call(&worker, "new")
      .args_json(serde_json::json!({
            "owner_id": owner.id(),
            "ft_contract_id": ft_contract.id(),
        }))?
      .transact()
      .await?;

  // Begin test
  test_pay_order(&user, &payment_contract, &worker).await?;

  Ok(())
}

async fn test_pay_order(
  user: &Account,
  contract: &Contract,
  worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
  let order_amount = parse_near!("1 N");

  // before transfer
  let _contract_balance = user
      .view_account(&worker)
      .await?
      .balance;

  user.
      call(&worker, contract.id(), "pay_order")
      .args_json(json!({
            "order_id": "order_1",
            "order_amount": U128(order_amount)
        }))?
      .deposit(order_amount)
      .transact()
      .await?;

  println!("      Passed ✅  pay_order");

  let res_order: OrderDetail = user.call(worker, contract.id(), "get_order")
      .args_json(json!({"order_id": "order_1"}))?
      .transact()
      .await?
      .json()?;

  // after transfer
  let _new_contract_balance = user
      .view_account(&worker)
      .await?
      .balance;

  assert_eq!(res_order.payer_id.to_string(), user.id().to_string());
  assert_eq!(res_order.amount, order_amount);
  println!("      Passed ✅  get_order");

  Ok(())
}
