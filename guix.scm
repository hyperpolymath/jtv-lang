; SPDX-License-Identifier: MPL-2.0
;; guix.scm — GNU Guix package definition for julia-the-viper
;; Usage: guix shell -f guix.scm

(use-modules (guix packages)
             (guix build-system gnu)
             (guix licenses))

(package
  (name "julia-the-viper")
  (version "0.1.0")
  (source #f)
  (build-system gnu-build-system)
  (synopsis "julia-the-viper")
  (description "julia-the-viper — part of the hyperpolymath ecosystem.")
  (home-page "https://github.com/hyperpolymath/julia-the-viper")
  (license ((@@ (guix licenses) license) "MPL-2.0"
             "https://github.com/hyperpolymath/palimpsest-license")))
