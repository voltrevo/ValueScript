use std::collections::{BTreeMap, HashMap};

use wasm_bindgen::prelude::*;

use valuescript_compiler::{
  asm::Value, assemble, assembly_parser::AssemblyParser, compile as compile_internal,
  CompileResult, Diagnostic, ResolvedPath,
};
use valuescript_vm::{
  vs_object::VsObject,
  vs_value::{ToVal, Val},
  LoadFunctionResult, ValTrait, VirtualMachine,
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
    _ => return Err("Expected array".to_val()),
  };

  let mut result = Vec::<Val>::new();

  for arg in arr.values {
    result.push(arg.try_to_val()?);
  }

  Ok(result)
}

pub trait TryToVal {
  fn try_to_val(self) -> Result<Val, Val>;
}

impl TryToVal for Value {
  fn try_to_val(self) -> Result<Val, Val> {
    Ok(match self {
      Value::Undefined => Val::Undefined,
      Value::Null => Val::Null,
      Value::Bool(b) => b.to_val(),
      Value::Number(n) => n.to_val(),
      Value::BigInt(n) => n.to_val(),
      Value::String(s) => s.to_val(),
      Value::Array(arr) => {
        let mut result = Vec::<Val>::new();

        for value in arr.values {
          result.push(value.try_to_val()?);
        }

        result.to_val()
      }
      Value::Object(obj) => {
        let mut string_map = BTreeMap::<String, Val>::new();

        for (key, value) in obj.properties {
          string_map.insert(key.try_to_val()?.val_to_string(), value.try_to_val()?);
        }

        VsObject {
          string_map,
          symbol_map: Default::default(),
          prototype: None,
        }
        .to_val()
      }

      Value::Void | Value::Register(..) | Value::Pointer(..) | Value::Builtin(..) => {
        return Err("Invalid argument".to_val());
      }
    })
  }
}
