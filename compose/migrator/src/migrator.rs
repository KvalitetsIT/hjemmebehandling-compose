use std::{fs::File, io::Write, thread::sleep, time::Duration};

use log::{error, info, warn};
use reqwest::Url;
use serde_json::Value;

use crate::client::Client;
pub struct Migrator {
    origin: Url,
    successor: Url,
    client: Client,
    validator: Validator,
}
impl Migrator {
    pub fn new(from: Url, to: Url) -> Self {
        Self {
            origin: from,
            successor: to,
            client: Client::new(),
            validator: Validator::new(),
        }
    }

    pub fn migrate(&self, resources: Vec<String>) {
        info!("Migrating data from: {} to {}", self.origin, self.successor);

        resources.iter().for_each(|resource_type| {
            match self.client.get(&Self::get_url(&self.origin, resource_type)) {
                Ok(response) => {
                    let value = Self::to_value(response);

                     match &value["entry"].as_array() {
                        Some(entries) => {
                            for entry in entries.iter() {
                                let full_url = Url::parse(entry["fullUrl"].as_str().unwrap())
                                    .expect("The expected field 'fullUrl' was not found");

                                let resource = &entry["resource"];
                                let destination = self.successor.join(full_url.path()).unwrap();

                                if let Err(error) = self.client.put(destination, &resource){
                                     error!("{}", error);
                                 }                               


                            }
                        }
                        None => warn!(
                            "The expected field 'entry' was not found for resource '{}': reasource may not have any entries yet",
                            resource_type
                        )
                    }
                }
                Err(err) => {
                    error!(
                        "Could not fetch resource '{}' due to: {}",
                        resource_type, err
                    );
                }
            }
        });

        Self::wait();

        self.verify_resources(resources);
    }

    fn get_url(origin: &Url, resource_type: &str) -> String {
        let path = format!("/fhir/{}/_history", resource_type);
        let origin = origin.join(path.as_str()).unwrap();
        origin.to_string()
    }

    fn create_reponse_file(name: &str, value: &Value) {
        const ERROR_MESSAGE: &str = "Could not create file";
        let path = format!("./resources/{}.json", name);
        File::create(path)
            .unwrap()
            .write_all(serde_json::to_string_pretty(value).unwrap().as_bytes())
            .expect(ERROR_MESSAGE);
    }

    fn to_value(response: reqwest::blocking::Response) -> Value {
        response.json::<Value>().unwrap()
    }

    fn wait() {
        let duration = Duration::from_secs(20);
        info!(
            "Waiting {} seconds before verifying migration...",
            duration.as_secs()
        );
        sleep(duration);
    }

    fn verify_resources(&self, resources: Vec<String>) {
        resources.iter().for_each(|resource_type| {
            let a = self
                .client
                .get(Self::get_url(&self.origin, resource_type))
                .map(|response| Self::to_value(response));

            let b = self
                .client
                .get(Self::get_url(&self.successor, resource_type))
                .map(|response| Self::to_value(response));

            match (a, b) {
                (Ok(a), Ok(b)) => {
                    // Create files for manual comparison
                    Self::create_reponse_file(format!("{}-origin", resource_type).as_str(), &a);
                    Self::create_reponse_file(format!("{}-successor", resource_type).as_str(), &b);

                    self.validator.verify_resource(a, b, &resource_type);
                }
                _ => {
                    error!("Validating resource '{resource_type}' was not possible");
                }
            }
        });
    }
}

struct Validator;

impl Validator {
    fn new() -> Self {
        Self {}
    }
    fn verify_resource(&self, a: Value, b: Value, resource_type: &str) {
        // Verify that the resources has been migrated correctly
        let a = a
            .get("total")
            .expect("field 'total' was not found")
            .as_u64()
            .expect("'field' total could not be parsed into a number");

        let b = b
            .get("total")
            .expect("field 'total' was not found")
            .as_u64()
            .expect("'field' total could not be parsed into a number");

        if !&a.eq(&b) {
            error!("Expected the same number ({b}/{a}) of entries for resource: {resource_type}");
        }
    }
}
