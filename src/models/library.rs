use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct LibraryPda {
    pub bump: u8,
    pub admin: Pubkey,
    pub treasure: Pubkey,
    pub name: String,
    pub description: String,
    pub books: u64,
    pub time: u64
}

impl LibraryPda {
    pub fn new(bp: u8, ad: Pubkey, tres: Pubkey, n: String, des: String, bk: u64, tm: u64) -> LibraryPda {
        LibraryPda { bump: bp, admin: ad, treasure: tres, name: n, description: des, books: bk, time: tm }
    }

    pub fn get_bump(&self) -> u8 {
        self.bump
    }

    pub fn get_admin(&self) -> &Pubkey {
        &self.admin
    }

    pub fn get_treasure(&self) -> &Pubkey {
        &self.treasure
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_description(&self) -> &String {
        &self.description
    }

    pub fn get_books_count(&self) -> u64 {
        self.books
    }

    pub fn get_time_when_the_library_pda_initialize(&self) -> u64 {
        self.time
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct TreasurePda {
    pub bump: u8
}

impl TreasurePda {
    pub fn new(bp: u8) -> TreasurePda {
        TreasurePda { bump: bp }
    }
    
    pub fn get_bump(&self) -> u8 { self.bump }
}