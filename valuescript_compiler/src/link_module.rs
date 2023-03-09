use std::collections::HashMap;

use crate::asm::{
  Array, Definition, DefinitionContent, Instruction, InstructionOrLabel, Object, Pointer, Value,
};
use crate::gather_modules::PathAndModule;
use crate::import_pattern::{ImportKind, ImportPattern};
use crate::name_allocator::NameAllocator;
use crate::resolve_path::{resolve_path, ResolvedPath};
use crate::DiagnosticLevel;
use crate::{asm::Module, Diagnostic};

pub struct LinkModuleResult {
  pub module: Option<Module>,
  pub diagnostics: Vec<Diagnostic>,
}

pub fn link_module(
  entry_point: &ResolvedPath,
  modules: &HashMap<ResolvedPath, PathAndModule>,
) -> LinkModuleResult {
  let mut result = LinkModuleResult {
    module: None,
    diagnostics: vec![],
  };

  let mut pointer_allocator = NameAllocator::default();
  let mut included_modules = HashMap::<ResolvedPath, (Value, Object)>::new();

  let mut path_and_module = match modules.get(&entry_point.clone()) {
    Some(path_and_module) => path_and_module.clone(),
    None => {
      result.diagnostics.push(Diagnostic {
        level: DiagnosticLevel::Error,
        message: format!("Module not found: {}", entry_point),
        span: swc_common::DUMMY_SP,
      });

      return result;
    }
  };

  let mut modules_to_include = resolve_and_rewrite_import_patterns(&mut path_and_module);
  let mut modules_to_include_i = 0;

  // No rewrites should actually occur here, but we still need to do this to get the names into the
  // allocator.
  rewrite_pointers(&mut path_and_module.module, &mut pointer_allocator);

  included_modules.insert(
    entry_point.clone(),
    (
      path_and_module.module.export_default.clone(),
      path_and_module.module.export_star.clone(),
    ),
  );

  while modules_to_include_i < modules_to_include.len() {
    let module_to_include = modules_to_include[modules_to_include_i].clone();
    modules_to_include_i += 1;

    if included_modules.contains_key(&module_to_include) {
      continue;
    }

    let mut including_path_and_module = match modules.get(&module_to_include) {
      Some(pm) => pm.clone(),
      None => {
        result.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::Error,
          message: format!("Module not found: {}", module_to_include),
          span: swc_common::DUMMY_SP,
        });

        continue;
      }
    };

    let mut new_modules_to_include =
      resolve_and_rewrite_import_patterns(&mut including_path_and_module);

    modules_to_include.append(&mut new_modules_to_include);

    rewrite_pointers(
      &mut including_path_and_module.module,
      &mut pointer_allocator,
    );

    included_modules.insert(
      module_to_include,
      (
        including_path_and_module.module.export_default,
        including_path_and_module.module.export_star,
      ),
    );

    path_and_module
      .module
      .definitions
      .append(&mut including_path_and_module.module.definitions);
  }

  result.module = Some(rewrite_import_patterns(
    path_and_module.module,
    &included_modules,
    &mut result.diagnostics,
  ));

  result
}

fn rewrite_pointers(module: &mut Module, pointer_allocator: &mut NameAllocator) {
  let mut pointer_rewriter = PointerRewriter::init(module, pointer_allocator);
  pointer_rewriter.module(module);
}

struct PointerRewriter {
  pointer_map: HashMap<Pointer, Pointer>,
}

impl PointerRewriter {
  pub fn init(module: &Module, pointer_allocator: &mut NameAllocator) -> Self {
    let mut self_ = Self {
      pointer_map: HashMap::new(),
    };

    for definition in &module.definitions {
      let mapped_pointer = Pointer {
        name: pointer_allocator.allocate(&definition.pointer.name),
      };

      if mapped_pointer != definition.pointer {
        self_
          .pointer_map
          .insert(definition.pointer.clone(), mapped_pointer);
      }
    }

    self_
  }

  pub fn module(&mut self, module: &mut Module) {
    self.value(&mut module.export_default);
    self.object(&mut module.export_star);

    for definition in &mut module.definitions {
      self.definition(definition);
    }
  }

  fn definition(&mut self, definition: &mut Definition) {
    self.pointer(&mut definition.pointer);

    match &mut definition.content {
      DefinitionContent::Function(function) => {
        self.body(&mut function.body);
      }
      DefinitionContent::Class(class) => {
        self.value(&mut class.constructor);
        self.value(&mut class.methods);
      }
      DefinitionContent::Value(value) => {
        self.value(value);
      }
      DefinitionContent::Lazy(lazy) => {
        self.body(&mut lazy.body);
      }
    }
  }

  fn pointer(&mut self, pointer: &mut Pointer) {
    if let Some(mapped_pointer) = self.pointer_map.get(&pointer) {
      *pointer = mapped_pointer.clone();
    }
  }

  fn array(&mut self, array: &mut Array) {
    for value in &mut array.values {
      self.value(value);
    }
  }

  fn object(&mut self, object: &mut Object) {
    for (key, value) in object.properties.iter_mut() {
      self.value(key);
      self.value(value);
    }
  }

  fn value(&mut self, value: &mut Value) {
    use Value::*;

    match value {
      Void => {}
      Undefined => {}
      Null => {}
      Bool(_) => {}
      Number(_) => {}
      String(_) => {}
      Array(array) => {
        self.array(array);
      }
      Object(object) => {
        self.object(object);
      }
      Register(_) => {}
      Pointer(pointer) => {
        self.pointer(pointer);
      }
      Builtin(_) => {}
    }
  }

  fn instruction(&mut self, instruction: &mut Instruction) {
    use Instruction::*;

    match instruction {
      End => {}
      OpInc(_) | OpDec(_) => {}
      Mov(arg, _)
      | OpNot(arg, _)
      | OpBitNot(arg, _)
      | TypeOf(arg, _)
      | UnaryPlus(arg, _)
      | UnaryMinus(arg, _)
      | Import(arg, _)
      | ImportStar(arg, _) => {
        self.value(arg);
      }
      OpPlus(arg1, arg2, _)
      | OpMinus(arg1, arg2, _)
      | OpMul(arg1, arg2, _)
      | OpDiv(arg1, arg2, _)
      | OpMod(arg1, arg2, _)
      | OpExp(arg1, arg2, _)
      | OpEq(arg1, arg2, _)
      | OpNe(arg1, arg2, _)
      | OpTripleEq(arg1, arg2, _)
      | OpTripleNe(arg1, arg2, _)
      | OpAnd(arg1, arg2, _)
      | OpOr(arg1, arg2, _)
      | OpLess(arg1, arg2, _)
      | OpLessEq(arg1, arg2, _)
      | OpGreater(arg1, arg2, _)
      | OpGreaterEq(arg1, arg2, _)
      | OpNullishCoalesce(arg1, arg2, _)
      | OpOptionalChain(arg1, arg2, _)
      | OpBitAnd(arg1, arg2, _)
      | OpBitOr(arg1, arg2, _)
      | OpBitXor(arg1, arg2, _)
      | OpLeftShift(arg1, arg2, _)
      | OpRightShift(arg1, arg2, _)
      | OpRightShiftUnsigned(arg1, arg2, _)
      | InstanceOf(arg1, arg2, _)
      | In(arg1, arg2, _)
      | Call(arg1, arg2, _)
      | Bind(arg1, arg2, _)
      | Sub(arg1, arg2, _)
      | SubMov(arg1, arg2, _)
      | New(arg1, arg2, _) => {
        self.value(arg1);
        self.value(arg2);
      }
      Apply(arg1, arg2, arg3, _) | SubCall(arg1, arg2, arg3, _) => {
        self.value(arg1);
        self.value(arg2);
        self.value(arg3);
      }
      Jmp(_) => {}
      JmpIf(arg, _) => {
        self.value(arg);
      }
    };
  }

  fn body(&mut self, body: &mut Vec<InstructionOrLabel>) {
    for instruction_or_label in body {
      match instruction_or_label {
        InstructionOrLabel::Instruction(instruction) => {
          self.instruction(instruction);
        }
        InstructionOrLabel::Label(_) => {}
      }
    }
  }
}

fn resolve_and_rewrite_import_patterns(path_and_module: &mut PathAndModule) -> Vec<ResolvedPath> {
  let mut resolved_paths = Vec::<ResolvedPath>::new();

  for definition in &mut path_and_module.module.definitions {
    match ImportPattern::decode(definition) {
      Some(_) => {}
      None => continue,
    }

    let lazy = match &mut definition.content {
      DefinitionContent::Lazy(lazy) => lazy,
      _ => panic!("Inconsistent with import pattern"),
    };

    let first_instruction = match lazy.body.first_mut() {
      Some(InstructionOrLabel::Instruction(instruction)) => instruction,
      _ => panic!("Inconsistent with import pattern"),
    };

    let import_string = match first_instruction {
      Instruction::Import(Value::String(string), _)
      | Instruction::ImportStar(Value::String(string), _) => string,
      _ => panic!("Inconsistent with import pattern"),
    };

    let resolved = resolve_path(&path_and_module.path, import_string);
    resolved_paths.push(resolved.clone());
    *import_string = resolved.path;
  }

  resolved_paths
}

fn rewrite_import_patterns(
  mut module: Module,
  included_modules: &HashMap<ResolvedPath, (Value, Object)>,
  diagnostics: &mut Vec<Diagnostic>,
) -> Module {
  for definition in &mut module.definitions {
    let import_pattern = match ImportPattern::decode(definition) {
      Some(import_pattern) => import_pattern,
      None => continue,
    };

    let resolved_path = ResolvedPath {
      // Should have been resolved already during resolve_and_rewrite_import_patterns
      path: import_pattern.path.clone(),
    };

    let (default, namespace) = match included_modules.get(&resolved_path) {
      Some(el) => el,
      None => continue,
    };

    let new_definition = Definition {
      pointer: import_pattern.pointer,
      content: match import_pattern.kind {
        ImportKind::Default => DefinitionContent::Value(default.clone()),
        ImportKind::Star => DefinitionContent::Value(Value::Object(Box::new(namespace.clone()))),
        ImportKind::Name(name) => match namespace.try_resolve_key(&name) {
          Some(value) => DefinitionContent::Value(value.clone()),
          None => {
            diagnostics.push(Diagnostic {
              level: DiagnosticLevel::Error,
              message: format!(
                "Imported name `{}` does not exist in `{}`",
                name, import_pattern.path
              ),
              span: swc_common::DUMMY_SP,
            });

            continue;
          }
        },
      },
    };

    *definition = new_definition;
  }

  module
}
