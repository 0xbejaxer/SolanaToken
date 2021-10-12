use {
    bincode::deserialize,
    solana_account_decoder::UiAccountEncoding,
    solana_client::{
        client_error::ClientError,
        rpc_client::RpcClient,
        rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
        rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
    },
    solana_program::{borsh::try_from_slice_unchecked, program_pack::Pack, pubkey::Pubkey},
    spl_stake_pool::{
        stake_program,
        state::{StakePool, ValidatorList},
    },
};

type Error = Box<dyn std::error::Error>;

pub fn get_stake_pool(
    rpc_client: &RpcClient,
    stake_pool_address: &Pubkey,
) -> Result<StakePool, Error> {
    let account_data = rpc_client.get_account_data(stake_pool_address)?;
    let stake_pool = try_from_slice_unchecked::<StakePool>(account_data.as_slice())
        .map_err(|err| format!("Invalid stake pool {}: {}", stake_pool_address, err))?;
    Ok(stake_pool)
}

pub fn get_validator_list(
    rpc_client: &RpcClient,
    validator_list_address: &Pubkey,
) -> Result<ValidatorList, Error> {
    let account_data = rpc_client.get_account_data(validator_list_address)?;
    let validator_list = try_from_slice_unchecked::<ValidatorList>(account_data.as_slice())
        .map_err(|err| format!("Invalid validator list {}: {}", validator_list_address, err))?;
    Ok(validator_list)
}

pub fn get_token_account(
    rpc_client: &RpcClient,
    token_account_address: &Pubkey,
    expected_token_mint: &Pubkey,
) -> Result<spl_token::state::Account, Error> {
    let account_data = rpc_client.get_account_data(token_account_address)?;
    let token_account = spl_token::state::Account::unpack_from_slice(account_data.as_slice())
        .map_err(|err| format!("Invalid token account {}: {}", token_account_address, err))?;

    if token_account.mint != *expected_token_mint {
        Err(format!(
            "Invalid token mint for {}, expected mint is {}",
            token_account_address, expected_token_mint
        )
        .into())
    } else {
        Ok(token_account)
    }
}

pub fn get_token_mint(
    rpc_client: &RpcClient,
    token_mint_address: &Pubkey,
) -> Result<spl_token::state::Mint, Error> {
    let account_data = rpc_client.get_account_data(token_mint_address)?;
    let token_mint = spl_token::state::Mint::unpack_from_slice(account_data.as_slice())
        .map_err(|err| format!("Invalid token mint {}: {}", token_mint_address, err))?;

    Ok(token_mint)
}

pub(crate) fn get_stake_state(
    rpc_client: &RpcClient,
    stake_address: &Pubkey,
) -> Result<stake_program::StakeState, Error> {
    let account_data = rpc_client.get_account_data(stake_address)?;
    let stake_state = deserialize(account_data.as_slice())
        .map_err(|err| format!("Invalid stake account {}: {}", stake_address, err))?;
    Ok(stake_state)
}

pub(crate) fn get_stake_accounts_by_withdraw_authority(
    rpc_client: &RpcClient,
    withdraw_authority: &Pubkey,
) -> Result<Vec<(Pubkey, u64, stake_program::StakeState)>, ClientError> {
    rpc_client
        .get_program_accounts_with_config(
            &stake_program::id(),
            #[allow(clippy::needless_update)] // TODO: Remove after updating to solana >=1.6.10
            RpcProgramAccountsConfig {
                filters: Some(vec![RpcFilterType::Memcmp(Memcmp {
                    offset: 44, // 44 is Withdrawer authority offset in stake account stake
                    bytes: MemcmpEncodedBytes::Binary(format!("{}", withdraw_authority)),
                    encoding: None,
                })]),
                account_config: RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    ..RpcAccountInfoConfig::default()
                },
                ..RpcProgramAccountsConfig::default()
            },
        )
        .map(|accounts| {
            accounts
                .into_iter()
                .filter_map(
                    |(address, account)| match deserialize(account.data.as_slice()) {
                        Ok(stake_state) => Some((address, account.lamports, stake_state)),
                        Err(err) => {
                            eprintln!("Invalid stake account data for {}: {}", address, err);
                            None
                        }
                    },
                )
                .collect()
        })
}

pub(crate) fn get_stake_pools(
    rpc_client: &RpcClient,
) -> Result<Vec<(Pubkey, StakePool, ValidatorList)>, ClientError> {
    rpc_client
        .get_program_accounts_with_config(
            &spl_stake_pool::id(),
            RpcProgramAccountsConfig {
                filters: Some(vec![RpcFilterType::Memcmp(Memcmp {
                    offset: 0, // 0 is the account type
                    bytes: MemcmpEncodedBytes::Binary("2".to_string()),
                    encoding: None,
                })]),
                account_config: RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    ..RpcAccountInfoConfig::default()
                },
                ..RpcProgramAccountsConfig::default()
            },
        )
        .map(|accounts| {
            accounts
                .into_iter()
                .filter_map(|(address, account)| {
                    match try_from_slice_unchecked::<StakePool>(account.data.as_slice()) {
                        Ok(stake_pool) => {
                            get_validator_list(rpc_client, &stake_pool.validator_list)
                                .map(|v| (address, stake_pool, v))
                                .ok()
                        }
                        Err(err) => {
                            eprintln!("Invalid stake pool data for {}: {}", address, err);
                            None
                        }
                    }
                })
                .collect()
        })
}
