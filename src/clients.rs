use chrono::Utc;
use futures::stream::TryStreamExt;

use mongodb::{
    bson::{doc, Bson, Document},
    options::{ClientOptions, FindOptions},
    Client,
};

use crate::types::Kline;

pub struct MongoClient {
    pub client: Client,
}

impl MongoClient {
    pub async fn new(connection_string: &str) -> MongoClient {
        let client_options = ClientOptions::parse(connection_string).await.unwrap();
        let client = Client::with_options(client_options).unwrap();
        MongoClient { client }
    }
    pub async fn get_klines(
        &self,
        database_name: &str,
        collection_name: &str,
        from_ts: i64,
        to_ts: Option<i64>,
    ) -> Vec<Kline> {
        let mut klines = Vec::new();
        let database = self.client.database(database_name);
        let collection = database.collection::<Document>(collection_name);
        let to_ts = if let Some(ts) = to_ts {
            ts
        } else {
            Utc::now().timestamp_millis()
        };
        let filter = doc! { "close_time": {"$gte": from_ts, "$lte": to_ts} };
        let find_options = FindOptions::builder()
            .sort(doc! { "close_time": 1 })
            .build();
        let mut cursor = collection.find(filter, find_options).await.unwrap();
        // Iterate over the results of the cursor.
        while let Some(doc) = cursor.try_next().await.unwrap() {
            let kline = Kline {
                open_time: doc.get("open_time").unwrap().as_i64().unwrap(),
                close_time: doc.get("close_time").unwrap().as_i64().unwrap(),
                open: parse_f64(doc.get("open")),
                high: parse_f64(doc.get("high")),
                low: parse_f64(doc.get("low")),
                close: parse_f64(doc.get("close")),
            };
            klines.push(kline);
        }
        klines
    }
}

pub fn parse_f64(bson: Option<&Bson>) -> f64 {
    bson.unwrap().as_str().unwrap().parse::<f64>().unwrap()
}
