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
use near_sdk::serde::{Serialize, Deserialize};
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

#[derive(Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SerializableDeed {
    id: u64,
    author: AccountId,
    title: String,
    description: String,
    proof: String,
    creditors: u64,
    is_creditor: bool
}

impl SerializableDeed {
    pub fn new(
        id: u64,
        author: AccountId,
        title: String,
        description: String,
        proof: String,
        creditors: u64,
        is_creditor: bool
    ) -> Self{
        Self { id, author, title, description, proof, creditors, is_creditor }
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

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3C?xml version='1.0' encoding='utf-8'?%3E %3C!-- Svg Vector Icons : http://www.onlinewebfonts.com/icon --%3E %3C!DOCTYPE svg PUBLIC '-//W3C//DTD SVG 1.1//EN' 'http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd'%3E %3Csvg version='1.1' xmlns='http://www.w3.org/2000/svg' xmlns:xlink='http://www.w3.org/1999/xlink' x='0px' y='0px' viewBox='0 0 1000 1000' enable-background='new 0 0 1000 1000' xml:space='preserve'%3E %3Cmetadata%3E Svg Vector Icons : http://www.onlinewebfonts.com/icon %3C/metadata%3E %3Cg%3E%3Cg transform='translate(0.000000,511.000000) scale(0.100000,-0.100000)'%3E%3Cpath d='M4627.9,4997.8c-783.1-81.8-1539.6-415.1-2122.3-932.3c-472.3-419.1-848.5-977.3-1053-1564.1c-392.6-1128.6-241.3-2292,449.8-3451.3c300.6-503,697.2-1005.9,1543.7-1954.6c711.5-797.4,1220.6-1425.1,1443.5-1778.8c92-143.1,128.8-143.1,222.9,6.1c102.2,161.5,523.4,713.6,750.4,985.5c118.6,141.1,439.6,509.1,713.5,817.8c703.3,793.3,954.8,1095.9,1241.1,1494.6c707.4,989.6,1030.5,2040.5,922.1,3019.9c-184,1686.8-1441.4,3032.1-3103.7,3318.4C5361.9,5005.9,4881.4,5024.3,4627.9,4997.8z M4227.1,3073.8c206.5-42.9,433.4-169.7,609.3-341.4l161.5-157.4l165.6,159.5c253.5,241.3,535.7,361.9,848.5,361.9c639.9,0,1153.1-537.7,1155.2-1206.3c0-331.2-102.2-682.9-318.9-1104.1C6521.2,150,5961-424.5,5259.7-841.6c-120.6-71.6-237.2-130.9-261.7-130.9c-22.5,0-149.3,65.4-280.1,145.2C3785.5-263,3139.4,532.4,2894.1,1419.7c-71.6,253.5-71.6,642-2,856.7C3082.2,2853,3648.5,3192.4,4227.1,3073.8z'/%3E%3C/g%3E%3C/g%3E %3C/svg%3E";

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
                name: "A non-transferable social reputation token.".to_string(),
                symbol: "DEED".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                reference: None,
                reference_hash: None,
                decimals: 0,
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
        self.deeds.replace(id, &deed);
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
        let description = format!("{} donated {} NEAR to all users. Thank you very much!", &env::predecessor_account_id(), deposit);
        self.deeds.push(&SocialDeed::new(self.deeds.len(), env::predecessor_account_id(), title, description, "https://gifimage.net/wp-content/uploads/2017/10/donation-gif-10.gif".into()));
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
                Promise::new(donatable).transfer(share);
            }
        }
    }

    pub fn is_registered(self, account_id: AccountId) -> bool{
        self.token.accounts.contains_key(&account_id)
    }

    pub fn get_deeds_count(self) -> u64{
        self.deeds.len()
    }

    pub fn social_deeds(&self, creditor_id: AccountId, from_index: Option<U128>, limit: Option<u64>) -> Vec<SerializableDeed> {
        let start_index: u128 = from_index.map(From::from).unwrap_or_default();
        require!(
            (self.deeds.len() as u128) > start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        require!(limit != 0, "Cannot provide limit of 0.");
        self.deeds
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|deed| SerializableDeed::new(deed.id, deed.author, deed.title, deed.description, deed.proof, 
                                                           deed.creditors.len(), deed.creditors.contains(&creditor_id)))
            .collect()
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
        
        assert_eq!(get_logs(), ["Donated 0.9963700000000001 NEAR to bob."], "Expected a donation log.");
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
        
        assert_eq!(get_logs(), ["Donated 0.6642466666666667 NEAR to bob.", "Donated 0.3321233333333333 NEAR to fargo."], "Expected a donation log.");
    }
    

    #[test]
    fn test_creditors_calculation() {
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
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        // Paying for account registration, aka storage deposit
        let deeds = contract.social_deeds(accounts(5), None, Some(2u64));
        let deed = deeds.first().unwrap();
        
        //This is always 0 - probabaly a mistake on my side
        assert_eq!(deed.creditors, 2, "creditors should be counted correctly.");
    }
}