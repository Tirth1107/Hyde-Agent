use rusqlite::{Connection, Result};
use std::sync::Mutex;
use lazy_static::lazy_static;


lazy_static! {
    pub static ref DB: Mutex<Option<Connection>> = Mutex::new(None);
}

pub fn init() -> Result<()> {
    let home = dirs::home_dir().expect("Failed to get home directory");
    let db_path = home.join(".hyde-agent").join("hyde_v2.db");
    
    // Create directory if it doesn't exist
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).unwrap_or_default();
    }
    
    let conn = Connection::open(db_path)?;
    
    // Create tables
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT
        )",
        [],
    )?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS active_jobs (
            id TEXT PRIMARY KEY,
            type TEXT,
            trigger_at INTEGER,
            payload TEXT,
            status TEXT
        )",
        [],
    )?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS context_memory (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id TEXT,
            role TEXT,
            content TEXT,
            entities TEXT,
            timestamp INTEGER
        )",
        [],
    )?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS notifications (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT,
            body TEXT,
            read BOOLEAN DEFAULT 0,
            timestamp INTEGER
        )",
        [],
    )?;
    
    *DB.lock().unwrap() = Some(conn);
    Ok(())
}

pub fn get_setting(key: &str) -> Option<String> {
    let guard = DB.lock().unwrap();
    let conn = guard.as_ref()?;
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1").ok()?;
    stmt.query_row([key], |row| row.get(0)).ok()
}

pub fn set_setting(key: &str, value: &str) -> Result<()> {
    let guard = DB.lock().unwrap();
    if let Some(conn) = guard.as_ref() {
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            [key, value],
        )?;
    }
    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ContextMessage {
    pub role: String,
    pub content: String,
}

pub fn add_memory(session_id: &str, role: &str, content: &str) -> Result<()> {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
    let guard = DB.lock().unwrap();
    if let Some(conn) = guard.as_ref() {
        conn.execute(
            "INSERT INTO context_memory (session_id, role, content, timestamp) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![session_id, role, content, now],
        )?;
    }
    Ok(())
}

pub fn get_memory(session_id: &str, limit: usize) -> Vec<ContextMessage> {
    let mut msgs = Vec::new();
    let guard = DB.lock().unwrap();
    if let Some(conn) = guard.as_ref() {
        if let Ok(mut stmt) = conn.prepare("SELECT role, content FROM context_memory WHERE session_id = ?1 ORDER BY timestamp DESC LIMIT ?2") {
            if let Ok(rows) = stmt.query_map(rusqlite::params![session_id, limit as i64], |row| {
                Ok(ContextMessage {
                    role: row.get(0)?,
                    content: row.get(1)?,
                })
            }) {
                for r in rows.flatten() {
                    msgs.push(r);
                }
            }
        }
    }
    msgs.reverse(); // old to new
    msgs
}
