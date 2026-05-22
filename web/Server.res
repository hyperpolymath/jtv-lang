// SPDX-License-Identifier: MPL-2.0
// Deno server for Julia the Viper PWA

/** Deno HTTP request type */
type request = {url: string}

/** Deno HTTP response type */
type response

/** Options for Deno.serve */
type serveOptions = {port: int}

/** Options for serveDir from the Deno standard library */
type serveDirOptions = {
  fsRoot: string,
  showDirListing: bool,
}

/** Deno.serve FFI binding */
@module("Deno") @val
external serve: (serveOptions, request => response) => unit = "serve"

/** serveDir from Deno standard library - serves static files */
@module("https://deno.land/std@0.224.0/http/file_server.ts")
external serveDir: (request, serveDirOptions) => response = "serveDir"

/** URL constructor FFI binding */
type url = {pathname: string}
@new external makeUrl: string => url = "URL"

/** Port the PWA server listens on */
let port = 8000

/** Request handler: routes /pkg/ to parent directory, everything else to web root */
let handler = (req: request): response => {
  let pathname = makeUrl(req.url).pathname

  if Js.String2.startsWith(pathname, "/pkg/") {
    serveDir(
      req,
      {
        fsRoot: "../",
        showDirListing: false,
      },
    )
  } else {
    serveDir(
      req,
      {
        fsRoot: ".",
        showDirListing: false,
      },
    )
  }
}

/** Start the server and log the listening address */
let () = {
  serve({port: port}, handler)
  Js.log(
    "Julia the Viper PWA server running at http://localhost:" ++
    Belt.Int.toString(port) ++
    "/",
  )
  Js.log("Serving WASM from ../pkg/")
}
