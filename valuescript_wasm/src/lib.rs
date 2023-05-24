use std::{
  collections::{BTreeMap, HashMap},
  rc::Rc,
};

use wasm_bindgen::prelude::*;

use valuescript_compiler::{
  asm::Value, assemble, assembly_parser::AssemblyParser, compile as compile_internal,
  CompileResult, Diagnostic, ResolvedPath,
};
use valuescript_vm::{
  vs_array::VsArray, vs_object::VsObject, vs_value::Val, LoadFunctionResult, ValTrait,
  VirtualMachine,
};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
  fn js_get_error_message(e: &JsValue) -> String;
  fn js_console_log(s: &str);
}

#[derive(serde::Serialize)]
struct RunResult {
  diagnostics: HashMap<String, Vec<Diagnostic>>,
  output: Result<String, String>,
}

#[derive(serde::Serialize)]
struct CompilerOutputWasm {
  diagnostics: HashMap<String, Vec<Diagnostic>>,
  assembly: Vec<String>,
}

impl CompilerOutputWasm {
  fn from_compile_result(result: CompileResult) -> CompilerOutputWasm {
    CompilerOutputWasm {
      diagnostics: result
        .diagnostics // TODO: Avoid conversion
        .into_iter()
        .map(|(path, diagnostics)| (path.to_string(), diagnostics))
        .collect(),
      assembly: match result.module {
        Some(module) => module.as_lines(),
        None => vec![],
      },
    }
  }
}

#[wasm_bindgen]
pub fn compile(entry_point: &str, read_file: &js_sys::Function) -> String {
  let compile_result = compile_internal(ResolvedPath::from(entry_point.to_string()), |path| {
    let call_result = read_file.call1(&JsValue::UNDEFINED, &JsValue::from_str(path));

    match call_result {
      Ok(result) => result
        .as_string()
        .ok_or_else(|| "read_file from JS produced non-string".into()),
      Err(err) => Err(js_get_error_message(&err)),
    }
  });

  serde_json::to_string(&CompilerOutputWasm::from_compile_result(compile_result))
    .expect("Failed json serialization")
}

fn run_to_result(entry_point: &str, read_file: &js_sys::Function, args: &str) -> RunResult {
  let compile_result = compile_internal(ResolvedPath::from(entry_point.to_string()), |path| {
    let call_result = read_file.call1(&JsValue::UNDEFINED, &JsValue::from_str(path));

    match call_result {
      Ok(result) => result
        .as_string()
        .ok_or_else(|| "read_file from JS produced non-string".into()),
      Err(err) => Err(js_get_error_message(&err)),
    }
  });

  let diagnostic_len = compile_result
    .diagnostics
    .iter()
    .map(|(_, v)| v.len())
    .sum::<usize>();

  if diagnostic_len > 0 {
    return RunResult {
      diagnostics: compile_result
        .diagnostics // TODO: Avoid conversion
        .into_iter()
        .map(|(path, diagnostics)| (path.to_string(), diagnostics))
        .collect(),
      output: Err("Compile failed".into()),
    };
  }

  let module = match compile_result.module {
    Some(module) => module,
    None => {
      return RunResult {
        diagnostics: HashMap::default(),
        output: Err("Compilation did not emit module".into()),
      }
    }
  };

  let bytecode = assemble(&module);

  match VirtualMachine::read_default_export(&bytecode).load_function() {
    LoadFunctionResult::NotAFunction => {
      return RunResult {
        diagnostics: HashMap::default(),
        output: Ok("(Default export is not a function)".into()),
      }
    }
    _ => {}
  };

  let mut vm = VirtualMachine::new();

  let val_args: Vec<Val> = match parse_args(args) {
    Ok(args) => args,
    Err(err) => {
      return RunResult {
        diagnostics: HashMap::default(),
        output: Err(err.codify()),
      }
    }
  };

  let vm_result = vm.run(&bytecode, None, &val_args);

  RunResult {
    diagnostics: HashMap::default(),
    output: match vm_result {
      Ok(result) => Ok(result.codify()),
      Err(err) => Err(err.codify()),
    },
  }
}

#[wasm_bindgen]
pub fn run(entry_point: &str, read_file: &js_sys::Function, args: &str) -> String {
  let result = run_to_result(entry_point, read_file, args);
  serde_json::to_string(&result).expect("Failed json serialization")
}

fn parse_args(args: &str) -> Result<Vec<Val>, Val> {
  let mut assembler = AssemblyParser {
    content: args,
    pos: args.chars().peekable(),
  };

  let value = assembler.assemble_value();

  let arr = match value {
    Value::Array(arr) => arr,
    _ => return Err(Val::String(Rc::new("Expected array".into()))),
  };

  let mut result = Vec::<Val>::new();

  for arg in arr.values {
    result.push(value_to_val(arg)?);
  }

  Ok(result)
}

fn value_to_val(value: Value) -> Result<Val, Val> {
  Ok(match value {
    Value::Undefined => Val::Undefined,
    Value::Null => Val::Null,
    Value::Bool(b) => Val::Bool(b),
    Value::Number(n) => Val::Number(n),
    Value::BigInt(n) => Val::BigInt(n),
    Value::String(s) => Val::String(Rc::new(s)),
    Value::Array(arr) => {
      let mut result = Vec::<Val>::new();

      for value in arr.values {
        result.push(value_to_val(value)?);
      }

      Val::Array(Rc::new(VsArray::from(result)))
    }
    Value::Object(obj) => {
      let mut string_map = BTreeMap::<String, Val>::new();

      for (key, value) in obj.properties {
        string_map.insert(value_to_val(key)?.val_to_string(), value_to_val(value)?);
      }

      Val::Object(Rc::new(VsObject {
        string_map,
        prototype: None,
      }))
    }

    Value::Void | Value::Register(..) | Value::Pointer(..) | Value::Builtin(..) => {
      return Err(Val::String(Rc::new("Invalid argument".into())));
    }
  })
}
