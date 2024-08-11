use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub async fn start_connection() -> Pool<Postgres> {
    let enviroment = std::env::var("DATABASE_URL");
        match enviroment {
            Err(e) => {
                println!("Error: {}", e);
                std::panic!("Database enviroment not found")
            }
            _ => (),
        }

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(enviroment.unwrap().as_str()).await;

    match pool {
        Ok(_) => println!("Database connected"),
        Err(e) => {
            println!("Error: {}", e);
            std::panic!("Database not connected")
        }
    };

    let connection = pool.unwrap();
    
    let check_migrate = sqlx::migrate!("./src/database/connection/migrations")
        .run(&connection)
        .await;

    match check_migrate {
        Ok(_) => println!("Migrations runned"),
        Err(e) => {
            println!("Error: {}", e);
            std::panic!("Migrations not runned")
        }
    };

    connection
}