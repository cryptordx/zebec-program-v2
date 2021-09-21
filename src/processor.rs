//! Program state processor
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo,
    program_error::{PrintProgramError,ProgramError},
    decode_error::DecodeError,
    system_instruction::create_account,
};
use num_traits::FromPrimitive;
use solana_program::{
    account_info::{next_account_info},
    entrypoint::ProgramResult,
    program::{invoke},
    msg,
    pubkey::Pubkey,
    sysvar::{rent::Rent,fees::Fees,clock::Clock},
};
use std::str::FromStr;
use solana_program::sysvar::Sysvar;
use crate::{
    instruction::{TokenInstruction,ProcessInitializeStream,Processwithdrawstream,ProcessUsdcStream},
    state::{Escrow,TokenInitializeAccountParams, TokenTransferParams,Pause},
    error::TokenError,
    spl_utils::{spl_token_transfer,spl_initialize}
};
/// Program state handler.
pub struct Processor {}
impl Processor {
    /// Function to initilize a stream
    pub fn _process_initialize_stream(program_id: &Pubkey, accounts: &[AccountInfo], start_time: u64, end_time: u64, amount: u64) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let source_account_info = next_account_info(account_info_iter)?; 
        let dest_account_info = next_account_info(account_info_iter)?;
        let lock_account_info = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;
        let space_size = std::mem::size_of::<Escrow>() as u64;
        // Get the rent sysvar via syscall
        let rent = Rent::get()?; //
        // Since we are performing system_instruction source account must be signer
        if !source_account_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature); 
        }
        // current time in unix time
        let now = Clock::get()?.unix_timestamp as u64; 
        if now > end_time{
            msg!("End time is already passed");
            return Err(TokenError::TimeEnd.into());
        }
        let create_account_instruction = create_account(
            source_account_info.key,
            lock_account_info.key,
            amount + rent.minimum_balance(std::mem::size_of::<Escrow>()),
            space_size,
            program_id,
        );
        invoke(
            &create_account_instruction,
            &[
                source_account_info.clone(),
                lock_account_info.clone(),
                system_program.clone(),
            ])?;
        // Sending transaction fee to destination account, to call withdraw instruction. 
        let fees = Fees::get()?;
        msg!("Total Payment: {}",amount + rent.minimum_balance(std::mem::size_of::<Escrow>()));
        msg!("Total Fees Required: {}",fees.fee_calculator.lamports_per_signature * 2);
        **lock_account_info.lamports.borrow_mut() = lock_account_info
        .lamports()
        .checked_sub(fees.fee_calculator.lamports_per_signature * 2)
        .unwrap();
        **dest_account_info.lamports.borrow_mut() = dest_account_info
        .lamports()
        .checked_add(fees.fee_calculator.lamports_per_signature * 2)
        .unwrap();

        let mut escrow = Escrow::try_from_slice(&lock_account_info.data.borrow())?;
        escrow.start_time = start_time;
        escrow.end_time = end_time;
        escrow.paused = 0;
        escrow.sender = *source_account_info.key;
        escrow.recipient = *dest_account_info.key;
        escrow.amount = amount;
        escrow.escrow = *lock_account_info.key;
        msg!("{:?}",escrow);
        escrow.serialize(&mut &mut lock_account_info.data.borrow_mut()[..])?;
        // let mut pause = Pause::try_from_slice(&lock_account_info.data.borrow())?;
        // pause.paused = true;
        // pause.serialize(&mut &mut lock_account_info.data.borrow_mut()[..])?;
        Ok(())
    }
    pub fn _process_usdc_stream(program_id: &Pubkey, accounts: &[AccountInfo], start_time: u64, end_time: u64, amount: u64) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let source_account_info = next_account_info(account_info_iter)?; 
        let dest_account_info = next_account_info(account_info_iter)?;
        let lock_account_info = next_account_info(account_info_iter)?;
        let token_program_info = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;
        let program_info = next_account_info(account_info_iter)?;
        let stream_info = next_account_info(account_info_iter)?;
        let rent_info = next_account_info(account_info_iter)?;
        let space_size = std::mem::size_of::<Escrow>() as u64;
        // Get the rent sysvar via syscall
        let rent = Rent::get()?; //

        // Since we are performing system_instruction source account must be signer
        if !source_account_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature); 
        }
        // current time in unix time
        let now = Clock::get()?.unix_timestamp as u64; 
        if now > end_time{
            msg!("End time is already passed");
            return Err(TokenError::TimeEnd.into());
        }
        let create_account_instruction = create_account(
            source_account_info.key,
            lock_account_info.key,
            rent.minimum_balance(std::mem::size_of::<Escrow>()),
            space_size as u64,
            program_id,
        );
        invoke(
            &create_account_instruction,
            &[
                source_account_info.clone(),
                lock_account_info.clone(),
                system_program.clone(),
            ])?;
            
        spl_initialize(
            program_info,
            lock_account_info,
            source_account_info,
            stream_info,
            rent_info,
        )?;
        // spl_token_transfer(
        //     token_program_info,
        //     source_account_info,
        //     lock_account_info,
        //     stream_info,
        //     amount
        // )?;
        // Sending transaction fee to destination account, to call withdraw instruction. 
        let fees = Fees::get()?;
        msg!("Total Payment: {}",amount + rent.minimum_balance(std::mem::size_of::<Escrow>()));
        msg!("Total Fees Required: {}",fees.fee_calculator.lamports_per_signature * 2);
        **lock_account_info.lamports.borrow_mut() = lock_account_info
        .lamports()
        .checked_sub(fees.fee_calculator.lamports_per_signature * 2)
        .unwrap();
        **dest_account_info.lamports.borrow_mut() = dest_account_info
        .lamports()
        .checked_add(fees.fee_calculator.lamports_per_signature * 2)
        .unwrap();

        let mut escrow = Escrow::try_from_slice(&lock_account_info.data.borrow())?;
        let mut pause = Pause::try_from_slice(&lock_account_info.data.borrow())?;
        escrow.start_time = start_time;
        escrow.end_time = end_time;
        escrow.sender = *source_account_info.key;
        escrow.recipient = *dest_account_info.key;
        escrow.amount = amount;
        escrow.escrow = *lock_account_info.key;
        // escrow.paused = false; 
        msg!("{:?}",escrow);
        escrow.serialize(&mut &mut lock_account_info.data.borrow_mut()[..])?;
        pause.paused = true; 
        pause.serialize(&mut &mut lock_account_info.data.borrow_mut()[..])?;
        Ok(())
    }
    /// Function to withdraw from a stream
    pub fn _process_withdraw_stream(program_id: &Pubkey, accounts: &[AccountInfo],amount: u64) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let dest_account_info = next_account_info(account_info_iter)?;
        let locked_fund = next_account_info(account_info_iter)?;
        // let data = locked_fund.try_borrow_mut_data()?;
        let mut escrow = Escrow::try_from_slice(&locked_fund.data.borrow())?;
        let now = Clock::get()?.unix_timestamp as u64;
        msg!("{}",amount);

        // Recipient can only withdraw the money that is already streamed. 
        let mut allowed_amt = (((now - escrow.start_time) as f64) / ((escrow.end_time - escrow.start_time) as f64) * escrow.amount as f64) as u64;
        if now >= escrow.end_time {
            msg!("Stream has been successfully completed");
            allowed_amt = escrow.amount;
        }
        // let rent = &Rent::from_account_info(dest_account_info)?;
        msg!("{} allowed_amt",allowed_amt);
        if *dest_account_info.key != escrow.recipient {
            return Err(TokenError::EscrowMismatch.into());
        }
        if escrow.paused != 0 {
            return Err(TokenError::EscrowMismatch.into());
        }
        if amount>allowed_amt {
            msg!("{} is not yet streamlined.",amount);
            return Err(ProgramError::InsufficientFunds);
        }
        msg!("{}",amount);

        **locked_fund.try_borrow_mut_lamports()? = locked_fund
        .lamports()
        .checked_sub(amount)
        .unwrap();
        
        **dest_account_info.try_borrow_mut_lamports()? = dest_account_info
        .lamports()
        .checked_add(amount)
        .unwrap();

        escrow.amount = escrow.amount-amount;
        escrow.serialize(&mut &mut locked_fund.data.borrow_mut()[..])?;
        Ok(())
    }
     /// Function to cancel a stream
    pub fn _process_cancel_stream(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let source_account_info = next_account_info(account_info_iter)?;
        let dest_account_info = next_account_info(account_info_iter)?;
        let locked_fund = next_account_info(account_info_iter)?;
        let mut escrow = Escrow::try_from_slice(&locked_fund.data.borrow())?;
        let now = Clock::get()?.unix_timestamp as u64;
        // Amount that recipient should receive.  
        let allowed_amt = (((now - escrow.start_time) as f64) / ((escrow.end_time - escrow.start_time) as f64) * escrow.amount as f64) as u64;
       
        if now >= escrow.end_time {
            msg!("Stream already completed");
            return Err(TokenError::TimeEnd.into());
        }
        if *source_account_info.key != escrow.sender {
            return Err(TokenError::OwnerMismatch.into());
        }
        let dest_account_amount = escrow.amount-allowed_amt;
        let source_account_amount = escrow.amount-dest_account_amount;

        **locked_fund.try_borrow_mut_lamports()? = locked_fund
        .lamports()
        .checked_sub(escrow.amount)
        .unwrap();
        
        // Send unstreamed fund to the sender. 
        **source_account_info.try_borrow_mut_lamports()? = source_account_info
        .lamports()
        .checked_add(source_account_amount)
        .unwrap();

        // Send streamed fund to the recipient. 
        **dest_account_info.try_borrow_mut_lamports()? = dest_account_info
        .lamports()
        .checked_add(dest_account_amount)
        .unwrap();
        escrow.amount = 0;
        escrow.serialize(&mut &mut locked_fund.data.borrow_mut()[..])?;
        Ok(())
    }
    //Function to pause a stream
    pub fn _process_pause(accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let locked_fund = next_account_info(account_info_iter)?;
        let mut escrow = Escrow::try_from_slice(&locked_fund.data.borrow())?;
        let now = Clock::get()?.unix_timestamp as u64;
        if now >= escrow.end_time {
            msg!("End time is already passed");
            return Err(TokenError::TimeEnd.into());
        }
        escrow.paused = 1;
        escrow.serialize(&mut &mut locked_fund.data.borrow_mut()[..])?;
        Ok(())
    }
    pub fn _process_resume(accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let locked_fund = next_account_info(account_info_iter)?;
        let mut escrow = Escrow::try_from_slice(&locked_fund.data.borrow())?;
        escrow.paused = 0;
        escrow.start_time =  Clock::get()?.unix_timestamp as u64;
        escrow.serialize(&mut &mut locked_fund.data.borrow_mut()[..])?;
        Ok(())
    }
    /// Processes an [Instruction](enum.Instruction.html).
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = TokenInstruction::unpack(input)?;
        match instruction {
            TokenInstruction::ProcessInitializeStream (ProcessInitializeStream{
                start_time,
                end_time,
                amount,
            }) => {
                msg!("Instruction: Processing Stream V1.0");
                Self::_process_initialize_stream(program_id,accounts,start_time, end_time, amount)
            }
            TokenInstruction::Processwithdrawstream(Processwithdrawstream {
                amount,
            }) => {
                msg!("Instruction: Processing Withdraw V1.0");
                Self::_process_withdraw_stream(program_id,accounts, amount)
            }
            TokenInstruction::Processcancelstream => {
                msg!("Instruction: Processing cancel V1.0");
                Self::_process_cancel_stream(program_id,accounts)
            }
            TokenInstruction::ProcessUsdcStream(ProcessUsdcStream {
                start_time,
                end_time,
                amount,
            }) => {
                msg!("Instruction: Initializing USDC stream V1.0");
                Self::_process_usdc_stream(program_id,accounts,start_time, end_time, amount)
            }
            TokenInstruction::ProcessPauseStream => {
                msg!("Instruction: Pausing stream");
                Self::_process_pause(accounts)
            }
            TokenInstruction::ProcessResumeStream=> {
                msg!("Instruction: Resuming stream");
                Self::_process_resume(accounts)
            }
        }
    }
}
impl PrintProgramError for TokenError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            TokenError::TimeEnd => msg!("Error: Stream already completed"),
            TokenError::OwnerMismatch => msg!("Error: Owner does not match"),
            TokenError::NotRentExempt => msg!("Error: Lamport balance below rent-exempt threshold"),
            TokenError::EscrowMismatch => msg!("Error: Account not associated with this Escrow"),
            TokenError::InvalidInstruction => msg!("Error: Invalid instruction"),
            TokenError::AlreadyCancel => msg!("Error: Invalid instruction"),
        }
    }
}