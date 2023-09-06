pub mod app;
use cfg_if::cfg_if;
use serde::{Serialize, Deserialize};

cfg_if! {
if #[cfg(feature = "hydrate")] {

  use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen]
    pub fn hydrate() {
      use app::*;
      use leptos::*;

      console_error_panic_hook::set_once();

      leptos::mount_to_body(move |cx| {
          view! { cx, <App/> }
      });
    }
}
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Priority {
  Low,
  Medium,
  High,
  Critical,
}

#[cfg(feature = "ssr")]
pub mod server {
  use actix_web::{web::{Json, Query, Path}, HttpResponse};
  use mongodb::{Collection, bson::oid::ObjectId};
  use mongodb::bson::doc;
  use mongodb::{Client, options::{ClientOptions, ConnectionString}};
  use anyhow::Result;
  use serde::{Serialize, Deserialize};
  use super::Priority;

  #[derive(Debug, Serialize, Deserialize)]
pub struct Task {
  #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
  pub id: Option<ObjectId>,
  pub name: String,
  pub priority: Priority,
}

  pub async fn connect_to_db(uri: &str) -> Result<Client> {
    let config = ClientOptions::parse_connection_string(ConnectionString::parse(uri)?)
      .await?;
  
    let client = Client::with_options(config)?;
  
    Ok(client)
  }

  #[derive(Debug, Clone)]
  pub struct State {
    pub mongo_client: Client,
  }
  
  impl State {
  
    pub async fn new() -> Result<Self> {
      Ok(Self {
        mongo_client: connect_to_db("mongodb://localhost:27017").await?
      })
    }
  }

  #[derive(Debug, Deserialize)]
  struct NewTask {
    name: String,
    priority: Priority,
  }

  #[actix_web::post("task")]
  async fn add_task(
      data: actix_web::web::Data<State>,
      task: Json<NewTask>,
  ) -> actix_web::Result<actix_web::HttpResponse> {
      let tasks: Collection<Task> = data.mongo_client.database("todo").collection("task");

      tasks.insert_one(&Task { id: None, name: task.name.clone(), priority: task.priority }, None).await.unwrap();

      Ok(HttpResponse::NoContent().finish())
  }

  #[actix_web::delete("task/{task_id}")]
  async fn delete_task(
    data: actix_web::web::Data<State>,
    task_id: Query<ObjectId>,
  ) -> actix_web::Result<actix_web::HttpResponse> {
    let tasks: Collection<Task> = data.mongo_client.database("todo").collection("task");

    let result = tasks.delete_one(doc! {
      "_id": task_id.0,
    }, None).await.unwrap();

    Ok(if result.deleted_count == 1 {
      HttpResponse::Ok().finish()
    } else {
      HttpResponse::NotFound().finish()
    })
  }
}