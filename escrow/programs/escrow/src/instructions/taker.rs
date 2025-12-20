use crate::{errors::EscrowError, state::Escrow};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },
};

#[derive(Accounts)]
pub struct Taker<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,

    #[account(mut)]
    pub maker: SystemAccount<'info>,

    #[account(
      mut,
      close = maker,
      seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
      bump = escrow.bump,
      has_one = maker @ EscrowError::InvalidMaker,
      has_one = mint_m @ EscrowError::InvalidMintM,
      has_one = mint_n @ EscrowError::InvalidMintN,
  )]
    pub escrow: Box<Account<'info, Escrow>>,

    /// Token Accounts
    pub mint_m: Box<InterfaceAccount<'info, Mint>>,
    pub mint_n: Box<InterfaceAccount<'info, Mint>>,

    #[account(
      mut,
      associated_token::mint = mint_m,
      associated_token::authority = escrow,
      associated_token::token_program = token_program
  )]
    pub vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
      init_if_needed,
      payer = taker,
      associated_token::mint = mint_m,
      associated_token::authority = taker,
      associated_token::token_program = token_program
  )]
    pub taker_ata_m: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
      mut,
      associated_token::mint = mint_n,
      associated_token::authority = taker,
      associated_token::token_program = token_program
  )]
    pub taker_ata_n: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
      init_if_needed,
      payer = taker,
      associated_token::mint = mint_n,
      associated_token::authority = maker,
      associated_token::token_program = token_program
  )]
    pub maker_ata_n: Box<InterfaceAccount<'info, TokenAccount>>,

    /// Programs
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Taker<'info> {
    fn transfer_to_maker(&mut self) -> Result<()> {
        transfer_checked(
            CpiContext::new(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.taker_ata_n.to_account_info(),
                    to: self.maker_ata_n.to_account_info(),
                    mint: self.mint_n.to_account_info(),
                    authority: self.taker.to_account_info(),
                },
            ),
            self.escrow.token_mint_n_expected,
            self.mint_n.decimals,
        )?;

        Ok(())
    }

    fn withdraw_and_close_vault(&mut self) -> Result<()> {
        // Create the signer seeds for the Vault
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"escrow",
            self.maker.to_account_info().key.as_ref(),
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump],
        ]];

        // Transfer Token M (Vault -> Taker)
        transfer_checked(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.vault.to_account_info(),
                    to: self.taker_ata_m.to_account_info(),
                    mint: self.mint_m.to_account_info(),
                    authority: self.escrow.to_account_info(),
                },
                &signer_seeds,
            ),
            self.vault.amount,
            self.mint_m.decimals,
        )?;

        // Close the Vault
        close_account(CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            CloseAccount {
                account: self.vault.to_account_info(),
                authority: self.escrow.to_account_info(),
                destination: self.maker.to_account_info(),
            },
            &signer_seeds,
        ))?;

        Ok(())
    }
}

pub fn handler(ctx: Context<Taker>) -> Result<()> {
    // Transfer Token B to Maker
    ctx.accounts.transfer_to_maker()?;

    // Withdraw and close the Vault
    ctx.accounts.withdraw_and_close_vault()?;

    Ok(())
}
