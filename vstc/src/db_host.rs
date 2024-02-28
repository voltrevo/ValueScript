use crate::exit_command_failed::exit_command_failed;
use actix_web::{dev, web, App, FromRequest, HttpRequest, HttpResponse, HttpServer, Responder};
use storage::{storage_head_ptr, SledBackend, Storage, StorageReader};
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

  let path = path.to_owned();

  // TODO: Multi-thread?
  let runtime = tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .unwrap();

  runtime.block_on(async {
    HttpServer::new(move || {
      App::new()
        .app_data(web::Data::new(path.clone()))
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
  data: web::Data<String>,
) -> impl Responder {
  let path = req.path();
  let method = req.method();
  let mut storage = Storage::new(SledBackend::open(data.as_ref()).unwrap());

  let body = match get_body(&req, payload.into_inner()).await {
    Ok(body) => body,
    Err(e) => return e.into(),
  };

  let mut instance: Val = storage
    .get_head(storage_head_ptr(b"state"))
    .unwrap()
    .unwrap();

  let fn_ = inline_valuescript(
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
  );

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

  let res = match vm.run(None, &mut instance, fn_, vec![req_val]) {
    Ok(res) => match res.to_json() {
      Some(json) => HttpResponse::Ok().json(json),
      None => HttpResponse::InternalServerError().body("Failed to serialize response"),
    },
    Err(err) => {
      println!("Uncaught exception: {}", err.pretty());
      HttpResponse::InternalServerError().body("Uncaught exception")
    }
  };

  storage
    .set_head(storage_head_ptr(b"state"), &instance)
    .unwrap();

  res
}

async fn get_body(req: &HttpRequest, mut payload: dev::Payload) -> Result<Val, actix_web::Error> {
  let payload = web::Payload::from_request(req, &mut payload).await?;

  let body = payload
    .to_bytes_limited(1_024 * 1_024)
    .await
    .map_err(|_| actix_web::error::PayloadError::Overflow)??;

  if body.is_empty() {
    Ok(Val::Undefined)
  } else {
    match serde_json::from_slice::<serde_json::Value>(&body) {
      Ok(json_value) => Ok(Val::from_json(&json_value)),
      Err(err) => Err(actix_web::error::ErrorBadRequest(err)),
    }
  }
}
