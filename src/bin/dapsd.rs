use std::{collections::HashMap, path::PathBuf, sync::Arc};

use async_std::sync::RwLock;
use tide::prelude::*;
use tide::Request;

#[derive(Clone, Debug)]
struct LanguageDirectory {
    languages: Arc<RwLock<HashMap<String, Language>>>,
}

impl Default for LanguageDirectory {
    fn default() -> LanguageDirectory {
        LanguageDirectory {
            languages: Arc::default(),
        }
    }
}

#[derive(Debug, Default)]
struct Language {
    name: String,
    projects: HashMap<String, Project>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Project {
    language: String,
    project_name: String,
    directory: PathBuf,
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    let mut app = tide::with_state(LanguageDirectory::default());
    app.at("/api/register/dir").post(register_dir);
    app.at("/*").all(serve_page);
    app.listen("127.0.10.1:8080").await?;
    Ok(())
}

async fn register_dir(mut req: Request<LanguageDirectory>) -> tide::Result {
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

async fn serve_page(mut _req: Request<LanguageDirectory>) -> tide::Result {
    Ok(format!("string").into())
}
