use recall_db::RecallDb;
use std::env;

#[tokio::test]
async fn test_db_connection_and_migrations() {
    dotenv::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    let db = RecallDb::new(&db_url).await.expect("Failed to connect to DB");
    
    // If new() succeeds, migrations have run.
    assert!(true);
}
