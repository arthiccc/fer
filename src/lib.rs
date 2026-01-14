use std::sync::Arc;
use parking_lot::RwLock;
use thiserror::Error;
use rusqlite::{params, Connection};
use std::thread;
use regex::Regex;
use std::time::{SystemTime, UNIX_EPOCH};
use secrecy::SecretString;
use wasm_bindgen::prelude::*;

use std::sync::mpsc;

uniffi::setup_scaffolding!();

#[derive(Debug, Error, uniffi::Error)]
pub enum TelcoError {
    #[error("Insufficient balance for this transaction.")]
    InsufficientBalance,
    #[error("Account is inactive.")]
    AccountInactive,
    #[error("Device is locked via biometrics.")]
    Locked,
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Internal error")]
    InternalError,
}

#[derive(Clone, Copy, Debug, uniffi::Enum, PartialEq)]
pub enum QuotaType { General, Social, Video }

#[derive(Clone, Debug, uniffi::Record)]
pub struct QuotaBucket {
    pub name: String,
    pub remaining_bytes: u64,
    pub category: QuotaType,
    pub expiry: u64,
}

#[derive(Clone, Debug, uniffi::Record)]
pub struct UserAccount {
    pub id: String,
    pub is_active: bool,
    pub biometric_locked: bool,
    pub buckets: Vec<QuotaBucket>,
    pub last_traffic_bytes: u64,
    pub data_balance_bytes: u64,
    pub current_latency_ms: u32,
}

#[uniffi::export(callback_interface)]
pub trait TelcoLiveUpdateHandler: Send + Sync {
    fn on_account_updated(&self, account: UserAccount);
}

struct PersistenceMsg {
    account: UserAccount,
    usage: Option<(u64, QuotaType, u64)>,
}

#[wasm_bindgen]
#[derive(uniffi::Object)]
pub struct TelcoSimulator {
    state: Arc<RwLock<UserAccount>>,
    db_key: Arc<RwLock<Option<SecretString>>>,
    update_handler: RwLock<Option<Box<dyn TelcoLiveUpdateHandler>>>,
    persistence_tx: mpsc::SyncSender<PersistenceMsg>,
}

#[uniffi::export]
impl TelcoSimulator {
    #[uniffi::constructor]
    pub fn new(id: String, db_path: String) -> Result<Arc<Self>, TelcoError> {
        let conn = Connection::open(&db_path).map_err(|e| TelcoError::DatabaseError(e.to_string()))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS accounts (id TEXT PRIMARY KEY, is_active BOOLEAN, locked BOOLEAN, last_traffic INTEGER);
             CREATE TABLE IF NOT EXISTS buckets (id INTEGER PRIMARY KEY, account_id TEXT, name TEXT, remaining_bytes INTEGER, category TEXT, expiry INTEGER);
             CREATE TABLE IF NOT EXISTS usage_history (timestamp INTEGER, amount INTEGER, category TEXT);"
        ).map_err(|e| TelcoError::DatabaseError(e.to_string()))?;

        let account = load_account_internal(&conn, &id).unwrap_or_else(|_| {
            UserAccount { 
                id: id.clone(), 
                is_active: true, 
                biometric_locked: false, 
                buckets: vec![], 
                last_traffic_bytes: 0,
                data_balance_bytes: 0,
                current_latency_ms: 46,
            }
        });

        let (tx, rx) = mpsc::sync_channel::<PersistenceMsg>(1000);
        let db_path_clone = db_path.clone();
        thread::spawn(move || {
            if let Ok(mut conn) = Connection::open(db_path_clone) {
                while let Ok(msg) = rx.recv() {
                    if let Some((bytes, category, now)) = msg.usage {
                        let _ = conn.execute("INSERT INTO usage_history (timestamp, amount, category) VALUES (?1, ?2, ?3)",
                            params![now, bytes, format!("{:?}", category)]);
                    }
                    if let Ok(tx) = conn.transaction() {
                        let _ = tx.execute("INSERT OR REPLACE INTO accounts (id, is_active, locked, last_traffic) VALUES (?1, ?2, ?3, ?4)", 
                            params![msg.account.id, msg.account.is_active, msg.account.biometric_locked, msg.account.last_traffic_bytes]);
                        let _ = tx.execute("DELETE FROM buckets WHERE account_id = ?1", params![msg.account.id]);
                        for b in msg.account.buckets {
                            let _ = tx.execute(
                                "INSERT INTO buckets (account_id, name, remaining_bytes, category, expiry) VALUES (?1, ?2, ?3, ?4, ?5)",
                                params![msg.account.id, b.name, b.remaining_bytes, format!("{:?}", b.category), b.expiry]
                            );
                        }
                        let _ = tx.commit();
                    }
                }
            }
        });

        Ok(Arc::new(Self { 
            state: Arc::new(RwLock::new(account)), 
            db_key: Arc::new(RwLock::new(None)),
            update_handler: RwLock::new(None),
            persistence_tx: tx,
        }))
    }

    pub fn set_update_handler(&self, handler: Box<dyn TelcoLiveUpdateHandler>) {
        let mut lock = self.update_handler.write();
        *lock = Some(handler);
        let account = self.state.read().clone();
        if let Some(h) = &*lock { h.on_account_updated(account); }
    }

    pub fn unlock_with_biometrics(&self) {
        let mut lock = self.state.write();
        lock.biometric_locked = false;
        let account = lock.clone();
        drop(lock);
        self.notify_and_persist(account, None);
    }

    pub fn secure_initialize(&self, key: String) {
        let mut lock = self.db_key.write();
        *lock = Some(SecretString::from(key));
    }

    pub fn get_account_info(&self) -> Result<UserAccount, TelcoError> {
        let state = self.state.read();
        if state.biometric_locked { return Err(TelcoError::Locked); }
        Ok(state.clone())
    }

    pub fn handle_command(&self, command: String) -> String {
        if self.state.read().biometric_locked { return "Unlock required.".to_string(); }
        let cmd = command.trim().to_lowercase();
        if cmd == "status" { return self.generate_insight(); }
        match self.parse_and_buy_topping(command) {
            Ok(_) => "Liquid Bubble growing...".to_string(),
            Err(e) => format!("Error: {}", e),
        }
    }

    pub fn simulate_usage(&self, bytes: u64, category: QuotaType) -> Result<(), TelcoError> {
        let mut lock = self.state.write();
        if lock.biometric_locked { return Err(TelcoError::Locked); }
        
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let new_state = (*lock).consume_data(bytes, category)?;
        *lock = new_state;
        
        let account = lock.clone();
        drop(lock);
        
        self.notify_and_persist(account, Some((bytes, category, now)));
        Ok(())
    }

    // Insight Logic
    fn generate_insight(&self) -> String {
        let total: u64 = self.state.read().buckets.iter().map(|b| b.remaining_bytes).sum();
        format!("You have {:.2} GB remaining. Native Rust Engine is standing by.", total as f64 / 1e9)
    }

    fn parse_and_buy_topping(&self, command: String) -> Result<(), TelcoError> {
        let re = Regex::new(r"(?i)(YouTube|Social|General)\s+(\d+)\s*(GB|MB)").unwrap();
        if let Some(caps) = re.captures(&command) {
            let cat_str = caps.get(1).unwrap().as_str().to_lowercase();
            let amount: u64 = caps.get(2).unwrap().as_str().parse().unwrap();
            let unit = caps.get(3).unwrap().as_str().to_uppercase();
            let bytes = if unit == "GB" { amount * 1024 * 1024 * 1024 } else { amount * 1024 * 1024 };
            let category = match cat_str.as_str() { "youtube" => QuotaType::Video, "social" => QuotaType::Social, _ => QuotaType::General };
            let topping = QuotaBucket {
                name: format!("{} {} Topping", amount, unit),
                remaining_bytes: bytes,
                category,
                expiry: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 86400 * 30,
            };
            let mut lock = self.state.write();
            lock.buckets.push(topping);
            lock.data_balance_bytes = lock.buckets.iter().map(|b| b.remaining_bytes).sum();
            let account = lock.clone();
            drop(lock);
            self.notify_and_persist(account, None);
            Ok(())
        } else {
            Err(TelcoError::InvalidCommand("Try 'YouTube 2GB'".to_string()))
        }
    }
    pub fn start_network_sensor(self: Arc<Self>) {
        thread::spawn(move || {
            let mut last_bytes = 0;
            loop {
                if let Ok(content) = std::fs::read_to_string("/proc/net/dev") {
                    for line in content.lines() {
                        // Monitor common interfaces
                        if line.contains("wlp3s0:") || line.contains("tun0:") || line.contains("eth0:") {
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if parts.len() > 1 {
                                let bytes: u64 = parts[1].parse().unwrap_or(0);
                                if last_bytes > 0 && bytes > last_bytes {
                                    let diff = bytes - last_bytes;
                                    // Map real traffic to Social quota for visibility in demo
                                    let _ = self.simulate_usage(diff, QuotaType::Social);
                                }
                                last_bytes = bytes;
                            }
                        }
                    }
                }
                thread::sleep(std::time::Duration::from_millis(500));
            }
        });
    }
}

impl TelcoSimulator {
    fn notify_and_persist(&self, account: UserAccount, usage: Option<(u64, QuotaType, u64)>) {
        if let Some(handler) = &*self.update_handler.read() { handler.on_account_updated(account.clone()); }
        let _ = self.persistence_tx.try_send(PersistenceMsg { account, usage });
    }
}

impl UserAccount {
    pub fn consume_data(&self, amount: u64, category: QuotaType) -> Result<Self, TelcoError> {
        if !self.is_active { return Err(TelcoError::AccountInactive); }
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut new_buckets = self.buckets.clone();
        let mut remaining = amount;
        let priorities = if category == QuotaType::General { vec![QuotaType::General] } else { vec![category, QuotaType::General] };
        for p in priorities {
            for bucket in new_buckets.iter_mut().filter(|b| b.category == p && b.expiry > now) {
                let deduction = std::cmp::min(bucket.remaining_bytes, remaining);
                bucket.remaining_bytes -= deduction;
                remaining -= deduction;
                if remaining == 0 { break; }
            }
            if remaining == 0 { break; }
        }
        if remaining > 0 { return Err(TelcoError::InsufficientBalance); }
        let total: u64 = new_buckets.iter().map(|b| b.remaining_bytes).sum();
        Ok(Self { 
            buckets: new_buckets, 
            data_balance_bytes: total,
            ..self.clone() 
        })
    }
}



fn load_account_internal(conn: &Connection, id: &str) -> Result<UserAccount, TelcoError> {
    let mut stmt = conn.prepare("SELECT is_active, locked, last_traffic FROM accounts WHERE id = ?1").ok().ok_or(TelcoError::InternalError)?;
    let (is_active, locked, last_traffic_bytes) = stmt.query_row(params![id], |row| Ok((row.get::<_, bool>(0)?, row.get::<_, bool>(1)?, row.get::<_, u64>(2)?)))
        .unwrap_or((true, false, 0));

    let mut stmt = conn.prepare("SELECT name, remaining_bytes, category, expiry FROM buckets WHERE account_id = ?1").ok().ok_or(TelcoError::InternalError)?;
    let buckets: Vec<QuotaBucket> = stmt.query_map(params![id], |row| {
        let cat_str: String = row.get(2)?;
        let category = match cat_str.as_str() { "Video" => QuotaType::Video, "Social" => QuotaType::Social, _ => QuotaType::General };
        Ok(QuotaBucket { name: row.get(0)?, remaining_bytes: row.get(1)?, category, expiry: row.get(3)? })
    }).ok().ok_or(TelcoError::InternalError)?.filter_map(|b| b.ok()).collect();

    Ok(UserAccount { 
        id: id.to_string(), 
        is_active, 
        biometric_locked: locked, 
        buckets: buckets.clone(), 
        last_traffic_bytes,
        data_balance_bytes: buckets.iter().map(|b| b.remaining_bytes).sum(),
        current_latency_ms: 46,
    })
}
