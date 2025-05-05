#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, Env, Symbol, String, symbol_short, log, BytesN};

// Store user accounts information
#[contracttype]
#[derive(Clone)]
pub struct User {
    pub user_id: BytesN<32>,         // Public key as user ID
    pub username: String,            // Username for display purposes
    pub registered_time: u64,        // Time of registration
    pub last_login: u64,             // Last login timestamp
    pub login_count: u64,            // Number of times logged in
    pub active: bool,                // Account status (active/deactivated)
}

// Map user ID to User struct
#[contracttype]
pub enum DataKey {
    User(BytesN<32>)
}

// Track system statistics
#[contracttype]
#[derive(Clone)]
pub struct SystemStats {
    pub total_users: u64,           // Total registered users
    pub active_users: u64,          // Number of active users
    pub inactive_users: u64,        // Number of inactive users
    pub total_logins: u64,          // Total number of logins across all users
}

// Key for system stats
const SYSTEM_STATS: Symbol = symbol_short!("SYS_STATS");

#[contract]
pub struct DecentralizedLoginSystem;

#[contractimpl]
impl DecentralizedLoginSystem {
    // Register a new user with a public key and username
    pub fn register_user(env: Env, user_id: BytesN<32>, username: String) -> bool {
        // Check if the user already exists
        let user_key = DataKey::User(user_id.clone());
        let existing_user: Option<User> = env.storage().instance().get(&user_key);
        
        if existing_user.is_some() {
            log!(&env, "User already exists with this ID: {}", username);
            return false;
        }
        
        let current_time = env.ledger().timestamp();
        
        // Create new user
        let user = User {
            user_id: user_id.clone(),
            username: username.clone(),
            registered_time: current_time,
            last_login: current_time,
            login_count: 0,
            active: true,
        };
        
        // Store user data
        env.storage().instance().set(&user_key, &user);
        
        // Update system stats
        let mut stats = Self::get_system_stats(env.clone());
        stats.total_users += 1;
        stats.active_users += 1;
        env.storage().instance().set(&SYSTEM_STATS, &stats);
        
        // Extend storage lifetime
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "User registered successfully: {}", username);
        true
    }
    
    // Record a login for a user
    pub fn login(env: Env, user_id: BytesN<32>) -> bool {
        let user_key = DataKey::User(user_id.clone());
        let user_option: Option<User> = env.storage().instance().get(&user_key);
        
        match user_option {
            Some(mut user) => {
                if !user.active {
                    log!(&env, "Account is deactivated for user: {}", user.username);
                    return false;
                }
                
                // Update user login information
                let current_time = env.ledger().timestamp();
                user.last_login = current_time;
                user.login_count += 1;
                
                // Store updated user data
                env.storage().instance().set(&user_key, &user);
                
                // Update system stats
                let mut stats = Self::get_system_stats(env.clone());
                stats.total_logins += 1;
                env.storage().instance().set(&SYSTEM_STATS, &stats);
                
                env.storage().instance().extend_ttl(5000, 5000);
                
                log!(&env, "User logged in: {}", user.username);
                true
            },
            None => {
                log!(&env, "User not found");
                false
            }
        }
    }
    
    // Get user data
    pub fn get_user(env: Env, user_id: BytesN<32>) -> Option<User> {
        let user_key = DataKey::User(user_id);
        env.storage().instance().get(&user_key)
    }
    
    // Get system statistics
    pub fn get_system_stats(env: Env) -> SystemStats {
        env.storage().instance().get(&SYSTEM_STATS).unwrap_or(SystemStats {
            total_users: 0,
            active_users: 0,
            inactive_users: 0,
            total_logins: 0
        })
    }
}