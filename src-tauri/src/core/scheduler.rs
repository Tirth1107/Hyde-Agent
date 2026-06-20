use tauri::{AppHandle, Emitter};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use rusqlite::params;
use tokio::time::{sleep, Duration};
use crate::core::db::DB;
use tauri_plugin_notification::NotificationExt;

#[derive(Serialize, Deserialize, Debug)]
pub struct JobPayload {
    pub message: String,
    // Optional additional payload
    #[serde(default)]
    pub title: String,
}

pub fn schedule_job(job_type: &str, delay_seconds: u64, payload: JobPayload) -> Result<String, String> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let trigger_at = now + delay_seconds;
    let id = format!("{}_{}", job_type, now);
    
    let payload_str = serde_json::to_string(&payload).unwrap_or_default();
    
    let guard = DB.lock().unwrap();
    if let Some(conn) = guard.as_ref() {
        conn.execute(
            "INSERT INTO active_jobs (id, type, trigger_at, payload, status) VALUES (?1, ?2, ?3, ?4, 'pending')",
            params![id, job_type, trigger_at, payload_str],
        ).map_err(|e| format!("DB Error: {}", e))?;
    }
    
    Ok(id)
}

pub fn cancel_job(id: &str) -> Result<(), String> {
    let guard = DB.lock().unwrap();
    if let Some(conn) = guard.as_ref() {
        conn.execute(
            "UPDATE active_jobs SET status = 'cancelled' WHERE id = ?1",
            params![id],
        ).map_err(|e| format!("DB Error: {}", e))?;
    }
    Ok(())
}

pub fn list_active_jobs() -> Result<Vec<(String, String, u64)>, String> {
    let guard = DB.lock().unwrap();
    let mut jobs = Vec::new();
    if let Some(conn) = guard.as_ref() {
        let mut stmt = conn.prepare("SELECT id, type, trigger_at FROM active_jobs WHERE status = 'pending'").map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        }).map_err(|e| e.to_string())?;
        
        for row in rows {
            if let Ok(job) = row {
                jobs.push(job);
            }
        }
    }
    Ok(jobs)
}

pub fn start_scheduler(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            sleep(Duration::from_secs(1)).await;
            check_and_run_jobs(&app);
        }
    });
}

fn check_and_run_jobs(app: &AppHandle) {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
    
    let guard = DB.lock().unwrap();
    let conn = if let Some(c) = guard.as_ref() { c } else { return; };
    
    // Find due jobs
    let mut stmt = match conn.prepare("SELECT id, type, payload FROM active_jobs WHERE status = 'pending' AND trigger_at <= ?1") {
        Ok(s) => s,
        Err(_) => return,
    };
    
    let mut due_jobs = Vec::new();
    if let Ok(rows) = stmt.query_map(params![now], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
    }) {
        for row in rows.flatten() {
            due_jobs.push(row);
        }
    }
    drop(stmt);
    drop(guard); // release lock so execute logic can potentially acquire it
    
    for (id, job_type, payload_str) in due_jobs {
        execute_job(app, &id, &job_type, &payload_str);
        
        // Mark as completed
        if let Ok(guard) = DB.lock() {
            if let Some(conn) = guard.as_ref() {
                let _ = conn.execute("UPDATE active_jobs SET status = 'completed' WHERE id = ?1", params![id]);
            }
        }
    }
}

fn execute_job(app: &AppHandle, _id: &str, job_type: &str, payload_str: &str) {
    let payload: JobPayload = serde_json::from_str(payload_str).unwrap_or(JobPayload {
        message: "Scheduled Task Complete".to_string(),
        title: "Hyde Agent".to_string()
    });
    
    let title = if payload.title.is_empty() {
        if job_type == "timer" { "Timer Complete" } else { "Reminder" }
    } else {
        &payload.title
    };
    
    // 1. Play sound (System Beep)
    print!("\x07");
    
    // 2. OS Notification
    let _ = app.notification()
        .builder()
        .title(title)
        .body(&payload.message)
        .show();
        
    // 3. Optional Overlay Toast (UI)
    let _ = app.emit("timer_done", &payload.message);
}
