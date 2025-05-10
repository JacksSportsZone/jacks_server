use std::process::exit;
use std::sync::Arc;
use actix_web::{App, HttpServer};
use jacks_sports_zone_api::{ start_scheduler, configure_app, shutdown_signal};
use jacks_sports_zone_api::college_hockey_zone::database;
use flexi_logger::{Age, Cleanup, Criterion, DeferredNow, FileSpec, Logger, Naming, WriteMode, Record};
use serde_json::json;
use jacks_card_games::card_games_app;
/// Sets up a logger that writes to both standard output and a daily rotating log file.
fn setup_logger(log_name: &str) -> Result<(), flexi_logger::FlexiLoggerError> {
    Logger::try_with_str("info")?
        .log_to_file(FileSpec::default().directory("logs").basename(log_name))
        .rotate(
            Criterion::Age(Age::Day),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(90),
        )
        .format(custom_format)
        .write_mode(WriteMode::Direct)
        .start()?;
    Ok(())
}

fn custom_format(
    w: &mut dyn std::io::Write,
    now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    let level = if record.target() == "JacksSportsZoneApi" && record.level() == log::Level::Info {
        "API"
    } else if record.target() == "JacksSportsZoneApi" && record.level() == log::Level::Debug {
        "Update"
    } else if record.target() == "JacksSportsZoneApi" && record.level() == log::Level::Error {
        "Update_Error"
    } else {
        record.level().as_str()
    };

    let log_entry = json!({
        "timestamp": now.format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string(),
        "level": level,
        "target": record.target(),
        "file": record.file().unwrap_or("<unnamed>"),
        "line": record.line().unwrap_or(0),
        "message": record.args().to_string(),
    });

    write!(w, "{}", log_entry)?;
    Ok(())
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting up...");

    if let Err(e) = setup_logger("CollegeHockeyZone") {
        println!("Failed to initialize logger: {}", e);
        exit(1);
    }
    println!("Logger initialized");

    // Establish the database connection pool
    let pool = database::connection::establish_connection().await;
    let db_pool = Arc::new(pool);

    // Start the scheduler
    if let Err(e) = start_scheduler(db_pool.clone()).await {
        println!("Failed to start scheduler: {}", e);
        exit(1);
    }

    // Start Actix Web server
    HttpServer::new({
        let db_pool = db_pool.clone();
        move || {
            App::new() // clone again to move into the App
                .service(configure_app(db_pool.clone()))
                .service(card_games_app())
        }
    })
        .bind("0.0.0.0:8080")?
        .run()
        .await?;

    shutdown_signal().await;

    println!("Shutting down gracefully...");
    Ok(())
}
