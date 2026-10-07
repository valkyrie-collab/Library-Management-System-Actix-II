use anchor_client::Signer;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::Keypair};
use sqlx::PgPool;

pub struct AppState {
    rpc_url: RpcClient,
    program_id: Pubkey,
}

impl AppState {
    pub fn new(url: RpcClient, pid: Pubkey) -> AppState {
        AppState { rpc_url: url, program_id: pid }
    }

    pub fn get_rpc_url(&self) -> &RpcClient { &self.rpc_url }

    pub fn get_program_id(&self) -> &Pubkey { &self.program_id }

}

pub struct Database {
    pg_pool: PgPool
}

impl Database {
    pub fn set_database(p: PgPool) -> Database {
        Database { pg_pool: p }
    }

    pub fn get_database(&self) -> &PgPool {
        &self.pg_pool
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Claims {
    sub: String,
    exp: u64,
    roles: Vec<String>
}

impl Claims {
    pub fn new(s: String, e: u64, r: Vec<String>) -> Claims {
        Claims {sub: s, exp: e, roles: r}
    }

    pub fn get_sub(&self) -> &String {
        &self.sub
    }

    pub fn get_exp(&self) -> u64 {
        u64::clone(&self.exp)
    }

    pub fn get_roles(&self) -> &Vec<String> {
        &self.roles
    }
}
