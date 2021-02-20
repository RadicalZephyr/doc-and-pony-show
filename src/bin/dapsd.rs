use std::path::PathBuf;

use tide::prelude::*;
use tide::Request;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Project {
    language: String,
    project_name: String,
    directory: PathBuf,
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    let mut app = tide::new();
    app.at("/register/dir").post(register_dir);
    app.listen("127.0.10.1:8080").await?;
    Ok(())
}

async fn register_dir(mut req: Request<()>) -> tide::Result {
    println!("{:?}", req.header_names());
    println!("{:?}", req.header("host"));
    let Project {
        language,
        project_name,
        directory,
    } = req.body_json().await?;
    Ok(format!(
        "Registered {} with language {} located at {:?}",
        project_name, language, directory
    )
    .into())
}
