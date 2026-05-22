use anyhow::{anyhow, Context, Result};
use cdrs_tokio::authenticators::StaticPasswordAuthenticatorProvider;
use cdrs_tokio::cluster::session::{RustlsSessionBuilder, Session, SessionBuilder};
use cdrs_tokio::cluster::{NodeRustlsConfigBuilder, RustlsConnectionManager};
use cdrs_tokio::frame::message_result::ColType;
use cdrs_tokio::load_balancing::RoundRobinLoadBalancingStrategy;
use cdrs_tokio::transport::TransportRustls;
use cdrs_tokio::types::blob::Blob;
use cdrs_tokio::types::decimal::Decimal;
use cdrs_tokio::types::rows::Row;
use cdrs_tokio::types::{ByIndex, IntoRustByIndex};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::env;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::TempDir;
use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName};
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tracing::{error, info, warn};
use zip::ZipArchive;

type AstraSession = Session<
    TransportRustls,
    RustlsConnectionManager,
    RoundRobinLoadBalancingStrategy<TransportRustls, RustlsConnectionManager>,
>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SecureConnectConfig {
    host: String,
    #[serde(alias = "cql_port")]
    cql_port: u16,
    keyspace: Option<String>,
    ca_cert_location: String,
    key_location: String,
    cert_location: String,
}

struct ExtractedBundle {
    _temp_dir: TempDir,
    root: PathBuf,
    config: SecureConnectConfig,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let bundle_path =
        env::var("ASTRA_SECURE_CONNECT_BUNDLE").expect("ASTRA_SECURE_CONNECT_BUNDLE not set");
    let client_id = env::var("ASTRA_CLIENT_ID").expect("ASTRA_CLIENT_ID not set");
    let client_secret = env::var("ASTRA_CLIENT_SECRET").expect("ASTRA_CLIENT_SECRET not set");

    info!("Connecting to Astra DB using bundle: {}", bundle_path);

    match connect_to_astra_db(&bundle_path, &client_id, &client_secret).await {
        Ok(session) => {
            println!("Successfully connected to Astra DB");
            info!("Successfully connected to Astra DB");

            if let Err(error) = query_system_local(&session).await {
                error!("Query failed: {}", error);
            } else {
                println!("Query executed successfully");
                info!("Query executed successfully");
            }
        }
        Err(error) => {
            error!("Connection failed: {:#}", error);
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn connect_to_astra_db(
    secure_connect_bundle_path: impl AsRef<Path>,
    client_id: &str,
    client_secret: &str,
) -> Result<AstraSession> {
    let bundle = extract_secure_connect_bundle(secure_connect_bundle_path.as_ref())?;
    let host = bundle.config.host.clone();
    let contact_point = format!("{}:{}", host, bundle.config.cql_port);

    info!("Connecting to Astra host: {}", contact_point);

    let tls_config = build_tls_config_from_bundle(&bundle)?;
    let dns_name = ServerName::try_from(host.clone())
        .map_err(|_| anyhow!("invalid Astra host name in bundle config: {host}"))?;
    let auth = StaticPasswordAuthenticatorProvider::new(client_id, client_secret);

    let node_config = NodeRustlsConfigBuilder::new(dns_name, Arc::new(tls_config))
        .with_contact_point(contact_point.into())
        .with_authenticator_provider(Arc::new(auth))
        .build()
        .await?;

    let session_builder =
        RustlsSessionBuilder::new(RoundRobinLoadBalancingStrategy::new(), node_config);

    let session_builder = match bundle.config.keyspace.as_deref() {
        Some(keyspace) if !keyspace.is_empty() => session_builder.with_keyspace(keyspace.into()),
        _ => session_builder,
    };

    Ok(session_builder.build().await?)
}

async fn query_system_local(session: &AstraSession) -> Result<()> {
    let query = "select * from default_keyspace.orders limit 5;";

    info!("Executing query: {}", query);

    let response_body = session
        .query(query)
        .await?
        .response_body()
        .context("query response did not contain a body")?;
    let column_specs = response_body
        .as_rows_metadata()
        .context("query response did not contain row metadata")?
        .col_specs
        .clone();
    let rows = response_body
        .into_rows()
        .context("query response did not contain rows")?;

    println!("Returned {} row(s)", rows.len());
    info!("Returned {} row(s)", rows.len());
    for row in rows {
        let row_values = format_row(&row, &column_specs);
        println!("row: {row_values:?}");
        info!("Query result row: {:?}", row_values);
    }

    Ok(())
}

fn format_row(
    row: &Row,
    column_specs: &[cdrs_tokio::frame::message_result::ColSpec],
) -> BTreeMap<String, String> {
    column_specs
        .iter()
        .enumerate()
        .map(|(index, column)| {
            let value = format_column_value(row, index, column.col_type.id);
            (column.name.clone(), value)
        })
        .collect()
}

fn format_column_value(row: &Row, index: usize, column_type: ColType) -> String {
    if row.is_empty(index) {
        return "null".into();
    }

    match column_type {
        ColType::Ascii | ColType::Varchar => format_value::<String>(row, index),
        ColType::Bigint | ColType::Counter | ColType::Timestamp | ColType::Time => {
            format_value::<i64>(row, index)
        }
        ColType::Blob => format_blob(row, index),
        ColType::Boolean => format_value::<bool>(row, index),
        ColType::Decimal => format_value::<Decimal>(row, index),
        ColType::Double => format_value::<f64>(row, index),
        ColType::Float => format_value::<f32>(row, index),
        ColType::Int | ColType::Date => format_value::<i32>(row, index),
        ColType::Uuid | ColType::Timeuuid => format_value::<uuid::Uuid>(row, index),
        ColType::Varint => format_value::<num_bigint::BigInt>(row, index),
        ColType::Inet => format_value::<std::net::IpAddr>(row, index),
        ColType::Smallint => format_value::<i16>(row, index),
        ColType::Tinyint => format_value::<i8>(row, index),
        unsupported => format!("<unsupported {unsupported:?}>"),
    }
}

fn format_value<T>(row: &Row, index: usize) -> String
where
    T: std::fmt::Debug,
    Row: IntoRustByIndex<T>,
{
    match row.by_index::<T>(index) {
        Ok(Some(value)) => format!("{value:?}"),
        Ok(None) => "null".into(),
        Err(error) => format!("<decode error: {error}>"),
    }
}

fn format_blob(row: &Row, index: usize) -> String {
    match row.by_index::<Blob>(index) {
        Ok(Some(blob)) => format!("<blob {} bytes>", blob.into_vec().len()),
        Ok(None) => "null".into(),
        Err(error) => format!("<decode error: {error}>"),
    }
}

fn extract_secure_connect_bundle(bundle_path: &Path) -> Result<ExtractedBundle> {
    let bundle_file = File::open(bundle_path)
        .with_context(|| format!("failed to open SCB zip: {}", bundle_path.display()))?;
    let mut archive = ZipArchive::new(bundle_file).context("failed to read SCB zip archive")?;
    let temp_dir = tempfile::tempdir().context("failed to create temporary SCB directory")?;

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let Some(enclosed_name) = entry.enclosed_name() else {
            continue;
        };

        let output_path = temp_dir.path().join(enclosed_name);
        if entry.is_dir() {
            fs::create_dir_all(&output_path)?;
            continue;
        }

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut output_file = File::create(&output_path)?;
        std::io::copy(&mut entry, &mut output_file)?;
    }

    let config_path = temp_dir.path().join("config.json");
    let config_json = fs::read_to_string(&config_path)
        .with_context(|| format!("SCB is missing {}", config_path.display()))?;
    log_scb_config_json(&config_path, &config_json);
    let config = serde_json::from_str(&config_json).with_context(|| {
        format!(
            "failed to parse SCB config.json at {}",
            config_path.display()
        )
    })?;

    Ok(ExtractedBundle {
        root: temp_dir.path().to_path_buf(),
        _temp_dir: temp_dir,
        config,
    })
}

fn log_scb_config_json(config_path: &Path, config_json: &str) {
    match serde_json::from_str::<serde_json::Value>(config_json) {
        Ok(serde_json::Value::Object(mut config)) => {
            for key in ["keyStorePassword", "trustStorePassword", "pfxCertPassword"] {
                if config.contains_key(key) {
                    config.insert(
                        key.to_string(),
                        serde_json::Value::String("<redacted>".into()),
                    );
                }
            }

            let formatted = serde_json::to_string_pretty(&serde_json::Value::Object(config))
                .unwrap_or_default();
            println!(
                "Extracted SCB config from {}:\n{}",
                config_path.display(),
                formatted
            );
        }
        Ok(config) => {
            warn!(
                "Extracted SCB config from {} is not a JSON object: {}",
                config_path.display(),
                config
            );
        }
        Err(error) => {
            warn!(
                "Failed to parse extracted SCB config as JSON for logging at {}: {}",
                config_path.display(),
                error
            );
            println!("Raw extracted SCB config:\n{}", config_json);
        }
    }
}

fn build_tls_config_from_bundle(bundle: &ExtractedBundle) -> Result<ClientConfig> {
    let ca_path = bundle_path(&bundle.root, &bundle.config.ca_cert_location);
    let cert_path = bundle_path(&bundle.root, &bundle.config.cert_location);
    let key_path = bundle_path(&bundle.root, &bundle.config.key_location);

    let mut root_store = RootCertStore::empty();
    let ca_certs = load_certs(&ca_path)?;
    if ca_certs.is_empty() {
        return Err(anyhow!("SCB CA file contains no certificates"));
    }
    for cert in ca_certs {
        root_store
            .add(cert)
            .with_context(|| format!("failed to add CA certificate from {}", ca_path.display()))?;
    }

    let client_certs = load_certs(&cert_path)?;
    if client_certs.is_empty() {
        return Err(anyhow!(
            "SCB client certificate file contains no certificates"
        ));
    }
    let client_key = load_private_key(&key_path)?;

    ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_client_auth_cert(client_certs, client_key)
        .context("failed to build mTLS client config from SCB")
}

fn bundle_path(root: &Path, location: &str) -> PathBuf {
    root.join(location.trim_start_matches("./"))
}

fn load_certs(path: &Path) -> Result<Vec<CertificateDer<'static>>> {
    let file = File::open(path)
        .with_context(|| format!("failed to open certificate file: {}", path.display()))?;
    let mut reader = BufReader::new(file);

    rustls_pemfile::certs(&mut reader)
        .collect::<std::result::Result<Vec<_>, _>>()
        .with_context(|| format!("failed to parse certificate file: {}", path.display()))
}

fn load_private_key(path: &Path) -> Result<PrivateKeyDer<'static>> {
    let file =
        File::open(path).with_context(|| format!("failed to open key file: {}", path.display()))?;
    let mut reader = BufReader::new(file);
    let mut key_bytes = Vec::new();
    reader.read_to_end(&mut key_bytes)?;

    let mut pkcs8_reader = BufReader::new(key_bytes.as_slice());
    if let Some(key) = rustls_pemfile::pkcs8_private_keys(&mut pkcs8_reader).next() {
        return Ok(PrivateKeyDer::from(key.with_context(|| {
            format!("failed to parse PKCS#8 private key: {}", path.display())
        })?));
    }

    let mut rsa_reader = BufReader::new(key_bytes.as_slice());
    if let Some(key) = rustls_pemfile::rsa_private_keys(&mut rsa_reader).next() {
        return Ok(PrivateKeyDer::from(key.with_context(|| {
            format!("failed to parse RSA private key: {}", path.display())
        })?));
    }

    Err(anyhow!(
        "SCB private key file contains no supported private key: {}",
        path.display()
    ))
}
