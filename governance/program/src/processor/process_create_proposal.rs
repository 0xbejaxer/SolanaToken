//! Program state processor

use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

use crate::{
    error::GovernanceError,
    state::{
        enums::{GovernanceAccountType, InstructionExecutionFlags, ProposalState},
        governance::get_governance_data_for_realm,
        proposal::{get_proposal_address_seeds, Proposal},
        realm::get_realm_data_for_governing_token_mint,
        token_owner_record::get_token_owner_record_data_for_realm,
    },
    tools::account::create_and_serialize_account_signed,
};

/// Processes CreateProposal instruction
pub fn process_create_proposal(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
    description_link: String,
    governing_token_mint: Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let realm_info = next_account_info(account_info_iter)?; // 0
    let proposal_info = next_account_info(account_info_iter)?; // 1
    let governance_info = next_account_info(account_info_iter)?; // 2

    let proposal_owner_record_info = next_account_info(account_info_iter)?; // 3
    let governance_authority_info = next_account_info(account_info_iter)?; // 4

    let payer_info = next_account_info(account_info_iter)?; // 5
    let system_info = next_account_info(account_info_iter)?; // 6

    let rent_sysvar_info = next_account_info(account_info_iter)?; // 7
    let rent = &Rent::from_account_info(rent_sysvar_info)?;

    let clock_info = next_account_info(account_info_iter)?; // 8
    let clock = Clock::from_account_info(clock_info)?;

    if !proposal_info.data_is_empty() {
        return Err(GovernanceError::ProposalAlreadyExists.into());
    }

    let realm_data =
        get_realm_data_for_governing_token_mint(program_id, realm_info, &governing_token_mint)?;

    let mut governance_data =
        get_governance_data_for_realm(program_id, governance_info, realm_info.key)?;

    let mut proposal_owner_record_data = get_token_owner_record_data_for_realm(
        program_id,
        proposal_owner_record_info,
        realm_info.key,
    )?;

    // Proposal owner (TokenOwner) or its governance_delegate must sign this transaction
    proposal_owner_record_data
        .assert_token_owner_or_delegate_is_signer(governance_authority_info)?;

    // Ensure proposal owner (TokenOwner) has enough tokens to create proposal and no outstanding proposals
    proposal_owner_record_data.assert_can_create_proposal(&realm_data, &governance_data.config)?;

    proposal_owner_record_data.outstanding_proposal_count = proposal_owner_record_data
        .outstanding_proposal_count
        .checked_add(1)
        .unwrap();
    proposal_owner_record_data.serialize(&mut *proposal_owner_record_info.data.borrow_mut())?;

    let proposal_data = Proposal {
        account_type: GovernanceAccountType::Proposal,
        governance: *governance_info.key,
        governing_token_mint,
        state: ProposalState::Draft,
        token_owner_record: *proposal_owner_record_info.key,

        signatories_count: 0,
        signatories_signed_off_count: 0,

        name,
        description_link,

        draft_at: clock.unix_timestamp,
        signing_off_at: None,
        voting_at: None,
        voting_at_slot: None,
        voting_completed_at: None,
        executing_at: None,
        closed_at: None,

        instructions_executed_count: 0,
        instructions_count: 0,
        instructions_next_index: 0,

        execution_flags: InstructionExecutionFlags::None,

        yes_votes_count: 0,
        no_votes_count: 0,
        max_vote_weight: None,
        vote_threshold_percentage: None,
    };

    create_and_serialize_account_signed::<Proposal>(
        payer_info,
        proposal_info,
        &proposal_data,
        &get_proposal_address_seeds(
            governance_info.key,
            &governing_token_mint,
            &governance_data.proposals_count.to_le_bytes(),
        ),
        program_id,
        system_info,
        rent,
    )?;

    governance_data.proposals_count = governance_data.proposals_count.checked_add(1).unwrap();
    governance_data.serialize(&mut *governance_info.data.borrow_mut())?;

    Ok(())
}
