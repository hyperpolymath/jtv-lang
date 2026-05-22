// SPDX-License-Identifier: MPL-2.0
// Julia the Viper - VS Code Extension
// Provides LSP client, run/debug/format commands for .jtv files

/** VS Code API bindings */
module Vscode = {
  type extensionContext
  type textEditor
  type terminal
  type disposable
  type uri = {fsPath: string}
  type textDocument = {uri: uri}
  type fileSystemWatcher

  @module("vscode")
  external showErrorMessage: string => unit = "window.showErrorMessage"

  @module("vscode")
  external getActiveTextEditor: unit => option<textEditor> = "window.activeTextEditor"

  /** Access the active text editor's document */
  @get external getDocument: textEditor => textDocument = "document"

  @module("vscode")
  external createTerminal: string => terminal = "window.createTerminal"

  @send external show: terminal => unit = "show"
  @send external sendText: (terminal, string) => unit = "sendText"

  @module("vscode")
  external registerCommand: (string, unit => unit) => disposable = "commands.registerCommand"

  @module("vscode")
  external getConfiguration: string => {..} = "workspace.getConfiguration"

  /** Type-safe accessor for workspace configuration values by key */
  @send external get: ({..}, string) => option<string> = "get"

  @module("vscode")
  external createFileSystemWatcher: string => fileSystemWatcher =
    "workspace.createFileSystemWatcher"

  /** Extension context subscriptions array */
  @get external getSubscriptions: extensionContext => array<disposable> = "subscriptions"
}

/** VS Code Language Client bindings */
module LanguageClient = {
  type t
  type serverOptions = {
    run: {"command": string},
    debug: {"command": string},
  }
  type documentFilter = {
    scheme: string,
    language: string,
  }
  type synchronize = {fileEvents: Vscode.fileSystemWatcher}
  type clientOptions = {
    documentSelector: array<documentFilter>,
    synchronize: synchronize,
  }

  @module("vscode-languageclient/node") @new
  external make: (string, string, serverOptions, clientOptions) => t = "LanguageClient"

  @send external start: t => unit = "start"
  @send external stop: t => Js.Promise2.t<unit> = "stop"
}

/** Module-level mutable reference to the LSP client */
let client: ref<option<LanguageClient.t>> = ref(None)

/** Run the active .jtv file in a terminal */
let runFile = (): unit => {
  switch Vscode.getActiveTextEditor() {
  | None => Vscode.showErrorMessage("No active editor")
  | Some(editor) => {
      let filePath = Vscode.getDocument(editor).uri.fsPath
      let terminal = Vscode.createTerminal("Julia the Viper")
      Vscode.show(terminal)
      Vscode.sendText(terminal, `jtv-cli run "${filePath}"`)
    }
  }
}

/** Debug the active .jtv file in a terminal */
let debugFile = (): unit => {
  switch Vscode.getActiveTextEditor() {
  | None => Vscode.showErrorMessage("No active editor")
  | Some(editor) => {
      let filePath = Vscode.getDocument(editor).uri.fsPath
      let terminal = Vscode.createTerminal("JtV Debug")
      Vscode.show(terminal)
      Vscode.sendText(terminal, `jtv-debug "${filePath}"`)
    }
  }
}

/** Format the active .jtv file in a terminal */
let formatFile = (): unit => {
  switch Vscode.getActiveTextEditor() {
  | None => Vscode.showErrorMessage("No active editor")
  | Some(editor) => {
      let filePath = Vscode.getDocument(editor).uri.fsPath
      let terminal = Vscode.createTerminal("JtV Format")
      Vscode.show(terminal)
      Vscode.sendText(terminal, `jtv-cli format "${filePath}"`)
    }
  }
}

/** Activate the extension: start LSP client and register commands */
let activate = (context: Vscode.extensionContext): unit => {
  Js.log("Julia the Viper extension activated")

  let config = Vscode.getConfiguration("jtv")
  let lspPath = switch Vscode.get(config, "lsp.path") {
  | Some(p) => p
  | None => "jtv-lsp"
  }

  let serverOptions: LanguageClient.serverOptions = {
    run: {"command": lspPath},
    debug: {"command": lspPath},
  }

  let clientOptions: LanguageClient.clientOptions = {
    documentSelector: [{scheme: "file", language: "jtv"}],
    synchronize: {
      fileEvents: Vscode.createFileSystemWatcher("**/*.jtv"),
    },
  }

  let lspClient = LanguageClient.make(
    "jtv",
    "Julia the Viper Language Server",
    serverOptions,
    clientOptions,
  )

  LanguageClient.start(lspClient)
  client := Some(lspClient)

  let subs = Vscode.getSubscriptions(context)
  ignore(Js.Array2.push(subs, Vscode.registerCommand("jtv.run", runFile)))
  ignore(Js.Array2.push(subs, Vscode.registerCommand("jtv.debug", debugFile)))
  ignore(Js.Array2.push(subs, Vscode.registerCommand("jtv.format", formatFile)))
}

/** Deactivate the extension: stop the LSP client */
let deactivate = (): option<Js.Promise2.t<unit>> => {
  switch client.contents {
  | None => None
  | Some(c) => Some(LanguageClient.stop(c))
  }
}
