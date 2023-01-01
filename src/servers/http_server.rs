use axum::{
	extract::State,
	http::{Method, StatusCode},
	routing::{get, get_service, post},
	Json, Router,
};

use serde_json::{json, Value};
use sp_core::{crypto::Ss58Codec, Pair};
use std::time::SystemTime;

use crate::servers::server_common;

use std::path::PathBuf;
use tower_http::{
	cors::{Any, CorsLayer},
	services::ServeDir,
};

use crate::chain::{
	chain::{get_nft_data_handler, rpc_query, submit_tx},
	nft::{retrieve_secret_shares, store_secret_shares},
};

use crate::attestation;
use crate::backup::admin::{backup_fetch_secrets, backup_push_secrets};

#[derive(Clone)]
pub struct StateConfig {
	pub owner_key: schnorrkel::Keypair,
	pub enclave_key: sp_core::sr25519::Pair,
	pub seal_path: String,
	pub identity: String,
}

/* HTTP Server */
pub async fn http_server(
	port: &u16,
	identity: &str,
	account: &str,
	certfile: &str,
	keyfile: &str,
	seal_path: &str,
) {
	let account_keys: Vec<&str> = account.split("_").collect();
	let private_bytes = hex::decode(account_keys[0]).expect("Error reading account data");
	let public_bytes = hex::decode(account_keys[1]).expect("Error reading account data");
	let account_pair = schnorrkel::Keypair {
		secret: schnorrkel::SecretKey::from_bytes(&private_bytes).unwrap(),
		public: schnorrkel::PublicKey::from_bytes(&public_bytes).unwrap(),
	};

	let (enclave_pair, _) = sp_core::sr25519::Pair::generate();

	let state_config = StateConfig {
		owner_key: account_pair,
		enclave_key: enclave_pair,
		seal_path: seal_path.to_owned(),
		identity: identity.to_string(),
	};

	let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");

	let cors = CorsLayer::new()
		// allow `GET` and `POST` when accessing the resource
		.allow_methods([Method::GET, Method::POST])
		// allow requests from any origin
		.allow_origin(Any);

	let http_app = Router::new()
		.fallback_service(
			get_service(ServeDir::new(assets_dir).append_index_html_on_directories(true))
				.handle_error(|error: std::io::Error| async move {
					(
						StatusCode::INTERNAL_SERVER_ERROR,
						format!("Unhandled internal error: {}", error),
					)
				}),
		)
		.route("/health", get(get_health_status))
		.layer(cors)
		// TEST APIS
		.route("/api/getNFTData/:nft_id", get(get_nft_data_handler))
		.route("/api/rpcQuery/:blocknumber", get(rpc_query))
		.route("/api/submitTx/:amount", get(submit_tx))
		// CENTRALIZED BACKUP API
		.route("/api/backup/fetchEnclaveSecrets", post(backup_fetch_secrets))
		.route("/api/backup/pushEnclaveSecrets", post(backup_push_secrets))
		// SECRET SHARING API
		.route("/api/nft/storeSecretShares", post(store_secret_shares))
		.route("/api/nft/retrieveSecretShares", post(retrieve_secret_shares))
		.with_state(state_config);

	server_common::serve(http_app, port, certfile, keyfile).await;
}

/*  -------------Handlers------------- */
async fn get_health_status(State(state): State<StateConfig>) -> Json<Value> {
	let time: chrono::DateTime<chrono::offset::Utc> = SystemTime::now().into();

	let quote_vec = attestation::ra::generate_quote();

	let operator =
		sp_core::sr25519::Public::from_raw(state.owner_key.public.to_bytes()).to_ss58check();
	let checksum = self_check();

	Json(json!({
		"status": 200,
		"date": time.format("%Y-%m-%d %H:%M:%S").to_string(),
		"description": "SGX server is running!".to_string(),
		"encalve_address": state.enclave_key.public().to_ss58check(),
		"operator_address": operator,
		"binary_hash" : checksum,
		"quote": quote_vec,
	}))
}

fn self_check() -> Result<String, String> {
	// Check running address

	use sysinfo::get_current_pid;

	let mut binary_path = match get_current_pid() {
		Ok(pid) => {
			let path_string = "/proc/".to_owned() + &pid.to_string() + "/exe";
			let binpath = std::path::Path::new(&path_string).read_link().unwrap();
			binpath
		},
		Err(e) => {
			tracing::error!("failed to get current pid: {}", e);
			std::path::PathBuf::new()
		},
	};

	// Verify Ternoa hash/signature
	let bytes = std::fs::read(binary_path.clone()).unwrap();
	let hash = sha256::digest(bytes.as_slice());

	binary_path.pop(); // remove binary name
	binary_path.push("checksum");

	let binary_hash = std::fs::read_to_string(binary_path.clone()).expect(&format!(
		"Binary-checksum path not found : {}",
		binary_path.clone().to_str().unwrap()
	));

	let binary_hash = binary_hash
		.strip_suffix("\r\n")
		.or(binary_hash.strip_suffix("\n"))
		.unwrap_or(&binary_hash);

	if binary_hash != hash {
		tracing::error!("Binary hash doesn't match!");
		return Err(hash);
	} else {
		tracing::info!("Binary hash match : {}", hash);
		return Ok(hash);
	}
}
