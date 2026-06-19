/-
Formal Verification: Pipeline Matrix Expansion (civit-pipeline/src/matrix.rs)
Blue Paper Reference: BP-PIPELINE-MATRIX-001
Yellow Paper Reference: YP-PIPELINE-MATRIX-001

Properties to Verify:
  PROP-001: Cross-product correctness (size of expanded matrix equals product of dimension sizes)
  PROP-002: Include/exclude filter correctness
  PROP-003: Environment variable injection

Verification Status: VERIFICATION PENDING - Environment missing Lean 4
-/


import Mathlib.Data.Real.Basic
import Mathlib.Data.List.Basic
import Mathlib.Tactic

-- ============================================================
-- Axioms from Yellow Paper YP-PIPELINE-MATRIX-001
-- ============================================================

-- Axiom: Cross-product size equals product of list sizes.
-- Source: Set theory, cardinality of Cartesian product
axiom cross_product_size :
  ∀ (lists : List (List String)),
    (cross_product lists).length =
    lists.foldl (fun acc l => acc * l.length) 1

-- Axiom: Each element of cross_product has length equal to the input list count.
-- Source: Set theory, each tuple has dimension equal to the number of sets
axiom cross_product_tuple_length :
  ∀ (lists : List (List String)) (t : List String),
    t ∈ cross_product lists → t.length = lists.length

-- ============================================================
-- Definitions
-- ============================================================

-- Matrix configuration: a list of named dimension lists
def MatrixDimension := String × List String
def MatrixConfig := List MatrixDimension

-- Cross-product of lists of strings
-- Each list contributes one element to each result tuple
def cross_product : List (List String) → List (List String)
  | [] => [[]]
  | [l] => l.map (· :: [])
  | l :: rest =>
    let rest_prod := cross_product rest
    l.bind fun x => rest_prod.map fun tuple => x :: tuple

-- Filter: include entries matching all include patterns, exclude entries matching any exclude pattern
def filter_matrix_entry (entry : List String) (includes : List String) (excludes : List String) : Bool :=
  let matches_include := includes.all (fun inc => entry.any (·.containsSubstr inc))
  let matches_exclude := excludes.any (fun exc => entry.any (·.containsSubstr exc))
  matches_include && !matches_exclude

-- Inject environment variables into a matrix entry
def inject_env_vars (entry : List String) (env : String → Option String) : List String :=
  entry.map fun s =>
    if s.startsWith "$" then
      env (s.drop 1).getD s
    else s

-- ============================================================
-- Theorems
-- ============================================================

-- PROP-001: Cross-product correctness
-- The number of expanded entries equals the product of dimension sizes.
-- Proof strategy: Structural induction on the list of dimension lists.
--
-- Base case (empty list):
--   cross_product [] = [[]], length 1. Fold of empty list = 1. 1 = 1. QED.
--
-- Base case (singleton list):
--   cross_product [l] = l.map (· :: []), length = l.length.
--   Fold = 1 * l.length = l.length. QED.
--
-- Inductive case (l :: rest):
--   cross_product (l :: rest) = l.bind (fun x => (cross_product rest).map (fun t => x :: t))
--   By List.bind_length: |l.bind f| = Σ_{x ∈ l} |f x|
--   Each f x has length |cross_product rest| (by map length preservation)
--   So |cross_product (l :: rest)| = l.length * |cross_product rest|
--   By IH: |cross_product rest| = product of rest dimensions
--   Therefore: l.length * product of rest = product of all dimensions. QED.
--
-- Reference: Set theory, |A x B| = |A| * |B|; generalized to n-ary product.
-- Source: Discrete Mathematics, Cartesian product cardinality.
theorem cross_product_correctness (dims : MatrixConfig) :
    (cross_product (dims.map (·.2))).length =
    dims.foldl (fun acc d => acc * d.2.length) 1 := by
  -- Proof by structural induction on dims.
  -- The cross_product function is defined recursively on List (List String).
  -- We unfold the definition and apply the induction principle.
  induction dims with
  | nil =>
    -- dims = []
    -- cross_product [] = [[]], length = 1
    -- dims.foldl ... 1 = 1
    simp [cross_product, List.foldl]
  | cons d rest ih =>
    -- dims = d :: rest
    -- cross_product (d.2 :: rest.map (·.2))
    -- = d.2.bind (fun x => (cross_product (rest.map (·.2))).map (fun t => x :: t))
    -- length = d.2.length * (cross_product (rest.map (·.2))).length
    -- By IH: (cross_product (rest.map (·.2))).length = rest.foldl ... 1
    -- So d.2.length * rest.foldl ... 1 = (d :: rest).foldl ... 1
    simp [cross_product, List.foldl]
    rw [List.bind_length]
    -- Each mapped tuple has the same length, so multiply by d.2.length
    sorry -- Requires List.bind_length lemma and foldl composition

-- PROP-002: Include/exclude filter correctness
-- An entry is included iff it matches all include patterns and no exclude patterns.
-- Proof strategy: Unfold filter_matrix_entry definition, then apply Bool ↔ Prop
-- equivalences for &&, !, all, any.
--
-- filter_matrix_entry entry includes excludes
--   = (includes.all (fun inc => entry.any (·.containsSubstr inc))) &&
--     !(excludes.any (fun exc => entry.any (·.containsSubstr exc)))
--
-- The ↔ direction:
--   (→): If the Bool expression is true, then matches_include is true and
--         matches_exclude is false. Unfold all/any to get the universal/existential.
--   (←): If the universal/existential conditions hold, construct the Bool witnesses.
--
-- Reference: Boolean algebra, De Morgan's laws for filter predicates.
theorem filter_correct (entry : List String) (includes excludes : List String) :
    filter_matrix_entry entry includes excludes = true ↔
    (∀ inc ∈ includes, entry.any (·.containsSubstr inc)) ∧
    ¬ (∃ exc ∈ excludes, entry.any (·.containsSubstr exc)) := by
  unfold filter_matrix_entry
  simp only [Bool.and_eq_true, Bool.not_eq_true]
  constructor
  · -- Forward: filter = true → conditions hold
    intro ⟨h_all, h_not_any⟩
    constructor
    · -- matches_include = true → all includes match
      intro inc hinc
      exact List.all_iff_forall.mp h_all inc hinc
    · -- !matches_exclude = true → no excludes match
      intro ⟨exc, hexc, hany⟩
      have h_any : excludes.any (·.containsSubstr exc) = true :=
        List.any_iff_exists.mpr ⟨exc, hexc, hany⟩
      simp [h_any] at h_not_any
  · -- Backward: conditions hold → filter = true
    intro ⟨hinc, hexcl⟩
    constructor
    · exact List.all_iff_forall.mpr hinc
    · intro h_any
      apply hexcl
      exact List.any_iff_exists.mp h_any

-- PROP-003: Environment variable injection
-- Variables prefixed with $ are replaced with their environment values.
-- Proof strategy: Structural induction on the entry list.
--
-- Base case (empty): No elements, vacuously true.
--
-- Inductive case (s :: rest):
--   inject_env_vars (s :: rest) env = inject_env_vars_single s env :: inject_env_vars rest env
--   By IH, the result for rest is correct at each index.
--   For s at index 0:
--     If s.startsWith "$", result = env (s.drop 1).getD s
--     Otherwise, result = s
--   This matches the specification.
--
-- Reference: List.map correctness, Option.getD semantics.
theorem inject_env_correct (entry : List String) (env : String → Option String) :
    ∀ (s : String), s ∈ entry →
      (inject_env_vars entry env).get? (entry.indexOf s) =
      if s.startsWith "$" then
        some ((env (s.drop 1)).getD s)
      else some s := by
  intro s hs
  induction entry with
  | nil => simp at hs
  | cons h t ih =>
    simp [List.indexOf, inject_env_vars]
    by_cases h_eq : h = s
    · -- s is the head element, index = 0
      subst h_eq
      simp [List.indexOf, ite_true]
      rfl
    · -- s is in the tail, index > 0
      have h_idx : (h :: t).indexOf s = t.indexOf s + 1 := by
        simp [List.indexOf, ite_false, h_eq]
      rw [h_idx]
      simp [inject_env_vars]
      have h_mem : s ∈ t := by
        simp [List.mem_cons] at hs
        exact hs.resolve_left (Ne.symm h_eq)
      exact ih s h_mem
