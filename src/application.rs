use rocket::{Request, State, catchers, get, post, routes};
use rocket_dyn_templates::{Template, context, tera::Tera};
// core.rs
use serde::Serialize;
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Serialize, Clone, Copy)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
}

#[derive(Debug, Serialize, Clone)]
pub struct Capsule {
    pub template: String,
    pub name: String,
    pub description: String,
    pub uri: String,
    pub method: Method,
    pub data: serde_json::Value,
}

impl Capsule {
    pub fn new<N: Into<String>, D: Into<String>, U: Into<String>, T: Into<String>>(
        name: N,
        description: D,
        uri: U,
        template: T,
        method: Method,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            uri: uri.into(),
            template: template.into(),
            method,
            data: serde_json::json!({}),
        }
    }
}

#[derive(Default, Clone)]
pub struct CapsuleRegistry {
    map: HashMap<String, Capsule>, // key = uri
}

impl CapsuleRegistry {
    pub fn add(&mut self, capsule: Capsule) {
        self.map.insert(capsule.uri.clone(), capsule);
    }
    pub fn get(&self, uri: &str) -> Option<&Capsule> {
        self.map.get(uri)
    }
    pub fn all(&self) -> impl Iterator<Item = (&String, &Capsule)> {
        self.map.iter()
    }
}

/// Abstract template engine: compiles templates and can produce a renderable context.
pub trait TemplateEngine: Send + Sync {
    /// Load templates from disk or memory; adapter decides how.
    fn load_all(&self) -> anyhow::Result<()>;
    /// Build the context map for a capsule (you can enrich this globally).
    fn context_for(&self, capsule: &Capsule) -> serde_json::Value {
        serde_json::json!({
            "name": capsule.name,
            "description": capsule.description,
            "uri": capsule.uri,
            "method": format!("{:?}", capsule.method),
            "data": capsule.data
        })
    }
}

/// Abstract HTTP server that knows how to route URIs to capsules and ask
/// the template engine to render them.
pub trait HttpServer: Send + Sync {
    /// Serve with the given registry + engine; blocks until shutdown.
    fn serve<'a>(
        &'a self,
        registry: Arc<CapsuleRegistry>,
        engine: Arc<dyn TemplateEngine>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>>;
}
pub struct TeraEngine {
    tera: parking_lot::RwLock<Tera>,
    /// Where your templates live, e.g. "templates"
    root: String,
}

impl TeraEngine {
    pub fn new(root: impl Into<String>) -> Self {
        Self {
            tera: parking_lot::RwLock::new(Tera::default()),
            root: root.into(),
        }
    }
}

impl TemplateEngine for TeraEngine {
    fn load_all(&self) -> anyhow::Result<()> {
        // Load all *.html.tera in the root directory
        let glob = format!("{}/**/*.html.tera", self.root);
        let mut tera = Tera::default();
        tera.add_template_files(
            globwalk::glob(&glob)?
                .filter_map(Result::ok)
                .map(|e| (e.path().to_path_buf(), None::<&str>)),
        )?;
        *self.tera.write() = tera;
        Ok(())
    }
}

#[derive(Clone)]
struct AppState {
    registry: Arc<CapsuleRegistry>,
    engine: Arc<dyn TemplateEngine>,
}

#[get("/<_..>", rank = 2)]
fn not_found() -> Template {
    Template::render("404", context! {})
}

#[get("/<path..>", rank = 1)]
fn catch_all(path: std::path::PathBuf, state: &State<AppState>) -> Template {
    // Normalize to "/xyz"
    let path = format!("/{}", path.display());
    let registry = state.registry.clone();
    let engine = state.engine.clone();
    if let Some(capsule) = registry.get(&path).cloned() {
        render_capsule(&capsule, engine.as_ref())
    } else {
        Template::render("404", context! { path })
    }
}
#[post("/<path..>", data = "<data>")]
fn handle_post(path: std::path::PathBuf, data: String, state: &State<AppState>) -> Template {
    let path_str = format!("/{}", path.display());
    let registry = state.registry.clone();
    let engine = state.engine.clone();
    if let Some(capsule) = registry.get(&path_str).cloned() {
        let mut capsule = capsule.clone();
        capsule.data = serde_json::json!({ "body": data });
        let ctx = engine.context_for(&capsule);
        return Template::render(capsule.template, ctx);
    }
    Template::render("404", context! { path: path_str })
}

// Helper function to render a capsule using the engine
fn render_capsule(capsule: &Capsule, engine: &dyn TemplateEngine) -> Template {
    let ctx = engine.context_for(capsule);
    Template::render(capsule.template.clone(), ctx)
}
/// Rocket server adapter
pub struct RocketTeraServer {
    /// Customize templates dir if needed (e.g., "templates")
    templates_dir: String,
}

impl RocketTeraServer {
    pub fn new(templates_dir: impl Into<String>) -> Self {
        Self {
            templates_dir: templates_dir.into(),
        }
    }
}

impl HttpServer for RocketTeraServer {
    fn serve<'a>(
        &'a self,
        registry: Arc<CapsuleRegistry>,
        engine: Arc<dyn TemplateEngine>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>> {
        Box::pin(async move {
            // Load templates once
            engine.load_all()?;

            // Build rocket with a custom Tera (so changes from engine.load_all are used)
            let state = AppState { registry, engine };

            let rocket = rocket::build()
                .manage(state)
                .attach(Template::custom({
                    let templates_dir = self.templates_dir.clone();
                    move |engines| {
                        // Tell Rocket to load *.html.tera from templates_dir
                        engines.tera.autoescape_on(vec![]);
                        engines
                            .tera
                            .add_template_files(
                                globwalk::glob(format!("{templates_dir}/**/*.html.tera"))
                                    .expect("glob ok")
                                    .filter_map(Result::ok)
                                    .map(|e| (e.path().to_path_buf(), None::<&str>)),
                            )
                            .expect("load templates");
                    }
                }))
                // You can mount once at "/" and let `catch_all` dispatch
                .mount("/", routes![catch_all, handle_post, not_found])
                .register("/", catchers![default_catcher]);

            rocket
                .ignite()
                .await
                .expect("msg")
                .launch()
                .await
                .expect("msg");
            Ok(())
        })
    }
}

#[rocket::catch(default)]
fn default_catcher(_status: rocket::http::Status, _req: &Request<'_>) -> Template {
    Template::render("404", context! {})
}
