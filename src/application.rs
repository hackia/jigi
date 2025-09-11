use rocket::{Request, State, catchers, get, post, routes};
use rocket_dyn_templates::{Template, context, tera::Tera};
// core.rs
use serde::Serialize;
use std::{collections::HashMap, sync::Arc};

/// Represents the HTTP methods supported by the application.
///
/// This enumeration provides a simple way to define and work with different
/// HTTP request methods such as `GET`, `POST`, `PUT`, and `DELETE`.
///
/// # Variants
/// - `GET`: Represents the HTTP GET method, typically used to retrieve data
///   from a server.
/// - `POST`: Represents the HTTP POST method, often used to send new data to
///   the server or submit form data.
/// - `PUT`: Represents the HTTP PUT method, generally used to update existing
///   resources or create a resource at a specific location.
/// - `DELETE`: Represents the HTTP DELETE method, used to delete resources
///   on the server.
///
/// # Traits
/// - `Debug`: Allows the `Method` enum variants to be formatted using the `Debug` trait.
/// - `Serialize`: Enables serialization of the `Method` enum variants when using
///   libraries like `serde`.
/// - `Clone`: Allows duplication of `Method` values.
/// - `Copy`: Permits the `Method` enum variants to be copied instead of moved.
///
/// # Examples
/// ```
/// use your_module::Method;
///
/// let method = Method::GET;
/// println!("{:?}", method); // Prints: GET
/// ```
#[derive(Debug, Serialize, Clone, Copy)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
}
/// Represents a `Capsule` structure that holds information about an HTTP request or operation.
///
/// This struct is commonly used to define and configure API calls or template-based payloads,
/// including metadata about the request and the associated HTTP method.
///
/// # Fields
///
/// * `template` - A `String` representing a predefined format or type for the capsule (e.g., a template identifier).
/// * `name` - A `String` defining the name of the capsule, which can serve as an identifier or description.
/// * `description` - A `String` providing additional details or information about what the capsule represents or its purpose.
/// * `uri` - A `String` representing the target URI (Uniform Resource Identifier) where the HTTP request is directed.
/// * `method` - A `Method` enumeration representing the HTTP method (e.g., GET, POST, PUT, DELETE) for the associated request.
/// * `data` - A `serde_json::Value` field used to store the payload or body of the HTTP request as a JSON structure.
///
/// # Traits
///
/// The struct derives the following traits:
/// * `Debug`: Enables formatting for debugging purposes.
/// * `Serialize`: Allows the struct to be serialized into formats like JSON.
/// * `Clone`: Enables creating deep copies of the struct.
///
/// # Example
///
/// ```rust
/// use serde_json::json;
/// use reqwest::Method;
/// use some_module::Capsule;
///
/// let capsule = Capsule {
///     template: "example_template".to_string(),
///     name: "Example Capsule".to_string(),
///     description: "This is an example capsule struct.".to_string(),
///     uri: "https://api.example.com/resource".to_string(),
///     method: Method::POST,
///     data: json!({
///         "key": "value"
///     }),
/// };
///
/// println!("{:?}", capsule);
/// ```
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
/// A trait defining the behavior of an HTTP server.
///
/// This trait provides the functionality required to start an HTTP server
/// with a provided template engine and registry. It ensures the server
/// can run asynchronously while supporting safe concurrency with `Send`
/// and `Sync` bounds. The server's execution is blocked until it is
/// explicitly shut down.
pub trait HttpServer: Send + Sync {
    ///
    /// Starts the service with the provided `registry` and `engine`.
    ///
    /// # Parameters
    ///
    /// * `&'a self` - A reference to the instance of the service that will handle the requests.
    /// * `registry` - An `Arc`-wrapped instance of `CapsuleRegistry` containing the registry information or handlers for the service.
    /// * `engine` - An `Arc`-wrapped implementation of the `TemplateEngine` trait, used for template rendering within the service.
    ///
    /// # Returns
    ///
    /// A pinned `Future` object that resolves to a `Result` containing `()` on success or an error of the type `anyhow::Error` upon failure.
    ///
    /// The returned future is asynchronous and must be awaited.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use your_crate::{CapsuleRegistry, TemplateEngine};
    ///
    /// let registry = Arc::new(CapsuleRegistry::new());
    /// let engine = Arc::new(MyTemplateEngine::new());
    /// let server = MyService::new();
    ///
    /// let result = server.serve(registry, engine).await;
    /// if let Err(e) = result {
    ///     eprintln!("Failed to run the server: {:?}", e);
    /// }
    /// ```
    ///
    /// This method allows for extensibility by passing shared, thread-safe resources and configurations
    /// (`CapsuleRegistry` and `TemplateEngine`) to the service to manage routes, templates, or other dynamic components.
    ///
    fn serve<'a>(
        &'a self,
        registry: Arc<CapsuleRegistry>,
        engine: Arc<dyn TemplateEngine>,
    ) -> std::pin::Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>>;
}

/// `TeraEngine` is a structure designed to manage and render templates using the Tera template engine.
///
/// # Fields
///
/// * `tera` - A thread-safe, read-write lock (`RwLock`) on a Tera instance. This enables safe concurrent access
///   to the template engine for loading, rendering, and managing templates dynamically.
///
/// * `root` - A `String` representing the directory where templates are stored. This is used to define the
///   base path for accessing template files.
///
/// # Examples
///
/// ```
/// use your_crate_name::TeraEngine;
/// use tera::Tera;
///
/// let tera = Tera::new("templates/**/*").unwrap();
/// let engine = TeraEngine {
///     tera: parking_lot::RwLock::new(tera),
///     root: "templates".to_string(),
/// };
/// // Now you can use `engine` to manage and render templates.
/// ```
pub struct TeraEngine {
    tera: parking_lot::RwLock<Tera>,
    /// Where your templates live, e.g. "templates"
    root: String,
}


impl TeraEngine {
    /// Creates a new instance of the struct containing a Tera template engine
    /// and a root directory.
    ///
    /// # Arguments
    ///
    /// * `root` - A value that can be converted into a `String`, representing the
    ///   root directory for the template engine.
    ///
    /// # Returns
    ///
    /// Returns a new instance of the struct initialized with a default `Tera`
    /// instance wrapped in a `parking_lot::RwLock` for thread-safe access,
    /// and the specified root directory.
    ///
    /// # Examples
    ///
    /// ```
    /// let instance = StructName::new("path/to/templates");
    /// ```
    pub fn new(root: impl Into<String>) -> Self {
        Self {
            tera: parking_lot::RwLock::new(Tera::default()),
            root: root.into(),
        }
    }
}

impl TemplateEngine for TeraEngine {
    ///
    /// Loads all `.html.tera` template files located in the root directory and its subdirectories into the `tera` template engine.
    ///
    /// # Arguments
    /// * `&self` - A reference to the current instance which holds the root directory path and handles the shared `tera` engine.
    ///
    /// # Behavior
    /// This method performs the following steps:
    /// 1. Creates a glob pattern to match all files with the `.html.tera` extension in the root directory and its subdirectories.
    /// 2. Initializes a default `Tera` template engine instance.
    /// 3. Uses the `globwalk` crate to walk the directory tree and find all matching `.html.tera` files, returning an iterator over their paths.
    /// 4. Adds the matching files to the `Tera` instance.
    /// 5. Updates the `tera` instance stored in the shared `tera` writeable reference with the newly loaded templates.
    ///
    /// # Returns
    /// * `Ok(())` - On successful loading of the templates.
    /// * `Err(anyhow::Error)` - If there is an error during the glob creation, file discovery, or template loading.
    ///
    /// # Errors
    /// This function returns an error in the following cases:
    /// - If the glob pattern fails to compile.
    /// - If there is an issue accessing the file system to enumerate matching template files.
    /// - If adding the template files to the `Tera` engine fails.
    ///
    /// # Example
    /// ```rust
    /// // Assuming `self` is an instance of a struct containing the `root` directory and `tera`.
    /// self.load_all()?;
    /// ```
    ///
    /// # Dependencies
    /// - This method relies on the `globwalk` crate to match files in the filesystem.
    /// - Uses the `Tera` crate for template engine management.
    /// - Requires `anyhow` for error handling.
    ///
    /// # Safety
    /// * Ensure that the `root` directory exists and is accessible.
    /// * Ensure no conflicts with other threads accessing or modifying the `tera` instance.
    ///
    /// Note: File loading is recursive, so deep directory structures with many matching files may impact performance.
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

/// Represents the state of the application.
///
/// This structure holds shared resources that are needed throughout the application.
/// It is designed to be passed around in a thread-safe manner using `Arc`.
///
/// # Fields
///
/// * `registry`: An `Arc` to a `CapsuleRegistry` instance.
///   - This registry is likely responsible for managing and accessing various capsules,
///     which may represent plugins, modules, or components within the application.
///
/// * `engine`: An `Arc` to a trait object implementing the `TemplateEngine` trait.
///   - This provides the functionality for rendering templates, enabling dynamic content
///     generation based on templates and data.
///
/// # Derives
///
/// * `Clone`: The structure can be cloned, ensuring that the underlying `Arc`
///   references are incremented properly and that multiple references to the same
///   data can safely coexist.
///
/// # Example
///
/// ```rust
/// use std::sync::Arc;
///
/// let registry = Arc::new(CapsuleRegistry::new());
/// let engine = Arc::new(MyTemplateEngine::new());
///
/// let app_state = AppState {
///     registry,
///     engine,
/// };
///
/// // Clone the app state if needed
/// let cloned_state = app_state.clone();
/// ```
#[derive(Clone)]
struct AppState {
    registry: Arc<CapsuleRegistry>,
    engine: Arc<dyn TemplateEngine>,
}
/// Handler function for the "Not Found" (404) error page.
///
/// This function is triggered when a user accesses a route that does not
/// explicitly match any defined routes in the application, except those
/// with higher rank. It uses Rocket's `Template` engine to render a
/// custom "404" error page.
///
/// # Route
/// - `/<_..>`: Matches any URL path with any number of segments (using `_..` as a wildcard).
/// - `rank = 2`: Specifies that this route has a lower precedence (rank) compared to other routes with a higher rank.
///
/// # Returns
/// - A rendered "404" template page with an empty context.
///
/// # Example
/// If a user navigates to a non-existent path like `/nonexistent/route`, this
/// handler will render the "404.html" template page with no additional context.
///
/// # Requirements
/// - Ensure that the "404" template is available in the template directory.
/// - Requires the `Template` feature to be enabled in the Rocket application.
///
/// # Usage
/// ```rust
/// use rocket_contrib::templates::Template;
///
/// #[get("/<_..>", rank = 2)]
/// fn not_found() -> Template {
///     Template::render("404", context! {})
/// }
/// ```
#[get("/<_..>", rank = 2)]
fn not_found() -> Template {
    Template::render("404", context! {})
}
/// Catch-all route handler for dynamic paths.
///
/// This route captures requests to paths that are not explicitly defined by other routes and processes them by checking
/// if the path corresponds to a known capsule in the registry. If the specified path matches a capsule, it renders the content
/// using the rendering engine. Otherwise, it returns a "404 Not Found" template.
///
/// # Arguments
///
/// * `path` - The path of the request, represented as a `std::path::PathBuf`. It is automatically extracted from the URL by Rocket
///   and normalized to begin with a forward slash (e.g., "/example").
/// * `state` - A reference to the shared application state (`AppState`) that contains important resources such as the registry
///   for looking up capsules and the rendering engine.
///
/// # Returns
///
/// * `Template` - If a matching capsule is found in the `registry`, its content is rendered using the shared `engine`
///   and returned as a `Template`. If no match is found, a "404 Not Found" page is rendered and returned.
///
/// # Behavior
///
/// 1. Normalizes the incoming path to ensure it begins with a "/".
/// 2. Checks the `registry` (a component of `AppState`) for a capsule corresponding to the requested path.
/// 3. If a capsule is found:
///     - The capsule is passed to the `render_capsule` function along with the rendering engine to generate the response content.
/// 4. If no capsule is found:
///     - A "404 Not Found" template is served with the requested path included in the template context.
///
/// # Route Details
///
/// * Method: `GET`
/// * Path: `/<path..>` (catches all paths that are not explicitly matched by other routes.)
/// * Rank: `1` (higher specificity for catching requests early.)
///
/// # Example
///
/// Assume the application has the following paths set up in the registry:
///
/// ```no_run
/// use rocket::State;
/// use rocket_contrib::templates::Template;
/// // ... setup code
/// state.registry.insert("/example", Capsule::new("Hello, world!"));
/// ```
///
/// 1. Request to `/example`:
///    - Matches a capsule in the registry.
///    - Renders "Hello, world!" as the response content via the rendering engine.
/// 2. Request to `/unknown`:
///    - Does not match any capsule in the registry.
///    - Returns a "404 Not Found" response.
///
/// # Dependencies
///
/// This function relies on:
/// * `AppState` to provide the `registry` and `engine`.
/// * `render_capsule` to process capsules.
/// * `rocket_contrib::templates::Template` for rendering templates.
///
/// # Note
///
/// Ensure that the `AppState` is properly initialized with a valid registry and rendering engine in the Rocket application
/// for this route to function correctly.
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
/// Handles POST requests to dynamic routes, parses the request body, and renders a template based on the request path.
///
/// # Parameters
/// - `path`: A `PathBuf` representing the dynamic route extracted from the URL.
/// - `data`: A `String` containing the body of the POST request.
/// - `state`: A reference to the application state (`State<AppState>`), which holds shared data like a registry and engine.
///
/// # Returns
/// A `Template` that is rendered based on the state and request data. If the path matches an entry in the registry,
/// the corresponding template is rendered with the appropriate context. Otherwise, a "404" template is rendered.
///
/// # Behavior
/// - The function converts the request path into a string and looks it up in the `registry` stored in the application state.
/// - If a corresponding "capsule" (a unit of template and data) is found in the registry:
///   - The capsule is cloned and the POST request body is embedded as JSON under the key `"body"`.
///   - A rendering context is generated for the capsule using the rendering engine.
///   - The specified template is rendered with the constructed context.
/// - If no matching capsule is found in the registry, it renders the "404" template, passing the requested path as part
///   of the rendering context.
///
/// # Example
/// ```
/// // Route: POST /example/path
/// // Request Body: "test data"
///
/// // Assuming a registry with "/example/path" mapped to a template:
/// // - Template will be rendered with the body "test data" injected into the context.
///
/// // If no entry exists for "/example/path":
/// // - The "404" template will be rendered with the path in the context.
/// ```
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

/// Renders a capsule using the provided template engine.
///
/// This function takes a reference to a `Capsule` and a dynamic reference to
/// a `TemplateEngine`. It generates a rendering context for the capsule using
/// the template engine and then renders the specified capsule template using
/// the generated context.
///
/// # Arguments
///
/// * `capsule` - A reference to the `Capsule` object to be rendered. The `Capsule`
///   contains the data and the name of the template that will be used for rendering.
/// * `engine` - A dynamic reference to an object implementing the `TemplateEngine`
///   trait, which provides the necessary functionality to create a rendering context
///   and render templates.
///
/// # Returns
///
/// A `Template` object, which is the rendered result of the specified capsule template.
///
/// # Example
///
/// ```rust
/// let capsule = Capsule::new("example_template", some_data);
/// let engine = MyTemplateEngine::new();
/// let rendered_template = render_capsule(&capsule, &engine);
/// println!("{}", rendered_template.content());
/// ```
///
/// # Panics
///
/// This function might panic if:
/// - The template specified in the `Capsule` does not exist or cannot be cloned.
/// - The `TemplateEngine` implementation encounters an error while creating the
///   rendering context or rendering the template.
///
/// # See Also
///
/// * `Capsule` - The data structure representing a unit of information to be rendered.
/// * `TemplateEngine` - The trait that must be implemented by the template engine used
///   for rendering.
/// * `Template::render` - The method used to render a template with a given context.
fn render_capsule(capsule: &Capsule, engine: &dyn TemplateEngine) -> Template {
    let ctx = engine.context_for(capsule);
    Template::render(capsule.template.clone(), ctx)
}
/// A structure representing a server configuration for Rocket with Tera templates.
///
/// This structure is used to define and customize the directory where Tera template files
/// are located, enabling the server to render dynamic content.
///
/// # Fields
///
/// * `templates_dir` - A `String` specifying the directory where the Tera templates are stored.
///   By default, this can be customized to match the specific path to the templates directory
///   (e.g., "templates").
///
/// # Example
///
/// ```rust
/// let server = RocketTeraServer {
///     templates_dir: String::from("custom_templates"),
/// };
/// println!("Templates directory: {}", server.templates_dir);
/// ```
pub struct RocketTeraServer {
    templates_dir: String,
}

impl RocketTeraServer {
    /// Creates a new instance of the struct with the specified templates' directory.
    ///
    /// # Arguments
    ///
    /// * `templates_dir` - A value that can be converted into a `String`, representing the path to the directory
    ///   where templates are stored.
    ///
    /// # Returns
    ///
    /// Returns an instance of the struct initialized with the given `templates_dir`.
    ///
    /// # Example
    ///
    /// ```
    /// let instance = StructName::new("path/to/templates");
    /// ```
    pub fn new(templates_dir: impl Into<String>) -> Self {
        Self {
            templates_dir: templates_dir.into(),
        }
    }
}

impl HttpServer for RocketTeraServer {
    /// Serves the application by setting up and launching a Rocket web server.
    ///
    /// # Parameters
    /// - `registry`: An `Arc` wrapped instance of `CapsuleRegistry` used to manage application data.
    /// - `engine`: An `Arc` wrapped trait object implementing `TemplateEngine` used to manage and render templates.
    ///
    /// # Returns
    /// - A pinned `Future` that resolves to a `Result<()>` indicating the success or failure of the Rocket server setup and execution.
    ///
    /// # Functionality
    /// 1. Loads all templates using the provided `TemplateEngine` instance.
    /// 2. Builds a Rocket instance with customized Tera template handling:
    ///    - Configures Rocket to load templates from the specified directory (`templates_dir`) with filenames ending in `.html.tera`.
    ///    - Disables auto-escaping for template rendering.
    /// 3. Manages application state using `AppState`, allowing access to the registry and template engine during request handling.
    /// 4. Mounts the Rocket instance to the root path (`"/"`) with predefined routes (`catch_all`, `handle_post`, `not_found`) and catchers (`default_catcher`).
    /// 5. Initiates and launches the Rocket server asynchronously.
    /// 6. Returns a `Result` indicating whether the Rocket server launched successfully or encountered an error.
    ///
    /// # Notes
    /// - The `engine.load_all()` call is critical for preloading all templates before initializing the Rocket server.
    /// - The `globwalk` dependency is used to iterate over template files matching the specified pattern.
    /// - Ensure that `self.templates_dir` is correctly set to the directory containing your `.html.tera` template files.
    ///
    /// # Errors
    /// - If the template loading (`engine.load_all()`) fails, an error is returned.
    /// - If the Rocket server fails to ignite or launch, an error is returned.
    ///
    /// # Example Usage
    /// ```rust
    /// let app = MyApp { templates_dir: "templates".to_string() };
    /// let registry = Arc::new(CapsuleRegistry::new());
    /// let engine = Arc::new(MyTemplateEngine::new());
    ///
    /// let server_future = app.serve(registry, engine);
    /// tokio::runtime::Runtime::new().unwrap().block_on(server_future).unwrap();
    /// ```
    fn serve<'a>(
        &'a self,
        registry: Arc<CapsuleRegistry>,
        engine: Arc<dyn TemplateEngine>,
    ) -> std::pin::Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>> {
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
/// A default error catcher for handling HTTP errors in a Rocket application.
///
/// This function is invoked whenever an unhandled error occurs that is not caught
/// by a more specific error catcher. It serves as a fallback to render a custom
/// error page for HTTP errors like "404 Not Found" or other server errors.
///
/// # Parameters
///
/// * `_status`:
///     - The HTTP status code associated with the error.
///     - It includes details about the nature of the error (e.g., 404, 500).
///
/// * `_req`:
///     - A reference to the HTTP `Request` object.
///     - Provides details about the incoming request, such as headers and URI,
///       but it is unused in this implementation.
///
/// # Returns
///
/// * `Template`:
///     - Renders an error template (in this case, "404") with no context data passed.
///     - This could be customized to display more detailed error information or redesign the error page.
///
/// # Usage
///
/// Add this function to your Rocket application to customize what users see
/// when they encounter an error status that doesn't have a specific handler.
///
/// Example:
///
/// ```rust
/// #[rocket::catch(default)]
/// fn default_catcher(_status: rocket::http::Status, _req: &Request<'_>) -> Template {
///     Template::render("404", context! {})
/// }
///
/// #[launch]
/// fn rocket() -> _ {
///     rocket::build()
///         .mount("/", routes![...])
///         .register("/", catchers![default_catcher])
/// }
/// ```
///
/// This example renders a "404" template for errors with a default context.
#[rocket::catch(default)]
fn default_catcher(_status: rocket::http::Status, _req: &Request<'_>) -> Template {
    Template::render("404", context! {})
}
