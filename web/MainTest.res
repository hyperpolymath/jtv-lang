// SPDX-License-Identifier: MPL-2.0
// Julia the Viper - Web module tests

/** Deno test runner FFI binding */
@module("Deno") @val
external test: (string, unit => unit) => unit = "test"

/** Deno assertion FFI binding */
@module("@std/assert")
external assertEquals: ('a, 'a) => unit = "assertEquals"

/** Test that the add function correctly sums two integers */
let () = test("addTest", () => {
  assertEquals(Main.add(2, 3), 5)
})
