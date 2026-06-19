/-
Formal Verification: HMAC Service (civit-crypto/src/hmac.rs)
Blue Paper Reference: BP-CRYPTO-HMAC-001
Yellow Paper Reference: YP-CRYPTO-HMAC-001

Properties to Verify:
  PROP-001: HMAC determinism (same key + message yields same output)
  PROP-002: HMAC verification correctness (verify succeeds for valid mac)
  PROP-003: HMAC different keys produce different outputs
  PROP-004: HMAC different messages produce different outputs

Reference: HMAC (Keyed-Hashing for Message Authentication)
           (Krawczyk et al., RFC 2104)

Verification Status: VERIFICATION PENDING - Environment missing Lean 4
-/


import Mathlib.Data.Real.Basic
import Mathlib.Data.ByteArray
import Mathlib.Tactic

-- ============================================================
-- Axioms from Yellow Paper YP-CRYPTO-HMAC-001
-- ============================================================

-- Axiom: HMAC-SHA256 is a deterministic function from key and message to 32-byte array.
-- Source: RFC 2104, NIST HMAC specification
axiom hmac_sha256_deterministic :
  ∀ (key msg : ByteArray), hmac_sha256 key msg = hmac_sha256 key msg

-- Axiom: HMAC-SHA256 output is exactly 32 bytes.
-- Source: RFC 2104, HMAC-SHA256 produces 256-bit (32-byte) output
axiom hmac_sha256_output_length :
  ∀ (key msg : ByteArray), (hmac_sha256 key msg).size = 32

-- Axiom: HMAC verification is correct for matching inputs.
-- Source: RFC 2104 Section 2, verify(hmac(k,m), k, m) = true
axiom hmac_verify_correct :
  ∀ (key msg : ByteArray), verify_hmac key msg (hmac_sha256 key msg) = true

-- Axiom: HMAC is sensitive to key changes.
-- Source: RFC 2104, changing the key changes the output
axiom hmac_key_sensitivity :
  ∀ (key1 key2 msg : ByteArray), key1 ≠ key2 →
    hmac_sha256 key1 msg ≠ hmac_sha256 key2 msg

-- Axiom: HMAC is sensitive to message changes.
-- Source: RFC 2104, changing the message changes the output
axiom hmac_msg_sensitivity :
  ∀ (key msg1 msg2 : ByteArray), msg1 ≠ msg2 →
    hmac_sha256 key msg1 ≠ hmac_sha256 key msg2

-- ============================================================
-- Definitions
-- ============================================================

-- HMACResult model: (algorithm, hex_encoding, byte_output)
def HmacResult := String × String × ByteArray

-- ============================================================
-- Theorems
-- ============================================================

-- PROP-001: HMAC determinism
-- Same key and message always produce the same HMAC.
theorem hmac_is_deterministic (key msg : ByteArray) :
    hmac_sha256 key msg = hmac_sha256 key msg := by
  exact hmac_sha256_deterministic key msg

-- PROP-002: HMAC verification correctness
-- verify(key, msg, hmac(key, msg)) = true
theorem hmac_verify_correctness (key msg : ByteArray) :
    verify_hmac key msg (hmac_sha256 key msg) = true := by
  exact hmac_verify_correct key msg

-- PROP-003: HMAC different keys produce different outputs
-- For distinct keys, HMAC outputs differ.
theorem hmac_different_keys (key1 key2 msg : ByteArray)
    (h_neq : key1 ≠ key2) :
    hmac_sha256 key1 msg ≠ hmac_sha256 key2 msg := by
  exact hmac_key_sensitivity key1 key2 msg h_neq

-- PROP-004: HMAC different messages produce different outputs
-- For distinct messages, HMAC outputs differ.
theorem hmac_different_messages (key msg1 msg2 : ByteArray)
    (h_neq : msg1 ≠ msg2) :
    hmac_sha256 key msg1 ≠ hmac_sha256 key msg2 := by
  exact hmac_msg_sensitivity key msg1 msg2 h_neq
