use std::{
    collections::{HashMap, HashSet}, env, fs::File, io::Write, str::FromStr, thread::sleep, time::Duration
};

use log::{error, info, warn};
use reqwest::{Method, Url};
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
                    let entry =  &value["entry"].as_array();
                      match entry {
                         Some(entries) => {

                            let resource_ids = entries.iter().map(|entry| {
                               entry.get("resource").and_then(|resource| resource.get("id")).and_then(|id| id.as_str())
                           }).filter(|id| id.is_some()).map(|id|{ id.unwrap().to_string()}).collect::<HashSet<String>>();


                            resource_ids.iter().for_each(|resource_id|{
                                let url =  Self::get_url(&self.origin, format!("{}/{}", resource_type, resource_id).as_str());
                                let response = Self::to_value(self.client.get(url).unwrap());
                                    let entries = response.get("entry").unwrap().as_array().unwrap();





                             for entry in entries {

                                let request = entry.get("request").expect("Could not extract request from entry");

                                let method = request.get("method").expect("Could not extract method from request").as_str().and_then(|method| Method::from_str(method).ok()).unwrap();
                                let full_url: Url = entry.get("fullUrl").and_then(|u| u.as_str()).and_then(|u| Url::parse(u).ok()).unwrap();
                                
                                 let destination = self.successor.join(full_url.path()).unwrap();

                                let response = match method {
                                    Method::PUT | Method::POST  => {
                                        let resource = entry.get("resource").expect("Could not extract resource");

                                        self.client.put(destination, resource)
                                    }
                                    Method::DELETE => {self.client.delete(self.successor.join(full_url.path()).unwrap())}
                                    _ => Err(String::from("Expected method {}, method").into())
                                };

                                match response {
                                    Ok(response) => {
                                        if response.status().is_success(){
                                            info!("{}: {}", response.status(), response.text().unwrap_or("".to_string()) )
                                        }else {
                                            error!("{}: {}", response.status(), response.text().unwrap_or("".to_string()))
                                        }                                     },
                                    Err(error) => error!("{:#?}", error),
                                }
                                

                                
                             }

                                    
                                



                            })
                                
                            


                         }
                         None => warn!(
                             "The expected field 'entry' was not found for resource '{}': reasource may not have any entries yet",
                             resource_type
                         )
                     }
                 }
                 Err(err) => {
                     error!(
                         "Could not fetch resource '{}' due to: {:#?}",
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

    #[allow(dead_code)]
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
        let secs: u64 = env::var("wait")
            .map(|secs| {
                secs.parse::<u64>()
                    .expect("Could not parse 'wait' as a u64")
            })
            .unwrap_or(20);
        let duration = Duration::from_secs(secs);

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
        }else {
            info!("Successfully migrated resouce: {resource_type}")
        }
    }
}
