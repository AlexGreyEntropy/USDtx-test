// Protocol Controller program
// central (master) program for the USDtx/Thaler dual-escrow stablecoin system
//coordinates between USDtx token, Thaler tokens, and dual vaults to maintain mathematical solvency and yield distribution

// main responsibilities are cross-program coordination and state sync, solvency monitor, yield harvesting and distribution
// this should handle the emergency triggers and circuit breakers, as well oracle price aggregation

use pinocchio::{
    account_info::AccountInfo,
    entrypoint,
    program_error::ProgramError,
    pubkey::Pubkey,
    msg,
};

pinocchio_pubkey::declare_id!("Protocol7er9K8kZyb4hV2XgJcNpHv3WrJ9mCdE1fA2bC");

mod instructions;
mod state;
mod error;
mod math;
mod oracle;
mod doppler_oracle;
mod constants;
mod cpi;
mod dynamic_fees;
mod user_mint_pda;
mod shared;
mod master_authority;

pub use instructions::*;
pub use state::*;
pub use error::*;
pub use math::*;
pub use oracle::*;
pub use doppler_oracle::*;
pub use dynamic_fees::*;
pub use user_mint_pda::*;
pub use shared::*;
pub use master_authority::*;

entrypoint!(process_instruction);

///main program instruction processor
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> Result<(), ProgramError> {
    if instruction_data.is_empty() {
        return Err(ProgramError::InvalidInstructionData);
    }
    
    match instruction_data[0] {
      
        //batch instruction discriminator (work in progress)
        255 => process_batch_instructions(program_id, accounts, &instruction_data[1..]),

      
        // initialization, update and pause
        0 => instructions::initialize_protocol(program_id, accounts, &instruction_data[1..]),
        1 => instructions::update_protocol_parameters(program_id, accounts, &instruction_data[1..]),
        2 => instructions::emergency_protocol_pause(program_id, accounts, &instruction_data[1..]),
        
        // cross-program
        10 => instructions::coordinate_usdtx_freeze_for_yield(program_id, accounts, &instruction_data[1..]),
        11 => instructions::coordinate_usdtx_unfreeze_from_yield(program_id, accounts, &instruction_data[1..]),
        12 => instructions::sync_freeze_states_across_programs(program_id, accounts, &instruction_data[1..]),
        13 => instructions::validate_system_solvency_freeze_based(program_id, accounts, &instruction_data[1..]),
        
        // orchestrate workflow for SOL strategies
        14 => instructions::orchestrate_sol_to_usdtx_workflow(program_id, accounts, &instruction_data[1..]),
        15 => instructions::coordinate_sol_strategy_deployment(program_id, accounts, &instruction_data[1..]),
        16 => instructions::collect_drift_usdc_yields_to_vault(program_id, accounts, &instruction_data[1..]),
        
        // orchestrate workflow for USDC strategies  
        17 => instructions::orchestrate_usdc_to_usdtx_workflow(program_id, accounts, &instruction_data[1..]),
        18 => instructions::coordinate_usdc_strategy_deployment(program_id, accounts, &instruction_data[1..]),
        19 => instructions::collect_kamino_usdc_yields_to_vault(program_id, accounts, &instruction_data[1..]),
        
        // yields vault (USDC)
        20 => instructions::coordinate_yields_vault_collection(program_id, accounts, &instruction_data[1..]),
        21 => instructions::distribute_yields_to_thaler_freezers(program_id, accounts, &instruction_data[1..]),
        22 => instructions::rebalance_all_strategies_with_vault(program_id, accounts, &instruction_data[1..]),
        23 => instructions::optimize_freeze_based_yield_allocation(program_id, accounts, &instruction_data[1..]),
        
        // oracle price aggregation (doppler)
        30 => instructions::aggregate_oracle_prices(program_id, accounts, &instruction_data[1..]),
        31 => instructions::validate_price_deviations(program_id, accounts, &instruction_data[1..]),
        32 => instructions::update_collateral_ratios(program_id, accounts, &instruction_data[1..]),
        
        // dynamic fees
        33 => instructions::update_tvl_data_and_dynamic_fees(program_id, accounts, &instruction_data[1..]),
        
        // oracles init, update and price fetch
        34 => instructions::initialize_doppler_oracle(program_id, accounts, &instruction_data[1..]),
        35 => instructions::update_switchboard_price(program_id, accounts, &instruction_data[1..]),
        36 => instructions::update_pyth_price(program_id, accounts, &instruction_data[1..]),
        37 => instructions::update_chainlink_price(program_id, accounts, &instruction_data[1..]),
        38 => instructions::get_doppler_aggregated_price(program_id, accounts, &instruction_data[1..]),
        
        // mint/redeem PDAs and merchant redemption
        39 => instructions::create_unique_mint_pda(program_id, accounts, &instruction_data[1..]),
        40 => instructions::validate_redemption_against_mint_pda(program_id, accounts, &instruction_data[1..]),
        41 => instructions::register_merchant_pda(program_id, accounts, &instruction_data[1..]),
        42 => instructions::process_merchant_redemption(program_id, accounts, &instruction_data[1..]),
        
        // monitors, circuit breakers and triggers
        45 => instructions::monitor_protocol_health(program_id, accounts, &instruction_data[1..]),
        46 => instructions::execute_emergency_procedures(program_id, accounts, &instruction_data[1..]),
        47 => instructions::liquidation_trigger(program_id, accounts, &instruction_data[1..]),
        
        // strategy management
        50 => instructions::whitelist_strategy(program_id, accounts, &instruction_data[1..]),
        51 => instructions::update_strategy_weights(program_id, accounts, &instruction_data[1..]),
        52 => instructions::pause_strategy(program_id, accounts, &instruction_data[1..]),
        
        // master overrides
        60 => master_authority::emergency_pause_all_programs(program_id, accounts, &instruction_data[1..]),
        61 => master_authority::emergency_recall_all_external_assets(program_id, accounts, &instruction_data[1..]),
        62 => master_authority::master_authority_override_program_config(program_id, accounts, &instruction_data[1..]),
        63 => master_authority::master_authority_update_all_dynamic_fees(program_id, accounts, &instruction_data[1..]),
        64 => master_authority::master_authority_emergency_circuit_breaker(program_id, accounts, &instruction_data[1..]),
        65 => master_authority::master_authority_resume_protocol_operations(program_id, accounts, &instruction_data[1..]),
        
        _ => {
            msg!("Unknown protocol controller instruction: {}", instruction_data[0]);
            Err(ProgramError::InvalidInstructionData)
        }
    }
}

// batch instructions for cross-program using discriminator 255
pub fn process_batch_instructions(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> Result<(), ProgramError> {
    if instruction_data.is_empty() {
        msg!("Empty batch instruction data");
        return Err(ProgramError::InvalidInstructionData);
    }
    
    // first byte is the number of instructions in the batch
    let instruction_count = instruction_data[0] as usize;


    // batch min size limit
    if instruction_count == 0 || instruction_count > 10 { 
        msg!("Invalid batch instruction count: {}", instruction_count);
        return Err(ProgramError::InvalidInstructionData);
    }
    
    msg!("processing batch of {} instructions", instruction_count);

  
    //skipping count byte
    let mut data_offset = 1;
    
    for i in 0..instruction_count {
        if data_offset >= instruction_data.len() {
            msg!("Batch instruction {} out of bounds", i);
            return Err(ProgramError::InvalidInstructionData);
        }
        
    // reading 2-byte header.. basically account count and data length
        if data_offset + 2 > instruction_data.len() {
            msg!("Insufficient data for batch instruction {} header", i);
            return Err(ProgramError::InvalidInstructionData);
        }
        
        let account_count = instruction_data[data_offset] as usize;
        let data_length = instruction_data[data_offset + 1] as usize;
        data_offset += 2;
        
    // skipping account indices (using all accounts for simplicity)
        data_offset += account_count;
        
        if data_offset + data_length > instruction_data.len() {
            msg!("Insufficient data for batch instruction {} payload", i);
            return Err(ProgramError::InvalidInstructionData);
        }

      
        // getting the exact instruction data slice
        let inner_instruction_data = &instruction_data[data_offset..data_offset + data_length];
        data_offset += data_length;

      
        
    //process the inner instruction (as a recursive call but with single instruction)
        if !inner_instruction_data.is_empty() && inner_instruction_data[0] != 255 {

          
            // this would prevent nested batch instructions
            match inner_instruction_data[0] {
              
                0 => instructions::initialize_protocol(program_id, accounts, &inner_instruction_data[1..])?,
                1 => instructions::update_protocol_parameters(program_id, accounts, &inner_instruction_data[1..])?,
                10 => instructions::coordinate_usdtx_freeze_for_yield(program_id, accounts, &inner_instruction_data[1..])?,
                11 => instructions::coordinate_usdtx_unfreeze_from_yield(program_id, accounts, &inner_instruction_data[1..])?,
                14 => instructions::orchestrate_sol_to_usdtx_workflow(program_id, accounts, &inner_instruction_data[1..])?,
                17 => instructions::orchestrate_usdc_to_usdtx_workflow(program_id, accounts, &inner_instruction_data[1..])?,
                20 => instructions::coordinate_yields_vault_collection(program_id, accounts, &inner_instruction_data[1..])?,
                21 => instructions::distribute_yields_to_thaler_freezers(program_id, accounts, &inner_instruction_data[1..])?,
                30 => instructions::aggregate_oracle_prices(program_id, accounts, &inner_instruction_data[1..])?,
                33 => instructions::update_tvl_data_and_dynamic_fees(program_id, accounts, &inner_instruction_data[1..])?,
                39 => instructions::create_unique_mint_pda(program_id, accounts, &inner_instruction_data[1..])?,
                40 => instructions::validate_redemption_against_mint_pda(program_id, accounts, &inner_instruction_data[1..])?,
                45 => instructions::monitor_protocol_health(program_id, accounts, &inner_instruction_data[1..])?,
                50 => instructions::whitelist_strategy(program_id, accounts, &inner_instruction_data[1..])?,
                _ => {
                    msg!("Unknown batch instruction discriminator: {}", inner_instruction_data[0]);
                    return Err(ProgramError::InvalidInstructionData);
                }
            }
            msg!("batch instruction {} completed", i);;

          
        } else {
            msg!("invalid batch instruction {} - nested batching not allowed", i);
            return Err(ProgramError::InvalidInstructionData);
        }
    }
    
    msg!("batch processing successful - {} instructions executed", instruction_count);
    Ok(())
}



// removing allocator.. using custom panic handler

pinocchio::no_allocator!();
pinocchio::nostd_panic_handler!();

}
