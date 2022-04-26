/*!
Fungible Token implementation with JSON serialization.
NOTES:
  - The maximum balance value is limited by U128 (2**128 - 1).
  - JSON calls should pass U128 as a base-10 string. E.g. "100".
  - The contract optimizes the inner trie structure by hashing account IDs. It will prevent some
    abuse of deep tries. Shouldn't be an issue, once NEAR clients implement full hashing of keys.
  - The contract tracks the change in storage before and after the call. If the storage increases,
    the contract requires the caller of the contract to attach enough deposit to the function call
    to cover the storage cost.
    This is done to prevent a denial of service attack on the contract by taking all available storage.
    If the storage decreases, the contract will issue a refund for the cost of the released storage.
    The unused tokens from the attached deposit are also refunded, so it's safe to
    attach more deposit than required.
  - To prevent the deployed contract from being modified or deleted, it should not have any access
    keys on its account.
*/

use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_contract_standards::fungible_token::core::FungibleTokenCore;
use near_contract_standards::fungible_token::resolver::FungibleTokenResolver;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, UnorderedSet, Vector};
use near_sdk::json_types::U128;
use near_sdk::{env, log, near_bindgen, AccountId, Balance, PanicOnDefault, PromiseOrValue,
               require, Promise};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
    deeds: Vector<SocialDeed>,
    owner: AccountId,
    donatable_accounts: UnorderedSet<AccountId>
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct SocialDeed {
    id: u64,
    author: AccountId,
    title: String,
    description: String,
    proof: String,
    creditors: UnorderedSet<AccountId>
}

impl SocialDeed {
    pub fn new(
        id: u64,
        author: AccountId,
        title: String,
        description: String,
        proof: String
    ) -> Self{
        let str_prefix = id.to_string();
        let prefix = str_prefix.as_bytes();
        Self { id, author, title, description, proof, creditors: UnorderedSet::new(prefix) }
    }
}

pub fn refund_deposit_to_account(storage_used: u64, account_id: AccountId) {
    let required_cost = env::storage_byte_cost() * Balance::from(storage_used);
    let attached_deposit = env::attached_deposit();

    require!(
        required_cost <= attached_deposit,
        format!("Must attach {} yoctoNEAR to cover storage", required_cost)
    );

    let refund = attached_deposit - required_cost;
    if refund > 1 {
        Promise::new(account_id).transfer(refund);
    }
}

pub fn calculate_and_check_deposit(storage_used: u64) -> u128 {
    let required_cost = env::storage_byte_cost() * Balance::from(storage_used);
    let attached_deposit = env::attached_deposit();

    require!(
        required_cost <= attached_deposit,
        format!("Must attach {} yoctoNEAR to cover storage", required_cost)
    );

    attached_deposit - required_cost
}

/// Assumes that the precedecessor will be refunded
pub fn refund_deposit(storage_used: u64) {
    refund_deposit_to_account(storage_used, env::predecessor_account_id())
}

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E";

#[near_bindgen]
impl Contract {
    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// default metadata (for example purposes only).
    #[init]
    pub fn new_default_meta(owner_id: AccountId, total_supply: U128) -> Self {
        Self::new(
            owner_id,
            total_supply,
            FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "Example NEAR fungible token".to_string(),
                symbol: "EXAMPLE".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                reference: None,
                reference_hash: None,
                decimals: 24,
            },
        )
    }

    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// the given fungible token metadata.
    #[init]
    pub fn new(
        owner_id: AccountId,
        total_supply: U128,
        metadata: FungibleTokenMetadata,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        let mut this = Self {
            token: FungibleToken::new(b"a".to_vec()),
            metadata: LazyOption::new(b"m".to_vec(), Some(&metadata)),
            deeds: Vector::new(b"d".to_vec()),
            owner: owner_id.clone(),
            donatable_accounts: UnorderedSet::new(b"s".to_vec())
        };
        this.token.internal_register_account(&owner_id);
        this.token.internal_deposit(&owner_id, total_supply.into());
        near_contract_standards::fungible_token::events::FtMint {
            owner_id: &owner_id,
            amount: &total_supply,
            memo: Some("Initial tokens supply is minted"),
        }
        .emit();
        this
    }

    #[payable]
    pub fn credit(
        &mut self,
        id: u64
    )
    {
        let initial_storage_usage = env::storage_usage();

        assert!(self.deeds.len() > id, "The id is out of range.");
        let mut deed = self.deeds.get(id).unwrap();
        assert_ne!(env::predecessor_account_id(), deed.author, "You cannot credit yourself.");
        assert!(deed.creditors.insert(&env::predecessor_account_id()), "{} cannot credit the deed of {} again.", env::predecessor_account_id(), deed.author);
        let memo = Some(format!("Social deed of {} credited by {}", deed.author, env::predecessor_account_id().to_string()));
        self.token.internal_transfer(&self.owner, &deed.author, 1u128, memo);

        refund_deposit(env::storage_usage() - initial_storage_usage);
    }

    #[payable]
    pub fn add_deed(
        &mut self,
        author: AccountId,
        title: String,
        description: String,
        proof: String
    )
    {
        let initial_storage_usage = env::storage_usage();
        
        assert_eq!(author, env::predecessor_account_id(), "The author must be the same as the calling account.");
        self.deeds.push(&SocialDeed::new(self.deeds.len(), author.clone(), title, description, proof));
        self.donatable_accounts.insert(&author);

        refund_deposit(env::storage_usage() - initial_storage_usage);
    }

    #[payable]
    pub fn donate(
        &mut self
    )
    {
        let initial_storage_usage = env::storage_usage();
        
        let title = "Donation to all users".to_string();
        let deposit = (env::attached_deposit() as f64)/(10u128.pow(24) as f64);
        let description = format!("{} donated {} NEAR to al users. Thank you very much!", &env::predecessor_account_id(), deposit);
        self.deeds.push(&SocialDeed::new(self.deeds.len(), env::predecessor_account_id(), title, description, "https://explorer.testnet.near.org/accounts/social-bounty.cryptosketches.testnet".into()));
        self.donatable_accounts.insert(&env::predecessor_account_id());

        let remaining = calculate_and_check_deposit(env::storage_usage() - initial_storage_usage);
        let minted_amount = self.token.total_supply - Into::<u128>::into(self.token.ft_balance_of(self.owner.clone())) - Into::<u128>::into(self.token.ft_balance_of(env::predecessor_account_id()));
        for donatable in self.donatable_accounts.iter() {
            if donatable == env::predecessor_account_id() {
                continue;
            }
            let share : u128 = ((Into::<u128>::into(self.token.ft_balance_of(donatable.clone())) as f64)/(minted_amount as f64) * (remaining as f64)) as u128;
            if share > 10u128.pow(22){
                let donation = (share as f64)/(10u128.pow(24) as f64);
                env::log_str(format!("Donated {} NEAR to {}.", donation, donatable).as_str());
                Promise::new(env::current_account_id()).transfer(share);
            }
        }
    }

    fn on_account_closed(&mut self, account_id: AccountId, balance: Balance) {
        log!("Closed @{} with {}", account_id, balance);
    }

    fn on_tokens_burned(&mut self, account_id: AccountId, amount: Balance) {
        log!("Account @{} burned {}", account_id, amount);
    }
}

#[near_bindgen]
impl FungibleTokenCore for Contract {
    #[payable]
    fn ft_transfer(
        &mut self,
        _receiver_id: AccountId,
        _amount: U128,
        _memo: Option<String>,
    ) {
        panic!("This token is not transferable!")
        //self.token.ft_transfer(receiver_id, amount, memo)
    }

    #[payable]
    fn ft_transfer_call(
        &mut self,
        _receiver_id: AccountId,
        _amount: U128,
        _memo: Option<String>,
        _msg: String,
    ) -> PromiseOrValue<U128> {
        panic!("This token is not transferable!")
        //self.token.ft_transfer_call(receiver_id, amount, memo, msg)
    }

    fn ft_total_supply(&self) -> U128 {
        self.token.ft_total_supply()
    }

    fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        self.token.ft_balance_of(account_id)
    }
}

#[near_bindgen]
impl FungibleTokenResolver for Contract {
    #[private]
    fn ft_resolve_transfer(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128 {
        let (used_amount, burned_amount) =
            self.token.internal_ft_resolve_transfer(&sender_id, receiver_id, amount);
        if burned_amount > 0 {
            self.on_tokens_burned(sender_id, burned_amount);
        }
        used_amount.into()
    }
}

near_contract_standards::impl_fungible_token_storage!(Contract, token, on_account_closed);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder, get_logs};
    use near_sdk::{testing_env, Balance};

    use super::*;

    const TOTAL_SUPPLY: Balance = 1_000_000_000_000_000;
    const SAFE_STORAGE_COST: u128 = 10000000000000000000000;

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new_default_meta(accounts(1).into(), TOTAL_SUPPLY.into());
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.ft_total_supply().0, TOTAL_SUPPLY);
        assert_eq!(contract.ft_balance_of(accounts(1)).0, TOTAL_SUPPLY);
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = Contract::default();
    }

    #[test]
    #[should_panic(expected = "This token is not transferable!")]
    fn test_transfer() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(2).into(), TOTAL_SUPPLY.into());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(SAFE_STORAGE_COST)
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(2))
            .build());
        let transfer_amount = TOTAL_SUPPLY / 3;
        contract.ft_transfer(accounts(1), transfer_amount.into(), None);
    }

    #[test]
    #[should_panic]
    fn test_add_deed_panics_on_different_author() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(2).into(), TOTAL_SUPPLY.into());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(SAFE_STORAGE_COST)
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);

        contract.add_deed(accounts(2), "".into(), "".into(), "".into());
    }

    #[test]
    #[should_panic]
    fn test_add_deed_panics_without_attached_deposit() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(2).into(), TOTAL_SUPPLY.into());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(SAFE_STORAGE_COST)
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);
        
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(0)
            .predecessor_account_id(accounts(1))
            .build());

        contract.add_deed(accounts(1), "".into(), "".into(), "".into());
    }

    #[test]
    fn test_add_deed_adds_all_data() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(2).into(), TOTAL_SUPPLY.into());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(SAFE_STORAGE_COST)
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);

        contract.add_deed(accounts(1), "title".into(), "description".into(), "proof".into());

        let deed = contract.deeds.get(0).unwrap();
        assert_eq!(deed.author, accounts(1));
        assert_eq!(deed.title, "title");
        assert_eq!(deed.description, "description");
        assert_eq!(deed.proof, "proof");
    }

    #[test]
    fn test_add_deed_sets_id_correctly() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(2).into(), TOTAL_SUPPLY.into());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(SAFE_STORAGE_COST)
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);

        contract.add_deed(accounts(1), "title".into(), "description".into(), "proof".into());
        contract.add_deed(accounts(1), "title".into(), "description".into(), "proof".into());

        let deed = contract.deeds.get(0).unwrap();
        assert_eq!(deed.id, 0u64);

        let deed = contract.deeds.get(1).unwrap();
        assert_eq!(deed.id, 1u64);
    }

    #[test]
    #[should_panic]
    fn cannot_credit_deed_twice() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(2).into(), TOTAL_SUPPLY.into());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(SAFE_STORAGE_COST)
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);
        contract.add_deed(accounts(1), "title".into(), "description".into(), "proof".into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(SAFE_STORAGE_COST)
            .predecessor_account_id(accounts(3))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);
        contract.credit(0);
        contract.credit(0);
    }

    #[test]
    #[should_panic]
    fn cannot_credit_own_deed() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(2).into(), TOTAL_SUPPLY.into());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(SAFE_STORAGE_COST)
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);
        contract.add_deed(accounts(1), "title".into(), "description".into(), "proof".into());
        
        contract.credit(0);
    }

    #[test]
    fn test_credit_transfers_token_to_author() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(2).into(), TOTAL_SUPPLY.into());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(SAFE_STORAGE_COST)
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);
        contract.add_deed(accounts(1), "title".into(), "description".into(), "proof".into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(SAFE_STORAGE_COST)
            .predecessor_account_id(accounts(3))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);
        contract.credit(0);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert_eq!(contract.ft_balance_of(accounts(2)).0, (TOTAL_SUPPLY - 1));
        assert_eq!(contract.ft_balance_of(accounts(1)).0, 1);
        assert_eq!(contract.ft_balance_of(accounts(3)).0, 0);
    }
    

    #[test]
    fn test_donation_donated_to_single_account() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(2).into(), TOTAL_SUPPLY.into());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(SAFE_STORAGE_COST)
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);
        contract.add_deed(accounts(1), "title".into(), "description".into(), "proof".into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(SAFE_STORAGE_COST)
            .predecessor_account_id(accounts(3))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);
        contract.credit(0);

        
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(10u128.pow(24))
            .predecessor_account_id(accounts(4))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);
        contract.donate();
        
        assert_eq!(get_logs(), ["Donated 0.99626 NEAR to bob."], "Expected a donation log.");
    }
    

    #[test]
    fn test_donation_donated_to_two_accounts() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(2).into(), TOTAL_SUPPLY.into());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(SAFE_STORAGE_COST)
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);
        contract.add_deed(accounts(1), "title".into(), "description".into(), "proof".into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(SAFE_STORAGE_COST)
            .predecessor_account_id(accounts(5))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);
        contract.add_deed(accounts(5), "title".into(), "description".into(), "proof".into());
        contract.credit(0);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(SAFE_STORAGE_COST)
            .predecessor_account_id(accounts(3))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);
        contract.credit(0);
        contract.credit(1);

        
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(10u128.pow(24))
            .predecessor_account_id(accounts(4))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);
        contract.donate();
        
        assert_eq!(get_logs(), ["Donated 0.6641733333333333 NEAR to bob.", "Donated 0.33208666666666664 NEAR to fargo."], "Expected a donation log.");
    }
}