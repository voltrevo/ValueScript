use crate::exit_command_failed::exit_command_failed;
use actix::{Actor, Addr, Context, Handler, Message};
use actix_web::{
  dev,
  http::{Method, StatusCode},
  web::{self, Bytes},
  App, FromRequest, HttpRequest, HttpResponse, HttpServer, Responder,
};
use storage::{storage_head_ptr, SledBackend, Storage, StorageReader};
use tokio::task::LocalSet;
use valuescript_compiler::inline_valuescript;
use valuescript_vm::{
  vs_object::VsObject,
  vs_value::{ToVal, Val},
  VirtualMachine,
};

pub fn db_host(path: &str, args: &[String]) {
  if !args.is_empty() {
    // TODO
    exit_command_failed(
      args,
      Some("Not implemented: host arguments"),
      "vstc db help",
    );
  }

  // TODO: Multi-thread?
  let runtime = tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .unwrap();

  let local = LocalSet::new();

  local.block_on(&runtime, async {
    let db_actor = DbActor::new(Storage::new(SledBackend::open(path).unwrap())).start();

    HttpServer::new(move || {
      App::new()
        .app_data(web::Data::new(db_actor.clone()))
        .default_service(web::route().to(handle_request))
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .run()
    .await
    .unwrap();
  });
}

async fn handle_request(
  req: HttpRequest,
  payload: web::Payload,
  data: web::Data<Addr<DbActor>>,
) -> impl Responder {
  let req_val = DbRequest {
    path: req.path().to_owned(),
    method: req.method().clone(),
    body: match get_body(&req, payload.into_inner()).await {
      Ok(body) => body,
      Err(_e) => todo!(), // handle error
    },
  };

  match data.send(req_val).await {
    Ok(Ok(res)) => HttpResponse::Ok()
      .content_type("application/json")
      .body(res),
    Ok(Err(err)) => {
      println!("{}", err.message);
      HttpResponse::build(StatusCode::from_u16(err.code).unwrap()).body(err.message)
    }
    Err(err) => {
      println!("{}", err);
      HttpResponse::InternalServerError().body("Mailbox has closed")
    }
  }
}

struct DbRequest {
  path: String,
  method: Method,
  body: Bytes,
}

struct HttpError {
  code: u16,
  message: String,
}

impl Message for DbRequest {
  type Result = Result<String, HttpError>;
}

struct DbActor {
  storage: Storage<SledBackend>,
  apply_fn: Val,
}

impl Actor for DbActor {
  type Context = Context<Self>;
}

impl DbActor {
  fn new(storage: Storage<SledBackend>) -> Self {
    Self {
      storage,
      apply_fn: inline_valuescript(
        r#"
          export default function(req) {
            if ("handleRequest" in this) {
              return this.handleRequest(req);
            }
  
            const handlerName = `${req.method} ${req.path}`;
  
            if (!this[handlerName]) {
              throw new Error("No handler for request");
            }
  
            if (req.method === "GET") {
              // Enforce GET as read-only
              const state = this;
              return state[handlerName](req.body);
            }
  
            return this[handlerName](req.body);
          }
        "#,
      ),
    }
  }
}

impl Handler<DbRequest> for DbActor {
  type Result = Result<String, HttpError>;

  fn handle(&mut self, msg: DbRequest, _ctx: &mut Self::Context) -> Self::Result {
    let mut instance: Val = self
      .storage
      .get_head(storage_head_ptr(b"state"))
      .unwrap()
      .unwrap();

    let DbRequest { path, method, body } = msg;

    let body = if body.is_empty() {
      Val::Undefined
    } else {
      match serde_json::from_slice::<serde_json::Value>(&body) {
        Ok(json_value) => Val::from_json(&json_value),
        Err(_err) => {
          return Err(HttpError {
            code: 400,
            message: "Bad request".to_owned(),
          })
        }
      }
    };

    let req_val = VsObject {
      string_map: vec![
        ("path".to_string(), path.to_val()),
        ("method".to_string(), method.to_string().to_val()),
        ("body".to_string(), body),
      ]
      .into_iter()
      .collect(),
      symbol_map: vec![].into_iter().collect(),
      prototype: Val::Void,
    }
    .to_val();

    let mut vm = VirtualMachine::default();

    let res = match vm.run(None, &mut instance, self.apply_fn.clone(), vec![req_val]) {
      Ok(res) => match res.to_json() {
        Some(json) => Ok(json.to_string()),
        None => Err(HttpError {
          code: 500,
          message: "Failed to serialize response".to_owned(),
        }),
      },
      Err(err) => {
        println!("Uncaught exception: {}", err.pretty());
        Err(HttpError {
          code: 500,
          message: "Uncaught exception".to_owned(),
        })
      }
    };

    if res.is_ok() {
      self
        .storage
        .set_head(storage_head_ptr(b"state"), &instance)
        .unwrap();
    }

    // TODO: Consider more cache retention
    self.storage.clear_read_cache();

    res
  }
}

async fn get_body(req: &HttpRequest, mut payload: dev::Payload) -> Result<Bytes, actix_web::Error> {
  let payload = web::Payload::from_request(req, &mut payload).await?;

  payload
    .to_bytes_limited(1_024 * 1_024)
    .await
    .map_err(|_| actix_web::error::PayloadError::Overflow)?
}
