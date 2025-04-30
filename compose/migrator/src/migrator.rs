use sqlx::{mysql::MySqlPoolOptions, postgres::PgPoolOptions};
use std::{
    collections::BTreeMap,
    env::{self, var},
    fs::File,
    io::{Read, Write},
    str::FromStr,
    thread::sleep,
    time::Duration,
};
use tokio::runtime::Runtime;

use crate::{bundle::Bundle, client::Client, record::Record};
use log::{debug, error, info, warn};
use reqwest::{Method, Url};
use serde_json::{json, Value};
use sqlx::{MySql, Pool, Postgres};
use urlencoding::encode;
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
                warn!("Environment variable 'origin' not found falling back to local data");

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

            let origin = self.origin.clone();
            let origin: (String, Value) = (origin .map(|x| Self::get_url(&x, resource_type.as_str())) .unwrap_or(String::from("/local/")), self.get_history(resource_type));

            let successor =  Self::get_url(&self.successor, format!("{}", resource_type).as_str());
            let successor: (String, Value) = (  successor.clone(), self.client.get(&successor).unwrap());

            let origin_total: u64 = origin.1.get("total")
                    .unwrap()
                    .as_u64()
                    .expect("Could not parse value of total as u64");

            let successor_total: u64 = successor.1.get("total")
                    .unwrap()
                    .as_u64()
                    .expect("Could not parse value of total as u64");


            let has_matching_amount_of_entries =  origin_total == successor_total;

            match has_matching_amount_of_entries {
                true => true,
                false => {
                    error!(
                        "Expected '{}' but was '{}':\n{} returned '{}' historical records\n while\n{} returned '{}' historical records.",
                        origin_total,
                        successor_total,
                        origin.0,
                        origin_total,
                        successor.0,
                        successor_total,
                    );
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

    async fn fetch_records(pool: DB) -> BTreeMap<String, BTreeMap<i64, Record>> {
        match pool {
            DB::Postgres(pg) => {
                let statement = "SELECT hfj_res_ver.res_id, hfj_res_ver.res_ver, hfj_res_ver.res_type, hfj_resource.fhir_id, hfj_res_ver.res_deleted_at, hfj_res_ver.res_text_vc, hfj_res_ver.res_published, hfj_res_ver.res_updated FROM hfj_res_ver JOIN hfj_resource ON hfj_res_ver.res_id = hfj_resource.res_id;";
                let query = sqlx::query(statement);
                let response = query.fetch_all(&pg).await.expect(
                    format!(
                        "Could not fetch rows - failed to execute statement '{}'",
                        statement
                    )
                    .as_str(),
                );

                let mut result: BTreeMap<String, BTreeMap<i64, Record>> = BTreeMap::new();

                for row in response {
                    let row = Record::from(&row);
                    let id = row.fhir_id.clone().unwrap_or(row.res_id.to_string());

                    let version = row.res_ver;

                    if let Some(versions) = result.get_mut(&id) {
                        versions.insert(version, row);
                    } else {
                        let mut versions = BTreeMap::new();
                        versions.insert(version, row);
                        result.insert(id, versions);
                    }
                }

                info!("Postgres records: {}", result.len());

                result
            }
            DB::MariaDB(maria) => {
                let statement = "SELECT HFJ_RES_VER.RES_ID, HFJ_RES_VER.RES_VER, HFJ_RES_VER.RES_TYPE, HFJ_FORCED_ID.FORCED_ID, HFJ_RES_VER.RES_DELETED_AT, HFJ_RES_VER.RES_PUBLISHED, HFJ_RES_VER.RES_UPDATED, HFJ_RES_VER.RES_TEXT FROM HFJ_RES_VER LEFT JOIN HFJ_FORCED_ID ON HFJ_RES_VER.RES_ID = HFJ_FORCED_ID.RESOURCE_PID ORDER BY HFJ_RES_VER.RES_ID, HFJ_RES_VER.RES_VER;";
                let query = sqlx::query(statement);
                let response = query.fetch_all(&maria).await.expect(
                    format!(
                        "Could not fetch rows - failed to execute statement '{}'",
                        statement
                    )
                    .as_str(),
                );

                let mut result: BTreeMap<String, BTreeMap<i64, Record>> = BTreeMap::new();

                for row in response {
                    let row = Record::from(&row);
                    let id = row.fhir_id.clone().unwrap_or(row.res_id.to_string());

                    let version = row.res_ver;

                    if let Some(versions) = result.get_mut(&id) {
                        versions.insert(version, row);
                    } else {
                        let mut versions = BTreeMap::new();
                        versions.insert(version, row);
                        result.insert(id, versions);
                    }
                }

                info!("MariaDB records: {}", result.len());

                result
            }
        }
    }

    async fn update_records(
        pool: Pool<Postgres>,
        mariadb_records: BTreeMap<String, BTreeMap<i64, Record>>,
    ) {
        for (_, versions) in mariadb_records {
            let mut index: i64 = versions.len() as i64;
            for (_, record) in versions.iter().rev() {
                let fhir_id = record.fhir_id.clone();
                let resource_id = record.res_id;

                let sql: String = match &fhir_id.is_some() {
                    true => String::from(
                        "WITH ranked_rows AS (
                                SELECT pid, ROW_NUMBER() OVER () AS row_num
                                FROM hfj_res_ver
                                LEFT JOIN hfj_resource
                                on hfj_res_ver.res_id = hfj_resource.res_id
                                WHERE hfj_resource.fhir_id = $1
                            )
                            UPDATE hfj_res_ver
                            SET res_published = $2, res_updated = $3, res_deleted_at = $4
                            WHERE pid = (SELECT pid FROM ranked_rows WHERE row_num = $5);",
                    ),
                    false => String::from(
                        "WITH ranked_rows AS (
                                SELECT pid, ROW_NUMBER() OVER () AS row_num
                                FROM hfj_res_ver
                                LEFT JOIN hfj_resource
                                on hfj_res_ver.res_id = hfj_resource.res_id
                                WHERE hfj_resource.fhir_id = $1
                            )
                            UPDATE hfj_res_ver
                            SET res_published = $2, res_updated= $3, res_deleted_at = $4
                            WHERE pid = (SELECT pid FROM ranked_rows WHERE row_num = $5);",
                    ),
                };

                let deleted = record
                    .res_deleted_at
                    .map(|x| x.to_string())
                    .unwrap_or(String::from("null"));

                let substituted_sql = sql
                    .replace(
                        "$1",
                        fhir_id.clone().unwrap_or(resource_id.to_string()).as_str(),
                    )
                    .replace("$2", record.res_published.to_string().as_str())
                    .replace("$3", record.res_updated.to_string().as_str())
                    .replace("$4", deleted.as_str())
                    .replace("$5", index.to_string().as_str());

                let query = sqlx::query(sql.as_str())
                    .bind(fhir_id.unwrap_or(resource_id.to_string()))
                    .bind(record.res_published)
                    .bind(record.res_updated)
                    .bind(record.res_deleted_at)
                    .bind(index);

                debug!("subject:\n{:?}", record);
                match query.execute(&pool).await {
                    Ok(res) => {
                        debug!("{}", substituted_sql);
                        if res.rows_affected() != 1 {
                            warn!(
                                "Expected exactly one row but '{}' was affected.",
                                res.rows_affected()
                            );
                        }
                    }
                    Err(_) => error!(
                        "something went wrong during execution of:\n{}",
                        substituted_sql
                    ),
                };

                index = index - 1;
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
                .expect(format!("Could not aquire connection for: {}", &mariadb_url).as_str());

            info!("Connection to '{}' aquired.", mariadb_url.to_string());

            let postgres: Pool<Postgres> = PgPoolOptions::new()
                .max_connections(1)
                .connect(&postgres_url)
                .await
                .expect(
                    format!(
                        "Could not aquire connection for: {}",
                        &postgres_url.to_string()
                    )
                    .as_str(),
                );

            info!("Connection to '{}' aquired.", postgres_url.to_string());

            let mariadb_records = Self::fetch_records(DB::MariaDB(mariadb)).await;

            let _postgres_records = Self::fetch_records(DB::Postgres(postgres.clone())).await;

            Migrator::update_records(postgres, mariadb_records).await;
        });
    }

    pub fn quit(&self) {
        let host = std::env::var("ISTIO_HOST").unwrap_or(String::from("localhost"));
        let port: u16 = std::env::var("ISTIO_PORT")
            .ok()
            .and_then(|x| x.parse::<u16>().ok())
            .unwrap_or(15020);

        info!(
            "Calling quit on istio sidecar proxy (http://{}:{}/quitquitquit)",
            host, port
        );

        let url = format!("http://{}:{}/quitquitquit", host, port);
        let data = json!({});
        self.client
            .post(url, &data)
            .expect("Something went wrong trying to quit istio sidecar");
    }
}

enum DB {
    Postgres(Pool<Postgres>),
    MariaDB(Pool<MySql>),
}

fn get_database_url(prefix: &str, protocol: &str) -> String {
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

    info!("database url: {}", url);
    let encoded_password = encode(password.as_str());

    let url = format!(
        "{}://{}:{}@{}:{}/{}",
        protocol, user, encoded_password, host, port, db
    );
    info!("encoded database url: {}", url);

    url.to_string()
}
