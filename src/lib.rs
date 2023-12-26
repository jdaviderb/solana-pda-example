use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
    program_error::ProgramError
};

entrypoint!(process_instruction);

enum Errors {
    Unauthorized,
    UnauthorizedAccount,
    CounterIsAlreadyInitialized,
    InvalidPda,
}

pub fn verify_account_owner<'a, 'b>(account: &'a AccountInfo<'a>, owner: &'b Pubkey) -> Result<&'a AccountInfo<'a>, ProgramError> where 'a: 'b {
    if account.owner != owner {
        return Err(Errors::Unauthorized.into());
    }

    Ok(account)
}

enum Instructions {
    InitializeCounter,
    IncrementCounter,
}

impl From<Errors> for ProgramError {
    fn from(e: Errors) -> Self {
        ProgramError::Custom(e as u32)
    }
}

struct Counter {
    owner: Pubkey, // 32
    count: u8,     // 1,
}

pub fn process_instruction<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    let mut accounts_iter = accounts.iter();
    let instruction = unsafe { &*(instruction_data.as_ptr() as *const Instructions) };

    match instruction {
        Instructions::InitializeCounter => {
            let counter_size = ::std::mem::size_of::<Counter>();
            msg!("Instruction: InitializeCounter");
            let signer = next_account_info(&mut accounts_iter)?;
            let counter = next_account_info(&mut accounts_iter)?;
            let (counter_pda, bump) =
                Pubkey::find_program_address(&[signer.key.as_ref(), b"counter"], program_id);
            let system_program = next_account_info(&mut accounts_iter)?;

            if &Pubkey::default() != counter.owner {
                return Err(Errors::CounterIsAlreadyInitialized.into());
            }

            if counter.key != &counter_pda {
                return Err(Errors::InvalidPda.into());
            }

            invoke_signed(
                &system_instruction::create_account(
                    signer.key,
                    counter.key,
                    Rent::get()?.minimum_balance(counter_size),
                    counter_size as u64,
                    program_id,
                ),
                &[signer.clone(), counter.clone(), system_program.clone()],
                &[&[signer.key.as_ref(), &b"counter"[..], &[bump]]],
            )?;

            let counter_data = &mut counter.data.borrow_mut();
            let counter = unsafe { &mut *(counter_data.as_mut_ptr() as *mut Counter) };

            counter.count = 1;
            counter.owner = *signer.key;
        }

        Instructions::IncrementCounter => {
            msg!("Instruction: IncrementCounter");
            let signer = next_account_info(&mut accounts_iter)?;
            let counter = verify_account_owner(
                next_account_info(&mut accounts_iter)?, 
                &program_id
            )?;
            let counter_data = &mut counter.data.borrow_mut();
            let counter = unsafe { &mut *(counter_data.as_mut_ptr() as *mut Counter) };

            if counter.owner != *signer.key || !signer.is_signer {
                return Err(Errors::UnauthorizedAccount.into());
            }
            counter.count += 1;
        }
    }

    Ok(())
}
