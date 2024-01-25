use crate::{error::Error::*, handler::BookRequest, Book, Result};
use chrono::prelude::*;
use futures::StreamExt;
use mongodb:: {
    bson::{doc, oid::ObjectId, Bson, Document, DateTime as BsonDateTime}, 
    options::{ClientOptions, FindOptions, UpdateOptions}, 
    Client, Collection,
};

const DB_NAME: &str = "booksDB";
const COLLECTION_NAME: &str = "books";

const ID: &str = "_id";
const NAME: &str = "name";
const AUTHOR: &str = "author";
const NUM_PAGES: &str = "num_pages";
const ADDED_AT: &str = "added_at";
const TAGS: &str = "tags";

#[derive(Clone, Debug)]
pub struct DB {
    pub client: Client,
    pub collection: Collection<Document>,
}

impl DB {
    // initialize the connection
    pub async fn init() -> Result<Self> {
        let client_options = ClientOptions::parse("mongodb://127.0.0.1:27017").await?;
        let client = Client::with_options(client_options)?;
        let collection = client.database(DB_NAME).collection(COLLECTION_NAME);

        Ok(Self { client, collection })
    }

    // GET /book
    pub async fn fetch_books(&self) -> Result<Vec<Book>> {
        let find_options = FindOptions::default();
        let mut cursor = self.collection
                            .find(None, find_options)
                            .await
                            .map_err(MongoQueryError)?;
        let mut result: Vec<Book> = Vec::new();
        while let Some(doc) = cursor.next().await {
            result.push(self.doc_to_book(&doc?)?);
        }

        Ok(result)
    }

    // POST /book
    pub async fn create_book (&self, entry: &BookRequest) -> Result<()> {
        let doc = doc! {
            NAME: entry.name.clone(),
            AUTHOR: entry.author.clone(),
            NUM_PAGES: entry.num_pages as i32,
            ADDED_AT: BsonDateTime::now(),
            TAGS: entry.tags.clone(),
        };

        self.client
            .database(DB_NAME)
            .collection(COLLECTION_NAME)
            .insert_one(doc, None)
            .await
            .map_err(MongoQueryError)?;

        Ok(())
    }
    
    // UPDATE 
    pub async fn edit_book (&self, id: &str, entry: &BookRequest) -> Result<()> {
        let oid = ObjectId::parse_str(id).map_err(|_| InvalidIDError(id.to_owned()))?;
        let query = doc! { "_id": oid, };
        let update_doc = doc! {
            "$set": {
                NAME: entry.name.clone(),
                AUTHOR: entry.author.clone(),
                NUM_PAGES: entry.num_pages as i32,
                ADDED_AT: BsonDateTime::now(),
                TAGS: entry.tags.clone(),
            }
        };
        let options = UpdateOptions::builder().upsert(false).build();
        self.client
            .database(DB_NAME)
            .collection::<Document>(COLLECTION_NAME)
            .update_one(query, update_doc, Some(options))
            .await
            .map_err(MongoQueryError)?;

        Ok(())
    }


    // DELETE
    pub async fn delete_book (&self, id: &str) -> Result<()> {
        let oid = ObjectId::parse_str(id).map_err(|_| InvalidIDError(id.to_owned()))?;
        let filter = doc! { "_id": oid, };
        self.client
            .database(DB_NAME)
            .collection::<Document>(COLLECTION_NAME)
            .delete_one(filter, None)
            .await
            .map_err(MongoQueryError)?;

        Ok(())
    }


    // doc to book
    fn doc_to_book (&self, doc: &Document) -> Result<Book> {
        let id = doc.get_object_id(ID)?.to_hex();
        let name = doc.get_str(NAME)?.to_owned();
        let author = doc.get_str(AUTHOR)?.to_owned();
        let num_pages = doc.get_i32(NUM_PAGES)? as usize;
        // let added_at = doc.get_datetime(ADDED_AT)?.clone();

        // let added_at = Utc.from_utc_datetime(&doc.get_datetime(ADDED_AT)?.clone());
        let added_at = Utc.timestamp_millis(doc.get_datetime(ADDED_AT)?.timestamp_millis());


        let tags = match doc.get_array(TAGS) {
            Ok(tags) => tags
                .iter()
                .filter_map(|entry| match entry {
                    Bson::String(v) => Some(v.to_owned()),
                    _ => None,
                })
                .collect(),
            Err(_) => Vec::new()
        };

        Ok(Book {
            id,
            name,
            author,
            num_pages,
            added_at,
            tags,
        })
    }

}