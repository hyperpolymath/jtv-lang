;; SPDX-FileCopyrightText: 2026 Hyperpolymath
;; SPDX-License-Identifier: MPL-2.0
;;
;; ANCHOR.scope-arrest.2026-01-01.Jewell.scm  (julia-the-viper)
;;
;; Purpose: Freeze v1 as runnable core; prevent v2/spec ambition from destabilising v1.

(define anchor
  '((schema . "hyperpolymath.anchor/1")
    (repo . "hyperpolymath/julia-the-viper")
    (date . "2026-01-01")
    (authority . "repo-superintendent")
    (purpose . ("Freeze v1 as runnable core; prevent v2/spec ambition from destabilising v1."))
    (identity
      . ((project . "Julia the Viper (JtV)")
         (kind . "security language")
         (domain . "grammar-enforced separation (control vs data)")
         (one-sentence . "A Harvard-architecture language where control and data are syntactically separated to reduce injection and hidden-control channels.")))

    (semantic-anchor
      . ((policy . "dual")
         (reference-impl . ("v1 interpreter/compiler behavior is authoritative"))
         (formal-spec . ("SPEC.core.scm defines the Control/Data split and core evaluation"))
         (freeze . ("v1 must be completed before any v2 implementation work"))))

    (allowed-implementation-languages
      . ("Rust"))
    (quarantined-optional
      . ("WASM build"
         "Editor tooling"
         "Secondary analyzers"))
    (forbidden
      . ("Starting v2 implementation in f0"
         "Adding new runtime languages"
         "Changing the Control/Data split model"))

    (golden-path
      . ((smoke-test-command . "just build && jtv run examples/hello.jtv")
         (success-criteria . ("one example runs deterministically"
                              "conformance corpus asserts separation properties"
                              "CLI exit codes stable"))))

    (first-pass-directives
      . ("Create conformance programs that try to smuggle control into data; they must fail."
         "Document the minimal grammar + semantics for v1 only."
         "If multiple READMEs exist, make one canonical and mark others as mirrors."
         "Any analyzer not in Rust must be declared optional and non-authoritative."))

    (rsr
      . ((target-tier . "silver-now") (upgrade-path . "gold-after-f1")))))
