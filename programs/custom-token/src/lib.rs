use anchor_lang::prelude::*;
use spl_token_2022::extension::ExtensionType;
use spl_token_2022::state::Mint;


declare_id!("CswZd8SPTPqQHbEx8ZVdwxGwWBoWBvvewc6EFkEDtyLH");

#[program]
pub mod custom_token {
    use super::*;

   pub fn create_token_with_extensions(ctx: Context<CreateTokenWithExtensions>, args: CreateTokenArgs) -> Result<()> {
        msg!("Creating token with extensions: {:?}", args);
        
    // Step 1: Calculate required mint account size
    let mint_size = custom_token::get_mint_account_size(&args);

    // Step 2: Calculate rent-exempt lamports
    let rent_lamports = Rent::get()?.minimum_balance(mint_size);
     
     // Step 3: Create mint account manually
    let create_mint_ix = system_instruction::create_account(
        &ctx.accounts.payer.key(),
        &ctx.accounts.mint.key(),
        rent_lamports,
        mint_size as u64,
        &ctx.accounts.token_program.key(),
    );

    invoke(
        &create_mint_ix,
        &[
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;


    Ok(())
   }

    /// Calculates the size of the mint account based on the provided arguments.
   pub fn get_mint_account_size(args: &CreateTokenArgs) -> usize {
    let mut extensions = vec![
        ExtensionType::TransferFeeConfig,
        ExtensionType::TransferFeeAmount,
    ];

    if args.require_memo {
        extensions.push(ExtensionType::MemoTransfer);
    }

    if args.default_frozen {
        extensions.push(ExtensionType::DefaultAccountState);
    }

    if args.immutable_owner {
        extensions.push(ExtensionType::ImmutableOwner);
    }

    ExtensionType::try_calculate_account_len::<Mint>(&extensions).unwrap()
   }



}

#[derive(Accounts)]
pub struct CreateTokenWithExtensions<'info> {
    #[account]
    pub payer: Signer<'info>,

    ///CHECK: we will create this manually using CPI
    #[account(mut)]
    pub mint : UncheckedAccount<'info>,

    ///CHECK:will be set as the mint authority
    pub mint_authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>, 
}


//not a data account, just a struct to hold the arguments
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CreateTokenArgs {
    pub decimals: u8,
    pub transfer_fee_basis_points: u16,
    pub max_fee: u64,
    pub require_memo: bool,
    pub default_frozen: bool,
    pub immutable_owner: bool,
}
