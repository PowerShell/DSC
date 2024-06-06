/*---------------------------------------------------------
 * Copyright (C) Microsoft Corporation. All rights reserved.
 *--------------------------------------------------------*/

import * as vscode from 'vscode';
import * as fs from 'fs';

export function activate(context: vscode.ExtensionContext) {

	const dsc_type_provider = vscode.languages.registerCompletionItemProvider({ language: 'yaml', pattern: '**/*.dsc.yaml' },
		{
			provideCompletionItems(document: vscode.TextDocument, position: vscode.Position) {

				const linePrefix = document.lineAt(position).text.slice(0, position.character);
				if (!linePrefix.endsWith('type:')) {
					return undefined;
				}

				let ps_cache_file_path: string = '';
				if (process.platform==='win32')
				{
					ps_cache_file_path = process.env.LocalAppData + '\\dsc\\PSAdapterCache.json';
				}
				else
				{
					ps_cache_file_path = process.env.HOME + '/.dsc/PSAdapterCache.json';
				}
				console.log("Using cache path " + ps_cache_file_path);

				if (!fs.existsSync(ps_cache_file_path)) {
					console.log("Cache file does not exist");
					return [];
				}

				const dataArray = JSON.parse(fs.readFileSync(ps_cache_file_path, 'utf-8'));
				var completions:vscode.CompletionItem[] = [];
				dataArray.ResourceCache.forEach(function (resouce: any) {
						
					const all_props_completion = new vscode.CompletionItem(resouce.Type);
					const key_props_completion = new vscode.CompletionItem(resouce.Type + " [keys only]");

					let all_props_comp_text: string = ' ' + resouce.Type;
					let key_props_comp_text: string = ' ' + resouce.Type;
					all_props_comp_text += '\nproperties:';
					key_props_comp_text += '\nproperties:';
					resouce.DscResourceInfo.Properties.forEach(function (prop: any) {
						all_props_comp_text += '\n  ' + prop.Name + ': ';
						if (prop.IsMandatory)
						{
							key_props_comp_text += '\n  ' + prop.Name + ': ';
						}
					});
					all_props_completion.insertText = new vscode.SnippetString(all_props_comp_text);
					key_props_completion.insertText = new vscode.SnippetString(key_props_comp_text);

					completions.push(all_props_completion);
					completions.push(key_props_completion);
				});

				return completions;
			}
		},
		':' // triggered whenever a ':' is being typed
	);

	context.subscriptions.push(dsc_type_provider);
}
