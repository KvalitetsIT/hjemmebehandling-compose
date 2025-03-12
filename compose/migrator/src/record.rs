use chrono::NaiveDateTime;
use sqlx::{mysql::MySqlRow, postgres::PgRow, Row};

#[derive(sqlx::FromRow, Debug)]
pub struct Record {
    #[allow(dead_code)]
    pub pid: i64,
    pub res_id: i64,
    pub res_ver: i64,
    #[allow(dead_code)]
    pub res_type: String,
    pub res_published: NaiveDateTime,
    pub res_updated: NaiveDateTime,
}

impl From<&MySqlRow> for Record {
    fn from(row: &MySqlRow) -> Self {
        let pid: i64 = row
            .try_get("PID")
            .expect("Could not aquire 'PID' from the database response");

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
            .expect("Could not aquire 'RES_UPDATED' from the database response");

        Record {
            pid,
            res_id,
            res_ver,
            res_type,
            res_published,
            res_updated,
        }
    }
}

impl From<&PgRow> for Record {
    fn from(row: &PgRow) -> Self {
        let pid: i64 = row
            .try_get("pid")
            .expect("Could not aquire 'PID' from the database response");

        let res_id: i64 = row
            .try_get("res_id")
            .expect("Could not aquire 'RES_ID' from the database response");

        let res_ver: i64 = row
            .try_get("res_ver")
            .expect("Could not aquire 'RES_VER' from the database response");

        let res_type: String = row
            .try_get("res_type")
            .expect("Could not aquire 'RES_TYPE' from the database response");

        let res_published: NaiveDateTime = row
            .try_get("res_published")
            .expect("Could not aquire 'RES_PUBLISHED' from the database response");

        let res_updated: NaiveDateTime = row
            .try_get("res_updated")
            .expect("Could not aquire 'RES_UPDATED' from the database response");

        Record {
            pid,
            res_id,
            res_ver,
            res_type,
            res_published,
            res_updated,
        }
    }
}
