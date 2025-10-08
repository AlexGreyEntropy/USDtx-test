// protocol-controller
// core instructions for cross-program coordination.
// work in progress!

use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    msg,
    clock::Clock,
    program::{invoke, invoke_signed},
    instruction::{AccountMeta, Instruction},
};

use crate::{
    error::*,
    state::*,
    math::*,
    oracle::*,
};
use bytemuck;




///initialize protocol controller with all program addresses
pub fn initialize_protocol(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> Result<(), ProgramError> {
    msg!("initializing protocol Controller");
    
    if accounts.len() < 10 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    
    if data.len() < 160 { // 5 program IDs * 32 bytes each
        return Err(ProtocolControllerError::ParameterValidationFailed.into());
    }
    
    //parse program addresses from instruction data
    // needs verificationn/testing for offset values
    let mut offset = 0;

  
    let usdtx_program_id = Pubkey::new_from_array(
        data[offset..offset+32].try_into().unwrap()
    );
    offset += 32;

  
    let thaler_program_id = Pubkey::new_from_array(
        data[offset..offset+32].try_into().unwrap()
    );
    offset += 32;


    let sol_strategy_program_id = Pubkey::new_from_array(
        data[offset..offset+32].try_into().unwrap()
    );
    offset += 32;

  
    let usdc_strategy_program_id = Pubkey::new_from_array(
        data[offset..offset+32].try_into().unwrap()
    );
    offset += 32;


  
    let magicblock_program_id = Pubkey::new_from_array(
        data[offset..offset+32].try_into().unwrap()
    );
    
    // now this would create the ProtocolController account with these addresses
    
    Ok(())
}




// update protocol parameters
pub fn update_protocol_parameters(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> Result<(), ProgramError> {
    msg!("updating protocol parameters");
    
    if data.len() < 8 {
        return Err(ProtocolControllerError::ParameterValidationFailed.into());
    }
    
    let parameter_type = data[0];
    let parameter_value = u64::from_le_bytes(data[1..9].try_into().unwrap());


  // matching correct parameter values, simplified
    match parameter_type {
        1 => msg!("updating minimum collateral ratio to: {}", parameter_value),
        2 => msg!("updating rebalance frequency to: {} seconds", parameter_value),
        3 => msg!("updating yield distribution frequency to: {} seconds", parameter_value),
        4 => msg!("updating emergency threshold to: {}", parameter_value),
        _ => return Err(ProtocolControllerError::ParameterValidationFailed.into()),
    }
    
    Ok(())
}



// emergency protocol pause.. manually but might change it to automatic
pub fn emergency_protocol_pause(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> Result<(), ProgramError> {
    msg!("pausing all protocol operations via CPIs");
    
    if data.len() < 4 {
        return Err(ProtocolControllerError::ParameterValidationFailed.into());
    }
    
    if accounts.len() < 6 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    
    let emergency_type = u32::from_le_bytes(data[0..4].try_into().unwrap());
    
    // get protocol controller PDA bump
    let protocol_controller_account = &accounts[0];
    let (_, protocol_controller_bump) = Pubkey::find_program_address(
        &[crate::constants::pda_seeds::PROTOCOL_CONTROLLER_SEED],
        &crate::ID,
    );


  
  // pause parameters... emergency type
    msg!("emergency type: {}", match emergency_type {
        1 => "low collateralization detected",
        2 => "oracle price deviation", 
        3 => "strategy failure cascade",
        4 => "liquidation threshold breached",
        5 => "manual override",
        _ => "unknown type",
    });


  
// pause for all program using CPIs
    crate::cpi::ProtocolCPI::invoke_emergency_pause(accounts, protocol_controller_bump)
        .map_err(|e| ProgramError::Custom(e as u32))?;


  
// update protocol controller state
    let mut controller_data = protocol_controller_account.try_borrow_mut_data()?;
    if controller_data.len() >= std::mem::size_of::<crate::state::ProtocolController>() {
        let controller_state = bytemuck::cast_mut::<crate::state::ProtocolController>(&mut controller_data);

        //emergency mode
        controller_state.is_paused = true;
        controller_state.emergency_mode = true;
        controller_state.last_emergency_action = pinocchio::clock::Clock::get()?.unix_timestamp;
        controller_state.emergency_override_count = controller_state.emergency_override_count.saturating_add(1);
        
        msg!("emergency override count: {}", controller_state.emergency_override_count);
        msg!("timestamp: {}", controller_state.last_emergency_action);

      
        //logs for emergency details
        match emergency_type {
            1 => {
                msg!("collateral ratio below emergency threshold");
            },
            2 => {
                msg!("oracle feeds excessive price deviation");
            },
            3 => {
                msg!("strategy manager {} failure or loss");
            },
            4 => {
                msg!("system is close to liquidation levels");
            },
            5 => {
                msg!("manual emergency pause");
            },
            _ => {
                msg!("emergency type {}", emergency_type);
            }
        }
    }
    
    msg!("all programs paused via CPIs");
    Ok(())
}


// cross-program coordination
// this coordinates operations between USDtx token, Thaler token, vaults and strategies
// batching instrusctions if needed


// coordinate mint across USDtx token, vaults, and strategies
pub fn coordinate_mint_operation(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> Result<(), ProgramError> {
    msg!("USDtx mint");
    
    if data.len() < 17 {
        return Err(ProtocolControllerError::ParameterValidationFailed.into());
    }
    
    let mint_amount = u64::from_le_bytes(data[0..8].try_into().unwrap());

  // 1=SOL, 2=USDC
    let collateral_type = data[8];
    let collateral_amount = u64::from_le_bytes(data[9..17].try_into().unwrap());

  
    ///work in progress
    // steps:
    // 1.validate solvency pre-mint
    // 2.call escrow program to accept collateral
    // 3.call strategy manager to deploy collateral
    // 4.call USDtx token program to mint tokens
    // 5.validate solvency post-mint
    
    Ok(())
}


// coordinate burn across all programs
pub fn coordinate_burn_operation(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> Result<(), ProgramError> {
    msg!("USDtx burn");
    
    if data.len() < 16 {
        return Err(ProtocolControllerError::ParameterValidationFailed.into());
    }
    
    let burn_amount = u64::from_le_bytes(data[0..8].try_into().unwrap());

  // 1=SOL, 2=USDC
    let redeem_type = data[8];
    let expected_collateral = u64::from_le_bytes(data[9..17].try_into().unwrap());

  
    ///work in progress
    // steps:
    // 1.call USDtx token program to burn tokens
    // 2.call strategy manager to withdraw collateral
    // 3.call escrow program to release collateral
    // 4.validate solvency post-burn
    
    Ok(())
}


// sync states accross programs
pub fn sync_vault_states(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> Result<(), ProgramError> {
    msg!("sync vault states");
    
    if data.len() < 1 {
        return Err(ProtocolControllerError::ParameterValidationFailed.into());
    }
    
    let sync_type = data[0];

  //types
    match sync_type {
        1 => msg!("sync collateral balances"),
        2 => msg!("sync yields accumulated"),
        3 => msg!("sync strategy allocations"),
        4 => msg!("full state sync"),
        _ => return Err(ProtocolControllerError::CoordinationOperationMismatch.into()),
    }
    //this would query all vault programs and states for consistency
    
    Ok(())
}



// full system solvency across all programs
pub fn validate_system_solvency(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> Result<(), ProgramError> {
    msg!("system solvency with CPIs");

  
    if data.len() < 8 {
        return Err(ProtocolControllerError::ParameterValidationFailed.into());
    }
    if accounts.len() < 5 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

  
    let min_collateral_ratio_bps = u16::from_le_bytes(data[0..2].try_into().unwrap());
    
    // protocol controller PDA bump
    let protocol_controller_account = &accounts[0];
    let (_, protocol_controller_bump) = Pubkey::find_program_address(
        &[crate::constants::pda_seeds::PROTOCOL_CONTROLLER_SEED],
        &crate::ID,
    );
    
    
    // invoke solvency checks for all vaults
    crate::cpi::ProtocolCPI::invoke_solvency_check(accounts, protocol_controller_bump)
        .map_err(|e| ProgramError::Custom(e as u32))?;
    
    //updated oracle prices
    let sol_price_usd = crate::cpi::ProtocolCPI::invoke_oracle_price_update(accounts, protocol_controller_bump)
        .map_err(|e| ProgramError::Custom(e as u32))?;
    
    //update protocol controller state with the solvency results
    let mut controller_data = protocol_controller_account.try_borrow_mut_data()?;
    if controller_data.len() >= std::mem::size_of::<crate::state::ProtocolController>() {
        let controller_state = bytemuck::cast_mut::<crate::state::ProtocolController>(&mut controller_data);
        
        // mathematical calculations with collateral data
        use crate::math::ProtocolMath;
        
        // these values would come from vault's CPIs
        let total_usdtx_supply = controller_state.total_usdtx_minted.saturating_sub(controller_state.total_usdtx_burned);
        let total_sol_collateral = controller_state.current_sol_tvl;
        let total_usdc_collateral = controller_state.current_usdc_tvl;
        
        let sol_value_usd = (total_sol_collateral as u128)
            .saturating_mul(sol_price_usd as u128)
            .saturating_div(1_000_000_000u128) as u64;
        
        let total_backing_value = sol_value_usd.saturating_add(total_usdc_collateral);
        
        // actual collateralization ratio using our math utilities
        let collateral_ratio = ProtocolMath::calculate_collateralization_ratio(
            total_usdtx_supply,
            total_backing_value,
        ).map_err(|e| ProgramError::Custom(e as u32))?;
        
        // updating state
        controller_state.global_collateral_ratio = collateral_ratio;
        controller_state.last_solvency_check = pinocchio::clock::Clock::get()?.unix_timestamp;

        msg!("check results");
        msg!("total USDtx supply: ${}", total_pusd_supply / 1_000_000);
        msg!("SOL collateral value: ${}", sol_value_usd / 1_000_000);
        msg!("USDC collateral: ${}", total_usdc_collateral / 1_000_000);
        msg!("total collateral value: ${}", total_backing_value / 1_000_000);
        msg!("collateral ratio: {:.2}%", collateral_ratio as u32 / 100.0);
      //should add more messages for the bellow cases
      //this check should be manually called to monitor the system
      //automatic checks to be implemented


      
///        //minimum ratio.. update emergency state
///        if collateral_ratio < min_collateral_ratio_bps {
///            msg!("collateral ratio below minimum");
///            controller_state.emergency_mode = true;
///            controller_state.last_emergency_action = pinocchio::clock::Clock::get()?.unix_timestamp;
///            return Err(ProtocolControllerError::InsufficientCollateralization.into());
///      
///      //1% threshold to minimum collateral ratio
///        } else if collateral_ratio < min_collateral_ratio_bps + 100 {
///            msg!("collateral ratio close to minimum");
///      
///        } else {
///            msg!("system healthy");
///            if controller_state.emergency_mode {
///                controller_state.emergency_mode = false;
///            }
///        }
///    }
    
    Ok(())
}


// harvesting yield from all strategies
// this is done with CPIs and based on timestamp or specific parameters
pub fn harvest_all_yield(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> Result<(), ProgramError> {
    msg!("yield harvest from all strategies via CPIs");
    
    if data.len() < 1 {
        return Err(ProtocolControllerError::ParameterValidationFailed.into());
    }
    
    if accounts.len() < 4 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    
    let harvest_mode = data[0];

  
    // protocol controller PDA bump
    let protocol_controller_account = &accounts[0];
    let (_, protocol_controller_bump) = Pubkey::find_program_address(
        &[crate::constants::pda_seeds::PROTOCOL_CONTROLLER_SEED],
        &crate::ID,
    );

  // yield harvest parameters
  // 3 different modes
    msg!("harvest mode: {}", match harvest_mode {
        1 => "SOL strategies only",
        2 => "USDC strategies only", 
        3 => "All strategies",
        _ => "Invalid mode",
    });
    
    if harvest_mode == 0 || harvest_mode > 3 {
        return Err(ProtocolControllerError::YieldHarvestingFailed.into());
    }
    
    //CPIs to strategy managers for harvesting the yields
    let total_yield_harvested = crate::cpi::ProtocolCPI::invoke_yield_harvesting(
        accounts, 
        protocol_controller_bump
    ).map_err(|e| ProgramError::Custom(e as u32))?;
    
    // update protocol controller state(with harvest results)
    let mut controller_data = protocol_controller_account.try_borrow_mut_data()?;
    if controller_data.len() >= std::mem::size_of::<crate::state::ProtocolController>() {
        let controller_state = bytemuck::cast_mut::<crate::state::ProtocolController>(&mut controller_data);
        
        controller_state.total_yield_harvested = controller_state.total_yield_harvested
            .saturating_add(total_yield_harvested);
        
        let current_time = pinocchio::clock::Clock::get()?.unix_timestamp;
        
    Ok(())
}


  
//distribute yields to stakers (through the thaler escrow)
pub fn distribute_yield_to_thaler(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> Result<(), ProgramError> {
    msg!("distributing yield");
    
    if data.len() < 16 {
        return Err(ProtocolControllerError::ParameterValidationFailed.into());
    }
    
    let total_yield_usdc = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let eligible_stakers = u32::from_le_bytes(data[8..12].try_into().unwrap());
    let treasury_fee_bps = u16::from_le_bytes(data[12..14].try_into().unwrap());


  
    //treasury fee (default 20%)
    let treasury_fee = (total_yield_usdc as u128)
        .saturating_mul(treasury_fee_bps as u128)
        .saturating_div(10_000u128) as u64;
    
    let distributable_yield = total_yield_usdc.saturating_sub(treasury_fee);
    
    //calculate Thalers to mint(each worth exactly $100 USDC)
    let thaler_tokens_to_mint = distributable_yield / 100_000_000;

    // steps:
    // 1.call Thaler program to mint new tokens
    // 2.call TWAB staking to distribute based on time-weighted balances
    // 3.update yields state
    
    Ok(())
}

//rebalance all strategy allocations
pub fn rebalance_all_strategies(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> Result<(), ProgramError> {
    msg!("Rebalancing all strategy allocations");
    
    if data.len() < 1 {
        return Err(ProtocolControllerError::ParameterValidationFailed.into());
    }
    
    let rebalance_trigger = data[0];
    
    match rebalance_trigger {
// work in progress
