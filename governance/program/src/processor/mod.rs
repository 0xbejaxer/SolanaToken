//! Program processor

mod process_add_signatory;
mod process_cancel_proposal;
mod process_cast_vote;
mod process_create_account_governance;
mod process_create_mint_governance;
mod process_create_program_governance;
mod process_create_proposal;
mod process_create_realm;
mod process_create_token_governance;
mod process_deposit_governing_tokens;
mod process_execute_instruction;
mod process_finalize_vote;
mod process_flag_instruction_error;
mod process_insert_instruction;
mod process_relinquish_vote;
mod process_remove_instruction;
mod process_remove_signatory;
mod process_set_governance_config;
mod process_set_governance_delegate;
mod process_set_realm_authority;
mod process_set_realm_config;
mod process_sign_off_proposal;
mod process_withdraw_governing_tokens;

use crate::instruction::GovernanceInstruction;
use borsh::BorshDeserialize;

use process_add_signatory::*;
use process_cancel_proposal::*;
use process_cast_vote::*;
use process_create_account_governance::*;
use process_create_mint_governance::*;
use process_create_program_governance::*;
use process_create_proposal::*;
use process_create_realm::*;
use process_create_token_governance::*;
use process_deposit_governing_tokens::*;
use process_execute_instruction::*;
use process_finalize_vote::*;
use process_flag_instruction_error::*;
use process_insert_instruction::*;
use process_relinquish_vote::*;
use process_remove_instruction::*;
use process_remove_signatory::*;
use process_set_governance_config::*;
use process_set_governance_delegate::*;
use process_set_realm_authority::*;
use process_set_realm_config::*;
use process_sign_off_proposal::*;
use process_withdraw_governing_tokens::*;

use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

/// Processes an instruction
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    let instruction = GovernanceInstruction::try_from_slice(input)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    if let GovernanceInstruction::InsertInstruction {
        index,
        hold_up_time,
        instruction: _,
    } = instruction
    {
        // Do not dump instruction data into logs
        msg!(
            "GOVERNANCE-INSTRUCTION: InsertInstruction {{ index: {:?}, hold_up_time: {:?} }}",
            index,
            hold_up_time
        );
    } else {
        msg!("GOVERNANCE-INSTRUCTION: {:?}", instruction);
    }

    match instruction {
        GovernanceInstruction::CreateRealm { name, config_args } => {
            process_create_realm(program_id, accounts, name, config_args)
        }

        GovernanceInstruction::DepositGoverningTokens {} => {
            process_deposit_governing_tokens(program_id, accounts)
        }

        GovernanceInstruction::WithdrawGoverningTokens {} => {
            process_withdraw_governing_tokens(program_id, accounts)
        }

        GovernanceInstruction::SetGovernanceDelegate {
            new_governance_delegate,
        } => process_set_governance_delegate(program_id, accounts, &new_governance_delegate),

        GovernanceInstruction::CreateProgramGovernance {
            config,
            transfer_upgrade_authority,
        } => process_create_program_governance(
            program_id,
            accounts,
            config,
            transfer_upgrade_authority,
        ),

        GovernanceInstruction::CreateMintGovernance {
            config,
            transfer_mint_authority,
        } => process_create_mint_governance(program_id, accounts, config, transfer_mint_authority),

        GovernanceInstruction::CreateTokenGovernance {
            config,
            transfer_token_owner,
        } => process_create_token_governance(program_id, accounts, config, transfer_token_owner),

        GovernanceInstruction::CreateAccountGovernance { config } => {
            process_create_account_governance(program_id, accounts, config)
        }

        GovernanceInstruction::CreateProposal {
            name,
            description_link,
            governing_token_mint,
        } => process_create_proposal(
            program_id,
            accounts,
            name,
            description_link,
            governing_token_mint,
        ),
        GovernanceInstruction::AddSignatory { signatory } => {
            process_add_signatory(program_id, accounts, signatory)
        }
        GovernanceInstruction::RemoveSignatory { signatory } => {
            process_remove_signatory(program_id, accounts, signatory)
        }
        GovernanceInstruction::SignOffProposal {} => {
            process_sign_off_proposal(program_id, accounts)
        }
        GovernanceInstruction::CastVote { vote } => process_cast_vote(program_id, accounts, vote),

        GovernanceInstruction::FinalizeVote {} => process_finalize_vote(program_id, accounts),

        GovernanceInstruction::RelinquishVote {} => process_relinquish_vote(program_id, accounts),

        GovernanceInstruction::CancelProposal {} => process_cancel_proposal(program_id, accounts),

        GovernanceInstruction::InsertInstruction {
            index,
            hold_up_time,
            instruction,
        } => process_insert_instruction(program_id, accounts, index, hold_up_time, instruction),

        GovernanceInstruction::RemoveInstruction {} => {
            process_remove_instruction(program_id, accounts)
        }
        GovernanceInstruction::ExecuteInstruction {} => {
            process_execute_instruction(program_id, accounts)
        }

        GovernanceInstruction::SetGovernanceConfig { config } => {
            process_set_governance_config(program_id, accounts, config)
        }

        GovernanceInstruction::FlagInstructionError {} => {
            process_flag_instruction_error(program_id, accounts)
        }
        GovernanceInstruction::SetRealmAuthority {
            new_realm_authority,
        } => process_set_realm_authority(program_id, accounts, new_realm_authority),
        GovernanceInstruction::SetRealmConfig { config_args } => {
            process_set_realm_config(program_id, accounts, config_args)
        }
    }
}
