//! Instruction types
use solana_program::{
    program_error::ProgramError,
    program::{invoke},
    entrypoint::ProgramResult,
};

use crate::{
    error::TokenError,
    state::{Escrow,TokenInitializeAccountParams, TokenTransferParams},

};
use std::convert::TryInto;

/// Initialize stream data
pub struct ProcessInitializeStream{
    pub start_time: u64,
    pub end_time: u64,
    pub amount: u64,
}
/// Initialize usdc stream data
pub struct ProcessUsdcStream{
    pub start_time: u64,
    pub end_time: u64,
    pub amount: u64,
}
pub struct Processwithdrawstream{
    /// Amount of funds locked
    pub amount: u64,
}

pub enum TokenInstruction {
    ProcessInitializeStream(ProcessInitializeStream),
    Processwithdrawstream(Processwithdrawstream),
    Processcancelstream ,
    ProcessUsdcStream(ProcessUsdcStream),
    ProcessPauseStream,
    ProcessResumeStream
}
impl TokenInstruction {
    /// Unpacks a byte buffer into a [TokenInstruction](enum.TokenInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        use TokenError::InvalidInstruction;
        let (&tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            // Initialize stream instruction 
            0 => {
                let (start_time, rest) = rest.split_at(8);
                let (end_time, rest) = rest.split_at(8);
                let (amount, _rest) = rest.split_at(8);
                let start_time = start_time.try_into().ok().map(u64::from_le_bytes).ok_or(InvalidInstruction)?;
                let end_time = end_time.try_into().ok().map(u64::from_le_bytes).ok_or(InvalidInstruction)?;
                let amount = amount.try_into().ok().map(u64::from_le_bytes).ok_or(InvalidInstruction)?;
                Self::ProcessInitializeStream (ProcessInitializeStream{start_time,end_time,amount})
            }
            // Withdraw stream instruction 
            1 => {
                let (amount, _rest) = rest.split_at(8);
                let amount = amount.try_into().ok().map(u64::from_le_bytes).ok_or(InvalidInstruction)?;
                Self::Processwithdrawstream (Processwithdrawstream{amount})
            }
            // Cancel stream instruction 
            2 => {
                Self:: Processcancelstream
            }
             // Initialize usdc stream 
             3 => {
                let (start_time, rest) = rest.split_at(8);
                let (end_time, rest) = rest.split_at(8);
                let (amount, _rest) = rest.split_at(8);
                let start_time = start_time.try_into().ok().map(u64::from_le_bytes).ok_or(InvalidInstruction)?;
                let end_time = end_time.try_into().ok().map(u64::from_le_bytes).ok_or(InvalidInstruction)?;
                let amount = amount.try_into().ok().map(u64::from_le_bytes).ok_or(InvalidInstruction)?;
                Self::ProcessUsdcStream (ProcessUsdcStream{start_time,end_time,amount})
            }
            4 =>{
                Self::ProcessPauseStream
            }
            5 =>{
                Self::ProcessResumeStream
            }
            _ => return Err(TokenError::InvalidInstruction.into()),
        })
    }
}