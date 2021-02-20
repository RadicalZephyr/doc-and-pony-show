use std::{
    collections::HashMap,
    ffi::OsStr,
    io,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_std::{path::PathBuf as AsyncPathBuf, sync::RwLock};
use tide::{http::headers::HeaderValues, log, prelude::*, Request, StatusCode};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct LanguageName(String);

impl LanguageName {
    fn as_str(&self) -> &String {
        &self.0
    }
}

impl LanguageName {
    fn from_host_name(host_name_opt: Option<&HeaderValues>) -> tide::Result<Self> {
        host_name_opt
            .ok_or(tide::Error::from_str(
                StatusCode::InternalServerError,
                "no hostname specified",
            ))
            .and_then(|host_name| {
                host_name
                    .to_string()
                    .strip_suffix(".docs")
                    .map(String::from)
                    .ok_or(tide::Error::from_str(
                        StatusCode::BadRequest,
                        "improper domain name",
                    ))
            })
            .map(|language_name| LanguageName(language_name))
    }
}

type SharedLanguageDirectory = Arc<RwLock<LanguageDirectory>>;

type LanguageMap = HashMap<String, Language>;

#[derive(Debug, Default)]
struct LanguageDirectory {
    languages: LanguageMap,
}

impl LanguageDirectory {
    fn language(&self, language_name: &LanguageName) -> tide::Result<&Language> {
        self.languages
            .get(language_name.as_str())
            .ok_or(tide::Error::from_str(
                StatusCode::NotFound,
                "language has no registered projects",
            ))
    }
}

type ProjectMap = HashMap<String, Project>;

#[derive(Debug, Default)]
struct Language {
    name: String,
    projects: ProjectMap,
}

impl Language {
    fn project(&self, project_name: &str) -> tide::Result<&Project> {
        self.projects.get(project_name).ok_or(tide::Error::from_str(
            StatusCode::NotFound,
            "Project not found",
        ))
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Project {
    language: String,
    project_name: String,
    directory: PathBuf,
}

impl Project {
    async fn serve_path(&self, path: &str) -> tide::Result {
        let file_path = self.full_path_to(path);
        if !file_path.starts_with(&self.directory) {
            log::info!("Unauthorized attempt to read: {:?}", &file_path);
            Ok(tide::Response::new(StatusCode::Forbidden))
        } else {
            let file_path = AsyncPathBuf::from(file_path);
            match tide::Body::from_file(&file_path).await {
                Ok(body) => Ok(tide::Response::builder(StatusCode::Ok).body(body).build()),
                Err(e) if e.kind() == io::ErrorKind::NotFound => {
                    log::warn!("File not found: {:?}", &file_path);
                    Ok(tide::Response::new(StatusCode::NotFound))
                }
                Err(e) => Err(e.into()),
            }
        }
    }

    fn full_path_to(&self, path: &str) -> PathBuf {
        let mut file_path = self.directory.clone();
        for p in Path::new(path) {
            if p == OsStr::new(".") {
                continue;
            } else if p == OsStr::new("..") {
                file_path.pop();
            } else {
                file_path.push(&p);
            }
        }
        file_path
    }
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    let mut app = tide::with_state(SharedLanguageDirectory::default());
    app.at("/api/register/dir").post(register_dir);
    app.at("/:project_name/*path").all(serve_page);
    app.listen("127.0.10.1:8080").await?;
    Ok(())
}

async fn register_dir(mut req: Request<SharedLanguageDirectory>) -> tide::Result {
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

async fn serve_page(req: Request<SharedLanguageDirectory>) -> tide::Result {
    let language_name = LanguageName::from_host_name(req.header("host"))?;
    let project_name = req.param("project_name")?;
    let path = req.param("path")?;
    let state = req.state();
    let language_directory = state.read().await;
    let language = language_directory.language(&language_name)?;
    let project = language.project(project_name)?;
    project.serve_path(path).await
}
