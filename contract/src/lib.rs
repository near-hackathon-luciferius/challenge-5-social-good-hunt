/*!
Non-Fungible Token implementation with JSON serialization.
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
use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::{
    env, near_bindgen, require, AccountId, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue,
};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
}

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3C?xml version='1.0' encoding='utf-8'?%3E %3C!-- Svg Vector Icons : http://www.onlinewebfonts.com/icon --%3E %3C!DOCTYPE svg PUBLIC '-//W3C//DTD SVG 1.1//EN' 'http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd'%3E %3Csvg version='1.1' xmlns='http://www.w3.org/2000/svg' xmlns:xlink='http://www.w3.org/1999/xlink' x='0px' y='0px' viewBox='0 0 1000 1000' enable-background='new 0 0 1000 1000' xml:space='preserve'%3E %3Cmetadata%3E Svg Vector Icons : http://www.onlinewebfonts.com/icon %3C/metadata%3E %3Cg%3E%3Cg transform='translate(0.000000,511.000000) scale(0.100000,-0.100000)'%3E%3Cpath d='M9142.9,4429.6c-128.4-46-172.5-80.5-601.8-490.7c-237.6-226.2-440.8-414-450.4-417.8c-7.7-1.9-74.7,34.5-147.6,82.4c-172.5,115-693.8,371.8-933.4,460c-1249.6,458.1-2591.3,483-3731.7,72.8c-701.5-251.1-1357-655.5-1857.2-1144.2c-402.5-394.8-655.5-741.7-893.2-1228.6C-137.4,408.5-20.5-1126.7,834.3-2267.1c182.1-243.4,615.2-693.8,881.7-916.1c622.9-523.3,1282.2-864.4,1945.4-1006.2c291.3-63.3,833.7-63.3,1081-1.9c306.7,76.7,603.7,231.9,824.2,427.4c373.8,331.6,638.2,897,948.7,2016.3c157.1,567.3,245.3,751.3,479.2,998.6c153.3,161,295.2,266.4,672.7,509.8c419.7,268.3,609.5,415.9,854.8,661.2c322,322,536.7,690,611.4,1046.5c72.8,348.8,3.8,787.7-174.4,1105.9l-49.8,90.1l216.6,210.8c120.8,116.9,320.1,310.5,444.7,431.2c285.6,276,329.7,350.7,329.7,565.4c0,187.8-44.1,297.1-172.5,419.7C9578,4437.3,9326.9,4496.7,9142.9,4429.6z M9470.7,4105.7c120.8-80.5,164.8-233.8,99.7-352.7C9518.6,3663,5811.8,124.9,5733.2,90.4c-235.7-97.7-473.4,155.2-350.7,373.7c38.3,69,3735.5,3612.8,3814.1,3655C9275.2,4163.2,9394,4157.5,9470.7,4105.7z M5409.3,3825.9c153.3-59.4,304.7-201.3,383.3-362.3c57.5-116.9,63.2-141.8,63.2-310.5c0-168.7-5.7-193.6-63.2-310.5c-76.7-159.1-228.1-302.8-383.3-362.2c-84.3-34.5-147.6-44.1-281.7-46c-155.3,0-185.9,7.7-300.9,61.3c-297.1,139.9-460,440.8-415.9,764.7c38.3,277.9,268.3,530.9,548.2,601.8C5072,3889.1,5286.7,3871.9,5409.3,3825.9z M3188,2524.5c157.2-78.6,300.9-230,360.3-383.3c65.2-166.7,63.2-396.7-7.7-548.1c-107.3-230-272.2-366.1-517.5-425.5c-392.9-97.8-795.4,172.5-877.8,590.3c-53.7,268.3,95.8,594.2,339.2,736c139.9,82.4,231.9,103.5,412.1,97.7C3038.5,2585.8,3080.6,2576.3,3188,2524.5z M2020.7,613.6c216.6-57.5,391-203.2,486.8-408.2c44.1-93.9,51.8-134.2,51.8-291.3c0-161-5.7-197.4-55.6-300.9c-67.1-145.7-216.6-293.2-369.9-366.1c-95.8-44.1-136.1-51.7-293.2-51.7c-164.8,0-193.6,5.7-310.5,61.3c-155.2,72.8-279.8,195.5-354.6,346.9c-47.9,97.8-53.7,128.4-53.7,310.5c0,182.1,5.7,210.8,53.7,306.7c99.7,195.5,270.2,337.3,473.4,391C1786.9,648.1,1884.7,648.1,2020.7,613.6z M5137.2,25.2c26.8-42.1,90.1-109.2,139.9-145.7c105.4-74.8,105.4-84.3,24.9-264.5c-101.6-222.3-297.1-368-640.1-479.2c-208.9-67.1-231.9-80.5-325.8-174.4l-99.7-103.5l-23,53.7c-13.4,30.7-23,141.8-24.9,256.8c0,201.3,0,203.2,84.3,369.9c138,279.8,389.1,502.2,622.9,548.1C5058.6,119.1,5085.4,111.5,5137.2,25.2z M3168.8-1385.4c218.5-101.6,350.7-258.8,408.2-483c84.3-329.7-78.6-674.7-391-828c-118.8-59.4-139.9-63.2-320.1-61.3c-164.8,0-207,7.7-302.8,51.8c-295.2,136.1-477.2,477.2-415.9,785.8c57.5,289.4,270.2,517.5,550.1,588.4C2827.6-1299.2,3036.5-1322.2,3168.8-1385.4z'/%3E%3C/g%3E%3C/g%3E %3C/svg%3E";

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
}

#[near_bindgen]
impl Contract {
    /// Initializes the contract owned by `owner_id` with
    /// default metadata (for example purposes only).
    #[init]
    pub fn new_default_meta(owner_id: AccountId) -> Self {
        Self::new(
            owner_id,
            NFTContractMetadata {
                spec: NFT_METADATA_SPEC.to_string(),
                name: "NftArt Minted Artwork".to_string(),
                symbol: "NFTART".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                base_uri: None,
                reference: None,
                reference_hash: None,
            },
        )
    }

    #[init]
    pub fn new(owner_id: AccountId, metadata: NFTContractMetadata) -> Self {
        require!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        Self {
            tokens: NonFungibleToken::new(
                StorageKey::NonFungibleToken,
                owner_id,
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
        }
    }

    /// Mint a new token with ID=`token_id` belonging to `token_owner_id`.
    ///
    /// Since this example implements metadata, it also requires per-token metadata to be provided
    /// in this call. `self.tokens.mint` will also require it to be Some, since
    /// `StorageKey::TokenMetadata` was provided at initialization.
    ///
    /// `self.tokens.mint` will enforce `predecessor_account_id` to equal the `owner_id` given in
    /// initialization call to `new`.
    #[payable]
    pub fn nft_mint_with_id(
        &mut self,
        token_id: TokenId,
        token_owner_id: AccountId,
        token_metadata: TokenMetadata,
    ) -> Token {
        if let 
                TokenMetadata{
                    title: Some(_),
                    description: Some(_),
                    media: Some(_),
                    media_hash: Some(_),
                    copies: Some(1u64),
                    issued_at: Some(_),
                    extra: None,
                    ..
                } = token_metadata
        {
            let mut metadata = token_metadata;
            metadata.extra = Some("W10=".into());
            self.tokens.internal_mint(token_id, token_owner_id, Some(metadata))
        }
        else
        {
            panic!("Token metadata does not have all required fields.");
        }
    }

    /// Mint a new token with ID=`token_id` belonging to `token_owner_id`.
    ///
    /// Since this example implements metadata, it also requires per-token metadata to be provided
    /// in this call. `self.tokens.mint` will also require it to be Some, since
    /// `StorageKey::TokenMetadata` was provided at initialization.
    ///
    /// `self.tokens.mint` will enforce `predecessor_account_id` to equal the `owner_id` given in
    /// initialization call to `new`.
    #[payable]
    pub fn nft_mint(
        &mut self,
        token_owner_id: AccountId,
        token_metadata: TokenMetadata,
    ) -> Token {
        let total_supply: u128 = self.tokens.nft_total_supply().into();
        let token_id = (total_supply+1u128).to_string();
        self.nft_mint_with_id(token_id, token_owner_id, token_metadata)
    }
}

near_contract_standards::impl_non_fungible_token_core!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_approval!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_enumeration!(Contract, tokens);

#[near_bindgen]
impl NonFungibleTokenMetadataProvider for Contract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;
    use std::collections::HashMap;

    use super::*;

    const MINT_STORAGE_COST: u128 = 10000000000000000000000;

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    fn sample_token_metadata() -> TokenMetadata {    
        TokenMetadata {
            title: Some("Olympus Mons".into()),
            description: Some("The tallest mountain in the charted solar system".into()),
            media: Some("uri".into()),
            media_hash: Some("RTBEMDBDNjZGODk1RTlEOEEyMTQzNjUyRjlCMUJGNEQ1MEU2NjQxNEM0RUI5NDQzMzdGRTcwMTk5NDFEMjkzQQ==".as_bytes().to_vec().into()),
            copies: Some(1u64),
            issued_at: Some("1970-01-01T00:00:00+00:00".into()),
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: None,
            reference: None,
            reference_hash: None,
        }
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new_default_meta(accounts(1).into());
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.nft_token("100".to_string()), None);
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = Contract::default();
    }

    #[test]
    fn test_mint() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());

        let token_id = "1".to_string();
        let token = contract.nft_mint_with_id(token_id.clone(), accounts(0), sample_token_metadata());
        assert_eq!(token.token_id, token_id);
        assert_eq!(token.owner_id, accounts(0));
        let mut metadata = token.metadata.unwrap();
        metadata.extra = None;
        assert_eq!(metadata, sample_token_metadata());
        assert_eq!(token.approved_account_ids.unwrap(), HashMap::new());
    }

    #[test]
    fn test_transfer() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "2".to_string();
        contract.nft_mint_with_id(token_id.clone(), accounts(0), sample_token_metadata());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_transfer(accounts(1), token_id.clone(), None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        if let Some(token) = contract.nft_token(token_id.clone()) {
            assert_eq!(token.token_id, token_id);
            assert_eq!(token.owner_id, accounts(1));
            let mut metadata = token.metadata.unwrap();
            metadata.extra = None;
            assert_eq!(metadata, sample_token_metadata());
            assert_eq!(token.approved_account_ids.unwrap(), HashMap::new());
        } else {
            panic!("token not correctly created, or not found by nft_token");
        }
    }

    #[test]
    fn test_approve() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "3".to_string();
        contract.nft_mint_with_id(token_id.clone(), accounts(0), sample_token_metadata());

        // alice approves bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_approve(token_id.clone(), accounts(1), None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert!(contract.nft_is_approved(token_id.clone(), accounts(1), Some(1)));
    }

    #[test]
    fn test_revoke() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "4".to_string();
        contract.nft_mint_with_id(token_id.clone(), accounts(0), sample_token_metadata());

        // alice approves bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_approve(token_id.clone(), accounts(1), None);

        // alice revokes bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_revoke(token_id.clone(), accounts(1));
        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert!(!contract.nft_is_approved(token_id.clone(), accounts(1), None));
    }

    #[test]
    fn test_revoke_all() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "5".to_string();
        contract.nft_mint_with_id(token_id.clone(), accounts(0), sample_token_metadata());

        // alice approves bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_approve(token_id.clone(), accounts(1), None);

        // alice revokes bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_revoke_all(token_id.clone());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert!(!contract.nft_is_approved(token_id.clone(), accounts(1), Some(1)));
    }
}