use std::process::exit;
use std::sync::Arc;
use actix_web::{App, HttpServer};
use jacks_sports_zone_api::{setup_logger, start_scheduler, configure_app, shutdown_signal};
use jacks_sports_zone_api::college_hockey_zone::database;

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
        }
    })
        .bind("0.0.0.0:8080")?
        .run()
        .await?;

    shutdown_signal().await;

    println!("Shutting down gracefully...");
    Ok(())
}
