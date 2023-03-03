use std::{
	collections::BTreeMap,
	fs::OpenOptions,
	io::{Read, Seek, Write},
};

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use super::verify::RequesterType;

/* ******************************
		LOG-FILE STRUCTURE
****************************** */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NFTType {
	SECRET,
	CAPSULE,
	NONE,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum LogType {
	STORE,
	VIEW,
	BURN,
	NONE,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LogAccount {
	pub address: String,
	pub role: RequesterType,
}

impl LogAccount {
	pub fn new(address: String, role: RequesterType) -> LogAccount {
		LogAccount { address, role }
	}
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LogStruct {
	pub date: String,
	pub account: LogAccount,
	pub event: LogType,
}

impl LogStruct {
	pub fn new(account: LogAccount, event: LogType) -> LogStruct {
		let current_date: chrono::DateTime<chrono::offset::Utc> =
			std::time::SystemTime::now().into();
		let date = current_date.format("%Y-%m-%d %H:%M:%S").to_string();
		LogStruct { date, account, event }
	}
}

type Index = u32;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LogFile {
	pub secret_nft: BTreeMap<Index, LogStruct>,
	pub capsule: BTreeMap<Index, LogStruct>,
}

impl LogFile {
	pub fn new() -> LogFile {
		LogFile { secret_nft: BTreeMap::new(), capsule: BTreeMap::new() }
	}

	pub fn insert_new_nft_log(&mut self, log: LogStruct) {
		let index = self.secret_nft.len() as u32;
		self.secret_nft.insert(index, log);
	}

	pub fn insert_new_capsule_log(&mut self, log: LogStruct) {
		let index = self.capsule.len() as u32;
		self.capsule.insert(index, log);
	}
}

pub fn update_log_file_view(
	file_path: String,
	requester_address: String,
	requester_type: RequesterType,
	log_type: LogType,
	nft_type: &str,
) {
	debug!("4-7 update log file view");

	let mut log_file = OpenOptions::new()
		.read(true)
		.write(true)
		.append(false)
		.open(file_path)
		.expect("Unable to open log file");

	let mut old_logs = String::new();
	log_file.read_to_string(&mut old_logs).unwrap(); // TODO: manage unwrap()
	log_file.seek(std::io::SeekFrom::Start(0)).unwrap();

	let mut log_file_struct: LogFile = serde_json::from_str(&old_logs).unwrap(); // TODO: manage unwrap()

	let log_account = LogAccount::new(requester_address, requester_type);
	let new_log = LogStruct::new(log_account, log_type);

	if nft_type == "capsule" {
		log_file_struct.insert_new_capsule_log(new_log);
	} else if nft_type == "secret-nft" {
		log_file_struct.insert_new_nft_log(new_log);
	}

	let log_buf = serde_json::to_vec(&log_file_struct).unwrap(); // TODO: manage unwrap()
	log_file.write_all(&log_buf).unwrap(); // TODO: manage unwrap()
}

/* **********************
		 TEST
********************** */

#[cfg(test)]
mod test {
	use super::*;

	#[tokio::test]
	async fn read_log_test() {
		let store_body = r#"
        {
            "secret_nft": {
                "0": {
                    "date": "2023-02-21 16:34:57",
                    "account": {
                        "address": "5CDGXH8Q9DzD3TnATTG6qm6f4yR1kbECBGUmh2XbEBQ8Jfa5",
                        "role": "OWNER"
                    },
                    "event": "STORE"
                },
        
                "1": {
                    "date": "2023-02-21 16:54:00",
                    "account": {
                        "address": "5CDGXH8Q9DzD3TnATTG6qm6f4yR1kbECBGUmh2XbEBQ8Jfa5",
                        "role": "DELEGATEE"
                    },
                    "event": "VIEW"
                }
            },
        
            "capsule": {
                "0": {
                    "date": "2024-03-22 17:35:58",
                    "account": {
                        "address": "5CDGXH8Q9DzD3TnATTG6qm6f4yR1kbECBGUmh2XbEBQ8Jfa5",
                        "role": "OWNER"
                    },
                    "event": "STORE"
                },
        
                "1": {
                    "date": "2024-03-22 17:45:10",
                    "account": {
                        "address": "5CDGXH8Q9DzD3TnATTG6qm6f4yR1kbECBGUmh2XbEBQ8Jfa5",
                        "role": "DELEGATEE"
                    },
                    "event": "VIEW"
                }
            }
        }"#;

		let mut log_file: LogFile =
			serde_json::from_str(&store_body).expect("error deserailizing json body");

		let nft_second_account_role = if let Some(event) = log_file.secret_nft.get(&1) {
			event.account.role
		} else {
			RequesterType::NONE
		};

		assert_eq!(nft_second_account_role, RequesterType::DELEGATEE);

		let new_log_body = r#"
        {
            "date": "2023-03-23 16:50:25",
            "account": {
                "address": "5TQAxH8Q9DzD3TnATTG6qm6f4yR1kbECBGUmh2XbEBQ8Jfa7",
                "role": "RENTEE"
            },
            "event": "VIEW"
        }"#;

		let new_log: LogStruct =
			serde_json::from_str(&new_log_body).expect("error deserailizing json body");
		log_file.insert_new_capsule_log(new_log);

		let correct_log = r#"
        {
            "secret_nft": {
                "0": {
                    "date": "2023-02-21 16:34:57",
                    "account": {
                        "address": "5CDGXH8Q9DzD3TnATTG6qm6f4yR1kbECBGUmh2XbEBQ8Jfa5",
                        "role": "OWNER"
                    },
                    "event": "STORE"
                },
        
                "1": {
                    "date": "2023-02-21 16:54:00",
                    "account": {
                        "address": "5CDGXH8Q9DzD3TnATTG6qm6f4yR1kbECBGUmh2XbEBQ8Jfa5",
                        "role": "DELEGATEE"
                    },
                    "event": "VIEW"
                }
            },
        
            "capsule": {
                "0":  {
                    "date": "2024-03-22 17:35:58",
                    "account":  {
                        "address": "5CDGXH8Q9DzD3TnATTG6qm6f4yR1kbECBGUmh2XbEBQ8Jfa5",
                        "role": "OWNER"
                    },
                    "event": "STORE"
                },

                "1": {
                    "date": "2024-03-22 17:45:10",
                    "account":  {
                        "address": "5CDGXH8Q9DzD3TnATTG6qm6f4yR1kbECBGUmh2XbEBQ8Jfa5",
                        "role": "DELEGATEE"
                    },
                    "event": "VIEW"
                },

                "2": {
                    "date": "2023-03-23 16:50:25",
                    "account": {
                        "address": "5TQAxH8Q9DzD3TnATTG6qm6f4yR1kbECBGUmh2XbEBQ8Jfa7",
                        "role": "RENTEE"
                    },
                    "event": "VIEW"
                }
            }
        }
        "#;

		assert_eq!(
			log_file,
			serde_json::from_str(&correct_log).expect("error deserailizing json body")
		);
	}

	#[tokio::test]
	async fn file_log_test() {
		// Simulating the Store keyshare process
		let mut file = std::fs::File::create("test.log").unwrap(); // TODO: manage unwrap()
		let owner = "5CDGXH8Q9DzD3TnATTG6qm6f4yR1kbECBGUmh2XbEBQ8Jfa5".to_string();

		let mut log_file_struct = LogFile::new();
		let log_account = LogAccount::new(owner, RequesterType::OWNER);
		let new_log = LogStruct::new(log_account, LogType::STORE);
		log_file_struct.insert_new_nft_log(new_log);

		let log_buf = serde_json::to_vec(&log_file_struct).unwrap(); // TODO: manage unwrap()
		file.write_all(&log_buf).unwrap(); // TODO: manage unwrap()
		std::mem::drop(file);

		// Simulating Retrive keyshare
		let requester_address = "5TQAxH8Q9DzD3TnATTG6qm6f4yR1kbECBGUmh2XbEBQ8Jfa7".to_string();
		update_log_file_view(
			"test.log".to_string(),
			requester_address,
			RequesterType::DELEGATEE,
			LogType::VIEW,
			"secret-nft",
		);

		// Simulating convert to capsule
		let requester_address = "5CDGXH8Q9DzD3TnATTG6qm6f4yR1kbECBGUmh2XbEBQ8Jfa5".to_string();
		update_log_file_view(
			"test.log".to_string(),
			requester_address,
			RequesterType::OWNER,
			LogType::STORE,
			"capsule",
		);

		// Simulate viewing the log
		let mut file = std::fs::File::open("test.log").unwrap(); // TODO: manage unwrap()
		let mut content = String::new();
		file.read_to_string(&mut content).unwrap();

		println!("{}", content);
	}
}