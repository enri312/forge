"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = require("vscode");
function activate(context) {
    console.log('FORGE Build System extension is now active!');
    // Inyectar la configuración del schema asociando al archivo local del VSIX
    const config = vscode.workspace.getConfiguration('evenBetterToml');
    const associations = config.get('schema.associations') || {};
    const schemaUri = vscode.Uri.joinPath(context.extensionUri, 'schemas', 'forge.schema.json').toString();
    if (associations['forge.toml'] !== schemaUri) {
        associations['forge.toml'] = schemaUri;
        associations['tests/**/forge.toml'] = schemaUri;
        config.update('schema.associations', associations, vscode.ConfigurationTarget.Workspace)
            .then(() => console.log('✅ Injected FORGE schema automatically'));
    }
    let buildCmd = vscode.commands.registerCommand('forge.build', () => {
        runForgeCommand('build');
    });
    let runCmd = vscode.commands.registerCommand('forge.run', () => {
        runForgeCommand('run');
    });
    let testCmd = vscode.commands.registerCommand('forge.test', () => {
        runForgeCommand('test');
    });
    context.subscriptions.push(buildCmd, runCmd, testCmd);
}
function runForgeCommand(command) {
    // Buscar si ya existe una terminal de forge
    let terminal = vscode.window.terminals.find(t => t.name === 'FORGE');
    if (!terminal) {
        terminal = vscode.window.createTerminal('FORGE');
    }
    terminal.show();
    terminal.sendText(`forge ${command}`);
}
function deactivate() { }
//# sourceMappingURL=extension.js.map