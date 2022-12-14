use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, AccountId, near_bindgen, PanicOnDefault, BorshStorageKey,
               Promise, PromiseOrValue, PromiseResult, Gas, ext_contract};
use near_sdk::collections::{LookupMap};

mod order;
use order::{PaymentMethod, Order};
mod ft_contract;
use ft_contract::*;

pub type OrderId = String;
pub const TRANSFER_GAS: Gas = Gas(10_000_000_000_000); //

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
#[near_bindgen]
struct EcommerceContract {
    pub owner_id: AccountId,
    pub ft_contract_id: AccountId,
    pub orders: LookupMap<OrderId, Order>
}

#[derive(BorshDeserialize, BorshSerialize, BorshStorageKey)]
enum StorageKey {
    OrderKey
}


#[ext_contract(ext_self)]
pub trait ExtEcommerceContract {
    fn pay_order_callbacks(&mut self, order_id: OrderId) -> PromiseOrValue<U128>;
}


#[near_bindgen]
impl ExtEcommerceContract for EcommerceContract {
    #[private]
    fn pay_order_callbacks(&mut self, order_id: OrderId) -> PromiseOrValue<U128> {
        assert_eq!(env::promise_results_count(), 1, "ERROR_TOO_MANY_RESULTS");
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_value) => {
                PromiseOrValue::Value(U128(0))
            },
            PromiseResult::Failed => {
                // update refund status
                let mut order = self.get_order(order_id.clone());
                order.is_refund = false;
                self.orders.insert(&order_id, &order);
                PromiseOrValue::Value(U128(order.amount))
            }
        }
    }
}

/**
 * Storage account key/ value,  ["Vu Nguyen", "Vu nguyen 123", "Vu nguyen abc"] = names
 * {key: value, number: 1, name: "Vu Nguyen"}
 * {"names[0]": "Vu nguyen", "names[1]": "Vu nguyen 123", key: value, number: 1}
 */

#[near_bindgen]
impl EcommerceContract {
    #[init]
    pub fn new(owner_id: AccountId, ft_contract_id: AccountId) -> Self {
        Self { 
            owner_id,
            ft_contract_id,
            orders: LookupMap::new(StorageKey::OrderKey)
        }
    }

    #[payable]
    pub fn pay_order(&mut self, order_id: OrderId, order_amount: U128) -> PromiseOrValue<U128> {
        // Lay thong tin so NEAR deposit cua user env::attached_deposit()
        assert!(env::attached_deposit() >= order_amount.0, "ERROR_DEPOSIT_NOT_ENOUGH");

        // Luu tru lai thong tin thanh toan cua user
        let order: Order = Order { 
            order_id: order_id.clone(),
            payer_id: env::signer_account_id(),
            payment_method: PaymentMethod::Near,
            amount: order_amount.0,
            received_amount: env::attached_deposit(),
            is_completed: true,
            is_refund: false,
            created_at: env::block_timestamp()
        };

        self.orders.insert(&order_id, &order);

        // Tra lai tien thua cho user
        if env::attached_deposit() > order_amount.0 {
            Promise::new(env::signer_account_id()).transfer(env::attached_deposit() - order_amount.0);
            PromiseOrValue::Value(U128(env::attached_deposit() - order_amount.0))
        } else {
            PromiseOrValue::Value(U128(0))
        }
    }

    /*
     * Refund money to user
     * - Check requested account is owner or not
     * - Check order is completed and is refund
     * - Update order status and refund money to user
     */
    pub fn refund(&mut self, order_id: OrderId) -> PromiseOrValue<U128> {
        assert_eq!(env::predecessor_account_id(), self.owner_id);

        let mut order = self.get_order(order_id.clone());
        assert!(order.is_completed && !order.is_refund);
        order.is_refund = true;

        self.orders.insert(&order_id, &order);

        if order.amount > 0 {
            // Cross contract call
            match order.payment_method {
                PaymentMethod::Near => {
                    let promise = Promise::new(order.payer_id)
                        .transfer(order.amount)
                        .then(
                            ext_self::ext(env::current_account_id())
                                .with_attached_deposit(0)
                                .with_static_gas(TRANSFER_GAS)
                                .pay_order_callbacks(order_id)
                        );
                    PromiseOrValue::Promise(promise)
                }
                PaymentMethod::FungibleToken => {
                    let promise = ext_ft::ext(self.ft_contract_id.clone())
                        .with_attached_deposit(1)
                        .with_static_gas(TRANSFER_GAS)
                        .ft_transfer(order.payer_id, U128(order.amount), Some("Refund order from payment contract".to_owned()))
                        .then(
                            ext_self::ext(env::current_account_id())
                                .with_attached_deposit(0)
                                .with_static_gas(TRANSFER_GAS)
                                .pay_order_callbacks(order_id)
                        );
                    PromiseOrValue::Promise(promise)

                }
            }
        } else {
            PromiseOrValue::Value(U128(0))
        }
    }

    pub fn get_order(&self, order_id: OrderId) -> Order {
        self.orders.get(&order_id).expect("NOT_FOUND_ORDER_ID")
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use near_sdk::test_utils::{VMContextBuilder, accounts};
    use near_sdk::{testing_env, MockedBlockchain};

    fn get_context(is_view: bool) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
        .current_account_id(accounts(0))
        .signer_account_id(accounts(0))
        .predecessor_account_id(accounts(0))
        .is_view(is_view);

        builder
    }

    #[test]
    fn test_pay_order() {
        let mut context = get_context(false);
        let alice: AccountId = accounts(0);
        let ft_contract_id: AccountId = accounts(1);

        context.account_balance(1000)
        .predecessor_account_id(alice.clone())
        .attached_deposit(1000)
        .signer_account_id(alice.clone());

        testing_env!(context.build());

        let mut contract = EcommerceContract::new(alice.clone(), ft_contract_id.clone());
        let order_amount = U128(1000);
        contract.pay_order("order_1".to_owned(), order_amount);

        let order = contract.get_order("order_1".to_owned());

        // Test
        assert_eq!(order.order_id, "order_1".to_owned());
        assert_eq!(order.amount, order_amount.0);
        assert_eq!(order.payer_id, alice);
        assert!(order.is_completed);
    }

    #[test]
    #[should_panic(expected = "ERROR_DEPOSIT_NOT_ENOUGH")]
    fn test_pay_order_with_lack_balance() {
        let mut context = get_context(false);
        let alice: AccountId = accounts(0);
        let ft_contract_id: AccountId = accounts(1);

        context.account_balance(1000)
        .predecessor_account_id(alice.clone())
        .attached_deposit(1000)
        .signer_account_id(alice.clone());

        testing_env!(context.build());

        let mut contract = EcommerceContract::new(alice.clone(), ft_contract_id.clone());
        let order_amount = U128(2000);
        contract.pay_order("order_1".to_owned(), order_amount);
    }
}