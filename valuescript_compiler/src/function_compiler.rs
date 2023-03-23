use queues::*;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use swc_common::Spanned;

use crate::asm::{
  Definition, DefinitionContent, Function, Instruction, InstructionOrLabel, Label, Pointer,
  Register, Value,
};
use crate::diagnostic::{Diagnostic, DiagnosticLevel};
use crate::expression_compiler::CompiledExpression;
use crate::expression_compiler::ExpressionCompiler;
use crate::name_allocator::{NameAllocator, RegAllocator};
use crate::scope_analysis::{NameId, OwnerId, ScopeAnalysis};

#[derive(Clone, Debug)]
pub enum Functionish {
  Fn(swc_ecma_ast::Function),
  Arrow(swc_ecma_ast::ArrowExpr),
  Constructor(
    Vec<InstructionOrLabel>,
    swc_common::Span,
    swc_ecma_ast::Constructor,
  ),
}

impl Spanned for Functionish {
  fn span(&self) -> swc_common::Span {
    match self {
      Functionish::Fn(fn_) => fn_.span,
      Functionish::Arrow(arrow) => arrow.span,
      Functionish::Constructor(_, class_span, _) => *class_span,
    }
  }
}

impl Functionish {
  pub fn owner_id(&self) -> OwnerId {
    OwnerId::Span(self.span().clone())
  }
}

#[derive(Clone, Debug)]
pub struct QueuedFunction {
  pub definition_pointer: Pointer,
  pub fn_name: Option<String>,
  pub functionish: Functionish,
}

pub struct LoopLabels {
  pub continue_: Label,
  pub break_: Label,
}

pub struct CatchSetting {
  pub label: Label,
  pub reg: Register,
}

pub struct FunctionCompiler {
  pub current: Function,
  pub definitions: Vec<Definition>,
  pub owner_id: OwnerId,
  pub scope_analysis: Rc<ScopeAnalysis>,
  pub definition_allocator: Rc<RefCell<NameAllocator>>,
  pub reg_allocator: RegAllocator,
  pub label_allocator: NameAllocator,
  pub queue: Queue<QueuedFunction>,
  pub loop_labels: Vec<LoopLabels>,
  pub catch_settings: Vec<CatchSetting>,
  pub end_label: Option<Label>,
  pub is_returning_register: Option<Register>,
  pub finally_labels: Vec<Label>,

  pub diagnostics: Vec<Diagnostic>,
}

impl FunctionCompiler {
  pub fn new(
    scope_analysis: &Rc<ScopeAnalysis>,
    owner_id: OwnerId,
    definition_allocator: Rc<RefCell<NameAllocator>>,
  ) -> FunctionCompiler {
    let reg_allocator = match scope_analysis.reg_allocators.get(&owner_id) {
      Some(reg_allocator) => reg_allocator.clone(),
      None => RegAllocator::default(),
    };

    return FunctionCompiler {
      current: Function::default(),
      definitions: vec![],
      owner_id,
      scope_analysis: scope_analysis.clone(),
      definition_allocator,
      reg_allocator,
      label_allocator: NameAllocator::default(),
      queue: Queue::new(),
      loop_labels: vec![],
      catch_settings: vec![],
      end_label: None,
      is_returning_register: None,
      finally_labels: vec![],
      diagnostics: vec![],
    };
  }

  pub fn push(&mut self, instruction: Instruction) {
    self
      .current
      .body
      .push(InstructionOrLabel::Instruction(instruction));
  }

  pub fn label(&mut self, label: Label) {
    self.current.body.push(InstructionOrLabel::Label(label));
  }

  pub fn lookup(&self, ident: &swc_ecma_ast::Ident) -> Option<Value> {
    self.scope_analysis.lookup(&self.owner_id, ident)
  }

  pub fn lookup_name_id(&self, name_id: &NameId) -> Option<Value> {
    self.scope_analysis.lookup_name_id(&self.owner_id, name_id)
  }

  pub fn todo(&mut self, span: swc_common::Span, message: &str) {
    self.diagnostics.push(Diagnostic {
      level: DiagnosticLevel::InternalError,
      message: format!("TODO: {}", message),
      span,
    });
  }

  pub fn allocate_defn(&mut self, name: &str) -> Pointer {
    let allocated_name = self
      .definition_allocator
      .borrow_mut()
      .allocate(&name.to_string());

    Pointer {
      name: allocated_name,
    }
  }

  pub fn allocate_defn_numbered(&mut self, name: &str) -> Pointer {
    let allocated_name = self
      .definition_allocator
      .borrow_mut()
      .allocate_numbered(&name.to_string());

    Pointer {
      name: allocated_name,
    }
  }

  pub fn allocate_tmp(&mut self) -> Register {
    self.reg_allocator.allocate_numbered("_tmp")
  }

  pub fn allocate_reg(&mut self, based_on: &String) -> Register {
    self.reg_allocator.allocate(based_on)
  }

  pub fn allocate_reg_fresh(&mut self, based_on: &String) -> Register {
    self.reg_allocator.allocate_fresh(based_on)
  }

  pub fn allocate_numbered_reg(&mut self, prefix: &str) -> Register {
    self.reg_allocator.allocate_numbered(prefix)
  }

  pub fn allocate_numbered_reg_fresh(&mut self, prefix: &str) -> Register {
    self
      .reg_allocator
      .allocate_numbered_fresh(&prefix.to_string())
  }

  pub fn release_reg(&mut self, reg: &Register) {
    self.reg_allocator.release(reg);
  }

  pub fn compile(
    definition_pointer: Pointer,
    fn_name: Option<String>,
    functionish: Functionish,
    scope_analysis: &Rc<ScopeAnalysis>,
    definition_allocator: Rc<RefCell<NameAllocator>>,
  ) -> (Vec<Definition>, Vec<Diagnostic>) {
    let mut self_ =
      FunctionCompiler::new(scope_analysis, functionish.owner_id(), definition_allocator);

    self_
      .queue
      .add(QueuedFunction {
        definition_pointer: definition_pointer.clone(),
        fn_name,
        functionish,
      })
      .expect("Failed to queue function");

    self_.process_queue();

    return (self_.definitions, self_.diagnostics);
  }

  pub fn process_queue(&mut self) {
    loop {
      match self.queue.remove() {
        Ok(qfn) => self.compile_functionish(qfn.definition_pointer, &qfn.functionish),
        Err(_) => {
          break;
        }
      }
    }
  }

  fn compile_functionish(&mut self, definition_pointer: Pointer, functionish: &Functionish) {
    // TODO: Use a new FunctionCompiler per function instead of this hack
    self.reg_allocator = match self
      .scope_analysis
      .reg_allocators
      .get(&functionish.owner_id())
    {
      Some(reg_allocator) => reg_allocator.clone(),
      None => RegAllocator::default(),
    };

    // TODO: Transitive captures
    let capture_params = self.scope_analysis.captures.get(&functionish.owner_id());

    for cap_param in capture_params.unwrap_or(&HashSet::new()) {
      let reg = match self
        .scope_analysis
        .lookup_capture(&self.owner_id, cap_param)
      {
        Some(Value::Register(reg)) => reg,
        _ => {
          self.diagnostics.push(Diagnostic {
            level: DiagnosticLevel::InternalError,
            message: format!("Expected capture to be a register"),
            span: functionish.span(),
          });

          self.reg_allocator.allocate_numbered("_error_cap_register")
        }
      };

      self.current.parameters.push(reg.clone());
    }

    let param_registers = self.get_param_registers(functionish);

    for reg in &param_registers {
      self.current.parameters.push(reg.clone());
    }

    self.add_param_code(functionish, &param_registers);

    match functionish {
      Functionish::Fn(fn_) => {
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

          expression_compiler.compile(expr, Some(Register::Return));
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

    if let Some(end_label) = self.end_label.as_ref() {
      self
        .current
        .body
        .push(InstructionOrLabel::Label(end_label.clone()));

      self.end_label = None;
      self.is_returning_register = None;
    }

    self.definitions.push(Definition {
      pointer: definition_pointer,
      content: DefinitionContent::Function(std::mem::take(&mut self.current)),
    });
  }

  fn handle_block_body(&mut self, block: &swc_ecma_ast::BlockStmt) {
    for i in 0..block.stmts.len() {
      self.statement(&block.stmts[i], i == block.stmts.len() - 1);
    }
  }

  fn get_param_registers(&mut self, functionish: &Functionish) -> Vec<Register> {
    let mut param_registers = Vec::<Register>::new();

    match functionish {
      Functionish::Fn(fn_) => {
        for p in &fn_.params {
          param_registers.push(self.get_pattern_register(&p.pat));
        }
      }
      Functionish::Arrow(arrow) => {
        for p in &arrow.params {
          param_registers.push(self.get_pattern_register(p));
        }
      }
      Functionish::Constructor(_, _class_span, constructor) => {
        for potspp in &constructor.params {
          match potspp {
            swc_ecma_ast::ParamOrTsParamProp::TsParamProp(ts_param_prop) => {
              self.todo(ts_param_prop.span(), "TypeScript parameter properties");
              param_registers.push(self.allocate_numbered_reg(&"_todo_ts_param_prop".to_string()));
            }
            swc_ecma_ast::ParamOrTsParamProp::Param(p) => {
              param_registers.push(self.get_pattern_register(&p.pat))
            }
          }
        }
      }
    };

    return param_registers;
  }

  pub fn get_pattern_register(&mut self, param_pat: &swc_ecma_ast::Pat) -> Register {
    use swc_ecma_ast::Pat;

    match param_pat {
      Pat::Ident(ident) => self.get_variable_register(&ident.id),
      Pat::Assign(assign) => self.get_pattern_register(&assign.left),
      Pat::Array(_) => self.allocate_numbered_reg(&"_array_pat".to_string()),
      Pat::Object(_) => self.allocate_numbered_reg(&"_object_pat".to_string()),
      Pat::Invalid(_) => self.allocate_numbered_reg(&"_invalid_pat".to_string()),
      Pat::Rest(_) => self.allocate_numbered_reg(&"_rest_pat".to_string()),
      Pat::Expr(_) => self.allocate_numbered_reg(&"_expr_pat".to_string()),
    }
  }

  pub fn get_variable_register(&mut self, ident: &swc_ecma_ast::Ident) -> Register {
    match self.scope_analysis.lookup(&self.owner_id, ident) {
      Some(Value::Register(reg)) => reg,
      _ => {
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::InternalError,
          message: format!(
            "Register should have been allocated for variable {}",
            ident.sym.to_string()
          ),
          span: ident.span(),
        });

        self.allocate_numbered_reg(&"_error_variable_without_register".to_string())
      }
    }
  }

  fn add_param_code(&mut self, functionish: &Functionish, param_registers: &Vec<Register>) {
    match functionish {
      Functionish::Fn(fn_) => {
        for (i, p) in fn_.params.iter().enumerate() {
          let mut ec = ExpressionCompiler { fnc: self };
          ec.pat(&p.pat, &param_registers[i], false);
        }
      }
      Functionish::Arrow(arrow) => {
        for (i, p) in arrow.params.iter().enumerate() {
          let mut ec = ExpressionCompiler { fnc: self };
          ec.pat(p, &param_registers[i], false);
        }
      }
      Functionish::Constructor(_, _class_span, constructor) => {
        for (i, potspp) in constructor.params.iter().enumerate() {
          match potspp {
            swc_ecma_ast::ParamOrTsParamProp::TsParamProp(_) => {
              // TODO (Diagnostic emitted elsewhere)
            }
            swc_ecma_ast::ParamOrTsParamProp::Param(p) => {
              let mut ec = ExpressionCompiler { fnc: self };
              ec.pat(&p.pat, &param_registers[i], false);
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
        self.diagnostics.push(Diagnostic {
          level: DiagnosticLevel::Error,
          message: "Not supported: With statement".to_string(),
          span: with.span,
        });
      }

      Return(ret_stmt) => {
        match &ret_stmt.arg {
          None => {}
          Some(expr) => {
            let mut expression_compiler = ExpressionCompiler { fnc: self };

            expression_compiler.compile(expr, Some(Register::Return));
          }
        }

        if !fn_last {
          if let Some(finally_label) = self.finally_labels.last().cloned() {
            let is_returning = match self.is_returning_register.clone() {
              Some(is_returning) => is_returning.clone(),
              None => {
                let is_returning = self.allocate_reg_fresh(&"_is_returning".to_string());
                self.is_returning_register = Some(is_returning.clone());
                is_returning
              }
            };

            self.push(Instruction::Mov(Value::Bool(true), is_returning.clone()));
            self.push(Instruction::Jmp(finally_label.ref_()));
          } else {
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
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::Error,
              message: "break statement outside loop".to_string(),
              span: break_.span,
            });
          }
        }
      }
      Continue(continue_) => {
        if continue_.label.is_some() {
          self.todo(continue_.span, "labeled continue statement");

          return;
        }

        match self.loop_labels.last() {
          Some(loop_labels) => {
            self.push(Instruction::Jmp(loop_labels.continue_.ref_()));
          }
          None => {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::Error,
              message: "continue statement outside loop".to_string(),
              span: continue_.span,
            });
          }
        }
      }
      If(if_) => {
        self.if_(if_);
      }
      Switch(switch) => self.todo(switch.span, "Switch statement"),
      Throw(throw) => {
        let mut expression_compiler = ExpressionCompiler { fnc: self };

        let arg = expression_compiler.compile(&throw.arg, None);
        let instr = Instruction::Throw(self.use_(arg));

        self.push(instr);
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
    let mut expression_compiler = ExpressionCompiler { fnc: self };

    let condition = expression_compiler.compile(&*if_.test, None);

    // Usually we wouldn't capture the value_assembly before release, but
    // it's safe to do so here, and allows cond_reg to re-use a register
    // from the condition
    let condition_asm = self.use_(condition);

    let cond_reg = self.allocate_numbered_reg("_cond");

    // TODO: Add negated jmpif instruction to avoid this
    self.push(Instruction::OpNot(condition_asm, cond_reg.clone()));

    let else_label = Label {
      name: self.label_allocator.allocate_numbered(&"else".to_string()),
    };

    self.push(Instruction::JmpIf(
      Value::Register(cond_reg.clone()),
      else_label.ref_(),
    ));

    self.release_reg(&cond_reg);

    self.statement(&*if_.cons, false);

    match &if_.alt {
      None => {
        self.label(else_label);
      }
      Some(alt) => {
        let after_else_label = Label {
          name: self
            .label_allocator
            .allocate_numbered(&"after_else".to_string()),
        };

        self.push(Instruction::Jmp(after_else_label.ref_()));

        self.label(else_label);
        self.statement(&*alt, false);
        self.label(after_else_label);
      }
    }
  }

  fn try_(&mut self, try_: &swc_ecma_ast::TryStmt) {
    let (catch_label, after_catch_label) = match try_.handler {
      Some(_) => (
        Some(Label {
          name: self.label_allocator.allocate_numbered(&"catch".to_string()),
        }),
        Some(Label {
          name: self
            .label_allocator
            .allocate_numbered(&"after_catch".to_string()),
        }),
      ),
      None => (None, None),
    };

    let finally_label = match &try_.finalizer {
      Some(_) => Some(Label {
        name: self
          .label_allocator
          .allocate_numbered(&"finally".to_string()),
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
        None => Register::Ignore,
      };

      catch_error_reg = Some(reg.clone());

      self.catch_settings.push(CatchSetting {
        label: label.clone(),
        reg,
      });
    }

    self.apply_catch_setting();
    self.block_statement(&try_.block);
    self.pop_catch_setting(); // TODO: Avoid redundant set_catch to our own finally

    if let Some(label) = &after_catch_label {
      self.push(Instruction::Jmp(label.ref_()));
    }

    if let Some(catch_clause) = &try_.handler {
      self.label(catch_label.unwrap());
      self.apply_catch_setting(); // TODO: Avoid redundant unset_catch

      if let Some(param) = &catch_clause.param {
        let mut ec = ExpressionCompiler { fnc: self };

        let pattern_reg = ec.fnc.get_pattern_register(&param);

        // TODO: Set up this register through set_catch instead of copying into it
        ec.fnc.push(Instruction::Mov(
          Value::Register(catch_error_reg.unwrap()),
          pattern_reg.clone(),
        ));

        ec.pat(&param, &pattern_reg, false);
      }

      self.block_statement(&catch_clause.body);

      if let Some(_) = finally_label {
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

      self.block_statement(&finally_clause);

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
            Value::Register(local_is_returning.clone()),
            end_label.ref_(),
          ));
        } else {
          self.push(Instruction::OpNot(
            Value::Register(local_is_returning.clone()),
            local_is_returning.clone(),
          ));

          let after_finally_label = Label {
            name: self
              .label_allocator
              .allocate_numbered(&"after_finally".to_string()),
          };

          self.push(Instruction::JmpIf(
            Value::Register(local_is_returning.clone()),
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

  fn while_(self: &mut Self, while_: &swc_ecma_ast::WhileStmt) {
    let start_label = Label {
      name: self.label_allocator.allocate_numbered(&"while".to_string()),
    };

    let end_label = Label {
      name: self
        .label_allocator
        .allocate_numbered(&"while_end".to_string()),
    };

    self.loop_labels.push(LoopLabels {
      continue_: start_label.clone(),
      break_: end_label.clone(),
    });

    self.label(start_label.clone());

    let mut expression_compiler = ExpressionCompiler { fnc: self };

    let condition = expression_compiler.compile(&*while_.test, None);

    // Usually we wouldn't capture the value_assembly before release, but
    // it's safe to do so here, and allows cond_reg to re-use a register
    // from the condition
    let condition_asm = self.use_(condition);

    let cond_reg = self.allocate_numbered_reg(&"_cond".to_string());

    // TODO: Add negated jmpif instruction to avoid this
    self.push(Instruction::OpNot(condition_asm, cond_reg.clone()));

    self.push(Instruction::JmpIf(
      Value::Register(cond_reg.clone()),
      end_label.ref_(),
    ));

    self.release_reg(&cond_reg);
    self.statement(&*while_.body, false);
    self.push(Instruction::Jmp(start_label.ref_()));
    self.label(end_label);

    self.loop_labels.pop();
  }

  fn do_while(self: &mut Self, do_while: &swc_ecma_ast::DoWhileStmt) {
    let start_label = Label {
      name: self
        .label_allocator
        .allocate_numbered(&"do_while".to_string()),
    };

    let continue_label = Label {
      name: self
        .label_allocator
        .allocate_numbered(&"do_while_continue".to_string()),
    };

    let end_label = Label {
      name: self
        .label_allocator
        .allocate_numbered(&"do_while_end".to_string()),
    };

    self.loop_labels.push(LoopLabels {
      continue_: continue_label.clone(),
      break_: end_label.clone(),
    });

    self.label(start_label.clone());

    self.statement(&*do_while.body, false);

    let mut expression_compiler = ExpressionCompiler { fnc: self };

    let condition = expression_compiler.compile(&*do_while.test, None);

    self.label(continue_label);

    let jmpif = Instruction::JmpIf(self.use_(condition), start_label.ref_());
    self.push(jmpif);

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
      name: self
        .label_allocator
        .allocate_numbered(&"for_test".to_string()),
    };

    let for_continue_label = Label {
      name: self
        .label_allocator
        .allocate_numbered(&"for_continue".to_string()),
    };

    let for_end_label = Label {
      name: self
        .label_allocator
        .allocate_numbered(&"for_end".to_string()),
    };

    self.label(for_test_label.clone());

    self.loop_labels.push(LoopLabels {
      continue_: for_continue_label.clone(),
      break_: for_end_label.clone(),
    });

    match &for_.test {
      Some(cond) => {
        let mut ec = ExpressionCompiler { fnc: self };

        let condition = ec.compile(cond, None);

        // Usually we wouldn't capture the value_assembly before release, but
        // it's safe to do so here, and allows cond_reg to re-use a register
        // from the condition
        let condition_asm = self.use_(condition);

        let cond_reg = self.allocate_numbered_reg("_cond");

        // TODO: Add negated jmpif instruction to avoid this
        self.push(Instruction::OpNot(condition_asm, cond_reg.clone()));

        self.push(Instruction::JmpIf(
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
    let index_reg = self.allocate_numbered_reg(&"_for_of_i".to_string());
    self.push(Instruction::Mov(Value::Number(0.0), index_reg.clone()));

    let array_reg = self.allocate_numbered_reg(&"_for_of_array".to_string());

    let mut ec = ExpressionCompiler { fnc: self };

    ec.compile(&for_of.right, Some(array_reg.clone()));

    let len_reg = ec.fnc.allocate_numbered_reg(&"_for_of_len".to_string());

    ec.fnc.push(Instruction::Sub(
      Value::Register(array_reg.clone()),
      Value::String("length".to_string()),
      len_reg.clone(),
    ));

    let for_test_label = Label {
      name: ec
        .fnc
        .label_allocator
        .allocate_numbered(&"for_test".to_string()),
    };

    let for_continue_label = Label {
      name: ec
        .fnc
        .label_allocator
        .allocate_numbered(&"for_continue".to_string()),
    };

    let for_end_label = Label {
      name: ec
        .fnc
        .label_allocator
        .allocate_numbered(&"for_end".to_string()),
    };

    ec.fnc.label(for_test_label.clone());

    ec.fnc.loop_labels.push(LoopLabels {
      continue_: for_continue_label.clone(),
      break_: for_end_label.clone(),
    });

    let cond_reg = ec.fnc.allocate_numbered_reg(&"_for_of_cond".to_string());

    ec.fnc.push(Instruction::OpTripleEq(
      Value::Register(index_reg.clone()),
      Value::Register(len_reg.clone()),
      cond_reg.clone(),
    ));

    ec.fnc.push(Instruction::JmpIf(
      Value::Register(cond_reg.clone()),
      for_end_label.ref_(),
    ));

    let pat = match &for_of.left {
      swc_ecma_ast::VarDeclOrPat::VarDecl(var_decl) => {
        if var_decl.decls.len() != 1 {
          panic!("Unexpected number of declarations on left side of for-of loop");
        }

        &var_decl.decls[0].name
      }
      swc_ecma_ast::VarDeclOrPat::Pat(pat) => pat,
    };

    let element_reg = ec.fnc.get_pattern_register(pat);

    ec.fnc.push(Instruction::Sub(
      Value::Register(array_reg.clone()),
      Value::Register(index_reg.clone()),
      element_reg.clone(),
    ));

    ec.pat(pat, &element_reg, true);

    self.statement(&for_of.body, false);

    self.label(for_continue_label);

    self.push(Instruction::OpInc(index_reg.clone()));
    self.push(Instruction::Jmp(for_test_label.ref_()));

    self.label(for_end_label);

    self.loop_labels.pop();
  }

  fn declaration(&mut self, decl: &swc_ecma_ast::Decl) {
    use swc_ecma_ast::Decl::*;

    match decl {
      Class(class) => self.todo(class.span(), "Class declaration"),
      Fn(_) => {}
      Var(var_decl) => self.var_declaration(var_decl),
      TsInterface(interface_decl) => self.todo(interface_decl.span, "TsInterface declaration"),
      TsTypeAlias(_) => {}
      TsEnum(ts_enum) => self.todo(ts_enum.span, "TsEnum declaration"),
      TsModule(ts_module) => self.todo(ts_module.span, "TsModule declaration"),
    };
  }

  fn var_declaration(&mut self, var_decl: &swc_ecma_ast::VarDecl) {
    for decl in &var_decl.decls {
      match &decl.init {
        Some(expr) => {
          let target_register = self.get_pattern_register(&decl.name);

          let mut ec = ExpressionCompiler { fnc: self };
          ec.compile(expr, Some(target_register.clone()));
          ec.pat(&decl.name, &target_register, false);
        }
        None => match &decl.name {
          swc_ecma_ast::Pat::Ident(_) => {
            // Nothing to do - identifier without initializer should be
            // undefined
          }
          _ => {
            self.diagnostics.push(Diagnostic {
              level: DiagnosticLevel::InternalError,
              message: "Expected destructuring declaration without initializer \
                to be caught in the parser. Pattern has not been compiled."
                .to_string(),
              span: decl.span(),
            });
          }
        },
      }
    }
  }

  fn expression(&mut self, expr: &swc_ecma_ast::Expr) {
    let mut expression_compiler = ExpressionCompiler { fnc: self };
    let compiled = expression_compiler.compile_top_level(expr, None);

    self.use_(compiled);
  }

  pub fn use_(&mut self, mut compiled_expr: CompiledExpression) -> Value {
    let asm = compiled_expr.value;

    for reg in &compiled_expr.nested_registers {
      self.release_reg(reg);
    }

    compiled_expr.release_checker.has_unreleased_registers = false;

    return asm;
  }

  pub fn use_ref(&mut self, compiled_expr: &mut CompiledExpression) -> Value {
    let asm = compiled_expr.value.clone();

    for reg in &compiled_expr.nested_registers {
      self.release_reg(reg);
    }

    compiled_expr.release_checker.has_unreleased_registers = false;

    return asm;
  }
}
