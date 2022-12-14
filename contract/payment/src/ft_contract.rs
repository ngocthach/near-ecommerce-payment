use near_sdk::{AccountId, PromiseOrValue, Promise, env, serde_json, ext_contract, near_bindgen};
use near_sdk::serde::{Serialize, Deserialize};
use crate::{Order, PaymentMethod, OrderId, U128, EcommerceContract, EcommerceContractExt};


pub trait FungibleTokenReceiver {
    fn ft_on_transfer(&mut self, sender_id: AccountId, amount: U128, msg: String) -> PromiseOrValue<U128>;
}

#[ext_contract(ext_ft)]
pub trait FungibleToken {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) -> Promise;
}

#[derive(Deserialize, Serialize)]
#[serde(crate="near_sdk::serde")]
struct FTMessage {
    order_id: OrderId,
    order_amount: U128,
}

#[near_bindgen]
impl FungibleTokenReceiver for EcommerceContract {
    fn ft_on_transfer(&mut self, sender_id: AccountId, amount: U128, msg: String) -> PromiseOrValue<U128> {
        assert_eq!(env::predecessor_account_id(), self.ft_contract_id);
        let FTMessage {order_id, order_amount} = serde_json::from_str(&msg).expect("ERROR_NOT_VALID_MESSAGE");

        assert!(amount.0 >= order_amount.0);

        // check order status: completed or not completed
        match self.orders.get(&order_id) {
            Some(order) => {
                assert!(!order.is_completed)
            },
            None => {}
        }

        // save order information of user
        let order: Order = Order {
            order_id: order_id.clone(),
            payer_id: sender_id,
            payment_method: PaymentMethod::FungibleToken,
            amount: order_amount.0,
            received_amount: amount.0,
            is_completed: true,
            is_refund: false,
            created_at: env::block_timestamp(),
        };

        self.orders.insert(&order_id, &order);

        // return balance to user
        if amount.0 > order_amount.0 {
            PromiseOrValue::Value(U128(amount.0 - order_amount.0))
        } else {
            PromiseOrValue::Value(U128(0))
        }
    }
}