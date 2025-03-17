use chrono::NaiveDateTime;
use sqlx::{mysql::MySqlRow, postgres::PgRow, Row};
#[derive(sqlx::FromRow, Debug)]
pub struct Record {
    pub res_id: i64,
    pub res_ver: i64,
    #[allow(dead_code)]
    pub res_type: String,
    pub res_published: NaiveDateTime,
    pub res_updated: NaiveDateTime,
    pub fhir_id: Option<String>,
}

impl From<&MySqlRow> for Record {
    fn from(row: &MySqlRow) -> Self {
        let res_id: i64 = row
            .try_get("RES_ID")
            .expect("Could not aquire 'RES_ID' from the database response");

        let res_ver: i64 = row
            .try_get("RES_VER")
            .expect("Could not aquire 'RES_VER' from the database response");

        let res_type: String = row
            .try_get("RES_TYPE")
            .expect("Could not aquire 'RES_TYPE' from the database response");

        let res_published: NaiveDateTime = row
            .try_get("RES_PUBLISHED")
            .expect("Could not aquire 'RES_PUBLISHED' from the database response");

        let res_updated: NaiveDateTime = row
            .try_get("RES_UPDATED")
            .expect("Could not aquire 'RES_PUBLISHED' from the database response");

        let fhir_id = row
            .try_get("FORCED_ID")
            .expect("Could not aquire 'FORCED_ID' from the database response");

        Record {
            res_id,
            res_ver,
            res_type,
            res_published,
            res_updated,
            fhir_id,
        }
    }
}

impl From<&PgRow> for Record {
    fn from(row: &PgRow) -> Self {
        let res_id: i64 = row
            .try_get("res_id")
            .expect("Could not aquire 'res_id' from the database response");

        let res_ver: i64 = row
            .try_get("res_ver")
            .expect("Could not aquire 'res_ver' from the database response");

        let res_type: String = row
            .try_get("res_type")
            .expect("Could not aquire 'res_type' from the database response");

        let res_published: NaiveDateTime = row
            .try_get("res_published")
            .expect("Could not aquire 'res_published' from the database response");

        let res_updated: NaiveDateTime = row
            .try_get("res_updated")
            .expect("Could not aquire 'res_updated' from the database response");

        let fhir_id = row
            .try_get("fhir_id")
            .expect("Could not aquire 'fhir_id' from the database response");

        Record {
            res_id,
            res_ver,
            res_type,
            res_published,
            res_updated,
            fhir_id,
        }
    }
}
