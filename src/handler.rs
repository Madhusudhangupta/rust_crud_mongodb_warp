use crate::{db::DB, WebResult};
use serde::{Deserialize, Serialize};
use warp::{http::StatusCode, reject, reply::json, Reply};

#[derive(Serialize, Deserialize, Debug)]
pub struct BookRequest {
    pub name: String,
    pub author: String,
    pub num_pages: usize,
    pub tags: Vec<String>,
}

// books_list_handler
pub async fn books_list_handler(db: DB) -> WebResult<impl Reply> {
    let books = db.fetch_books().await.map_err(|e| reject::custom(e))?;
    Ok(json(&books))
}

// create_book_handler
pub async fn create_book_handler(body: BookRequest, db: DB) -> WebResult<impl Reply> {
    db.create_book(&body).await.map_err(|e| reject::custom(e))?;
    Ok(StatusCode::CREATED)
}

// edit_book_handler
pub async fn edit_book_handler(id: String, body: BookRequest, db: DB) -> WebResult<impl Reply> {
    db.edit_book(&id, &body)
        .await
        .map_err(|e| reject::custom(e))?;
    Ok(StatusCode::OK)
}

// delete_book_handler
pub async fn delete_book_handler(id: String, db: DB) -> WebResult<impl Reply> {
    db.delete_book(&id).await.map_err(|e| reject::custom(e))?;
    Ok(StatusCode::OK)
}
