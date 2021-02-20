use std::{collections::HashMap, path::PathBuf, sync::Arc};

use async_std::sync::{RwLock, RwLockReadGuard};
use tide::{http::headers::HeaderValues, prelude::*};
use tide::{Request, StatusCode};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct LanguageName(String);

impl LanguageName {
    fn as_str(&self) -> &String {
        &self.0
    }
}

impl LanguageName {
    fn from_host_name(host_name_opt: Option<&HeaderValues>) -> Option<Self> {
        host_name_opt
            .and_then(|host_name| {
                host_name
                    .to_string()
                    .strip_suffix(".docs")
                    .map(String::from)
            })
            .map(|language_name| LanguageName(language_name))
    }
}

type LanguageMap = HashMap<String, Language>;

#[derive(Clone, Debug)]
struct LanguageDirectory {
    languages: Arc<RwLock<LanguageMap>>,
}

impl Default for LanguageDirectory {
    fn default() -> LanguageDirectory {
        LanguageDirectory {
            languages: Arc::default(),
        }
    }
}

impl LanguageDirectory {
    async fn languages(&self) -> RwLockReadGuard<'_, LanguageMap> {
        self.languages.read().await
    }
}

type ProjectMap = HashMap<String, Project>;

#[derive(Debug, Default)]
struct Language {
    name: String,
    projects: ProjectMap,
}

impl Language {
    fn project(&self, project_name: &str) -> Option<&Project> {
        self.projects.get(project_name)
    }
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
    app.at("/:project_name/*path").all(serve_page);
    app.listen("127.0.10.1:8080").await?;
    Ok(())
}

async fn register_dir(mut req: Request<LanguageDirectory>) -> tide::Result {
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

async fn serve_page(req: Request<LanguageDirectory>) -> tide::Result {
    let language_name = LanguageName::from_host_name(req.header("host"))
        .ok_or(tide::Error::from_str(StatusCode::NotFound, ""))?;
    let project_name = req.param("project_name")?;
    let path = req.param("path")?;
    let state = req.state();
    let languages = state.languages().await;
    let language = languages
        .get(language_name.as_str())
        .ok_or(tide::Error::from_str(
            StatusCode::NotFound,
            "language has no items",
        ))?;
    let project = language.project(project_name).ok_or(tide::Error::from_str(
        StatusCode::NotFound,
        "Project not found",
    ))?;
    Ok(format!("string").into())
}
