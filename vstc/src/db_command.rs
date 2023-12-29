use std::{io::Write, process::exit, rc::Rc};

use actix_web::{dev, web, App, FromRequest, HttpRequest, HttpResponse, HttpServer, Responder};

use storage::{storage_head_ptr, SledBackend, Storage, StorageReader};
use valuescript_compiler::{assemble, compile_str, inline_valuescript};
use valuescript_vm::{
  vs_object::VsObject,
  vs_value::{ToVal, Val},
  Bytecode, DecoderMaker, VirtualMachine,
};

use crate::{
  create_db::create_db,
  exit_command_failed::exit_command_failed,
  handle_diagnostics_cli::handle_diagnostics_cli,
  parse_command_line::parse_command_line,
  to_bytecode::{format_from_path, to_bytecode},
};

pub fn db_command(args: &[String]) {
  let mut help_case = false;

  match args.get(2).map(|s| s.as_str()) {
    Some("help") | Some("-h") | Some("--help") => help_case = true,
    _ => {}
  };

  match args.get(3).map(|s| s.as_str()) {
    Some("help") | Some("-h") | Some("--help") => help_case = true,
    _ => {}
  };

  if help_case {
    show_help();
    return;
  }

  let path = match args.get(2) {
    Some(path) => path.clone(),
    None => {
      exit_command_failed(args, Some("Missing db path"), "vstc db help");
    }
  };

  // let mut storage = Storage::new(SledBackend::open(path).unwrap());

  match args.get(3).map(|s| s.as_str()) {
    Some("new") => db_new(&path, args.get(4..).unwrap_or_default()),
    Some("call") => db_call(&path, args.get(4..).unwrap_or_default()),
    Some("host") => db_host(&path, args.get(4..).unwrap_or_default()),
    Some("-i") => db_interactive(&path),
    arg => 'b: {
      if let Some(arg) = arg {
        if arg.starts_with('{') || arg.starts_with('(') {
          break 'b db_run_inline(&path, arg);
        }
      }

      exit_command_failed(args, None, "vstc db help");
    }
  }
}

fn show_help() {
  println!("vstc db");
  println!();
  println!("ValueScript database functionality");
  println!();
  println!("USAGE:");
  println!("  vstc db [DB_PATH] [COMMAND] [ARGS]");
  println!();
  println!("Commands:");
  println!("  help, -h, --help          Show this message");
  println!("  new [CLASS_FILE] [ARGS]   Create a new database");
  println!("  call [FN_FILE] [ARGS]     Call a function on the database");
  println!("  '([EXPRESSION])'          Run expression with database as `this`");
  println!("  '{{[FN BODY]}}'             Run code block with database as `this`");
  println!("  -i                        Enter interactive mode");
  println!();
  println!("Examples:");
  println!("  vstc db path/widget.vsdb new Widget.ts       Create a new widget database");
  println!("  vstc db path/widget.vsdb call useWidget.ts   Call useWidget.ts on the widget");
  println!("  vstc db path/widget.vsdb '(this.info())'     Call info method");
  println!("  vstc db path/widget.vsdb '{{ const t = this; return t.info(); }}'");
  println!("                                               Call info method (enforcing read-only)");
  println!("  vstc db path/widget.vsdb -i                  Enter interactive mode");
}

fn make_storage(path: &String) -> Storage<SledBackend> {
  Storage::new(SledBackend::open(path).unwrap())
}

fn db_new(path: &String, args: &[String]) {
  let class_path = match args.get(0) {
    Some(class_path) => class_path,
    None => {
      exit_command_failed(args, Some("Missing class file"), "vstc db help");
    }
  };

  let args = args
    .get(1..)
    .unwrap_or_default()
    .iter()
    .map(|s| s.clone().to_val())
    .collect::<Vec<_>>();

  create_db(&mut make_storage(path), class_path, &args).expect("Failed to write to db");

  println!("Created database");
}

fn db_call(path: &String, args: &[String]) {
  let fn_file = match args.get(0) {
    Some(fn_file) => fn_file,
    None => exit_command_failed(args, Some("Missing function file"), "vstc db help"),
  };

  let fn_ = Rc::new(to_bytecode(format_from_path(fn_file), fn_file))
    .decoder(0)
    .decode_val(&mut vec![]);

  let args = args
    .get(1..)
    .unwrap_or_default()
    .iter()
    .map(|s| s.clone().to_val())
    .collect::<Vec<_>>();

  let mut vm = VirtualMachine::default();

  let mut storage = make_storage(path);

  let mut instance = storage
    .get_head(storage_head_ptr(b"state"))
    .unwrap()
    .unwrap();

  match vm.run(None, &mut instance, fn_, args) {
    Ok(res) => {
      println!("{}", res.pretty());
    }
    Err(err) => {
      println!("Uncaught exception: {}", err.pretty());
      exit(1);
    }
  }

  storage
    .set_head(storage_head_ptr(b"state"), &instance)
    .unwrap();
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

fn db_host(path: &str, args: &[String]) {
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

fn db_run_inline(path: &String, source: &str) {
  let mut vm = VirtualMachine::default();

  let mut storage = make_storage(path);

  let mut instance = storage
    .get_head::<Val>(storage_head_ptr(b"state"))
    .unwrap()
    .unwrap();

  let full_source = {
    if source.starts_with('(') {
      format!("export default function() {{ return (\n  {}\n); }}", source)
    } else if source.starts_with('{') {
      format!("export default function() {}", source)
    } else {
      panic!("Unrecognized inline code: {}", source);
    }
  };

  let compile_result = compile_str(&full_source);

  for (path, diagnostics) in compile_result.diagnostics.iter() {
    // TODO: Fix exit call
    handle_diagnostics_cli(&path.path, diagnostics);
  }

  let bytecode = Rc::new(Bytecode::new(assemble(
    &compile_result
      .module
      .expect("Should have exited if module is None"),
  )));

  let fn_ = bytecode.decoder(0).decode_val(&mut vec![]);

  match vm.run(None, &mut instance, fn_, vec![]) {
    Ok(res) => {
      println!("{}", res.pretty());
    }
    Err(err) => {
      println!("Uncaught exception: {}", err.pretty());
      exit(1);
    }
  }

  storage
    .set_head(storage_head_ptr(b"state"), &instance)
    .unwrap();
}

fn db_interactive(path: &String) {
  loop {
    let mut input = String::new();

    print!("> ");
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut input).unwrap();
    input.pop();

    let args = parse_command_line(&input);

    match args.get(0).map(|s| s.as_str()) {
      Some("help") => {
        // TODO: help (it's a bit different - code isn't quoted (TODO: quoted should work too))
        println!("TODO: help");
      }
      Some("exit" | "quit") => break,
      Some("new") => db_new(path, args.get(1..).unwrap_or_default()),
      Some("call") => db_call(path, args.get(1..).unwrap_or_default()),
      _ => 'b: {
        if input.starts_with('{') || input.starts_with('(') {
          break 'b db_run_inline(path, &input);
        }

        println!("Command failed: {:?}", args);
        println!("  For help: help");
      }
    }
  }
}
