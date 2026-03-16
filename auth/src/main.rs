use auth::init_actix_web_server;


fn print_type_of<T>(_:&T){
    println!("{}", std::any::type_name::<T>())
}
#[actix_web::main]
async fn main() {
    let listener = std::net::TcpListener::bind("127.0.0.1:3003").unwrap();
    let db = sea_orm::Database::connect("sqlite://db.sqlite3?mode=rwc").await.unwrap();

    db.get_schema_registry("auth::accounts::models::*").sync(&db).await.unwrap();

    init_actix_web_server(
        listener,
        db
    ).unwrap()
        .await
        .unwrap();

}
