use meilibridge::{
    api::{ApiServer, ApiState},
    config::{Config, ConfigLoader, ConfigValidator},
    error::Result,
    pipeline::{PipelineOrchestrator, StartupChecker},
    sync::SyncTaskManager,
};
use clap::{Parser, Subcommand};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tokio::signal;

#[derive(Parser)]
#[command(
    name = "meilibridge",
    version = env!("CARGO_PKG_VERSION"),
    about = "High-performance PostgreSQL to Meilisearch synchronization service",
    long_about = None
)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, env = "MEILIBRIDGE_CONFIG")]
    config: Option<String>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, env = "MEILIBRIDGE_LOG_LEVEL", default_value = "info")]
    log_level: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the synchronization service
    Run {
        /// Dry run mode (validate configuration without starting)
        #[arg(long)]
        dry_run: bool,
    },
    /// Validate configuration
    Validate,
    /// Generate sample configuration
    GenerateSample,
    /// Show version information
    Version,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing with the specified log level
    init_tracing(&cli.log_level);
    
    // Initialize Prometheus metrics
    meilibridge::metrics::init_process_metrics();

    info!("MeiliBridge v{}", env!("CARGO_PKG_VERSION"));

    // Handle commands
    match cli.command {
        Some(Commands::GenerateSample) => {
            println!("{}", generate_sample_config());
            return Ok(());
        }
        Some(Commands::Version) => {
            print_version_info();
            return Ok(());
        }
        Some(Commands::Validate) => {
            info!("Validating configuration...");
            let config = load_config(cli.config.as_deref())?;
            validate_config(&config)?;
            info!("Configuration is valid");
            return Ok(());
        }
        Some(Commands::Run { dry_run }) => {
            if dry_run {
                info!("Running in dry-run mode");
                let config = load_config(cli.config.as_deref())?;
                validate_config(&config)?;
                info!("Dry run completed successfully");
                return Ok(());
            }
        }
        None => {
            // Default command is run
        }
    }

    // Load configuration
    let config = load_config(cli.config.as_deref())?;
    
    // Validate configuration
    validate_config(&config)?;

    // Start the service
    run_service(config).await
}

fn init_tracing(log_level: &str) {
    use tracing_subscriber::fmt::time::ChronoLocal;
    
    // Custom event formatter
    let timer = ChronoLocal::new("%Y-%m-%d %H:%M:%S%.3f".to_string());
    
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_timer(timer)
        .with_ansi(true)
        .with_target(false)
        .with_thread_names(true)
        .with_thread_ids(true)
        .fmt_fields(tracing_subscriber::fmt::format::DefaultFields::new())
        .event_format(CustomFormatter);

    let filter = format!("meilibridge={},info", log_level);
    let filter_layer = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(filter));

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();
}

// Custom formatter for the log output
struct CustomFormatter;

impl<S, N> tracing_subscriber::fmt::FormatEvent<S, N> for CustomFormatter
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    N: for<'a> tracing_subscriber::fmt::FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &tracing_subscriber::fmt::FmtContext<'_, S, N>,
        mut writer: tracing_subscriber::fmt::format::Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        use tracing_subscriber::fmt::time::{FormatTime, ChronoLocal};
        
        // Format timestamp
        let timer = ChronoLocal::new("%Y-%m-%d %H:%M:%S%.3f".to_string());
        timer.format_time(&mut writer)?;
        write!(writer, " ")?;
        
        // Format level
        let level = event.metadata().level();
        match *level {
            tracing::Level::ERROR => write!(writer, "\x1b[31mERROR\x1b[0m")?,
            tracing::Level::WARN => write!(writer, "\x1b[33m WARN\x1b[0m")?,
            tracing::Level::INFO => write!(writer, " INFO")?,
            tracing::Level::DEBUG => write!(writer, "\x1b[36mDEBUG\x1b[0m")?,
            tracing::Level::TRACE => write!(writer, "\x1b[35mTRACE\x1b[0m")?,
        }
        write!(writer, " ")?;
        
        // Format thread name and ID
        let current_thread = std::thread::current();
        if let Some(name) = current_thread.name() {
            // Take last 8 characters of thread name
            let truncated_name = if name.len() > 8 {
                &name[name.len() - 8..]
            } else {
                name
            };
            write!(writer, "[{:8}", truncated_name)?;
        } else {
            write!(writer, "[{:8}", "unnamed")?;
        }
        
        // Add thread ID
        let thread_id = format!("{:?}", current_thread.id());
        if let Some(id_num) = thread_id.strip_prefix("ThreadId(").and_then(|s| s.strip_suffix(")")) {
            write!(writer, "-{}] ", id_num)?;
        } else {
            write!(writer, "-??] ")?;
        }
        
        // Format the actual message
        ctx.field_format().format_fields(writer.by_ref(), event)?;
        
        writeln!(writer)
    }
}

fn load_config(path: Option<&str>) -> Result<Config> {
    match path {
        Some(path) => {
            info!("Loading configuration from: {}", path);
            ConfigLoader::load_from_file(path)
        }
        None => {
            info!("Loading configuration from default locations");
            ConfigLoader::load()
        }
    }
}

fn validate_config(config: &Config) -> Result<()> {
    // Use the ConfigValidator for comprehensive validation
    let validator = ConfigValidator::new(config.clone());
    let report = validator.validate()?;
    
    // Print the validation report
    report.print();
    
    // Check if configuration is valid
    if !report.is_valid() {
        return Err(meilibridge::MeiliBridgeError::Validation(
            "Configuration validation failed. See errors above.".to_string(),
        ));
    }

    info!("  Application: {}", config.app.name);
    info!("  Instance ID: {}", config.app.instance_id);
    info!("  Sync tasks: {}", config.sync_tasks.len());
    
    for task in &config.sync_tasks {
        info!("  - Task '{}': {} -> {}", task.id, task.table, task.index);
    }

    Ok(())
}

async fn run_service(config: Config) -> Result<()> {
    info!("Starting MeiliBridge service");

    // Perform startup checks
    let startup_checker = StartupChecker::new(config.clone());
    startup_checker.check().await?;

    // Create the pipeline orchestrator
    let orchestrator = Arc::new(RwLock::new(PipelineOrchestrator::new(config.clone())?));
    
    // Create the sync task manager
    let task_manager = Arc::new(RwLock::new(SyncTaskManager::new(config.clone())));
    
    // Start the orchestrator
    {
        let mut orchestrator_guard = orchestrator.write().await;
        orchestrator_guard.start().await?;
    }
    
    // Create health registry and register health checks
    let health_registry = Arc::new(meilibridge::health::HealthRegistry::new());
    
    // Register PostgreSQL health check if available
    {
        let orchestrator_guard = orchestrator.read().await;
        if let Some(pg_health_check) = orchestrator_guard.create_postgres_health_check() {
            // Connect the health check connector
            // The health check will handle connection internally when checking health
            health_registry.register(pg_health_check).await;
        }
    }
    
    // Register Meilisearch health check
    health_registry.register(Box::new(
        meilibridge::health::MeilisearchHealthCheck::new(config.meilisearch.clone())
    )).await;
    
    // Register Redis health check if configured
    if !config.redis.url.is_empty() {
        health_registry.register(Box::new(
            meilibridge::health::RedisHealthCheck::new(config.redis.url.clone())
        )).await;
    }
    
    // Create API state with health registry and statement cache if using PostgreSQL
    let mut api_state = ApiState::new(orchestrator.clone(), task_manager.clone())
        .with_health_registry(health_registry.clone());
    
    // If using PostgreSQL, create a statement cache reference
    if let Some(source) = &config.source {
        if let meilibridge::config::SourceConfig::PostgreSQL(ref pg_config) = source {
            let cache_config = meilibridge::source::postgres::CacheConfig {
                max_size: pg_config.statement_cache.max_size,
                enabled: pg_config.statement_cache.enabled,
            };
            let statement_cache = Arc::new(meilibridge::source::postgres::StatementCache::new(cache_config));
            api_state = api_state.with_postgres_cache(statement_cache);
        }
    }
    
    // Also check multiple sources for PostgreSQL (use first one found)
    for named_source in &config.sources {
        if let meilibridge::config::SourceConfig::PostgreSQL(ref pg_config) = &named_source.config {
            let cache_config = meilibridge::source::postgres::CacheConfig {
                max_size: pg_config.statement_cache.max_size,
                enabled: pg_config.statement_cache.enabled,
            };
            let statement_cache = Arc::new(meilibridge::source::postgres::StatementCache::new(cache_config));
            api_state = api_state.with_postgres_cache(statement_cache);
            break; // Use first PostgreSQL source found
        }
    }
    
    // Start API server if enabled
    let api_handle = if config.api.host != "" && config.api.port != 0 {
        info!("Starting API server on {}:{}", config.api.host, config.api.port);
        let api_server = ApiServer::new(config.clone(), api_state);
        
        // Start API server in a separate task with error handling
        let api_handle = tokio::spawn(async move {
            match api_server.start().await {
                Ok(_) => info!("API server terminated normally"),
                Err(e) => {
                    error!("API server failed to start: {}", e);
                    // Don't crash the whole application if API server fails
                    // The service can still function without the API
                    warn!("Service will continue without API server");
                }
            }
        });
        
        // Give the API server a moment to start and check if it failed immediately
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        if api_handle.is_finished() {
            warn!("API server failed to start, but service will continue");
        }
        
        Some(api_handle)
    } else {
        info!("API server is disabled");
        None
    };
    
    info!("Service started successfully");
    info!("Press Ctrl+C to stop");

    // Wait for shutdown signal
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Shutdown signal received");
        }
        Err(err) => {
            error!("Unable to listen for shutdown signal: {}", err);
        }
    }

    // Stop the service
    info!("Stopping service...");
    
    // Stop API server
    if let Some(handle) = api_handle {
        handle.abort();
    }
    
    // Stop orchestrator
    {
        let mut orchestrator_guard = orchestrator.write().await;
        orchestrator_guard.stop().await?;
    }
    
    info!("Service stopped");
    
    // Give a brief moment for final logs to be written
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Force exit to ensure all threads are terminated
    std::process::exit(0);
}

fn generate_sample_config() -> &'static str {
    r#"# MeiliBridge Configuration Example
# This file demonstrates all available configuration options

# Application metadata
app:
  name: "MeiliBridge"
  instance_id: "prod-01"  # Optional, auto-generated if not specified
  tags:
    environment: "production"
    region: "us-east-1"

# PostgreSQL source configuration
source:
  type: postgresql
  postgresql:
    host: "localhost"
    port: 5432
    database: "myapp"
    user: "replication_user"
    password: "your_password"  # Can use ${POSTGRES_PASSWORD} for env var
    
    # Connection pool settings
    pool:
      max_size: 10
      min_idle: 2
      acquire_timeout: 30
      idle_timeout: 600
      max_lifetime: 1800
    
    # Replication settings
    replication:
      slot_name: "meilibridge_slot"
      publication_name: "meilibridge_pub"
      create_slot: true
      temporary_slot: false
      start_lsn: null  # Optional: specify starting position

# Meilisearch destination configuration
meilisearch:
  url: "http://localhost:7700"
  api_key: "your_master_key"  # Can use ${MEILI_MASTER_KEY}
  timeout: 30
  
  # Optional settings
  max_retries: 3
  retry_on_timeout: true

# Redis configuration for distributed mode
redis:
  url: "redis://localhost:6379"
  password: null  # Can use ${REDIS_PASSWORD}
  database: 0
  key_prefix: "meilibridge"
  
  pool:
    max_size: 10
    min_idle: 2
    connection_timeout: 5
    idle_timeout: 60

# Sync task definitions
sync_tasks:
  - id: "users_sync"
    table: "public.users"
    index: "users"
    primary_key: "id"
    
    # Start with full sync
    full_sync_on_start: true
    
    # Auto-start this task
    auto_start: true
    
    # Filter configuration (optional)
    filter:
      tables:
        whitelist: ["public.users"]
      event_types: ["create", "update", "delete"]
      conditions:
        - op: not_equals
          field: "deleted"
          value: true
    
    # Transform configuration (optional)
    transform:
      fields:
        public.users:
          email:
            type: lowercase
            fields: ["email"]
          full_name:
            type: compute
            expression: "concat(first_name, last_name)"
            to: "full_name"
    
    # Field mapping (optional)
    mapping:
      tables:
        public.users:
          name: "users"  # Rename table
          fields:
            user_id: "id"  # Rename field
            created_at: "created_timestamp"
      
      unmapped_fields_strategy: "include"  # include, exclude, or prefix
      unmapped_fields_prefix: "_"
    
    # Sync options
    options:
      batch_size: 1000
      batch_timeout_ms: 1000
      deduplicate: false
      
      retry:
        max_retries: 3
        initial_delay: 1000
        max_delay: 60000
        multiplier: 2.0

  - id: "products_sync"
    table: "public.products"
    index: "products"
    primary_key: "sku"
    full_sync_on_start: false
    auto_start: true
    
    options:
      batch_size: 500
      batch_timeout_ms: 2000

# API server configuration
api:
  enabled: true
  host: "0.0.0.0"
  port: 8080
  
  # CORS settings
  cors:
    enabled: true
    origins: ["http://localhost:3000"]
    methods: ["GET", "POST", "PUT", "DELETE"]
    headers: ["Content-Type", "Authorization"]
  
  # Authentication
  auth:
    enabled: true
    type: "bearer"
    tokens:
      - name: "admin"
        token: "your-admin-token"  # Can use ${API_ADMIN_TOKEN}
        role: "admin"
      - name: "readonly"
        token: "your-readonly-token"
        role: "read"

# Logging configuration
logging:
  level: "info"  # trace, debug, info, warn, error
  format: "pretty"  # pretty, json, compact
  
  # File output
  file:
    enabled: true
    path: "/var/log/meilibridge/app.log"
    rotation: "daily"  # daily, size, never
    max_size: "100MB"
    max_age: 7
    max_backups: 5
    compress: true
  
  # Optional: send logs to external service
  export:
    enabled: false
    endpoint: "https://logs.example.com"
    api_key: "your-api-key"
    batch_size: 100
    flush_interval: 10

# Feature flags
features:
  auto_recovery: true      # Automatically recover from failures
  health_checks: true      # Enable health check endpoints
  metrics_export: true     # Export Prometheus metrics
  distributed_mode: false  # Enable distributed mode with Redis

# Plugin configuration (optional)
plugins:
  directory: "./plugins"
  enabled:
    - "custom_transformer"
    - "slack_notifier"
"#
}

fn print_version_info() {
    println!("MeiliBridge v{}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("A high-performance PostgreSQL to Meilisearch synchronization service");
    println!("Repository: https://github.com/binary-touch/meilibridge");
    println!("License: MIT");
}
