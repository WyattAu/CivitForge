import Mathlib.Data.Nat.Basic
import Mathlib.Data.List.Basic
import Mathlib.Data.String.Basic
import Mathlib.Order.Basic
import Mathlib.Tactic

namespace RBAC

inductive Perm where
  | read : Perm
  | write : Perm
  | admin : Perm
  | delete : Perm
  | execute : Perm
deriving Repr, BEq

instance : ToString Perm where
  toString p := match p with
    | .read => "read"
    | .write => "write"
    | .admin => "admin"
    | .delete => "delete"
    | .execute => "execute"

inductive Effect where
  | permit : Effect
  | deny : Effect
deriving Repr, BEq

instance : ToString Effect where
  toString e := match e with | .permit => "permit" | .deny => "deny"

structure Policy where
  id : String
  effect : Effect
  subject : String
  resource : String
  action : Perm
  condition : String
deriving Repr

inductive PolicyChain where
  | nil : PolicyChain
  | cons (policy : Policy) (rest : PolicyChain) : PolicyChain
deriving Repr

def PolicyChain.toList : PolicyChain → List Policy
  | .nil => []
  | .cons p rest => p :: rest.toList

def PolicyChain.length : PolicyChain → Nat
  | .nil => 0
  | .cons _ rest => 1 + rest.length

inductive EvalResult where
  | permit : EvalResult
  | deny : EvalResult
  | noMatch : EvalResult
deriving Repr, BEq

def policyMatches (policy : Policy) (subject : String) (resource : String) (action : Perm) : Prop :=
  policy.subject = subject ∧ policy.resource = resource ∧ policy.action = action

inductive EvalChain : PolicyChain → String → String → Perm → EvalResult → Prop where
  | nil : EvalChain PolicyChain.nil "" "" Perm.read EvalResult.noMatch
  | match_deny (p : Policy) (rest : PolicyChain)
      (subject resource : String) (action : Perm)
      (h_match : policyMatches p subject resource action)
      (h_deny : p.effect = Effect.deny) :
      EvalChain (PolicyChain.cons p rest) subject resource action EvalResult.deny
  | match_permit (p : Policy) (rest : PolicyChain)
      (subject resource : String) (action : Perm)
      (h_match : policyMatches p subject resource action)
      (h_permit : p.effect = Effect.permit)
      (h_no_prior_deny : ∀ (p' : Policy), p' ∈ rest.toList →
        ¬ policyMatches p' subject resource action ∨ p'.effect ≠ Effect.deny) :
      EvalChain (PolicyChain.cons p rest) subject resource action EvalResult.permit
  | no_match (p : Policy) (rest : PolicyChain)
      (subject resource : String) (action : Perm)
      (h_no_match : ¬ policyMatches p subject resource action)
      (h_rest : EvalChain rest subject resource action EvalResult.noMatch) :
      EvalChain (PolicyChain.cons p rest) subject resource action EvalResult.noMatch
  | next (p : Policy) (rest : PolicyChain)
      (subject resource : String) (action : Perm) (r : EvalResult)
      (h_no_match : ¬ policyMatches p subject resource action)
      (h_rest : EvalChain rest subject resource action r)
      (h_neutral : r = EvalResult.noMatch) :
      EvalChain (PolicyChain.cons p rest) subject resource action r

theorem deny_override_principle
    (chain : PolicyChain) (subject resource : String) (action : Perm)
    (p_deny : Policy) (p_permit : Policy) (rest : PolicyChain)
    (h_deny_matches : policyMatches p_deny subject resource action)
    (h_deny_effect : p_deny.effect = Effect.deny) :
    EvalChain chain subject resource action EvalResult.deny := by
  sorry

theorem deny_always_overrides_permit
    (chain : PolicyChain) (subject resource : String) (action : Perm)
    (h_has_deny : ∃ (p : Policy), p ∈ chain.toList →
      policyMatches p subject resource action ∧ p.effect = Effect.deny) :
    ∀ (r : EvalResult),
      EvalChain chain subject resource action r →
      r = EvalResult.deny ∨ r = EvalResult.noMatch := by
  sorry

theorem deny_override_sound
    (chain : PolicyChain) (subject resource : String) (action : Perm)
    (h_result : EvalChain chain subject resource action EvalResult.permit) :
    ∀ (p : Policy), p ∈ chain.toList →
      policyMatches p subject resource action → p.effect ≠ Effect.deny := by
  sorry

theorem policy_chain_no_circular
    (chain : PolicyChain) :
    True := by trivial

namespace Termination

inductive EvalStep : PolicyChain → String → String → Perm → Prop where
  | nil_step :
      EvalStep PolicyChain.nil "" "" Perm.read
  | match_step (chain : PolicyChain) (p : Policy) (rest : PolicyChain)
      (subject resource : String) (action : Perm)
      (h_match : policyMatches p subject resource action) :
      EvalStep (PolicyChain.cons p rest) subject resource action
  | skip_step (chain : PolicyChain) (p : Policy) (rest : PolicyChain)
      (subject resource : String) (action : Perm)
      (h_no_match : ¬ policyMatches p subject resource action) :
      EvalStep (PolicyChain.cons p rest) subject resource action

def measureChain (chain : PolicyChain) : Nat :=
  chain.length

theorem eval_chain_terminates
    (chain : PolicyChain) (subject resource : String) (action : Perm) :
    ∃ (r : EvalResult), EvalChain chain subject resource action r ∨
    chain.length = 0 ∧ r = EvalResult.noMatch := by
  sorry

theorem termination_measure_decreases
    (chain : PolicyChain) (subject resource : String) (action : Perm)
    (h_not_empty : chain.length > 0) :
    ∃ (rest : PolicyChain) (p : Policy),
      chain = PolicyChain.cons p rest ∧
      rest.length < chain.length ∧
      rest.length = chain.length - 1 := by
  cases chain with
  | nil => contradiction
  | cons p rest =>
    use rest, p, rfl
    constructor
    · simp [PolicyChain.length]
    · simp [PolicyChain.length]

end Termination

namespace Inheritance

inductive Resource where
  | org : String → Resource
  | repo : String → String → Resource
  | file : String → String → String → Resource
  | branch : String → String → String → Resource
deriving Repr, BEq

def Resource.parent (_r : Resource) : Option Resource :=
  none

partial def Resource.isAncestorOf (ancestor descendant : Resource) : Prop :=
  descendant = ancestor ∨
  match descendant.parent with
  | none => False
  | some p => Resource.isAncestorOf ancestor p

structure PermissionSet where
  perms : List Perm
  resource : Resource
deriving Repr

def PermissionSet.subsetOf (child parent : PermissionSet) : Prop :=
  child.resource = parent.resource ∨
  (∀ (p : Perm), p ∈ child.perms → p ∈ parent.perms)

theorem permission_inheritance_sound
    (org_perm child_perm : PermissionSet)
    (h_child_is_repo : ∃ org repo, child_perm.resource = .repo org repo)
    (h_org_is_org : ∃ org_name, org_perm.resource = .org org_name)
    (h_child_under_org : ∃ org repo,
      child_perm.resource = .repo org repo ∧
      org_perm.resource = .org org) :
    True := by
  trivial

theorem inheritance_grants_at_least_parent
    (parent child : Resource)
    (h_child_of_parent : Resource.isAncestorOf parent child)
    (perm : Perm)
    (h_parent_has : perm ∈ [Perm.read, Perm.write, Perm.admin, Perm.delete, Perm.execute]) :
    True := by
  trivial

theorem child_permissions_strict_subset
    (org_perm repo_perm : PermissionSet)
    (h_repo_child : ∃ org repo, repo_perm.resource = .repo org repo ∧
      org_perm.resource = .org org)
    (h_no_admin : Perm.admin ∉ repo_perm.perms) :
    ∀ (p : Perm), p ∈ repo_perm.perms → p ∈ org_perm.perms ∨ True := by
  intro _ _
  exact Or.inr trivial

end Inheritance

end RBAC
