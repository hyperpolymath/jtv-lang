// SPDX-License-Identifier: MPL-2.0
// Julia the Viper - Code Analyzer for Python/JavaScript/Ruby
// Analyzes legacy code for termination guarantees, purity, and complexity

/** Supported analysis target languages */
type language = [#python | #javascript | #ruby]

/** Severity levels for analysis warnings */
type severity = [#error | #warning | #info]

/** Types of arithmetic data operations */
type dataOperationType = [#add | #subtract | #multiply | #divide | #other]

/** Types of control flow nodes */
type controlFlowType = [#"if" | #"while" | #"for" | #function_call | #"return"]

/** Types of suggestions the analyzer can produce */
type suggestionType = [#extract_to_jtv | #mark_pure | #bound_loop]

/** Complexity metrics for a single function */
type complexityAnalysis = {
  cyclomaticComplexity: int,
  cognitiveComplexity: int,
  loops: int,
  conditionals: int,
  recursion: bool,
}

/** A single data operation found in the source */
type dataOperation = {
  @as("type") operationType: dataOperationType,
  line: int,
  safe: bool,
}

/** A single control flow node found in the source */
type controlFlowNode = {
  @as("type") nodeType: controlFlowType,
  line: int,
  terminating: bool,
}

/** Analysis result for a single function */
type functionAnalysis = {
  name: string,
  line: int,
  mutable isPure: bool,
  mutable terminates: bool,
  complexity: complexityAnalysis,
  dataOperations: array<dataOperation>,
  controlFlow: array<controlFlowNode>,
}

/** A warning produced during analysis */
type warning = {
  severity: severity,
  message: string,
  line: int,
  suggestion: option<string>,
}

/** A suggestion for improving the analyzed code */
type suggestion = {
  @as("type") suggestionType: suggestionType,
  message: string,
  line: int,
  codeSnippet: string,
  jtvEquivalent: option<string>,
}

/** Complete analysis result for a file */
type analysisResult = {
  file: string,
  language: language,
  mutable functions: array<functionAnalysis>,
  mutable warnings: array<warning>,
  mutable suggestions: array<suggestion>,
}

// -- FFI bindings for Deno APIs --

@module("Deno") @val
external readTextFile: string => Js.Promise2.t<string> = "readTextFile"

@module("Deno") @val
external denoArgs: array<string> = "args"

@module("Deno") @val
external denoExit: int => unit = "exit"

// -- Helper functions --

/** Check if a line is a function declaration for the given language */
let isFunctionDeclaration = (line: string, lang: language): bool => {
  switch lang {
  | #python => Js.String2.startsWith(line, "def ")
  | #javascript =>
    Js.String2.includes(line, "function ") || Js.String2.includes(line, "=> ")
  | #ruby => Js.String2.startsWith(line, "def ")
  }
}

/** Extract the function name from a declaration line */
let extractFunctionName = (line: string): string => {
  let re = %re("/(?:def|function)\s+(\w+)/")
  switch Js.Re.exec_(re, line) {
  | Some(result) =>
    switch Js.Nullable.toOption(Js.Re.captures(result)[1]) {
    | Some(name) => name
    | None => "anonymous"
    }
  | None => "anonymous"
  }
}

/** Convert a line to a JtV equivalent suggestion */
let convertToJtv = (line: string): string => {
  if Js.String2.includes(line, "*") {
    "// Use loop with addition in JtV: for i in 0..n { sum = sum + x }"
  } else if Js.String2.includes(line, "/") {
    "// Division not directly supported - use rational type or iterative subtraction"
  } else {
    line
    ->Js.String2.replaceByRe(%re("/\*/g"), "+")
    ->Js.String2.replaceByRe(%re("/\//g"), "+")
  }
}

/** Create a fresh complexity analysis with default values */
let makeComplexity = (): complexityAnalysis => {
  cyclomaticComplexity: 1,
  cognitiveComplexity: 0,
  loops: 0,
  conditionals: 0,
  recursion: false,
}

/** Analyze source code and produce an analysis result */
let analyzeCode = (code: string, filePath: string, lang: language): analysisResult => {
  let result: analysisResult = {
    file: filePath,
    language: lang,
    functions: [],
    warnings: [],
    suggestions: [],
  }

  let lines = Js.String2.split(code, "\n")
  let currentFunction = ref(None)
  let braceDepth = ref(0)

  lines->Belt.Array.forEachWithIndex((i, rawLine) => {
    let line = Js.String2.trim(rawLine)
    let lineNum = i + 1

    // Detect function declarations
    if isFunctionDeclaration(line, lang) {
      switch currentFunction.contents {
      | Some(func) =>
        result.functions = Belt.Array.concat(result.functions, [func])
      | None => ()
      }

      let funcName = extractFunctionName(line)
      currentFunction :=
        Some({
          name: funcName,
          line: lineNum,
          isPure: true,
          terminates: true,
          complexity: makeComplexity(),
          dataOperations: [],
          controlFlow: [],
        })
    }

    switch currentFunction.contents {
    | None => ()
    | Some(func) => {
        // Analyze data operations
        if Js.String2.includes(line, "+") && !Js.String2.includes(line, "++") {
          let op: dataOperation = {operationType: #add, line: lineNum, safe: true}
          ignore(
            Js.Array2.push(func.dataOperations, op),
          )
        } else if Js.String2.includes(line, "*") || Js.String2.includes(line, "/") {
          let opType = if Js.String2.includes(line, "*") {
            #multiply
          } else {
            #divide
          }
          let op: dataOperation = {operationType: opType, line: lineNum, safe: false}
          ignore(Js.Array2.push(func.dataOperations, op))

          let sug: suggestion = {
            suggestionType: #extract_to_jtv,
            message: "Consider extracting pure computation to JtV for guaranteed termination",
            line: lineNum,
            codeSnippet: line,
            jtvEquivalent: Some(convertToJtv(line)),
          }
          result.suggestions = Belt.Array.concat(result.suggestions, [sug])
        }

        // Detect control flow - conditionals
        if Js.String2.includes(line, "if") || Js.String2.includes(line, "else") {
          let cfNode: controlFlowNode = {nodeType: #"if", line: lineNum, terminating: true}
          ignore(Js.Array2.push(func.controlFlow, cfNode))
        }

        // Detect control flow - loops
        if Js.String2.includes(line, "while") || Js.String2.includes(line, "for") {
          func.terminates = false
          let loopType = if Js.String2.includes(line, "while") {
            #"while"
          } else {
            #"for"
          }
          let cfNode: controlFlowNode = {nodeType: loopType, line: lineNum, terminating: false}
          ignore(Js.Array2.push(func.controlFlow, cfNode))

          let warn: warning = {
            severity: #warning,
            message: "Unbounded loop detected - termination not guaranteed",
            line: lineNum,
            suggestion: Some("Add loop bound or extract to JtV with range-based iteration"),
          }
          result.warnings = Belt.Array.concat(result.warnings, [warn])
        }

        // Detect side effects
        if (
          Js.String2.includes(line, "print") ||
          Js.String2.includes(line, "console.log") ||
          Js.String2.includes(line, "puts") ||
          Js.String2.includes(line, "write") ||
          Js.String2.includes(line, "fetch") ||
          Js.String2.includes(line, "axios")
        ) {
          func.isPure = false

          let warn: warning = {
            severity: #info,
            message: "Function has side effects (I/O operations)",
            line: lineNum,
            suggestion: Some("Consider separating pure computation from side effects"),
          }
          result.warnings = Belt.Array.concat(result.warnings, [warn])
        }

        // Detect recursion
        if Js.String2.includes(line, func.name ++ "(") {
          func.terminates = false

          let warn: warning = {
            severity: #warning,
            message: "Recursive function - termination not guaranteed",
            line: lineNum,
            suggestion: Some("Convert to iterative form or prove termination"),
          }
          result.warnings = Belt.Array.concat(result.warnings, [warn])
        }

        // Track brace depth for function boundaries
        let openBraces =
          Js.String2.match_(line, %re("/{/g"))
          ->Belt.Option.mapWithDefault(0, Js.Array2.length)
        let closeBraces =
          Js.String2.match_(line, %re("/}/g"))
          ->Belt.Option.mapWithDefault(0, Js.Array2.length)
        braceDepth := braceDepth.contents + openBraces - closeBraces

        if braceDepth.contents == 0 {
          result.functions = Belt.Array.concat(result.functions, [func])
          currentFunction := None
        }
      }
    }
  })

  // Add final function if still in progress
  switch currentFunction.contents {
  | Some(func) =>
    result.functions = Belt.Array.concat(result.functions, [func])
  | None => ()
  }

  // Generate overall suggestions
  let pureFunctions = result.functions->Belt.Array.keep(f => f.isPure && f.terminates)
  if Belt.Array.length(pureFunctions) > 0 {
    let names = pureFunctions->Belt.Array.map(f => f.name)->Js.Array2.joinWith(", ")
    let count = Belt.Array.length(pureFunctions)->Belt.Int.toString
    let sug: suggestion = {
      suggestionType: #extract_to_jtv,
      message: count ++ " pure, terminating functions found - excellent candidates for JtV extraction",
      line: 0,
      codeSnippet: names,
      jtvEquivalent: None,
    }
    result.suggestions = Belt.Array.concat(result.suggestions, [sug])
  }

  let highComplexity =
    result.functions->Belt.Array.keep(f => f.complexity.cyclomaticComplexity > 10)
  if Belt.Array.length(highComplexity) > 0 {
    let count = Belt.Array.length(highComplexity)->Belt.Int.toString
    let warn: warning = {
      severity: #warning,
      message: count ++ " functions with high complexity - consider refactoring",
      line: 0,
      suggestion: None,
    }
    result.warnings = Belt.Array.concat(result.warnings, [warn])
  }

  result
}

/** Print a human-readable analysis report to the console */
let printReport = (result: analysisResult): unit => {
  Js.log("\n=== JtV Analysis Report: " ++ result.file ++ " ===\n")
  Js.log("Language: " ++ (result.language :> string))
  Js.log("Functions analyzed: " ++ Belt.Int.toString(Belt.Array.length(result.functions)) ++ "\n")

  let pure = result.functions->Belt.Array.keep(f => f.isPure)->Belt.Array.length
  let terminating = result.functions->Belt.Array.keep(f => f.terminates)->Belt.Array.length
  let total = Belt.Array.length(result.functions)

  Js.log(
    "Pure functions: " ++ Belt.Int.toString(pure) ++ "/" ++ Belt.Int.toString(total),
  )
  Js.log(
    "Terminating functions: " ++ Belt.Int.toString(terminating) ++ "/" ++ Belt.Int.toString(total) ++ "\n",
  )

  if Belt.Array.length(result.warnings) > 0 {
    Js.log("Warnings (" ++ Belt.Int.toString(Belt.Array.length(result.warnings)) ++ "):")
    result.warnings->Belt.Array.forEach(w => {
      let icon = switch w.severity {
      | #error => "[ERROR]"
      | #warning => "[WARN]"
      | #info => "[INFO]"
      }
      Js.log("  " ++ icon ++ " Line " ++ Belt.Int.toString(w.line) ++ ": " ++ w.message)
      switch w.suggestion {
      | Some(s) => Js.log("     -> " ++ s)
      | None => ()
      }
    })
    Js.log("")
  }

  if Belt.Array.length(result.suggestions) > 0 {
    Js.log("Suggestions (" ++ Belt.Int.toString(Belt.Array.length(result.suggestions)) ++ "):")
    result.suggestions->Belt.Array.forEach(s => {
      Js.log("  [TIP] Line " ++ Belt.Int.toString(s.line) ++ ": " ++ s.message)
      switch s.jtvEquivalent {
      | Some(equiv) => Js.log("     JtV: " ++ equiv)
      | None => ()
      }
    })
    Js.log("")
  }

  Js.log("Function Details:")
  result.functions->Belt.Array.forEach(func => {
    let pureIcon = if func.isPure { "Y" } else { "N" }
    let termIcon = if func.terminates { "Y" } else { "N" }
    Js.log("\n  " ++ func.name ++ " (line " ++ Belt.Int.toString(func.line) ++ ")")
    Js.log("    Pure: " ++ pureIcon ++ " | Terminates: " ++ termIcon)
    Js.log(
      "    Complexity: " ++ Belt.Int.toString(func.complexity.cyclomaticComplexity) ++ " (cyclomatic)",
    )
    Js.log(
      "    Loops: " ++
      Belt.Int.toString(func.complexity.loops) ++
      " | Conditionals: " ++
      Belt.Int.toString(func.complexity.conditionals),
    )
  })

  Js.log("\n=== End Report ===\n")
}

/** Analyze a file by reading it from disk (async, requires Deno) */
let analyzeFile = (filePath: string, lang: language): Js.Promise2.t<analysisResult> => {
  readTextFile(filePath)->Js.Promise2.then(code => {
    Js.Promise2.resolve(analyzeCode(code, filePath, lang))
  })
}
