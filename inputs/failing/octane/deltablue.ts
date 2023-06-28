// //! bench()

// Work in progress. Challenges:
// - Lots of shared reference mutation
// - Use of inheritance and constants requiring compile-time evaluation

// Copyright 2008 the V8 project authors. All rights reserved.
// Copyright 1996 John Maloney and Mario Wolczko.

// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 2 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program; if not, write to the Free Software
// Foundation, Inc., 59 Temple Place, Suite 330, Boston, MA  02111-1307  USA

// This implementation of the DeltaBlue benchmark is derived
// from the Smalltalk implementation by John Maloney and Mario
// Wolczko. Some parts have been translated directly, whereas
// others have been modified more aggresively to make it feel
// more like a JavaScript program.

/**
 * A JavaScript implementation of the DeltaBlue constraint-solving
 * algorithm, as described in:
 *
 * "The DeltaBlue Algorithm: An Incremental Constraint Hierarchy Solver"
 *   Bjorn N. Freeman-Benson and John Maloney
 *   January 1990 Communications of the ACM,
 *   also available as University of Washington TR 89-08-06.
 *
 * Beware: this benchmark is written in a grotesque style where
 * the constraint model is built by side-effects from constructors.
 * I've kept it this way to avoid deviating too much from the original
 * implementation.
 */

/* --- O b j e c t   M o d e l --- */

class OrderedCollection<T> {
  elms: T[] = [];

  add(elm: T) {
    this.elms.push(elm);
  }

  at(index: number) {
    return this.elms[index];
  }

  size() {
    return this.elms.length;
  }

  removeFirst() {
    return this.elms.pop();
  }

  remove(elm: T) {
    let index = 0, skipped = 0;
    for (let i = 0; i < this.elms.length; i++) {
      let value = this.elms[i];
      if (value != elm) {
        this.elms[index] = value;
        index++;
      } else {
        skipped++;
      }
    }
    for (let i = 0; i < skipped; i++) {
      this.elms.pop();
    }
  }
}

/* --- *
 * S t r e n g t h
 * --- */

type StrengthValue = 0 | 1 | 2 | 3 | 4 | 5 | 6;

/**
 * Strengths are used to measure the relative importance of constraints.
 * New strengths may be inserted in the strength hierarchy without
 * disrupting current constraints.  Strengths cannot be created outside
 * this class, so pointer comparison can be used for value comparison.
 */

class Strength {
  strengthValue;
  name;

  constructor(strengthValue: StrengthValue, name: string) {
    this.strengthValue = strengthValue;
    this.name = name;
  }

  static stronger(s1: Strength, s2: Strength) {
    return s1.strengthValue < s2.strengthValue;
  }

  static weaker(s1: Strength, s2: Strength) {
    return s1.strengthValue > s2.strengthValue;
  }

  static weakestOf(s1: Strength, s2: Strength) {
    return this.weaker(s1, s2) ? s1 : s2;
  }

  static strongest(s1: Strength, s2: Strength) {
    return this.stronger(s1, s2) ? s1 : s2;
  }

  nextWeaker() {
    switch (this.strengthValue) {
      case 0:
        return Strength.WEAKEST;
      case 1:
        return Strength.WEAK_DEFAULT;
      case 2:
        return Strength.NORMAL;
      case 3:
        return Strength.STRONG_DEFAULT;
      case 4:
        return Strength.PREFERRED;
      case 5:
        return Strength.REQUIRED;
    }
  }

  static REQUIRED = new Strength(0, "required");
  static STONG_PREFERRED = new Strength(1, "strongPreferred");
  static PREFERRED = new Strength(2, "preferred");
  static STRONG_DEFAULT = new Strength(3, "strongDefault");
  static NORMAL = new Strength(4, "normal");
  static WEAK_DEFAULT = new Strength(5, "weakDefault");
  static WEAKEST = new Strength(6, "weakest");
}

/* --- *
 * C o n s t r a i n t
 * --- */

/**
 * An abstract class representing a system-maintainable relationship
 * (or "constraint") between a set of variables. A constraint supplies
 * a strength instance variable; concrete subclasses provide a means
 * of storing the constrained variables and other information required
 * to represent a constraint.
 */
abstract class Constraint {
  strength;

  constructor(strength: Strength) {
    this.strength = strength;
  }

  abstract addToGraph(): void;
  abstract removeFromGraph(): void;
  abstract chooseMethod(mark: number): void;
  abstract markInputs(mark: number): void;
  abstract output(): Variable;
  abstract isSatisfied(): boolean;
  abstract execute(): void;
  abstract markUnsatisfied(): void;
  abstract recalculate(): void;
  abstract inputsKnown(mark: number): boolean;

  /**
   * Attempt to find a way to enforce this constraint. If successful,
   * record the solution, perhaps modifying the current dataflow
   * graph. Answer the constraint that this constraint overrides, if
   * there is one, or nil, if there isn't.
   * Assume: I am not already satisfied.
   */
  satisfy(planner: Planner, mark: number) {
    this.chooseMethod(mark);
    if (!this.isSatisfied()) {
      if (this.strength == Strength.REQUIRED) {
        throw new Error("Could not satisfy a required constraint!");
      }
      return null;
    }
    this.markInputs(mark);
    let out = this.output();
    let overridden = out.determinedBy;
    if (overridden != null) overridden.markUnsatisfied();
    out.determinedBy = this;
    if (!planner.addPropagate(this, mark)) {
      throw new Error("Cycle encountered");
    }
    out.mark = mark;
    return overridden;
  }

  destroyConstraint(planner: Planner) {
    if (this.isSatisfied()) planner.incrementalRemove(this);
    else this.removeFromGraph();
  }

  /**
   * Normal constraints are not input constraints.  An input constraint
   * is one that depends on external state, such as the mouse, the
   * keybord, a clock, or some arbitraty piece of imperative code.
   */
  isInput() {
    return false;
  }
}

/* --- *
 * U n a r y   C o n s t r a i n t
 * --- */

/**
 * Abstract superclass for constraints having a single possible output
 * variable.
 */
abstract class UnaryConstraint extends Constraint {
  myOutput: Variable;
  satisfied;

  constructor(v: Variable, strength: Strength) {
    super(strength);
    this.myOutput = v;
    this.satisfied = false;
  }

  /**
   * Adds this constraint to the constraint graph
   */
  addToGraph() {
    this.myOutput.addConstraint(this);
    this.satisfied = false;
  }

  /**
   * Decides if this constraint can be satisfied and records that
   * decision.
   */
  chooseMethod(mark: number) {
    this.satisfied = (this.myOutput.mark != mark) &&
      Strength.stronger(this.strength, this.myOutput.walkStrength);
  }

  /**
   * Returns true if this constraint is satisfied in the current solution.
   */
  isSatisfied() {
    return this.satisfied;
  }

  markInputs(_mark: number) {
    // has no inputs
  }

  /**
   * Returns the current output variable.
   */
  output() {
    return this.myOutput;
  }

  /**
   * Calculate the walkabout strength, the stay flag, and, if it is
   * 'stay', the value for the current output of this constraint. Assume
   * this constraint is satisfied.
   */
  recalculate() {
    this.myOutput.walkStrength = this.strength;
    this.myOutput.stay = !this.isInput();
    if (this.myOutput.stay) this.execute(); // Stay optimization
  }

  /**
   * Records that this constraint is unsatisfied
   */
  markUnsatisfied() {
    this.satisfied = false;
  }

  inputsKnown() {
    return true;
  }

  removeFromGraph() {
    if (this.myOutput != null) this.myOutput.removeConstraint(this);
    this.satisfied = false;
  }
}

/* --- *
 * S t a y   C o n s t r a i n t
 * --- */

/**
 * Variables that should, with some level of preference, stay the same.
 * Planners may exploit the fact that instances, if satisfied, will not
 * change their output during plan execution.  This is called "stay
 * optimization".
 */
class StayConstraint extends UnaryConstraint {
  constructor(v: Variable, str: Strength) {
    super(v, str);
  }

  execute() {
    // Stay constraints do nothing
  }
}

/* --- *
 * E d i t   C o n s t r a i n t
 * --- */

/**
 * A unary input constraint used to mark a variable that the client
 * wishes to change.
 */
class EditConstraint extends UnaryConstraint {
  constructor(v: Variable, str: Strength) {
    super(v, str);
  }

  /**
   * Edits indicate that a variable is to be changed by imperative code.
   */
  isInput() {
    return true;
  }

  execute() {
    // Edit constraints do nothing
  }
}

/* --- *
 * B i n a r y   C o n s t r a i n t
 * --- */

const Direction = {
  NONE: 0,
  FORWARD: 1,
  BACKWARD: -1,
};

/**
 * Abstract superclass for constraints having two possible output
 * variables.
 */
abstract class BinaryConstraint extends Constraint {
  v1;
  v2;
  direction;

  constructor(var1: Variable, var2: Variable, strength: Strength) {
    super(strength);

    this.v1 = var1;
    this.v2 = var2;
    this.direction = Direction.NONE;
  }

  /**
   * Decides if this constraint can be satisfied and which way it
   * should flow based on the relative strength of the variables related,
   * and record that decision.
   */
  chooseMethod(mark: number) {
    if (this.v1.mark == mark) {
      this.direction = (this.v2.mark != mark &&
          Strength.stronger(this.strength, this.v2.walkStrength))
        ? Direction.FORWARD
        : Direction.NONE;
    }
    if (this.v2.mark == mark) {
      this.direction = (this.v1.mark != mark &&
          Strength.stronger(this.strength, this.v1.walkStrength))
        ? Direction.BACKWARD
        : Direction.NONE;
    }
    if (Strength.weaker(this.v1.walkStrength, this.v2.walkStrength)) {
      this.direction = Strength.stronger(this.strength, this.v1.walkStrength)
        ? Direction.BACKWARD
        : Direction.NONE;
    } else {
      this.direction = Strength.stronger(this.strength, this.v2.walkStrength)
        ? Direction.FORWARD
        : Direction.BACKWARD;
    }
  }

  /**
   * Add this constraint to the constraint graph
   */
  addToGraph() {
    this.v1.addConstraint(this);
    this.v2.addConstraint(this);
    this.direction = Direction.NONE;
  }

  /**
   * Answer true if this constraint is satisfied in the current solution.
   */
  isSatisfied() {
    return this.direction != Direction.NONE;
  }

  /**
   * Mark the input variable with the given mark.
   */
  markInputs(mark: number) {
    this.input().mark = mark;
  }

  /**
   * Returns the current input variable
   */
  input() {
    return (this.direction == Direction.FORWARD) ? this.v1 : this.v2;
  }

  /**
   * Returns the current output variable
   */
  output() {
    return (this.direction == Direction.FORWARD) ? this.v2 : this.v1;
  }

  /**
   * Calculate the walkabout strength, the stay flag, and, if it is
   * 'stay', the value for the current output of this
   * constraint. Assume this constraint is satisfied.
   */
  recalculate() {
    let ihn = this.input(), out = this.output();
    out.walkStrength = Strength.weakestOf(this.strength, ihn.walkStrength);
    out.stay = ihn.stay;
    if (out.stay) this.execute();
  }

  /**
   * Record the fact that this constraint is unsatisfied.
   */
  markUnsatisfied() {
    this.direction = Direction.NONE;
  }

  inputsKnown(mark: number) {
    let i = this.input();
    return i.mark == mark || i.stay || i.determinedBy == null;
  }

  removeFromGraph() {
    if (this.v1 != null) this.v1.removeConstraint(this);
    if (this.v2 != null) this.v2.removeConstraint(this);
    this.direction = Direction.NONE;
  }
}

/* --- *
 * S c a l e   C o n s t r a i n t
 * --- */

/**
 * Relates two variables by the linear scaling relationship: "v2 =
 * (v1 * scale) + offset". Either v1 or v2 may be changed to maintain
 * this relationship but the scale factor and offset are considered
 * read-only.
 */
class ScaleConstraint extends BinaryConstraint {
  direction;
  scale;
  offset;

  constructor(
    src: Variable,
    scale: Variable,
    offset: Variable,
    dest: Variable,
    strength: Strength,
  ) {
    super(src, dest, strength);
    this.direction = Direction.NONE;
    this.scale = scale;
    this.offset = offset;
  }

  /**
   * Adds this constraint to the constraint graph.
   */
  addToGraph() {
    super.addToGraph();
    this.scale.addConstraint(this);
    this.offset.addConstraint(this);
  }

  removeFromGraph() {
    super.removeFromGraph();
    if (this.scale != null) this.scale.removeConstraint(this);
    if (this.offset != null) this.offset.removeConstraint(this);
  }

  markInputs(mark: number) {
    super.markInputs(mark);
    this.scale.mark = this.offset.mark = mark;
  }

  /**
   * Enforce this constraint. Assume that it is satisfied.
   */
  execute() {
    if (this.direction == Direction.FORWARD) {
      this.v2.value = this.v1.value * this.scale.value + this.offset.value;
    } else {
      this.v1.value = (this.v2.value - this.offset.value) / this.scale.value;
    }
  }

  /**
   * Calculate the walkabout strength, the stay flag, and, if it is
   * 'stay', the value for the current output of this constraint. Assume
   * this constraint is satisfied.
   */
  recalculate() {
    let ihn = this.input(), out = this.output();
    out.walkStrength = Strength.weakestOf(this.strength, ihn.walkStrength);
    out.stay = ihn.stay && this.scale.stay && this.offset.stay;
    if (out.stay) this.execute();
  }
}

/* --- *
 * E q u a l i t  y   C o n s t r a i n t
 * --- */

/**
 * Constrains two variables to have the same value.
 */
class EqualityConstraint extends BinaryConstraint {
  constructor(
    var1: Variable,
    var2: Variable,
    strength: Strength,
  ) {
    super(var1, var2, strength);
  }

  /**
   * Enforce this constraint. Assume that it is satisfied.
   */
  execute() {
    this.output().value = this.input().value;
  }
}

/* --- *
 * V a r i a b l e
 * --- */

/**
 * A constrained variable. In addition to its value, it maintain the
 * structure of the constraint graph, the current dataflow graph, and
 * various parameters of interest to the DeltaBlue incremental
 * constraint solver.
 */
class Variable {
  value;
  constraints;
  determinedBy: Constraint | null;
  mark;
  walkStrength;
  stay;
  name;

  constructor(name: string, initialValue = 0) {
    this.value = initialValue;
    this.constraints = new OrderedCollection<Constraint>();
    this.determinedBy = null;
    this.mark = 0;
    this.walkStrength = Strength.WEAKEST;
    this.stay = true;
    this.name = name;
  }

  /**
   * Add the given constraint to the set of all constraints that refer
   * this variable.
   */
  addConstraint(c: Constraint) {
    this.constraints.add(c);
  }

  /**
   * Removes all traces of c from this variable.
   */
  removeConstraint(c: Constraint) {
    this.constraints.remove(c);
    if (this.determinedBy == c) this.determinedBy = null;
  }
}

/* --- *
 * P l a n n e r
 * --- */

/**
 * The DeltaBlue planner
 */
class Planner {
  currentMark;

  constructor() {
    this.currentMark = 0;
  }

  /**
   * Attempt to satisfy the given constraint and, if successful,
   * incrementally update the dataflow graph.  Details: If satifying
   * the constraint is successful, it may override a weaker constraint
   * on its output. The algorithm attempts to resatisfy that
   * constraint using some other method. This process is repeated
   * until either a) it reaches a variable that was not previously
   * determined by any constraint or b) it reaches a constraint that
   * is too weak to be satisfied using any of its methods. The
   * variables of constraints that have been processed are marked with
   * a unique mark value so that we know where we've been. This allows
   * the algorithm to avoid getting into an infinite loop even if the
   * constraint graph has an inadvertent cycle.
   */
  incrementalAdd(c: Constraint) {
    let mark = this.newMark();
    let overridden = c.satisfy(this, mark);
    while (overridden != null) {
      overridden = overridden.satisfy(this, mark);
    }
  }

  /**
   * Entry point for retracting a constraint. Remove the given
   * constraint and incrementally update the dataflow graph.
   * Details: Retracting the given constraint may allow some currently
   * unsatisfiable downstream constraint to be satisfied. We therefore collect
   * a list of unsatisfied downstream constraints and attempt to
   * satisfy each one in turn. This list is traversed by constraint
   * strength, strongest first, as a heuristic for avoiding
   * unnecessarily adding and then overriding weak constraints.
   * Assume: c is satisfied.
   */
  incrementalRemove(c: Constraint) {
    let out = c.output();
    c.markUnsatisfied();
    c.removeFromGraph();
    let unsatisfied = this.removePropagateFrom(out);
    let strength: Strength | undefined = Strength.REQUIRED;
    do {
      for (let i = 0; i < unsatisfied.size(); i++) {
        let u = unsatisfied.at(i);
        if (u.strength == strength) {
          this.incrementalAdd(u);
        }
      }
      strength = strength!.nextWeaker();
    } while (strength != Strength.WEAKEST);
  }

  /**
   * Select a previously unused mark value.
   */
  newMark() {
    return ++this.currentMark;
  }

  /**
   * Extract a plan for resatisfaction starting from the given source
   * constraints, usually a set of input constraints. This method
   * assumes that stay optimization is desired; the plan will contain
   * only constraints whose output variables are not stay. Constraints
   * that do no computation, such as stay and edit constraints, are
   * not included in the plan.
   * Details: The outputs of a constraint are marked when it is added
   * to the plan under construction. A constraint may be appended to
   * the plan when all its input variables are known. A variable is
   * known if either a) the variable is marked (indicating that has
   * been computed by a constraint appearing earlier in the plan), b)
   * the variable is 'stay' (i.e. it is a constant at plan execution
   * time), or c) the variable is not determined by any
   * constraint. The last provision is for past states of history
   * variables, which are not stay but which are also not computed by
   * any constraint.
   * Assume: sources are all satisfied.
   */
  makePlan(sources: OrderedCollection<Constraint>) {
    let mark = this.newMark();
    let plan = new Plan();
    let todo = sources;
    while (todo.size() > 0) {
      let c = todo.removeFirst()!;
      if (c.output().mark != mark && c.inputsKnown(mark)) {
        plan.addConstraint(c);
        c.output().mark = mark;
        this.addConstraintsConsumingTo(c.output(), todo);
      }
    }
    return plan;
  }

  /**
   * Extract a plan for resatisfying starting from the output of the
   * given constraints, usually a set of input constraints.
   */
  extractPlanFromConstraints(constraints: OrderedCollection<Constraint>) {
    let sources = new OrderedCollection<Constraint>();
    for (let i = 0; i < constraints.size(); i++) {
      let c = constraints.at(i);
      if (c.isInput() && c.isSatisfied()) {
        // not in plan already and eligible for inclusion
        sources.add(c);
      }
    }
    return this.makePlan(sources);
  }

  /**
   * Recompute the walkabout strengths and stay flags of all variables
   * downstream of the given constraint and recompute the actual
   * values of all variables whose stay flag is true. If a cycle is
   * detected, remove the given constraint and answer
   * false. Otherwise, answer true.
   * Details: Cycles are detected when a marked variable is
   * encountered downstream of the given constraint. The sender is
   * assumed to have marked the inputs of the given constraint with
   * the given mark. Thus, encountering a marked node downstream of
   * the output constraint means that there is a path from the
   * constraint's output to one of its inputs.
   */
  addPropagate(c: Constraint, mark: number) {
    let todo = new OrderedCollection<Constraint>();
    todo.add(c);
    while (todo.size() > 0) {
      let d = todo.removeFirst()!;
      if (d.output().mark == mark) {
        this.incrementalRemove(c);
        return false;
      }
      d.recalculate();
      this.addConstraintsConsumingTo(d.output(), todo);
    }
    return true;
  }

  /**
   * Update the walkabout strengths and stay flags of all variables
   * downstream of the given constraint. Answer a collection of
   * unsatisfied constraints sorted in order of decreasing strength.
   */
  removePropagateFrom(out: Variable) {
    out.determinedBy = null;
    out.walkStrength = Strength.WEAKEST;
    out.stay = true;
    let unsatisfied = new OrderedCollection<Constraint>();
    let todo = new OrderedCollection<Variable>();
    todo.add(out);
    while (todo.size() > 0) {
      let v = todo.removeFirst()!;
      for (let i = 0; i < v.constraints.size(); i++) {
        let c = v.constraints.at(i);
        if (!c.isSatisfied()) {
          unsatisfied.add(c);
        }
      }
      let determining = v.determinedBy;
      for (let i = 0; i < v.constraints.size(); i++) {
        let next = v.constraints.at(i);
        if (next != determining && next.isSatisfied()) {
          next.recalculate();
          todo.add(next.output());
        }
      }
    }
    return unsatisfied;
  }

  addConstraintsConsumingTo(v: Variable, coll: OrderedCollection<Constraint>) {
    let determining = v.determinedBy;
    let cc = v.constraints;
    for (let i = 0; i < cc.size(); i++) {
      let c = cc.at(i);
      if (c != determining && c.isSatisfied()) {
        coll.add(c);
      }
    }
  }

  change(v: Variable, newValue: number) {
    let edit = new EditConstraint(v, Strength.PREFERRED);
    this.addConstraint(edit);
    let edits = new OrderedCollection<Constraint>();
    edits.add(edit);
    let plan = this.extractPlanFromConstraints(edits);
    for (let i = 0; i < 10; i++) {
      v.value = newValue;
      plan.execute();
    }
    edit.destroyConstraint(this);
  }

  /**
   * Activate this constraint and attempt to satisfy it.
   */
  addConstraint(c: Constraint) {
    c.addToGraph();
    this.incrementalAdd(c);
  }
}

/* --- *
 * P l a n
 * --- */

/**
 * A Plan is an ordered list of constraints to be executed in sequence
 * to resatisfy all currently satisfiable constraints in the face of
 * one or more changing inputs.
 */
class Plan {
  v;

  constructor() {
    this.v = new OrderedCollection<Constraint>();
  }

  addConstraint(c: Constraint) {
    this.v.add(c);
  }

  size() {
    return this.v.size();
  }

  constraintAt(index: number) {
    return this.v.at(index);
  }

  execute() {
    for (let i = 0; i < this.size(); i++) {
      let c = this.constraintAt(i);
      c.execute();
    }
  }
}

/* --- *
 * M a i n
 * --- */

/**
 * This is the standard DeltaBlue benchmark. A long chain of equality
 * constraints is constructed with a stay constraint on one end. An
 * edit constraint is then added to the opposite end and the time is
 * measured for adding and removing this constraint, and extracting
 * and executing a constraint satisfaction plan. There are two cases.
 * In case 1, the added constraint is stronger than the stay
 * constraint and values must propagate down the entire length of the
 * chain. In case 2, the added constraint is weaker than the stay
 * constraint so it cannot be accomodated. The cost in this case is,
 * of course, very low. Typical situations lie somewhere between these
 * two extremes.
 */
function chainTest(n: number) {
  let planner = new Planner();
  let prev = null, first: Variable | null = null, last: Variable | null = null;

  // Build chain of n equality constraints
  for (let i = 0; i <= n; i++) {
    let name = "v" + i;
    let v = new Variable(name);
    if (prev != null) {
      planner.addConstraint(new EqualityConstraint(prev, v, Strength.REQUIRED));
    }
    if (i == 0) first = v;
    if (i == n) last = v;
    prev = v;
  }

  planner.addConstraint(new StayConstraint(last!, Strength.STRONG_DEFAULT));

  let edit = new EditConstraint(first!, Strength.PREFERRED);
  planner.addConstraint(edit);
  let edits = new OrderedCollection<EditConstraint>();
  edits.add(edit);
  let plan = planner.extractPlanFromConstraints(edits);
  for (let i = 0; i < 100; i++) {
    first!.value = i;
    plan.execute();
    if (last!.value != i) {
      throw new Error("Chain test failed.");
    }
  }
}

/**
 * This test constructs a two sets of variables related to each
 * other by a simple linear transformation (scale and offset). The
 * time is measured to change a variable on either side of the
 * mapping and to change the scale and offset factors.
 */
function projectionTest(n: number) {
  let planner = new Planner();
  let scale = new Variable("scale", 10);
  let offset = new Variable("offset", 1000);
  let src: Variable | null = null, dst: Variable | null = null;

  let dests = new OrderedCollection<Variable>();
  for (let i = 0; i < n; i++) {
    src = new Variable("src" + i, i);
    dst = new Variable("dst" + i, i);
    dests.add(dst);
    planner.addConstraint(new StayConstraint(src, Strength.NORMAL));
    planner.addConstraint(
      new ScaleConstraint(src, scale, offset, dst, Strength.REQUIRED),
    );
  }

  planner.change(src!, 17);
  if (dst!.value != 1170) throw new Error("Projection 1 failed");
  planner.change(dst!, 1050);
  if (src!.value != 5) throw new Error("Projection 2 failed");
  planner.change(scale, 5);
  for (let i = 0; i < n - 1; i++) {
    if (dests.at(i).value != i * 5 + 1000) {
      throw new Error("Projection 3 failed");
    }
  }
  planner.change(offset, 2000);
  for (let i = 0; i < n - 1; i++) {
    if (dests.at(i).value != i * 5 + 2000) {
      throw new Error("Projection 4 failed");
    }
  }
}

export function deltaBlue() {
  chainTest(100);
  projectionTest(100);
}
