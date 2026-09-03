//! Power Notifier is a small utility that sends a notification when power is restored or
//! when a system reboots. The notification provides:
//! * the hostname or machine id 
//! * the date and time when power was restored or when the system booted
//! * the date and time when power was last known to be on
//!
//! This Rust utility uses Tokio for learning and demonstration purposes.
//! This could easily be rewritten in conventional sync Rust or in a script.

use std::env;
use chrono::{DateTime, Local};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::fs::File;
use reqwest::Client;

/// the main function
#[tokio::main]
async fn main() {
    println!("Power Notifier");

    // read CLI args
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} <hostname or machine id> <path to log/state file> <ntfy.sh url>", args[0]);
        return;
    }
    let hostname = args[1].clone();
    let logfile = args[2].clone();
    let ntfyurl = args[3].clone();
    println!("hostname: {}", hostname);
    println!("logfile: {}", logfile);
    println!("ntfyurl: {}", ntfyurl);

    // get the current local date and time
    let current: DateTime<Local> = Local::now();
    let current_fmt = current.format("%Y-%m-%d %H:%M:%S").to_string();
    println!("Started: {}", current_fmt);

    // read the previous date and time from the log/state file
    let last_fmt: String = read_log(&logfile).await;

    // build the notification string
    let notification: String = hostname +
        " P " + &current_fmt +
        " L " + &last_fmt;

    // send the notification and
    // retry once per minute if there is a problem, like the network not being up yet
    send_notification(ntfyurl, notification).await;

    // spawn tasks to periodically update the log/state file
    update_log(logfile).await;
}

/// read the previous date and time from the log/state file
async fn read_log(logfile: &String) -> String {
    let mut buffer: Vec<u8> = Vec::new();
    let f = File::open(&logfile).await;
    match f {
        Ok(mut f) => {
            f.read_to_end(&mut buffer).await.unwrap_or(0);
        }
        Err(e) => {
            eprintln!("Error opening log file: {}", e);
        }
    };
    let last_fmt: String = String::from_utf8(buffer).unwrap_or("error".to_string());
    println!("Previous: {}", last_fmt);
    last_fmt
}

/// send the notification and
/// retry once per minute if there is a problem, like the network not being up yet
async fn send_notification(ntfyurl:String, notification: String) {
    let client = Client::new();
    loop {
        let res = client.post(&ntfyurl)
            .body(notification.clone())
            .send()
            .await;
        match res {
            Ok(_r) => {
                println!("Notification Sent!");
                break;
            },
            Err(e) => {
                eprintln!("Notification Error, {}, retrying after 60s", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            }
        };
    }
}

/// spawn tasks to periodically update the log/state file
async fn update_log(logfile: String) {
    loop {
        let lf = logfile.clone();
        let current: DateTime<Local> = Local::now();
        let current_fmt = current.format("%Y-%m-%d %H:%M:%S").to_string();
        println!("Updating log file at {}", current_fmt);

        tokio::spawn(async move {
            // any unwrap panic here interrupts the task,
            // but the parent loop and program keep running
            let mut f = File::create(&lf).await.unwrap();
            f.write_all(&current_fmt.into_bytes()).await.unwrap();
        });

        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
    }
}
