// File: EsmInterceptor/Dynamic/Dynamic.ts
// Defines the function responsible for creating the dynamic JavaScript module
// content that shims the `vscode` API for ES Module imports.

import type * as vscode from "vscode";

import DynamicModuleTemplateContent from "./DynamicTemplate.js";

/**
 * Creates the full JavaScript source code for a dynamic `vscode` module.
 *
 * It takes a template, an API key, and the actual `vscode` API object, then injects
 * the key and dynamically generates `export` statements for all properties on the API object.
 * This allows extensions using ES Modules (`import { window } from 'vscode'`) to receive the
 * correctly shimmed and context-aware API.
 *
 * @param ApiKey A unique key that identifies the specific instance of the `vscode` API for a given extension context.
 * @param VscodeApiInstance The actual, fully-constructed `vscode` API object to be exposed by the module.
 * @returns A string containing the complete JavaScript source code for the dynamic module.
 */
export const CreateDynamicVscodeModuleScript = (
	ApiKey: string,
	VscodeApiInstance: typeof vscode,
): string => {
	if (typeof ApiKey !== "string" || ApiKey.length === 0) {
		console.error(
			"[DynamicScriptGenerator] Invalid or empty API key provided. The resulting module will likely fail.",
		);
	}
	if (typeof VscodeApiInstance !== "object" || VscodeApiInstance === null) {
		console.error(
			"[DynamicScriptGenerator] Invalid API instance provided. Cannot generate export statements.",
		);
	}

	// Generate `export const ...` statements for every property on the API object.
	const ExportablePropertyNames = Object.keys(VscodeApiInstance);
	const ExportStatementsString = ExportablePropertyNames.map(
		(PropertyName) =>
			`export const ${PropertyName} = __apiInstance['${PropertyName}'];`,
	).join("\n");

	// Replace placeholders in the template with the runtime-generated content.
	let PopulatedScriptContent = DynamicModuleTemplateContent;
	PopulatedScriptContent = PopulatedScriptContent.replace(
		/__RUNTIME_API_KEY__/g,
		ApiKey,
	);
	PopulatedScriptContent = PopulatedScriptContent.replace(
		/__RUNTIME_EXPORT_STATEMENTS__/g,
		ExportStatementsString,
	);

	return PopulatedScriptContent;
};
