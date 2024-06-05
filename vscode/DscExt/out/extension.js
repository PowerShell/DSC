"use strict";
/*---------------------------------------------------------
 * Copyright (C) Microsoft Corporation. All rights reserved.
 *--------------------------------------------------------*/
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = void 0;
const vscode = require("vscode");
const fs = require("fs");
function activate(context) {
    const dsc_type_provider = vscode.languages.registerCompletionItemProvider({ language: 'yaml', pattern: '**/*.dsc.yaml' }, {
        provideCompletionItems(document, position) {
            const linePrefix = document.lineAt(position).text.slice(0, position.character);
            if (!linePrefix.endsWith('type:')) {
                return undefined;
            }
            let ps_cache_file_path = '';
            if (process.platform === 'win32') {
                ps_cache_file_path = process.env.LocalAppData + '\\dsc\\PSAdapterCache.json';
            }
            else {
                ps_cache_file_path = process.env.HOME + '/.dsc/PSAdapterCache.json';
            }
            console.log("Using cache path " + ps_cache_file_path);
            if (!fs.existsSync(ps_cache_file_path)) {
                console.log("Cache file does not exist");
                return [];
            }
            const dataArray = JSON.parse(fs.readFileSync(ps_cache_file_path, 'utf-8'));
            const all_props_completion = new vscode.CompletionItem(dataArray.ResourceCache[0].Type);
            const key_props_completion = new vscode.CompletionItem(dataArray.ResourceCache[0].Type + " [keys only]");
            let all_props_comp_text = ' ' + dataArray.ResourceCache[0].Type;
            let key_props_comp_text = ' ' + dataArray.ResourceCache[0].Type;
            all_props_comp_text += '\nproperties:';
            key_props_comp_text += '\nproperties:';
            dataArray.ResourceCache[0].DscResourceInfo.Properties.forEach(function (value) {
                all_props_comp_text += '\n  ' + value.Name + ':';
                if (value.IsMandatory) {
                    key_props_comp_text += '\n  ' + value.Name + ':';
                }
            });
            all_props_completion.insertText = new vscode.SnippetString(all_props_comp_text);
            key_props_completion.insertText = new vscode.SnippetString(key_props_comp_text);
            return [
                all_props_completion,
                key_props_completion
            ];
        }
    }, ':' // triggered whenever a ':' is being typed
    );
    context.subscriptions.push(dsc_type_provider);
}
exports.activate = activate;
//# sourceMappingURL=extension.js.map