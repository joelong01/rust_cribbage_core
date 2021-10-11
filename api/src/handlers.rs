use actix_web::{web::Path, HttpResponse, Responder};
use azure_sdk_cosmos::{prelude::*, };
use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
struct CosmosConnectionInfo {
    pub key: String,
    pub connection_string: String,
    pub database_name: String,
    pub collection_name: String,
    pub account_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CribbageAi {
    pub name: String,
    pub by: String,
    pub description: String,
    pub uri: String,
}

#[allow(unused_macros)]
macro_rules! get_connection_info {
    () => {{
        // let account_name = std::env::var("RUST_CRIBBAGE_COSMOS_ACCOUNT_NAME")
        //     .expect("RUST_CRIBBAGE_COSMOS_ACCOUNT_NAME must be set");
        let account_key: String = std::env::var("RUST_CRIBBAGE_COSMOS_KEY")
            .expect("RUST_CRIBBAGE_COSMOS_KEY must be set");
        let account_connection: String = std::env::var("RUST_CRIBBAGE_COSMOS_CONNECTION_STRING")
            .expect("RUST_CRIBBAGE_COSMOS_CONNECTION_STRING must be set");

        let cci = CosmosConnectionInfo {
            key: account_key,
            connection_string: account_connection,
            database_name: "CribbageDb".to_owned(),
            collection_name: "registeredAis".to_owned(),
            account_name: "rust-cribbage-db".to_owned(),
        };

        cci
    }};
}
#[allow(unused_macros)]
macro_rules! cosmos_connection {
    ($connection_info: expr) => {{
        let authorization_token = match AuthorizationToken::new_master(&$connection_info.key) {
            Ok(authorization_token) => authorization_token,
            Err(e) => {
                return HttpResponse::Ok().body(e.to_string());
            }
        };

        let client = match ClientBuilder::new(&$connection_info.account_name, authorization_token) {
            Ok(client) => client,
            Err(e) => {
                return HttpResponse::Ok().body(e.to_string());
            }
        };

        client
    }};
}

pub async fn get_registered_ais() -> impl Responder {
    // let mut ai = CribbageAi {
    //     name: "Hard".to_owned(),
    //     by: "joe".to_owned(),
    //     description: "used drop table to optimize crib".to_owned(),
    //     uri: "localhost:8081".to_owned(),
    // };

    // let mut lst: Vec<CribbageAi> = Vec::<CribbageAi>::new();
    // lst.push(ai);
    // ai = CribbageAi {
    //     name: "Easy".to_owned(),
    //     by: "joe".to_owned(),
    //     description: "ignores crib cards and optimizes for hand only".to_owned(),
    //     uri: "localhost:8082".to_owned(),
    // };

    // lst.push(ai);

    // ai = CribbageAi {
    //     name: "Random".to_owned(),
    //     by: "joe".to_owned(),
    //     description: "picks a valid but random card to play".to_owned(),
    //     uri: "localhost:8083".to_owned(),
    // };
    // lst.push(ai);
    // let json = serde_json::to_string(&lst).unwrap();

    let connection_info = get_connection_info!();
    let client = cosmos_connection!(connection_info);
    let client = client.with_database_client(&connection_info.database_name);
    let client = client.with_collection_client(connection_info.collection_name);

    let response = match client
        .list_documents()
        .with_max_item_count(100)
        .execute::<CribbageAi>()
        .await
    {
        Ok(response) => response,
        Err(e) => {
            return HttpResponse::Ok().body(e.to_string());
        }
    };

    let json = serde_json::to_string(&response.documents).unwrap();

    HttpResponse::Ok().body(json)
}

pub async fn add_ai(info: Path<(String, String, String, String)>) -> impl Responder {
    let info = info.into_inner();
    let ai = CribbageAi {
        name: info.0,
        by: info.1,
        description: info.2,
        uri: info.3,
    };

    let connection_info = get_connection_info!();

    let client = cosmos_connection!(connection_info);

    let client = client.into_database_client(&connection_info.database_name);
    let client = client.into_collection_client(&connection_info.collection_name);

    // let response = client.list_documents().execute::<CribbageAi>().await;
    // if response.is_err() {
    //     return HttpResponse::NotFound().body(format!(
    //         "No Collection installed for db: {}",
    //         connection_info.database_name
    //     ));
    // }

    // let mut found = false;
    // for docs in response.unwrap().documents {
    //     if docs.id == connection_info.collection_name {
    //         found = true;
    //         break;
    //     }
    // }

    // if found == false {
    //     return HttpResponse::NotFound().body(format!(
    //         "Collection {} not found.  Configure Cosmos!",
    //         connection_info.collection_name
    //     ));
    // }

    // let client = client.with_collection_client(connection_info.collection_name);

    let response = client
        .list_documents()
        .with_max_item_count(100)
        .execute::<CribbageAi>()
        .await;

    //
    //  check for duplicate names
    //
    if response.is_ok() {
        for doc in response.unwrap().documents {
            if doc.document.name == ai.name {
                return HttpResponse::Ok().body(format!("found duplicate name: {}\n.  Remove it then add again if you want to update it.\nDoing nothing.", ai.name));
            };
        }
    }

    let mut partition_keys = PartitionKeys::default();
    match partition_keys.push(&ai.name) {
        Ok(_) => {}
        Err(e) => {
            return HttpResponse::Ok()
                .body(format!("Error creating partition key: {}", e.to_string()));
        }
    }

    // let response = Some(
    //     client
    //         .create_document(Context::new(), &ai, CreateDocumentOptions::new())
    //         .await,
    // );

    match client
        .create_document()
        .with_partition_keys(&partition_keys)
        .execute_with_document(&ai)
        .await
    {
        Ok(_) => {
            return HttpResponse::Ok().body(serde_json::to_string(&ai).unwrap());
        }
        Err(e) => {
            return HttpResponse::Ok().body(format!("Error adding document: {}", e.to_string()));
        }
    };
}

pub async fn test_cosmos() -> impl Responder {
    let connection_info = get_connection_info!();
    let client = cosmos_connection!(connection_info);
    let client = client.into_database_client(connection_info.database_name);
    let client = client.into_collection_client(connection_info.collection_name);
    let ai = CribbageAi {
        name: "Hard".to_owned(),
        by: "joe".to_owned(),
        description: "used drop table to optimize crib".to_owned(),
        uri: "localhost:8081".to_owned(),
    };
    let mut partition_keys = PartitionKeys::default();
    match partition_keys.push(&ai.name) {
        Ok(_) => {}
        Err(e) => {
            return HttpResponse::Ok()
                .body(format!("Error creating partition key: {}", e.to_string()));
        }
    }
    let response = client
        .create_document()
        .with_partition_keys(&partition_keys)
        .execute_with_document(&ai)
        .await;

    // let database_client = client.with_database_client(&connection_info.database_name);
    // let response = database_client.list_collections().execute().await;

    match response {
        Ok(_) => HttpResponse::Ok().body(serde_json::to_string(&ai).unwrap()),
        Err(e) => HttpResponse::Ok().body(format!("Error adding document: {}", e)),
    }
}
