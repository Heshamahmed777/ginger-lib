use std::marker::PhantomData;
use algebra::Field;
use radix_trie::{Trie, TrieCommon};

use crate::{Index, Variable, LinearCombination, SynthesisError};

/// Represents a constraint system which can have new variables
/// allocated and constrains between them formed.
pub trait ConstraintSystem<F: Field>: Sized {
    /// Represents the type of the "root" of this constraint system
    /// so that nested namespaces can minimize indirection.
    type Root: ConstraintSystem<F>;

    /// Return the "one" input variable
    fn one() -> Variable {
        Variable::new_unchecked(Index::Input(0))
    }

    /// Allocate a private variable in the constraint system. The provided
    /// function is used to determine the assignment of the variable. The
    /// given `annotation` function is invoked in testing contexts in order
    /// to derive a unique name for this variable in the current namespace.
    fn alloc<FN, A, AR>(&mut self, annotation: A, f: FN) -> Result<Variable, SynthesisError>
        where
            FN: FnOnce() -> Result<F, SynthesisError>,
            A: FnOnce() -> AR,
            AR: Into<String>;

    /// Allocate a public variable in the constraint system. The provided
    /// function is used to determine the assignment of the variable.
    fn alloc_input<FN, A, AR>(&mut self, annotation: A, f: FN) -> Result<Variable, SynthesisError>
        where
            FN: FnOnce() -> Result<F, SynthesisError>,
            A: FnOnce() -> AR,
            AR: Into<String>;

    /// Enforce that `A` * `B` = `C`. The `annotation` function is invoked in
    /// testing contexts in order to derive a unique name for the constraint
    /// in the current namespace.
    fn enforce<A, AR, LA, LB, LC>(&mut self, annotation: A, a: LA, b: LB, c: LC)
        where
            A: FnOnce() -> AR,
            AR: Into<String>,
            LA: FnOnce(LinearCombination<F>) -> LinearCombination<F>,
            LB: FnOnce(LinearCombination<F>) -> LinearCombination<F>,
            LC: FnOnce(LinearCombination<F>) -> LinearCombination<F>;

    /// Create a new (sub)namespace and enter into it. Not intended
    /// for downstream use; use `namespace` instead.
    fn push_namespace<NR, N>(&mut self, name_fn: N)
        where
            NR: Into<String>,
            N: FnOnce() -> NR;

    /// Exit out of the existing namespace. Not intended for
    /// downstream use; use `namespace` instead.
    fn pop_namespace(&mut self);

    /// Gets the "root" constraint system, bypassing the namespacing.
    /// Not intended for downstream use; use `namespace` instead.
    fn get_root(&mut self) -> &mut Self::Root;

    /// Begin a namespace for this constraint system.
    fn ns<'a, NR, N>(&'a mut self, name_fn: N) -> Namespace<'a, F, Self::Root>
        where
            NR: Into<String>,
            N: FnOnce() -> NR,
    {
        self.get_root().push_namespace(name_fn);

        Namespace(self.get_root(), PhantomData)
    }

    /// Output the number of constraints in the system.
    fn num_constraints(&self) -> usize;
}

/// Represents the actual implementation of a rank-1 constraint system (R1CS)
#[derive(Debug, Clone)]
pub struct ConstraintSystemImpl<F: Field> {
    /// The mode in which the constraint system is operating. `self` can either
    /// be in setup mode (i.e., `self.mode == SynthesisMode::Setup`) or in
    /// proving mode (i.e., `self.mode == SynthesisMode::Prove`). If we are
    /// in proving mode, then we have the additional option of whether or
    /// not to construct the A, B, and C matrices of the constraint system
    /// (see below).
    pub mode: SynthesisMode,
    /// The number of variables that are "public inputs" to the constraint
    /// system.
    pub num_inputs: usize,
    /// The number of variables that are "private inputs" to the constraint
    /// system.
    pub num_aux: usize,
    /// The number of constraints in the constraint system.
    pub num_constraints: usize,
    /// Assignments to the public input variables. This is empty if `self.mode
    /// == SynthesisMode::Setup`.
    pub aux_assignment: Vec<F>,
    /// Assignments to the private input variables. This is empty if `self.mode
    /// == SynthesisMode::Setup`.
    pub input_assignment: Vec<F>,
    /// `A` matrix of the R1CS.
    pub at: Vec<Vec<(F, Index)>>,
    /// `B` matrix of the R1CS.
    pub bt: Vec<Vec<(F, Index)>>,
    /// `C` matrix of the R1CS.
    pub ct: Vec<Vec<(F, Index)>>,
    named_objects: Trie<String, NamedObject>,
    current_namespace: Vec<String>,
    constraint_names: Vec<String>,
    input_names: Vec<String>,
    aux_names: Vec<String>,
}

/// Defines the mode of operation of `ConstraintSystemImpl`.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SynthesisMode {
    /// Indicate to `ConstraintSystemImpl` that it should only generate
    /// constraint matrices and not populate the variable assignments.
    Setup,
    /// Indicate to `ConstraintSystemImpl` that it populate the variable
    /// assignments. If additionally `construct_matrices == true`, then generate
    /// the matrices as in the `Setup` case.
    Prove {
        /// If `construct_matrices == true`, then generate
        /// the matrices as in the `Setup` case.
        construct_matrices: bool,
    },
}

#[derive(Debug, Clone)]
enum NamedObject {
    Constraint(usize),
    Var(Variable),
    Namespace,
}

fn compute_path(ns: &[String], this: String) -> String {
    if this.chars().any(|a| a == '/') {
        panic!("'/' is not allowed in names");
    }

    let mut name = String::new();

    let mut needs_separation = false;
    for ns in ns.iter().chain(Some(&this).into_iter()) {
        if needs_separation {
            name += "/";
        }

        name += ns;
        needs_separation = true;
    }

    name
}

impl<F: Field> ConstraintSystem<F> for ConstraintSystemImpl<F> {
    type Root = ConstraintSystemImpl<F>;
    #[inline]
    fn one() -> Variable {
        Variable::new_unchecked(Index::Input(0))
    }
    #[inline]
    fn alloc<FN, A, AR>(&mut self, annotation: A, f: FN) -> Result<Variable, SynthesisError>
        where
            FN: FnOnce() -> Result<F, SynthesisError>,
            A: FnOnce() -> AR,
            AR: Into<String>,
    {
        let index = self.num_aux;
        self.num_aux += 1;

        let path = compute_path(&self.current_namespace, annotation().into());
        self.aux_names.push(path.clone());
        let var = Variable::new_unchecked(Index::Aux(index));
        self.set_named_obj(path, NamedObject::Var(var));

        if !self.is_in_setup_mode() {
            self.aux_assignment.push(f()?);
        }
        Ok(Variable::new_unchecked(Index::Aux(index)))
    }
    #[inline]
    fn alloc_input<FN, A, AR>(&mut self, annotation: A, f: FN) -> Result<Variable, SynthesisError>
        where
            FN: FnOnce() -> Result<F, SynthesisError>,
            A: FnOnce() -> AR,
            AR: Into<String>,
    {
        let index = self.num_inputs;
        self.num_inputs += 1;

        let path = compute_path(&self.current_namespace, annotation().into());
        self.input_names.push(path.clone());
        let var = Variable::new_unchecked(Index::Input(index));
        self.set_named_obj(path, NamedObject::Var(var));

        if !self.is_in_setup_mode() {
            self.input_assignment.push(f()?);
        }
        Ok(Variable::new_unchecked(Index::Input(index)))
    }
    #[inline]
    fn enforce<A, AR, LA, LB, LC>(&mut self, annotation: A, a: LA, b: LB, c: LC)
        where
            A: FnOnce() -> AR,
            AR: Into<String>,
            LA: FnOnce(LinearCombination<F>) -> LinearCombination<F>,
            LB: FnOnce(LinearCombination<F>) -> LinearCombination<F>,
            LC: FnOnce(LinearCombination<F>) -> LinearCombination<F>,
    {
        let path = compute_path(&self.current_namespace, annotation().into());
        let index = self.at.len();
        self.set_named_obj(path.clone(), NamedObject::Constraint(index));

        self.constraint_names.push(path.clone());

        if self.should_construct_matrices() {
            self.at.push(vec![]);
            self.bt.push(vec![]);
            self.ct.push(vec![]);
            Self::push_constraints(
                a(LinearCombination::zero()),
                &mut self.at,
                self.num_constraints,
            );
            Self::push_constraints(
                b(LinearCombination::zero()),
                &mut self.bt,
                self.num_constraints,
            );
            Self::push_constraints(
                c(LinearCombination::zero()),
                &mut self.ct,
                self.num_constraints,
            );
        }

        self.num_constraints += 1;
    }
    fn push_namespace<NR, N>(&mut self, name_fn: N)
        where
            NR: Into<String>,
            N: FnOnce() -> NR
    {
        let name = name_fn().into();
        let path = compute_path(&self.current_namespace, name.clone());
        self.set_named_obj(path.clone(), NamedObject::Namespace);
        self.current_namespace.push(name);
    }
    fn pop_namespace(&mut self) {
        assert!(self.current_namespace.pop().is_some());
    }
    fn get_root(&mut self) -> &mut Self::Root {
        self
    }
    fn num_constraints(&self) -> usize {
        self.num_constraints
    }
}

impl<F: Field> ConstraintSystemImpl<F> {
    /// Construct an empty `ConstraintSystem`.
    pub fn new() -> Self {
        let mut map = Trie::new();
        map.insert(
            "ONE".into(),
            NamedObject::Var(ConstraintSystemImpl::<F>::one()),
        );
        Self {
            num_inputs: 1,
            num_aux: 0,
            num_constraints: 0,
            at: Vec::new(),
            bt: Vec::new(),
            ct: Vec::new(),
            input_assignment: vec![F::one()],
            aux_assignment: Vec::new(),
            mode: SynthesisMode::Prove {
                construct_matrices: true,
            },
            named_objects: map,
            current_namespace: vec![],
            constraint_names: vec![],
            input_names: vec!["ONE".into()],
            aux_names: vec![],
        }
    }
    /// Set `self.mode` to `mode`.
    pub fn set_mode(&mut self, mode: SynthesisMode) {
        self.mode = mode;
    }

    /// Check whether `self.mode == SynthesisMode::Setup`.
    pub fn is_in_setup_mode(&self) -> bool {
        self.mode == SynthesisMode::Setup
    }

    /// Check whether or not `self` will construct matrices.
    pub fn should_construct_matrices(&self) -> bool {
        match self.mode {
            SynthesisMode::Setup => true,
            SynthesisMode::Prove { construct_matrices } => construct_matrices,
        }
    }
    fn push_constraints(
        l: LinearCombination<F>,
        constraints: &mut [Vec<(F, Index)>],
        this_constraint: usize,
    ) {
        for (var, coeff) in l.as_ref() {
            match var.get_unchecked() {
                Index::Input(i) => constraints[this_constraint].push((*coeff, Index::Input(i))),
                Index::Aux(i) => constraints[this_constraint].push((*coeff, Index::Aux(i))),
            }
        }
    }
    fn eval_lc(
        terms: &[(F, Index)],
        inputs: &[F],
        aux: &[F],
    ) -> F {
        let mut acc = F::zero();

        for &(ref coeff, idx) in terms {
            let mut tmp = match idx {
                Index::Input(index) => inputs[index],
                Index::Aux(index) => aux[index],
            };

            tmp.mul_assign(coeff);
            acc.add_assign(tmp);
        }

        acc
    }

    /// Prints the list of named object currently registered into the constraint system
    pub fn print_named_objects(&self) {
        for (name, _) in self.named_objects.iter() {
            println!("{}", name);
        }
    }

    /// Returns an Option containing the name of the first constraint which is not satisfied
    /// or None if all the constraints are satisfied.
    pub fn which_is_unsatisfied(&self) -> Option<&str> {
        for i in 0..self.num_constraints {
            let mut a = Self::eval_lc(&self.at[i], &self.input_assignment, &self.aux_assignment);
            let b = Self::eval_lc(&self.bt[i], &self.input_assignment, &self.aux_assignment);
            let c = Self::eval_lc(&self.ct[i], &self.input_assignment, &self.aux_assignment);
            a.mul_assign(&b);

            if a != c {
                return Some(&self.constraint_names[i]);
            }
        }
        None
    }

    /// Checks whether all the constraints are satisfied.
    pub fn is_satisfied(&self) -> bool {
        self.which_is_unsatisfied().is_none()
    }
    /// Sets the the variable named `path` to the value `to`.
    /// Panics in setup mode and if `path` does not indicate a variable.
    pub fn set(&mut self, path: &str, to: F) {
        if self.is_in_setup_mode() {
            panic!("it is not possible to set the value of variables during setup.")
        }
        match self.named_objects.get(path) {
            Some(&NamedObject::Var(ref v)) => match v.get_unchecked() {
                Index::Input(index) => self.input_assignment[index] = to,
                Index::Aux(index) => self.aux_assignment[index] = to,
            },
            Some(e) => panic!(
                "tried to set path `{}` to value, but `{:?}` already exists there.",
                path, e
            ),
            _ => panic!("no variable exists at path: {}", path),
        }
    }
    /// Gets the value of the variable named `path`.
    /// Panics in setup mode and if `path` does not indicate a variable.
    pub fn get(&mut self, path: &str) -> F {
        if self.is_in_setup_mode() {
            panic!("it is not possible to get the value of variables during setup.")
        }
        match self.named_objects.get(path) {
            Some(&NamedObject::Var(ref v)) => match v.get_unchecked() {
                Index::Input(index) => self.input_assignment[index],
                Index::Aux(index) => self.aux_assignment[index],
            },
            Some(e) => panic!(
                "tried to get value of path `{}`, but `{:?}` exists there (not a variable)",
                path, e
            ),
            _ => panic!("no variable exists at path: {}", path),
        }
    }
    fn set_named_obj(&mut self, path: String, to: NamedObject) {
        if self.named_objects.get(&path).is_some() {
            panic!("tried to create object at existing path: {}", path);
        }

        self.named_objects.insert(path, to);
    }
}

/// This is a "namespaced" constraint system which borrows a constraint system
/// (pushing a namespace context) and, when dropped, pops out of the namespace
/// context.
pub struct Namespace<'a, F: Field, CS: ConstraintSystem<F>>(&'a mut CS, PhantomData<F>);

/// Computations are expressed in terms of rank-1 constraint systems (R1CS).
/// The `generate_constraints` method is called to generate constraints for
/// both CRS generation and for proving.
pub trait ConstraintSynthesizer<F: Field> {
    /// Drives generation of new constraints inside `CS`.
    fn generate_constraints<CS: ConstraintSystem<F>>(self, cs: &mut CS) -> Result<(), SynthesisError>;
}

impl<F: Field, CS: ConstraintSystem<F>> ConstraintSystem<F> for Namespace<'_, F, CS> {
    type Root = CS::Root;

    #[inline]
    fn one() -> Variable {
        CS::one()
    }

    #[inline]
    fn alloc<FN, A, AR>(&mut self, annotation: A, f: FN) -> Result<Variable, SynthesisError>
        where
            FN: FnOnce() -> Result<F, SynthesisError>,
            A: FnOnce() -> AR,
            AR: Into<String>,
    {
        self.0.alloc(annotation, f)
    }

    #[inline]
    fn alloc_input<FN, A, AR>(&mut self, annotation: A, f: FN) -> Result<Variable, SynthesisError>
        where
            FN: FnOnce() -> Result<F, SynthesisError>,
            A: FnOnce() -> AR,
            AR: Into<String>,
    {
        self.0.alloc_input(annotation, f)
    }

    #[inline]
    fn enforce<A, AR, LA, LB, LC>(&mut self, annotation: A, a: LA, b: LB, c: LC)
        where
            A: FnOnce() -> AR,
            AR: Into<String>,
            LA: FnOnce(LinearCombination<F>) -> LinearCombination<F>,
            LB: FnOnce(LinearCombination<F>) -> LinearCombination<F>,
            LC: FnOnce(LinearCombination<F>) -> LinearCombination<F>,
    {
        self.0.enforce(annotation, a, b, c)
    }

    // Downstream users who use `namespace` will never interact with these
    // functions and they will never be invoked because the namespace is
    // never a root constraint system.

    #[inline]
    fn push_namespace<NR, N>(&mut self, _: N)
        where
            NR: Into<String>,
            N: FnOnce() -> NR,
    {
        panic!("only the root's push_namespace should be called");
    }

    #[inline]
    fn pop_namespace(&mut self) {
        panic!("only the root's pop_namespace should be called");
    }

    #[inline]
    fn get_root(&mut self) -> &mut Self::Root {
        self.0.get_root()
    }

    #[inline]
    fn num_constraints(&self) -> usize {
        self.0.num_constraints()
    }
}

impl<F: Field, CS: ConstraintSystem<F>> Drop for Namespace<'_, F, CS> {
    #[inline]
    fn drop(&mut self) {
        self.get_root().pop_namespace()
    }
}

/// Convenience implementation of ConstraintSystem<F> for mutable references to
/// constraint systems.
impl<F: Field, CS: ConstraintSystem<F>> ConstraintSystem<F> for &mut CS {
    type Root = CS::Root;

    #[inline]
    fn one() -> Variable {
        CS::one()
    }

    #[inline]
    fn alloc<FN, A, AR>(&mut self, annotation: A, f: FN) -> Result<Variable, SynthesisError>
        where
            FN: FnOnce() -> Result<F, SynthesisError>,
            A: FnOnce() -> AR,
            AR: Into<String>,
    {
        (**self).alloc(annotation, f)
    }

    #[inline]
    fn alloc_input<FN, A, AR>(&mut self, annotation: A, f: FN) -> Result<Variable, SynthesisError>
        where
            FN: FnOnce() -> Result<F, SynthesisError>,
            A: FnOnce() -> AR,
            AR: Into<String>,
    {
        (**self).alloc_input(annotation, f)
    }

    #[inline]
    fn enforce<A, AR, LA, LB, LC>(&mut self, annotation: A, a: LA, b: LB, c: LC)
        where
            A: FnOnce() -> AR,
            AR: Into<String>,
            LA: FnOnce(LinearCombination<F>) -> LinearCombination<F>,
            LB: FnOnce(LinearCombination<F>) -> LinearCombination<F>,
            LC: FnOnce(LinearCombination<F>) -> LinearCombination<F>,
    {
        (**self).enforce(annotation, a, b, c)
    }

    #[inline]
    fn push_namespace<NR, N>(&mut self, name_fn: N)
        where
            NR: Into<String>,
            N: FnOnce() -> NR,
    {
        (**self).push_namespace(name_fn)
    }

    #[inline]
    fn pop_namespace(&mut self) {
        (**self).pop_namespace()
    }

    #[inline]
    fn get_root(&mut self) -> &mut Self::Root {
        (**self).get_root()
    }

    #[inline]
    fn num_constraints(&self) -> usize {
        (**self).num_constraints()
    }
}
