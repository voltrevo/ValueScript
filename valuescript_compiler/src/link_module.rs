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

  link_import_patterns(
    &mut path_and_module.module,
    &included_modules,
    &mut result.diagnostics,
  );

  collapse_pointers_of_pointers(&mut path_and_module.module);
  // TODO: shake_tree(&mut path_and_module.module);

  result.module = Some(path_and_module.module);
  result
}

fn rewrite_pointers(module: &mut Module, pointer_allocator: &mut NameAllocator) {
  let mut pointer_map = HashMap::<Pointer, Pointer>::new();

  for definition in &module.definitions {
    let mapped_pointer = Pointer {
      name: pointer_allocator.allocate(&definition.pointer.name),
    };

    if mapped_pointer != definition.pointer {
      pointer_map.insert(definition.pointer.clone(), mapped_pointer);
    }
  }

  visit_pointers(module, |visitation| match visitation {
    PointerVisitation::Export(pointer)
    | PointerVisitation::Definition(pointer)
    | PointerVisitation::Reference(_, pointer) => {
      if let Some(mapped_pointer) = pointer_map.get(pointer) {
        *pointer = mapped_pointer.clone();
      }
    }
  });
}

fn visit_pointers<Visitor>(module: &mut Module, visitor: Visitor)
where
  Visitor: Fn(PointerVisitation) -> (),
{
  let pointer_visitor = VisitPointerImpl::new(visitor);
  pointer_visitor.module(module);
}

#[derive(PartialEq)]
enum PointerVisitation<'a> {
  Export(&'a mut Pointer),
  Definition(&'a mut Pointer),
  Reference(&'a Pointer, &'a mut Pointer),
}

struct VisitPointerImpl<Visitor>
where
  Visitor: Fn(PointerVisitation) -> (),
{
  visitor: Visitor,
}

impl<Visitor> VisitPointerImpl<Visitor>
where
  Visitor: Fn(PointerVisitation) -> (),
{
  fn new(visitor: Visitor) -> Self {
    Self { visitor }
  }

  pub fn module(&self, module: &mut Module) {
    self.value(None, &mut module.export_default);
    self.object(None, &mut module.export_star);

    for definition in &mut module.definitions {
      self.definition(definition);
    }
  }

  fn definition(&self, definition: &mut Definition) {
    (self.visitor)(PointerVisitation::Definition(&mut definition.pointer));

    match &mut definition.content {
      DefinitionContent::Function(function) => {
        self.body(&definition.pointer, &mut function.body);
      }
      DefinitionContent::Class(class) => {
        self.value(Some(&definition.pointer), &mut class.constructor);
        self.value(Some(&definition.pointer), &mut class.methods);
      }
      DefinitionContent::Value(value) => {
        self.value(Some(&definition.pointer), value);
      }
      DefinitionContent::Lazy(lazy) => {
        self.body(&definition.pointer, &mut lazy.body);
      }
    }
  }

  fn array(&self, owner: Option<&Pointer>, array: &mut Array) {
    for value in &mut array.values {
      self.value(owner, value);
    }
  }

  fn object(&self, owner: Option<&Pointer>, object: &mut Object) {
    for (key, value) in object.properties.iter_mut() {
      self.value(owner, key);
      self.value(owner, value);
    }
  }

  fn value(&self, owner: Option<&Pointer>, value: &mut Value) {
    use Value::*;

    match value {
      Void => {}
      Undefined => {}
      Null => {}
      Bool(_) => {}
      Number(_) => {}
      String(_) => {}
      Array(array) => {
        self.array(owner, array);
      }
      Object(object) => {
        self.object(owner, object);
      }
      Register(_) => {}
      Pointer(pointer) => {
        (self.visitor)(match owner {
          Some(owner) => PointerVisitation::Reference(owner, pointer),
          None => PointerVisitation::Export(pointer),
        });
      }
      Builtin(_) => {}
    }
  }

  fn instruction(&self, owner: &Pointer, instruction: &mut Instruction) {
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
        self.value(Some(owner), arg);
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
        self.value(Some(owner), arg1);
        self.value(Some(owner), arg2);
      }
      Apply(arg1, arg2, arg3, _) | SubCall(arg1, arg2, arg3, _) => {
        self.value(Some(owner), arg1);
        self.value(Some(owner), arg2);
        self.value(Some(owner), arg3);
      }
      Jmp(_) => {}
      JmpIf(arg, _) => {
        self.value(Some(owner), arg);
      }
    };
  }

  fn body(&self, owner: &Pointer, body: &mut Vec<InstructionOrLabel>) {
    for instruction_or_label in body {
      match instruction_or_label {
        InstructionOrLabel::Instruction(instruction) => {
          self.instruction(owner, instruction);
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

fn link_import_patterns(
  module: &mut Module,
  included_modules: &HashMap<ResolvedPath, (Value, Object)>,
  diagnostics: &mut Vec<Diagnostic>,
) {
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
}

fn collapse_pointers_of_pointers(module: &mut Module) {
  let mut double_pointer_map = HashMap::<Pointer, Pointer>::new();

  for definition in &mut module.definitions {
    let pointer = match &definition.content {
      DefinitionContent::Value(Value::Pointer(pointer)) => pointer,
      _ => continue,
    };

    double_pointer_map.insert(definition.pointer.clone(), pointer.clone());
  }

  visit_pointers(module, |visitation| match visitation {
    PointerVisitation::Definition(_) => {}
    PointerVisitation::Export(pointer) | PointerVisitation::Reference(_, pointer) => {
      let mut mapped_pointer: &Pointer = pointer;

      loop {
        if let Some(new_pointer) = double_pointer_map.get(mapped_pointer) {
          mapped_pointer = new_pointer;
          continue;
        }

        break;
      }

      *pointer = mapped_pointer.clone();
    }
  });
}
