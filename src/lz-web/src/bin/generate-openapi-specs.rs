use utoipa::OpenApi;

fn main() -> Result<(), anyhow::Error> {
    println!("{}", lz_web::api::ApiDoc::openapi().to_pretty_json()?);
    Ok(())
}
