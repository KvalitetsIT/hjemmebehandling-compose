use sqlx::{
    any::AnyRow,
    mysql::MySqlPoolOptions,
    pool::PoolOptions,
    postgres::{PgConnectOptions, PgPoolOptions, PgRow},
    Acquire, Any, AnyConnection, Connection, Database, Decode, PgConnection, Row,
};
use std::{
    env::{self, var},
    fmt::Debug,
    fs::File,
    io::{Read, Write},
    ops::Deref,
    str::FromStr,
    thread::sleep,
    time::Duration,
};
use tokio::runtime::Runtime;

use chrono::NaiveDateTime;
use log::{debug, error, info, warn};
use reqwest::{Method, Url};
use serde_json::Value;
use sqlx::{mysql::MySqlRow, MySql, Pool, Postgres};

use crate::{bundle::Bundle, client::Client, record::Record};
pub struct Migrator {
    successor: Url,
    origin: Option<Url>,
    client: Client,
}
impl Migrator {
    pub fn new(origin: Option<Url>, successor: Url) -> Self {
        Self {
            origin,
            successor,
            client: Client::new(),
        }
    }

    pub fn start(&self, resources: Vec<String>) {
        let data = self.aquire(resources);
        self.migrate(&data);

        Self::update_timestamps();

        Self::wait();

        self.validate_data(data);
    }

    fn aquire(&self, resources: Vec<String>) -> Vec<(String, Bundle)> {
        info!("Aquiring data...");
        resources
            .into_iter()
            .fold(Vec::<(String, Bundle)>::new(), |mut a, resource_type| {
                let json = self.get_history(&resource_type);

                let bundle = Bundle::from((resource_type.clone(), &json));
                a.push((resource_type, bundle));
                a
            })
    }

    fn get_history(&self, resource_type: &String) -> Value {
        let json = match &self.origin {
            // Fetch data from remote api
            Some(origin) => self
                .client
                .get(&Self::get_url(&origin, resource_type.as_str()))
                .unwrap(),

            // Fetch data from local storage
            None => {
                warn!("Environment variable 'successor' not found falling back to local data");

                let path = format!("./data/{}.json", resource_type);
                File::open(&path)
                    .and_then(|mut file| {
                        debug!("Read: {:?}", file);
                        let mut buf = String::new();
                        file.read_to_string(&mut buf).map(|_| buf)
                    })
                    .and_then(|s| serde_json::from_str::<Value>(s.as_str()).map_err(|e| e.into()))
                    .expect(format!("Expected: {}", path).as_str())
            }
        };
        json
    }

    fn get_url(origin: &Url, resource_type: &str) -> String {
        let path = format!("/fhir/{}/_history", resource_type);
        let origin = origin.join(path.as_str()).unwrap();
        origin.to_string()
    }

    #[allow(dead_code)]
    fn create_reponse_file(name: &str, value: &Value) {
        const ERROR_MESSAGE: &str = "Could not create file";
        let path = format!("./resources/{}.json", name);
        File::create(path)
            .unwrap()
            .write_all(serde_json::to_string_pretty(value).unwrap().as_bytes())
            .expect(ERROR_MESSAGE);
    }

    fn wait() {
        let secs: u64 = env::var("WAIT")
            .map(|secs| {
                secs.parse::<u64>()
                    .expect("Could not parse 'WAIT' as a u64")
            })
            .unwrap_or(20);
        let duration = Duration::from_secs(secs);

        info!(
            "Waiting {} seconds before verifying migration...",
            duration.as_secs()
        );
        sleep(duration);
    }

    fn validate_data(&self, resources: Vec<(String, Bundle)>) {
        info!("Validating migration...");
        let t = resources.iter().all(|(resource_type, _)| {
            let a: Value = self
                .client
                .get(Self::get_url(
                    &self.successor,
                    format!("{}", resource_type).as_str(),
                ))
                .unwrap();

            let b: Value = self.get_history(resource_type);

            let total: (u64, u64) = (
                a.get("total")
                    .unwrap()
                    .as_u64()
                    .expect("Could not parse value of total as u64"),
                b.get("total")
                    .unwrap()
                    .as_u64()
                    .expect("Could not parse value of total as u64"),
            );

            match total.0 == total.1 {
                true => true,
                false => {
                    error!("Expected {} but was {}", total.0, total.1);
                    false
                }
            }
        });

        match t {
            true => info!("Success! 🥳"),
            false => error!("Failed! 👎"),
        }
    }

    fn migrate(&self, data: &Vec<(String, Bundle)>) {
        info!(
            "Migrating data from: {} to {}",
            self.origin
                .as_ref()
                .map(|u| u.to_string())
                .unwrap_or("./data/<resource>.json".to_string()),
            self.successor
        );

        data.iter().for_each(|(_, bundle)| {
            bundle.get_entries().for_each(|entry| {
                let versions: Vec<(&u64, &Value)> = entry.get_versions().collect();

                versions.iter().for_each(|(_version, value)| {
                    let request = value
                        .get("request")
                        .expect("Could not extract request from entry");

                    let method = request
                        .get("method")
                        .and_then(|m| m.as_str())
                        .and_then(|method| Method::from_str(method).ok())
                        .expect("Could not extract method from request");

                    let full_url: Url = value
                        .get("fullUrl")
                        .and_then(|u| u.as_str())
                        .and_then(|u| Url::parse(u).ok())
                        .expect("Could not extract field 'fullUrl'");

                    let destination = self.successor.join(full_url.path()).unwrap();

                    let response = match method {
                        Method::PUT | Method::POST => {
                            let resource =
                                value.get("resource").expect("Could not extract resource");

                            self.client.put(destination, resource)
                        }
                        Method::DELETE => self.client.delete(destination),
                        _ => Err(String::from("Expected method {}, method").into()),
                    };

                    match response {
                        Ok(_) => {}
                        Err(error) => error!("{:#?}", error),
                    }
                });
            });
        });
    }

    async fn fetch_records(pool: DB) -> Vec<Record> {
        let statement = "SELECT HFJ_RES_VER.PID, HFJ_RES_VER.RES_ID, HFJ_RES_VER.RES_VER, HFJ_RES_VER.RES_TYPE, HFJ_RES_VER.RES_PUBLISHED, HFJ_RES_VER.RES_UPDATED FROM HFJ_RESOURCE  INNER JOIN HFJ_RES_VER ON HFJ_RESOURCE.RES_ID = HFJ_RES_VER.RES_ID" ;
        match pool {
            DB::Postgres(pg) => {
                let query = sqlx::query(statement);
                let response = query.fetch_all(&pg).await.expect(
                    format!(
                        "Could not fetch rows - failed to execute statement '{}'",
                        statement
                    )
                    .as_str(),
                );

                info!("Postgres records:");
                response
                    .iter()
                    .map(|row: &PgRow| {
                        let row = Record::from(row);
                        info!("{:?}", &row);
                        row
                    })
                    .collect::<Vec<Record>>()
            }
            DB::MariaDB(maria) => {
                let query = sqlx::query(statement);
                let response = query.fetch_all(&maria).await.expect(
                    format!(
                        "Could not fetch rows - failed to execute statement '{}'",
                        statement
                    )
                    .as_str(),
                );

                info!("MariaDB records:");
                response
                    .iter()
                    .map(|row: &MySqlRow| {
                        let row = Record::from(row);
                        info!("{:?}", &row);
                        row
                    })
                    .collect::<Vec<Record>>()
            }
        }
    }

    async fn update_records(pool: Pool<Postgres>, data: Vec<Record>) {
        let statement =
            "UPDATE hfj_res_ver SET res_published = $1, res_updated = $2 WHERE res_id = $3 AND res_ver= $4";

        for resource in data {
            let query = sqlx::query(statement)
                .bind(resource.res_published)
                .bind(resource.res_updated)
                .bind(resource.res_id)
                .bind(resource.res_ver);

            let s = format!( "UPDATE hfj_res_ver SET res_published = {}, res_updated = {} WHERE res_id = {} AND res_ver= {}", resource.res_published,  resource.res_updated, resource.res_id, resource.res_ver );

            let res = query
                .execute(&pool)
                .await
                .expect(format!("Could not execute statement '{}'", s).as_str());
            if res.rows_affected() != 1 {
                warn!(
                    "During execution of '{}'\nRows affected: {}\nExpected: 1\nNote: This might be ok, compare the response by GET: /fhir/{}/_history",
                    s,
                                        res.rows_affected(),
                    resource.res_type,
                )
            }
        }
    }
    fn update_timestamps() {
        info!("Updating timestamps");
        let rt = Runtime::new().unwrap();

        let mariadb_url = get_database_url("MARIADB", "mysql");
        let postgres_url = get_database_url("POSTGRES", "postgres");

        rt.block_on(async {
            let mariadb: Pool<MySql> = MySqlPoolOptions::new()
                .max_connections(1)
                .connect(&mariadb_url.to_string())
                .await
                .expect(
                    format!(
                        "Could not aquire connection for: {}",
                        &mariadb_url.to_string()
                    )
                    .as_str(),
                );

            info!("Connection to '{}' aquired.", mariadb_url.to_string());

            let postgres: Pool<Postgres> = PgPoolOptions::new()
                .max_connections(1)
                .connect(&postgres_url.to_string())
                .await
                .expect(
                    format!(
                        "Could not aquire connection for: {}",
                        &postgres_url.to_string()
                    )
                    .as_str(),
                );

            info!("Connection to '{}' aquired.", postgres_url.to_string());

            let records = Self::fetch_records(DB::MariaDB(mariadb)).await;

            // Added just to see logs
            Self::fetch_records(DB::Postgres(postgres.clone())).await;

            Migrator::update_records(postgres, records).await;
        });
    }
}

enum DB {
    Postgres(Pool<Postgres>),
    MariaDB(Pool<MySql>),
}

fn get_database_url(prefix: &str, protocol: &str) -> Url {
    let (user, password, host, port, db) = (
        var(format!("{}_USER", prefix)).unwrap(),
        var(format!("{}_PASSWORD", prefix)).unwrap(),
        var(format!("{}_HOST", prefix)).unwrap(),
        var(format!("{}_PORT", prefix)).unwrap(),
        var(format!("{}_DB", prefix)).unwrap(),
    );

    let url = format!(
        "{}://{}:{}@{}:{}/{}",
        protocol, user, password, host, port, db
    );

    Url::parse(url.as_str()).unwrap()
}
