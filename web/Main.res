// SPDX-License-Identifier: MPL-2.0
// Julia the Viper - Web module entry point

/** Add two numbers together.
 * The fundamental operation in JtV's addition-only Data Language.
 */
let add = (a: int, b: int): int => {
  a + b
}

/** Entry point: demonstrate addition when run directly */
let () = {
  Js.log("Add 2 + 3 = " ++ Belt.Int.toString(add(2, 3)))
}
