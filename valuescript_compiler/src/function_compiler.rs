use queues::*;
use std::collections::BTreeSet;
use std::mem::take;

use swc_common::Spanned;

use crate::asm::{
  Builtin, Definition, DefinitionContent, FnLine, Function, Instruction, Label, Pointer, Register,
  Value,
};
use crate::diagnostic::{Diagnostic, DiagnosticContainer, DiagnosticReporter};
use crate::expression_compiler::CompiledExpression;
use crate::expression_compiler::ExpressionCompiler;
use crate::ident::Ident;
use crate::module_compiler::ModuleCompiler;
use crate::name_allocator::{NameAllocator, RegAllocator};
use crate::scope::{NameId, OwnerId};
use crate::scope_analysis::{fn_to_owner_id, Name};

#[derive(Clone, Debug)]
pub enum Functionish {
  Fn(Option<swc_ecma_ast::Ident>, swc_ecma_ast::Function),
  Arrow(swc_ecma_ast::ArrowExpr),
  Constructor(Vec<FnLine>, swc_common::Span, swc_ecma_ast::Constructor),
}

impl Spanned for Functionish {
  fn span(&self) -> swc_common::Span {
    match self {
      Functionish::Fn(_, fn_) => fn_.span,
      Functionish::Arrow(arrow) => arrow.span,
      Functionish::Constructor(_, class_span, _) => *class_span,
    }
  }
}

impl Functionish {
  pub fn owner_id(&self) -> OwnerId {
    match self {
      Functionish::Fn(ident, fn_) => fn_to_owner_id(ident, fn_),
      _ => OwnerId::Span(self.span()),
    }
  }
}

#[derive(Clone, Debug)]
pub struct QueuedFunction {
  pub definition_pointer: Pointer,
  pub fn_name: Option<String>,
  pub functionish: Functionish,
}

pub struct LoopLabels {
  pub continue_: Option<Label>,
  pub break_: Label,
}

pub struct CatchSetting {
  pub label: Label,
  pub reg: Register,
}

pub struct FunctionCompiler<'a> {
  pub mc: &'a mut ModuleCompiler,
  pub current: Function,
  pub owner_id: OwnerId,
  pub reg_allocator: RegAllocator,
  pub label_allocator: NameAllocator,
  pub queue: Queue<QueuedFunction>,
  pub loop_labels: Vec<LoopLabels>,
  pub catch_settings: Vec<CatchSetting>,
  pub end_label: Option<Label>,
  pub is_returning_register: Option<Register>,
  pub finally_labels: Vec<Label>,
}

impl<'a> DiagnosticContainer for FunctionCompiler<'a> {
  fn diagnostics_mut(&mut self) -> &mut Vec<Diagnostic> {
    &mut self.mc.diagnostics
  }
}

impl<'a> FunctionCompiler<'a> {
  pub fn new(mc: &'a mut ModuleCompiler, owner_id: OwnerId) -> Self {
    let reg_allocator = match mc.scope_analysis.reg_allocators.get(&owner_id) {
      Some(reg_allocator) => reg_allocator.clone(),
      None => RegAllocator::default(),
    };

    FunctionCompiler {
      mc,
      current: Function::default(),
      owner_id,
      reg_allocator,
      label_allocator: NameAllocator::default(),
      queue: Queue::new(),
      loop_labels: vec![],
      catch_settings: vec![],
      end_label: None,
      is_returning_register: None,
      finally_labels: vec![],
    }
  }

  pub fn push(&mut self, mut instruction: Instruction) {
    if instruction_needs_mutable_this(&mut instruction) {
      self.push_raw(Instruction::RequireMutableThis);
    }

    self.push_raw(instruction);
  }

  pub fn push_raw(&mut self, instruction: Instruction) {
    self.current.body.push(FnLine::Instruction(instruction));
  }

  pub fn label(&mut self, label: Label) {
    self.current.body.push(FnLine::Label(label));
  }

  #[allow(dead_code)]
  pub fn comment(&mut self, message: String) {
    self.current.body.push(FnLine::Comment(message));
  }

  pub fn lookup(&mut self, ident: &Ident) -> Option<&Name> {
    let name = self.mc.scope_analysis.lookup(ident);

    if name.is_none() {
      self.mc.diagnostics.push(Diagnostic::internal_error(
        ident.span,
        &format!("Could not find name for ident {:?}", ident),
      ));
    }

    name
  }

  pub fn lookup_value(&self, ident: &Ident) -> Option<Value> {
    self.mc.scope_analysis.lookup_value(&self.owner_id, ident)
  }

  pub fn lookup_by_name_id(&self, name_id: &NameId) -> Option<Value> {
    self
      .mc
      .scope_analysis
      .lookup_by_name_id(&self.owner_id, name_id)
  }

  pub fn allocate_defn(&mut self, name: &str) -> Pointer {
    let allocated_name = self.mc.definition_allocator.allocate(&name.to_string());

    Pointer {
      name: allocated_name,
    }
  }

  pub fn allocate_defn_numbered(&mut self, name: &str) -> Pointer {
    let allocated_name = self.mc.definition_allocator.allocate_numbered(name);

    Pointer {
      name: allocated_name,
    }
  }

  pub fn allocate_tmp(&mut self) -> Register {
    self.reg_allocator.allocate_numbered("_tmp")
  }

  pub fn allocate_reg(&mut self, based_on: &str) -> Register {
    self.reg_allocator.allocate(based_on)
  }

  pub fn allocate_reg_fresh(&mut self, based_on: &str) -> Register {
    self.reg_allocator.allocate_fresh(based_on)
  }

  pub fn allocate_numbered_reg(&mut self, prefix: &str) -> Register {
    self.reg_allocator.allocate_numbered(prefix)
  }

  pub fn allocate_numbered_reg_fresh(&mut self, prefix: &str) -> Register {
    self.reg_allocator.allocate_numbered_fresh(prefix)
  }

  pub fn release_reg(&mut self, reg: &Register) {
    // Note: We no longer release registers back into the name allocator. See
    // `NameAllocator::release` for more information.
    // self.reg_allocator.release(reg);

    self.current.body.push(FnLine::Release(reg.clone()));
  }

  pub fn insert_all_releases(&mut self) {
    for reg in self.reg_allocator.all_used() {
      if !reg.is_special() {
        self.current.body.push(FnLine::Release(reg));
      }
    }
  }

  pub fn compile(
    mc: &'a mut ModuleCompiler,
    definition_pointer: Pointer,
    fn_name: Option<String>,
    functionish: Functionish,
  ) {
    let mut self_ = FunctionCompiler::new(mc, functionish.owner_id());

    self_
      .queue
      .add(QueuedFunction {
        definition_pointer,
        fn_name,
        functionish,
      })
      .expect("Failed to queue function");

    self_.process_queue();
  }

  pub fn process_queue(&mut self) {
    while let Ok(qfn) = self.queue.remove() {
      self.compile_functionish(qfn.definition_pointer, &qfn.functionish);
    }
  }

  fn compile_functionish(&mut self, definition_pointer: Pointer, functionish: &Functionish) {
    self.current.is_generator = match functionish {
      Functionish::Fn(_, fn_) => fn_.is_generator,

      // Note: It isn't currently possible to have an arrow generator, but SWC includes the
      // possibility in the ast.
      Functionish::Arrow(arrow_expr) => arrow_expr.is_generator,

      Functionish::Constructor(..) => false,
    };

    // TODO: Use a new FunctionCompiler per function instead of this hack
    self.reg_allocator = match self
      .mc
      .scope_analysis
      .reg_allocators
      .get(&functionish.owner_id())
    {
      Some(reg_allocator) => reg_allocator.clone(),
      None => RegAllocator::default(),
    };

    self.owner_id = functionish.owner_id();

    let capture_params = self
      .mc
      .scope_analysis
      .get_register_captures(&functionish.owner_id());

    for cap_param in capture_params {
      let reg = match self
        .mc
        .scope_analysis
        .lookup_capture(&self.owner_id, &cap_param)
      {
        Some(Value::Register(reg)) => reg,
        _ => {
          self.internal_error(
            cap_param.span(),
            "Unexpected non-register in captured_registers",
          );
          continue;
        }
      };

      self.current.parameters.push(reg.clone());
    }

    let param_registers = self.get_param_registers(functionish);

    for reg in param_registers.iter().flatten() {
      self.current.parameters.push(reg.clone());
    }

    self.add_param_code(functionish, &param_registers);

    match functionish {
      Functionish::Fn(_, fn_) => {
        match &fn_.body {
          Some(block) => {
            self.handle_block_body(block);
          }
          None => self.todo(
            fn_.span(),
            "function without body (abstract/interface method?)",
          ),
        };
      }
      Functionish::Arrow(arrow) => match &arrow.body {
        swc_ecma_ast::BlockStmtOrExpr::BlockStmt(block) => {
          self.handle_block_body(block);
        }
        swc_ecma_ast::BlockStmtOrExpr::Expr(expr) => {
          let mut expression_compiler = ExpressionCompiler { fnc: self };

          expression_compiler.compile_into(expr, Register::return_());
        }
      },
      Functionish::Constructor(member_initializers_assembly, _class_span, constructor) => {
        let mut mia_copy = member_initializers_assembly.clone();
        self.current.body.append(&mut mia_copy);

        match &constructor.body {
          Some(block) => {
            self.handle_block_body(block);
          }
          None => self.todo(constructor.span(), "constructor without body"),
        };
      }
    };

    self.insert_all_releases();

    if let Some(end_label) = self.end_label.as_ref() {
      self.current.body.push(FnLine::Label(end_label.clone()));

      self.end_label = None;
      self.is_returning_register = None;
    }

    self.mc.module.definitions.push(Definition {
      pointer: definition_pointer,
      content: DefinitionContent::Function(take(&mut self.current)),
    });
  }

  fn handle_block_body(&mut self, block: &swc_ecma_ast::BlockStmt) {
    for i in 0..block.stmts.len() {
      self.statement(&block.stmts[i], i == block.stmts.len() - 1);
    }
  }

  fn get_param_registers(&mut self, functionish: &Functionish) -> Vec<Option<Register>> {
    let mut param_registers = Vec::<Option<Register>>::new();

    match functionish {
      Functionish::Fn(_, fn_) => {
        for p in &fn_.params {
          param_registers.push(self.get_pattern_register_opt(&p.pat));
        }
      }
      Functionish::Arrow(arrow) => {
        for p in &arrow.params {
          param_registers.push(self.get_pattern_register_opt(p));
        }
      }
      Functionish::Constructor(_, _class_span, constructor) => {
        for potspp in &constructor.params {
          match potspp {
            swc_ecma_ast::ParamOrTsParamProp::TsParamProp(ts_param_prop) => {
              let reg = match &ts_param_prop.param {
                swc_ecma_ast::TsParamPropParam::Ident(ident) => {
                  match ident.id.sym.to_string().as_str() {
                    "this" => None,
                    _ => Some(self.get_variable_register(&Ident::from_swc_ident(&ident.id))),
                  }
                }
                swc_ecma_ast::TsParamPropParam::Assign(assign) => {
                  self.get_pattern_register_opt(&assign.left)
                }
              };

              param_registers.push(reg);
            }
            swc_ecma_ast::ParamOrTsParamProp::Param(p) => {
              param_registers.push(self.get_pattern_register_opt(&p.pat))
            }
          }
        }
      }
    };

    param_registers
  }

  pub fn get_pattern_register_opt(&mut self, param_pat: &swc_ecma_ast::Pat) -> Option<Register> {
    use swc_ecma_ast::Pat;

    Some(match param_pat {
      Pat::Ident(ident) => match ident.id.sym.to_string().as_str() {
        "this" => return None,
        _ => self.get_variable_register(&Ident::from_swc_ident(&ident.id)),
      },
      Pat::Assign(assign) => return self.get_pattern_register_opt(&assign.left),
      Pat::Array(_) => self.allocate_numbered_reg("_array_pat"),
      Pat::Object(_) => self.allocate_numbered_reg("_object_pat"),
      Pat::Invalid(_) => self.allocate_numbered_reg("_invalid_pat"),
      Pat::Rest(_) => self.allocate_numbered_reg("_rest_pat"),
      Pat::Expr(_) => self.allocate_numbered_reg("_expr_pat"),
    })
  }

  pub fn get_pattern_register(&mut self, param_pat: &swc_ecma_ast::Pat) -> Register {
    match self.get_pattern_register_opt(param_pat) {
      Some(r) => r,
      None => {
        self.error(param_pat.span(), "Invalid pattern");
        self.allocate_reg("_invalid_pattern")
      }
    }
  }

  pub fn get_variable_register(&mut self, ident: &Ident) -> Register {
    match self.mc.scope_analysis.lookup_value(&self.owner_id, ident) {
      Some(Value::Register(reg)) => reg,
      lookup_result => {
        self.internal_error(
          ident.span,
          &format!(
            "Register should have been allocated for variable {}, instead: {:?}",
            ident.sym, lookup_result,
          ),
        );

        self.allocate_numbered_reg("_error_variable_without_register")
      }
    }
  }

  fn add_param_code(&mut self, functionish: &Functionish, param_registers: &[Option<Register>]) {
    match functionish {
      Functionish::Fn(_, fn_) => {
        for (i, p) in fn_.params.iter().enumerate() {
          if let Some(reg) = &param_registers[i] {
            let mut ec = ExpressionCompiler { fnc: self };
            ec.pat(&p.pat, reg, false);
          }
        }
      }
      Functionish::Arrow(arrow) => {
        for (i, p) in arrow.params.iter().enumerate() {
          if let Some(reg) = &param_registers[i] {
            let mut ec = ExpressionCompiler { fnc: self };
            ec.pat(p, reg, false);
          }
        }
      }
      Functionish::Constructor(_, _class_span, constructor) => {
        for (i, potspp) in constructor.params.iter().enumerate() {
          match potspp {
            swc_ecma_ast::ParamOrTsParamProp::TsParamProp(tpp) => {
              if let Some(reg) = &param_registers[i] {
                let mut ec = ExpressionCompiler { fnc: self };

                let field_name = match &tpp.param {
                  swc_ecma_ast::TsParamPropParam::Ident(bi) => bi.id.sym.to_string(),
                  swc_ecma_ast::TsParamPropParam::Assign(assign) => {
                    ec.assign_pat(assign, reg);

                    match &*assign.left {
                      swc_ecma_ast::Pat::Ident(bi) => bi.id.sym.to_string(),
                      _ => {
                        ec.fnc.error(assign.left.span(), "Invalid pattern");
                        "_invalid_pattern".to_string()
                      }
                    }
                  }
                };

                ec.fnc.push(Instruction::SubMov(
                  Value::String(field_name),
                  Value::Register(reg.clone()),
                  Register::this(),
                ));
              }
            }
            swc_ecma_ast::ParamOrTsParamProp::Param(p) => {
              if let Some(reg) = &param_registers[i] {
                let mut ec = ExpressionCompiler { fnc: self };
                ec.pat(&p.pat, reg, false);
              }
            }
          }
        }
      }
    };
  }

  fn statement(&mut self, statement: &swc_ecma_ast::Stmt, fn_last: bool) {
    use swc_ecma_ast::Stmt::*;

    match statement {
      Block(block) => self.block_statement(block),
      Empty(_) => {}
      Debugger(debugger) => self.todo(debugger.span, "Debugger statement"),
      With(with) => {
        self.not_supported(with.span, "With statement");
      }

      Return(ret_stmt) => {
        match &ret_stmt.arg {
          None => {}
          Some(expr) => {
            let mut ec = ExpressionCompiler { fnc: self };
            ec.compile_into(expr, Register::return_());
          }
        }

        if !fn_last {
          if let Some(finally_label) = self.finally_labels.last().cloned() {
            let is_returning = match self.is_returning_register.clone() {
              Some(is_returning) => is_returning,
              None => {
                let is_returning = self.allocate_reg_fresh("_is_returning");
                self.is_returning_register = Some(is_returning.clone());
                is_returning
              }
            };

            self.push(Instruction::Mov(Value::Bool(true), is_returning));
            self.push(Instruction::Jmp(finally_label.ref_()));
          } else {
            self.insert_all_releases();
            self.push(Instruction::End);
          }
        }
      }

      Labeled(labeled) => self.todo(labeled.span, "Labeled statement"),

      Break(break_) => {
        if break_.label.is_some() {
          self.todo(break_.span, "labeled break statement");

          return;
        }

        let loop_labels = self.loop_labels.last();

        match loop_labels {
          Some(loop_labels) => {
            self.push(Instruction::Jmp(loop_labels.break_.ref_()));
          }
          None => {
            self.error(break_.span, "break statement outside loop");
          }
        }
      }
      Continue(continue_) => {
        if continue_.label.is_some() {
          self.todo(continue_.span, "labeled continue statement");

          return;
        }

        for label_pair in self.loop_labels.iter().rev() {
          match &label_pair.continue_ {
            Some(continue_label) => {
              self.push(Instruction::Jmp(continue_label.ref_()));
              return;
            }
            None => {}
          }
        }

        self.error(continue_.span, "continue statement outside loop");
      }
      If(if_) => {
        self.if_(if_);
      }
      Switch(switch) => self.switch(switch),
      Throw(throw) => {
        let mut expression_compiler = ExpressionCompiler { fnc: self };

        let arg = expression_compiler.compile(&throw.arg, None);
        let instr = Instruction::Throw(arg.value.clone());

        self.push(instr);
        self.release_ce(arg);

        if self.catch_settings.is_empty() {
          self.insert_all_releases();
        }
      }
      Try(try_) => {
        self.try_(try_);
      }
      While(while_) => {
        self.while_(while_);
      }
      DoWhile(do_while) => {
        self.do_while(do_while);
      }
      For(for_) => {
        self.for_(for_);
      }
      ForIn(for_in) => self.todo(for_in.span, "ForIn statement"),
      ForOf(for_of) => {
        self.for_of(for_of);
      }
      Decl(decl) => {
        self.declaration(decl);
      }
      Expr(expr) => {
        self.expression(&expr.expr);
      }
    }
  }

  fn block_statement(&mut self, block: &swc_ecma_ast::BlockStmt) {
    for stmt in &block.stmts {
      self.statement(stmt, false);
    }
  }

  fn if_(&mut self, if_: &swc_ecma_ast::IfStmt) {
    let mut ec = ExpressionCompiler { fnc: self };

    let cond_reg = ec.fnc.allocate_numbered_reg("_cond");
    ec.compile_into(&if_.test, cond_reg.clone());

    let else_label = Label {
      name: self.label_allocator.allocate_numbered("else"),
    };

    self.push(Instruction::JmpIfNot(
      Value::Register(cond_reg.clone()),
      else_label.ref_(),
    ));

    self.release_reg(&cond_reg);

    self.statement(&if_.cons, false);

    match &if_.alt {
      None => {
        self.label(else_label);
      }
      Some(alt) => {
        let after_else_label = Label {
          name: self.label_allocator.allocate_numbered("after_else"),
        };

        self.push(Instruction::Jmp(after_else_label.ref_()));

        self.label(else_label);
        self.statement(alt, false);
        self.label(after_else_label);
      }
    }
  }

  fn switch(&mut self, switch: &swc_ecma_ast::SwitchStmt) {
    let mut ec = ExpressionCompiler { fnc: self };

    let sw_expr_reg = ec.fnc.allocate_numbered_reg("_sw_expr");
    ec.compile_into(&switch.discriminant, sw_expr_reg.clone());

    let end_label = Label {
      name: ec.fnc.label_allocator.allocate_numbered("sw_end"),
    };

    ec.fnc.loop_labels.push(LoopLabels {
      continue_: None,
      break_: end_label.clone(),
    });

    let case_labels = (0..switch.cases.len())
      .map(&mut |i| Label {
        name: ec
          .fnc
          .label_allocator
          .allocate_numbered(&format!("sw_{}_case", i)),
      })
      .collect::<Vec<Label>>();

    let cond_reg = ec.fnc.allocate_numbered_reg("_sw_cond");
    let mut default_i: Option<usize> = None;

    for (i, case) in switch.cases.iter().enumerate() {
      match &case.test {
        // case test:
        Some(test) => {
          ec.compile_into(test, cond_reg.clone());

          ec.fnc.push(Instruction::OpTripleEq(
            Value::Register(sw_expr_reg.clone()),
            Value::Register(cond_reg.clone()),
            cond_reg.clone(),
          ));

          ec.fnc.push(Instruction::JmpIf(
            Value::Register(cond_reg.clone()),
            case_labels[i].ref_(),
          ));
        }
        // default:
        None => {
          if default_i.is_some() {
            ec.error(case.span, "A switch can only have one default");
          }

          default_i = Some(i);
        }
      };
    }

    ec.fnc.push(Instruction::Jmp(match default_i {
      Some(default_i) => case_labels[default_i].ref_(),
      None => end_label.ref_(),
    }));

    for (i, case) in switch.cases.iter().enumerate() {
      ec.fnc.label(case_labels[i].clone());

      for stmt in &case.cons {
        ec.fnc.statement(stmt, false);
      }
    }

    self.release_reg(&cond_reg);
    self.release_reg(&sw_expr_reg);

    self.label(end_label);
  }

  fn try_(&mut self, try_: &swc_ecma_ast::TryStmt) {
    let (catch_label, after_catch_label) = match try_.handler {
      Some(_) => (
        Some(Label {
          name: self.label_allocator.allocate_numbered("catch"),
        }),
        Some(Label {
          name: self.label_allocator.allocate_numbered("after_catch"),
        }),
      ),
      None => (None, None),
    };

    let finally_label = match &try_.finalizer {
      Some(_) => Some(Label {
        name: self.label_allocator.allocate_numbered("finally"),
      }),
      None => None,
    };

    let mut finally_error_reg: Option<Register> = None;

    if let Some(label) = &finally_label {
      // We use a fresh register here because if we don't put anything in it, it's meaningful. It
      // tells finally not to re-throw.
      let reg = self.allocate_numbered_reg_fresh("_finally_error");
      self.finally_labels.push(label.clone());

      finally_error_reg = Some(reg.clone());

      self.catch_settings.push(CatchSetting {
        label: label.clone(),
        reg,
      });
    }

    let mut catch_error_reg: Option<Register> = None;

    if let Some(label) = &catch_label {
      let reg = match try_
        .handler
        .as_ref()
        .expect("catch label without handler")
        .param
      {
        Some(_) => self.allocate_numbered_reg("_error"),
        None => Register::ignore(),
      };

      catch_error_reg = Some(reg.clone());

      self.catch_settings.push(CatchSetting {
        label: label.clone(),
        reg,
      });
    }

    self.apply_catch_setting();

    let mut snap_pairs = BTreeSet::<(Register, Register)>::new();

    if try_.handler.is_some() {
      let snap_registers = self.get_mutated_registers(try_.block.span);

      for reg in snap_registers {
        let snap_reg = self.allocate_reg_fresh(&format!("snap_{}", reg.name));

        self.push(Instruction::Mov(
          Value::Register(reg.clone()),
          snap_reg.clone(),
        ));

        snap_pairs.insert((reg, snap_reg));
      }
    }

    self.block_statement(&try_.block);
    self.pop_catch_setting(); // TODO: Avoid redundant set_catch to our own finally

    if let Some(label) = &after_catch_label {
      self.push(Instruction::Jmp(label.ref_()));
    }

    if let Some(catch_clause) = &try_.handler {
      self.label(catch_label.unwrap());
      self.apply_catch_setting(); // TODO: Avoid redundant unset_catch

      for (reg, snap_reg) in snap_pairs {
        self.push(Instruction::Mov(Value::Register(snap_reg), reg));
      }

      if let Some(param) = &catch_clause.param {
        let mut ec = ExpressionCompiler { fnc: self };

        let pattern_reg = ec.fnc.get_pattern_register(param);

        // TODO: Set up this register through set_catch instead of copying into it
        ec.fnc.push(Instruction::Mov(
          Value::Register(catch_error_reg.unwrap()),
          pattern_reg.clone(),
        ));

        ec.pat(param, &pattern_reg, false);
      }

      self.block_statement(&catch_clause.body);

      if finally_label.is_some() {
        self.pop_catch_setting();
      }

      self.label(after_catch_label.unwrap());

      // TODO: Shouldn't we be releasing registers from the scope when we don't need it anymore?
    }

    if let Some(finally_clause) = &try_.finalizer {
      self.label(finally_label.unwrap());
      self.finally_labels.pop();
      self.apply_catch_setting();

      let local_is_returning = match self.is_returning_register.clone() {
        Some(is_returning) => {
          let local_is_returning = self.allocate_numbered_reg_fresh("_local_is_returning");

          self.push(Instruction::Mov(
            Value::Register(is_returning.clone()),
            local_is_returning.clone(),
          ));

          self.push(Instruction::Mov(Value::Bool(false), is_returning));

          Some(local_is_returning)
        }
        None => None,
      };

      self.block_statement(finally_clause);

      self.push(Instruction::Throw(Value::Register(
        finally_error_reg.unwrap(),
      )));

      if let Some(local_is_returning) = local_is_returning {
        if self.finally_labels.is_empty() {
          let end_label = match &self.end_label {
            Some(end_label) => end_label.clone(),
            None => {
              let end_label = Label {
                name: self.label_allocator.allocate(&"end".to_string()),
              };

              self.end_label = Some(end_label.clone());
              end_label
            }
          };

          self.push(Instruction::JmpIf(
            Value::Register(local_is_returning),
            end_label.ref_(),
          ));
        } else {
          // TODO: Why are we negating `local_is_returning` here? Can we use `jmpif_not` instead?
          self.push(Instruction::OpNot(
            Value::Register(local_is_returning.clone()),
            local_is_returning.clone(),
          ));

          let after_finally_label = Label {
            name: self.label_allocator.allocate_numbered("after_finally"),
          };

          self.push(Instruction::JmpIf(
            Value::Register(local_is_returning),
            after_finally_label.ref_(),
          ));

          self.push(Instruction::Mov(
            Value::Bool(true),
            self.is_returning_register.clone().unwrap(),
          ));

          self.push(Instruction::Jmp(self.finally_labels.last().unwrap().ref_()));

          self.label(after_finally_label);
        }
      }
    }
  }

  fn apply_catch_setting(&mut self) {
    if let Some(catch_setting) = self.catch_settings.last() {
      self.push(Instruction::SetCatch(
        catch_setting.label.ref_(),
        catch_setting.reg.clone(),
      ));
    } else {
      self.push(Instruction::UnsetCatch);
    }
  }

  fn pop_catch_setting(&mut self) {
    self.catch_settings.pop().expect("no catch setting to pop");
    self.apply_catch_setting();
  }

  fn while_(&mut self, while_: &swc_ecma_ast::WhileStmt) {
    let start_label = Label {
      name: self.label_allocator.allocate_numbered("while"),
    };

    let end_label = Label {
      name: self.label_allocator.allocate_numbered("while_end"),
    };

    self.loop_labels.push(LoopLabels {
      continue_: Some(start_label.clone()),
      break_: end_label.clone(),
    });

    self.label(start_label.clone());

    let mut ec = ExpressionCompiler { fnc: self };

    let cond_reg = ec.fnc.allocate_numbered_reg("_cond");
    ec.compile_into(&while_.test, cond_reg.clone());

    self.push(Instruction::JmpIfNot(
      Value::Register(cond_reg.clone()),
      end_label.ref_(),
    ));

    self.release_reg(&cond_reg);
    self.statement(&while_.body, false);
    self.push(Instruction::Jmp(start_label.ref_()));
    self.label(end_label);

    self.loop_labels.pop();
  }

  fn do_while(&mut self, do_while: &swc_ecma_ast::DoWhileStmt) {
    let start_label = Label {
      name: self.label_allocator.allocate_numbered("do_while"),
    };

    let continue_label = Label {
      name: self.label_allocator.allocate_numbered("do_while_continue"),
    };

    let end_label = Label {
      name: self.label_allocator.allocate_numbered("do_while_end"),
    };

    self.loop_labels.push(LoopLabels {
      continue_: Some(continue_label.clone()),
      break_: end_label.clone(),
    });

    self.label(start_label.clone());

    self.statement(&do_while.body, false);

    let mut expression_compiler = ExpressionCompiler { fnc: self };

    let condition = expression_compiler.compile(&do_while.test, None);

    self.label(continue_label);

    self.push(Instruction::JmpIf(
      condition.value.clone(),
      start_label.ref_(),
    ));

    self.release_ce(condition);

    self.label(end_label);

    self.loop_labels.pop();
  }

  fn for_(&mut self, for_: &swc_ecma_ast::ForStmt) {
    match &for_.init {
      Some(var_decl_or_expr) => match var_decl_or_expr {
        swc_ecma_ast::VarDeclOrExpr::VarDecl(var_decl) => {
          self.var_declaration(var_decl);
        }
        swc_ecma_ast::VarDeclOrExpr::Expr(expr) => {
          self.expression(expr);
        }
      },
      None => {}
    }

    let for_test_label = Label {
      name: self.label_allocator.allocate_numbered("for_test"),
    };

    let for_continue_label = Label {
      name: self.label_allocator.allocate_numbered("for_continue"),
    };

    let for_end_label = Label {
      name: self.label_allocator.allocate_numbered("for_end"),
    };

    self.label(for_test_label.clone());

    self.loop_labels.push(LoopLabels {
      continue_: Some(for_continue_label.clone()),
      break_: for_end_label.clone(),
    });

    match &for_.test {
      Some(cond) => {
        let mut ec = ExpressionCompiler { fnc: self };

        let cond_reg = ec.fnc.allocate_numbered_reg("_cond");
        ec.compile_into(cond, cond_reg.clone());

        self.push(Instruction::JmpIfNot(
          Value::Register(cond_reg.clone()),
          for_end_label.ref_(),
        ));

        self.release_reg(&cond_reg);
      }
      None => {}
    }

    self.statement(&for_.body, false);

    self.label(for_continue_label);

    match &for_.update {
      Some(update) => self.expression(update),
      None => {}
    }

    self.push(Instruction::Jmp(for_test_label.ref_()));

    self.label(for_end_label);

    self.loop_labels.pop();
  }

  fn for_of(&mut self, for_of: &swc_ecma_ast::ForOfStmt) {
    let mut ec = ExpressionCompiler { fnc: self };

    let pat = match &for_of.left {
      swc_ecma_ast::VarDeclOrPat::VarDecl(var_decl) => {
        if var_decl.decls.len() != 1 {
          panic!("Unexpected number of declarations on left side of for-of loop");
        }

        &var_decl.decls[0].name
      }
      swc_ecma_ast::VarDeclOrPat::Pat(pat) => pat,
    };

    let value_reg = ec.fnc.get_pattern_register(pat);

    let iter_reg = ec.fnc.allocate_numbered_reg("_iter");
    let iter_res_reg = ec.fnc.allocate_numbered_reg("_iter_res");
    let done_reg = ec.fnc.allocate_numbered_reg("_done");

    ec.compile_into(&for_of.right, iter_reg.clone());

    ec.fnc.push(Instruction::ConstSubCall(
      Value::Register(iter_reg.clone()),
      Value::Builtin(Builtin {
        name: "SymbolIterator".to_string(),
      }),
      Value::Array(Box::default()),
      iter_reg.clone(),
    ));

    let for_test_label = Label {
      name: ec.fnc.label_allocator.allocate_numbered("for_test"),
    };

    let for_continue_label = Label {
      name: ec.fnc.label_allocator.allocate_numbered("for_continue"),
    };

    let for_end_label = Label {
      name: ec.fnc.label_allocator.allocate_numbered("for_end"),
    };

    ec.fnc.push(Instruction::Jmp(for_continue_label.ref_()));

    ec.fnc.label(for_test_label.clone());

    ec.fnc.loop_labels.push(LoopLabels {
      continue_: Some(for_continue_label.clone()),
      break_: for_end_label.clone(),
    });

    ec.fnc.push(Instruction::JmpIf(
      Value::Register(done_reg.clone()),
      for_end_label.ref_(),
    ));

    ec.pat(pat, &value_reg, true);

    self.statement(&for_of.body, false);

    self.label(for_continue_label);
    self.push(Instruction::Next(iter_reg, iter_res_reg.clone()));

    self.push(Instruction::UnpackIterRes(
      iter_res_reg.clone(),
      value_reg,
      done_reg,
    ));

    self.release_reg(&iter_res_reg);

    self.push(Instruction::Jmp(for_test_label.ref_()));

    self.label(for_end_label);

    self.loop_labels.pop();
  }

  fn declaration(&mut self, decl: &swc_ecma_ast::Decl) {
    use swc_ecma_ast::Decl::*;

    match decl {
      Class(class) => self.todo(class.span(), "Class declaration"),
      Fn(fn_decl) => {
        self
          .queue
          .add(QueuedFunction {
            definition_pointer: match self.lookup_value(&Ident::from_swc_ident(&fn_decl.ident)) {
              Some(Value::Pointer(p)) => p,
              _ => {
                self.internal_error(
                  fn_decl.ident.span,
                  &format!(
                    "Lookup of function {} was not a pointer, lookup_result: {:?}",
                    fn_decl.ident.sym,
                    self.lookup_value(&Ident::from_swc_ident(&fn_decl.ident))
                  ),
                );

                return;
              }
            },
            fn_name: Some(fn_decl.ident.sym.to_string()),
            functionish: Functionish::Fn(Some(fn_decl.ident.clone()), fn_decl.function.clone()),
          })
          .expect("Failed to add function to queue");
      }
      Var(var_decl) => self.var_declaration(var_decl),
      TsInterface(interface_decl) => self.todo(interface_decl.span, "TsInterface declaration"),
      TsTypeAlias(_) => {}
      TsEnum(ts_enum) => {
        let pointer = match self
          .mc
          .scope_analysis
          .lookup_value(&OwnerId::Module, &Ident::from_swc_ident(&ts_enum.id))
        {
          Some(Value::Pointer(p)) => p,
          _ => {
            self.internal_error(
              ts_enum.id.span,
              &format!("Pointer for {} should have been in scope", ts_enum.id.sym),
            );

            return;
          }
        };

        let enum_value = self.mc.compile_enum_value(ts_enum);

        self.mc.module.definitions.push(Definition {
          pointer,
          content: DefinitionContent::Value(enum_value),
        });
      }
      TsModule(ts_module) => self.todo(ts_module.span, "TsModule declaration"),
    };
  }

  fn var_declaration(&mut self, var_decl: &swc_ecma_ast::VarDecl) {
    for decl in &var_decl.decls {
      match &decl.init {
        Some(expr) => {
          let target_register = self.get_pattern_register(&decl.name);

          let mut ec = ExpressionCompiler { fnc: self };
          ec.compile_into(expr, target_register.clone());
          ec.pat(&decl.name, &target_register, false);
        }
        None => match &decl.name {
          swc_ecma_ast::Pat::Ident(_) => {
            // Nothing to do - identifier without initializer should be
            // undefined
          }
          _ => {
            self.internal_error(
              decl.span(),
              "Expected destructuring declaration without initializer to be caught in the parser. \
                Pattern has not been compiled.",
            );
          }
        },
      }
    }
  }

  fn expression(&mut self, expr: &swc_ecma_ast::Expr) {
    let mut expression_compiler = ExpressionCompiler { fnc: self };
    let compiled = expression_compiler.compile_top_level(expr, None);

    self.release_ce(compiled);
  }

  pub fn release_ce(&mut self, mut compiled_expr: CompiledExpression) {
    for reg in &compiled_expr.nested_registers {
      self.release_reg(reg);
    }

    compiled_expr.release_checker.has_unreleased_registers = false;
  }

  fn get_mutated_registers(&self, span: swc_common::Span) -> BTreeSet<Register> {
    let start = swc_common::Span {
      lo: span.lo,
      hi: span.lo,
      ctxt: span.ctxt,
    };

    let end = swc_common::Span {
      lo: span.hi,
      hi: span.hi,
      ctxt: span.ctxt,
    };

    let mut mutated_registers = BTreeSet::<Register>::new();

    for (_span, mutated_name_id) in self.mc.scope_analysis.mutations.range(start..end) {
      if let Some(Value::Register(reg)) = self.lookup_by_name_id(mutated_name_id) {
        mutated_registers.insert(reg);
      }
    }

    // TODO: Avoid doing this. This is a workaround to include mutations of variables that are
    // supposed to be const, because we don't yet protect these variables from mutation that occurs
    // via method calls. Once that is implemented, this shouldn't be needed.
    for (_span, mutated_name_id) in self.mc.scope_analysis.optional_mutations.range(start..end) {
      if let Some(Value::Register(reg)) = self.lookup_by_name_id(mutated_name_id) {
        mutated_registers.insert(reg);
      }
    }

    mutated_registers
  }
}

fn instruction_needs_mutable_this(
  // We don't really mutate `instruction`, but we're using visit_registers_mut_rev which doesn't
  // have a non-mut equivalent. Writing it just for this seems unnecessary.
  instruction: &mut Instruction,
) -> bool {
  if let Instruction::ThisSubCall(_this, _, _, dst) = instruction {
    // visit_registers_mut_rev flags `this` as write:true since a write can occur, but the whole
    // purpose of this instruction is to conditionally propagate constness into the next call to
    // deal with this issue. Therefore, we only check `dst` here.

    return dst.is_this();
  }

  let mut result = false;

  instruction.visit_registers_mut_rev(&mut |rvm| {
    if rvm.write && rvm.register.is_this() {
      result = true;
    }
  });

  result
}
