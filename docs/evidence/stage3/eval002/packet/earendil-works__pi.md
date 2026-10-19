## earendil-works__pi: 113 proposal(s), first 20 adjudicated

### earendil-works__pi 1: code:types.ts->index.ts
- E2 AgentToolResult declared at packages/agent/src/types.ts:424
- E4 AgentToolResult imported at packages/coding-agent/examples/extensions/subagent/index.ts:19 from "@earendil-works/pi-agent-core", resolved to packages/agent/src/index.ts, re-exported by barrel packages/agent/src/index.ts:5 from packages/agent/src/types.ts
- E5 AgentToolResult fields read: content (packages/coding-agent/examples/extensions/subagent/index.ts:101), details (packages/coding-agent/examples/extensions/subagent/index.ts:560), usage (packages/coding-agent/examples/extensions/subagent/index.ts:365)
- E5 AgentToolResult stoplisted reads: (none)
```
// packages/agent/src/types.ts:424
export interface AgentToolResult<T = JsonValue | undefined> {
	/** Text or image content returned to the model. */
	content: (TextContent | ImageContent)[];
	/** Arbitrary structured details for logs or UI rendering. */
	details: T;
	/**
	 * Machine-readable result matching the tool's `outputSchema`, for programmatic callers. Not sent
	 * to the model; `content` remains the model-facing result.
	 */
	structuredContent?: JsonValue;
	/** Usage from the final tool execution itself, if available. Not used for main LLM context accounting. */
	usage?: Usage;
```
```
// packages/coding-agent/examples/extensions/subagent/index.ts:101
			const filePath = shortenPath(rawPath);
			const content = (args.content || "") as string;
			const lines = content.split("\n").length;
```
```
// packages/coding-agent/examples/extensions/subagent/index.ts:560
								// Combine completed results with current streaming result
								const currentResult = partial.details?.results[0];
								if (currentResult) {
```
```
// packages/coding-agent/examples/extensions/subagent/index.ts:365
					if (msg.role === "assistant") {
						currentResult.usage.turns++;
						const usage = msg.usage;
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 2: code:types.ts->agent-session.ts
- E2 AfterToolCallContext declared at packages/agent/src/types.ts:119
- E2 AfterToolCallResult declared at packages/agent/src/types.ts:92
- E2 AgentContext declared at packages/agent/src/types.ts:502
- E2 AgentState declared at packages/agent/src/types.ts:382
- E2 AgentToolCallOutcome declared at packages/agent/src/types.ts:449
- E2 BeforeToolCallContext declared at packages/agent/src/types.ts:107
- E4 AfterToolCallContext imported at packages/coding-agent/src/core/agent-session.ts:18 from "@earendil-works/pi-agent-core", resolved to packages/agent/src/index.ts, re-exported by barrel packages/agent/src/index.ts:5 from packages/agent/src/types.ts
- E4 AfterToolCallResult imported at packages/coding-agent/src/core/agent-session.ts:18 from "@earendil-works/pi-agent-core", resolved to packages/agent/src/index.ts, re-exported by barrel packages/agent/src/index.ts:5 from packages/agent/src/types.ts
- E4 AgentContext imported at packages/coding-agent/src/core/agent-session.ts:18 from "@earendil-works/pi-agent-core", resolved to packages/agent/src/index.ts, re-exported by barrel packages/agent/src/index.ts:5 from packages/agent/src/types.ts
- E4 AgentState imported at packages/coding-agent/src/core/agent-session.ts:18 from "@earendil-works/pi-agent-core", resolved to packages/agent/src/index.ts, re-exported by barrel packages/agent/src/index.ts:5 from packages/agent/src/types.ts
- E4 AgentToolCallOutcome imported at packages/coding-agent/src/core/agent-session.ts:18 from "@earendil-works/pi-agent-core", resolved to packages/agent/src/index.ts, re-exported by barrel packages/agent/src/index.ts:5 from packages/agent/src/types.ts
- E4 BeforeToolCallContext imported at packages/coding-agent/src/core/agent-session.ts:18 from "@earendil-works/pi-agent-core", resolved to packages/agent/src/index.ts, re-exported by barrel packages/agent/src/index.ts:5 from packages/agent/src/types.ts
- E5 AfterToolCallContext fields read: args (packages/coding-agent/src/core/agent-session.ts:1343), context (packages/coding-agent/src/core/agent-session.ts:796), isError (packages/coding-agent/src/core/agent-session.ts:722)
- E5 AfterToolCallContext stoplisted reads: result
- E5 AfterToolCallResult fields read: content (packages/coding-agent/src/core/agent-session.ts:697), details (packages/coding-agent/src/core/agent-session.ts:698), isError (packages/coding-agent/src/core/agent-session.ts:722), structuredContent (packages/coding-agent/src/core/agent-session.ts:699), usage (packages/coding-agent/src/core/agent-session.ts:701)
- E5 AfterToolCallResult stoplisted reads: (none)
- E5 AgentContext fields read: messages (packages/coding-agent/src/core/agent-session.ts:585), tools (packages/coding-agent/src/core/agent-session.ts:752)
- E5 AgentContext stoplisted reads: (none)
- E5 AgentState fields read: errorMessage (packages/coding-agent/src/core/agent-session.ts:1969), isStreaming (packages/coding-agent/src/core/agent-session.ts:2087), model (packages/coding-agent/src/core/agent-session.ts:617), systemPrompt (packages/coding-agent/src/core/agent-session.ts:3495), thinkingLevel (packages/coding-agent/src/core/agent-session.ts:587)
- E5 AgentState stoplisted reads: (none)
- E5 AgentToolCallOutcome fields read: durationMs (packages/coding-agent/src/core/agent-session.ts:1362), isError (packages/coding-agent/src/core/agent-session.ts:722)
- E5 AgentToolCallOutcome stoplisted reads: result
- E5 BeforeToolCallContext fields read: args (packages/coding-agent/src/core/agent-session.ts:1343), context (packages/coding-agent/src/core/agent-session.ts:796)
- E5 BeforeToolCallContext stoplisted reads: (none)
```
// packages/agent/src/types.ts:107
export interface BeforeToolCallContext {
	/** The assistant message that requested the tool call. */
	assistantMessage: AssistantMessage;
	/** The raw tool call block from `assistantMessage.content`. */
	toolCall: AgentToolCall;
	/** Validated tool arguments for the target tool schema. */
	args: unknown;
	/** Current agent context at the time the tool call is prepared. */
	context: AgentContext;
}

/** Context passed to `afterToolCall`. */
```
```
// packages/coding-agent/src/core/agent-session.ts:1343
				toolName: event.toolName,
				args: event.args,
			};
```
```
// packages/coding-agent/src/core/agent-session.ts:796
				const canonicalContext = {
					...request.context,
					messages: projection.messages,
```
```
// packages/coding-agent/src/core/agent-session.ts:722
			structuredContent: hookResult ? hookResult.structuredContent : result.structuredContent,
			isError: hookResult?.isError ?? isError,
			usage: hookResult?.usage,
```
```
// packages/coding-agent/src/core/agent-session.ts:697
					input: args as Record<string, unknown>,
					content: result.content,
					details: result.details,
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 3: code:types.ts->nested-tool-calls.ts
- E2 AgentToolCallOutcome declared at packages/agent/src/types.ts:449
- E2 AgentToolResult declared at packages/agent/src/types.ts:424
- E4 AgentToolCallOutcome imported at packages/coding-agent/src/core/nested-tool-calls.ts:11 from "@earendil-works/pi-agent-core", resolved to packages/agent/src/index.ts, re-exported by barrel packages/agent/src/index.ts:5 from packages/agent/src/types.ts
- E4 AgentToolResult imported at packages/coding-agent/src/core/nested-tool-calls.ts:11 from "@earendil-works/pi-agent-core", resolved to packages/agent/src/index.ts, re-exported by barrel packages/agent/src/index.ts:5 from packages/agent/src/types.ts
- E5 AgentToolCallOutcome fields read: durationMs (packages/coding-agent/src/core/nested-tool-calls.ts:82), isError (packages/coding-agent/src/core/nested-tool-calls.ts:237)
- E5 AgentToolCallOutcome stoplisted reads: result
- E5 AgentToolResult fields read: content (packages/coding-agent/src/core/nested-tool-calls.ts:155), isError (packages/coding-agent/src/core/nested-tool-calls.ts:237), usage (packages/coding-agent/src/core/nested-tool-calls.ts:88)
- E5 AgentToolResult stoplisted reads: (none)
```
// packages/agent/src/types.ts:424
export interface AgentToolResult<T = JsonValue | undefined> {
	/** Text or image content returned to the model. */
	content: (TextContent | ImageContent)[];
	/** Arbitrary structured details for logs or UI rendering. */
	details: T;
	/**
	 * Machine-readable result matching the tool's `outputSchema`, for programmatic callers. Not sent
	 * to the model; `content` remains the model-facing result.
	 */
	structuredContent?: JsonValue;
	/** Usage from the final tool execution itself, if available. Not used for main LLM context accounting. */
	usage?: Usage;
```
```
// packages/coding-agent/src/core/nested-tool-calls.ts:82
		record.status = isError ? "error" : "ok";
		record.durationMs = Math.round(performance.now() - (this.startedAt.get(record) ?? performance.now()));
		this.startedAt.delete(record);
```
```
// packages/coding-agent/src/core/nested-tool-calls.ts:237

		scope.recorder.finish(record, outcome.isError, textOf(outcome.result));
		// Nested results are not persisted, so their usage is only counted through the recorder.
```
```
// packages/coding-agent/src/core/nested-tool-calls.ts:155
function textOf(result: AgentToolResult<unknown>): string {
	return (result.content ?? [])
		.filter((block): block is TextContent => block.type === "text")
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 4: code:types.ts->tool-definition-wrapper.ts
- E2 AgentTool declared at packages/agent/src/types.ts:466
- E4 AgentTool imported at packages/coding-agent/src/core/tools/tool-definition-wrapper.ts:1 from "@earendil-works/pi-agent-core", resolved to packages/agent/src/index.ts, re-exported by barrel packages/agent/src/index.ts:5 from packages/agent/src/types.ts
- E5 AgentTool fields read: execute (packages/coding-agent/src/core/tools/tool-definition-wrapper.ts:22), executionMode (packages/coding-agent/src/core/tools/tool-definition-wrapper.ts:20), outputSchema (packages/coding-agent/src/core/tools/tool-definition-wrapper.ts:17), prepareArguments (packages/coding-agent/src/core/tools/tool-definition-wrapper.ts:19)
- E5 AgentTool stoplisted reads: label
```
// packages/agent/src/types.ts:466
export interface AgentTool<TParameters extends TSchema = TSchema, TDetails = any> extends Tool<TParameters> {
	/** Human-readable label for UI display. */
	label: string;
	/**
	 * Optional compatibility shim for raw tool-call arguments before schema validation.
	 * Must return an object that matches `TParameters`.
	 */
	prepareArguments?: (args: unknown) => Static<TParameters>;
	/**
	 * JSON Schema of `structuredContent` in successful results. Tools that declare it should always
	 * set `structuredContent`.
	 */
```
```
// packages/coding-agent/src/core/tools/tool-definition-wrapper.ts:22
		execute: (toolCallId, params, signal, onUpdate, ctx?: ExtensionToolContext) =>
			definition.execute(
				toolCallId,
```
```
// packages/coding-agent/src/core/tools/tool-definition-wrapper.ts:20
		prepareArguments: definition.prepareArguments,
		executionMode: definition.executionMode,
		execute: (toolCallId, params, signal, onUpdate, ctx?: ExtensionToolContext) =>
```
```
// packages/coding-agent/src/core/tools/tool-definition-wrapper.ts:17
		parameters: definition.parameters,
		outputSchema: definition.outputSchema,
		constrainedSampling: definition.constrainedSampling,
```
```
// packages/coding-agent/src/core/tools/tool-definition-wrapper.ts:19
		constrainedSampling: definition.constrainedSampling,
		prepareArguments: definition.prepareArguments,
		executionMode: definition.executionMode,
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 5: code:types.ts->execute.ts
- E2 AgentTool declared at packages/agent/src/types.ts:466
- E2 AgentToolCallOutcome declared at packages/agent/src/types.ts:449
- E2 AgentToolResult declared at packages/agent/src/types.ts:424
- E4 AgentTool imported at packages/coding-agent/src/extensions/codemode/execute.ts:6 from "@earendil-works/pi-agent-core", resolved to packages/agent/src/index.ts, re-exported by barrel packages/agent/src/index.ts:5 from packages/agent/src/types.ts
- E4 AgentToolCallOutcome imported at packages/coding-agent/src/extensions/codemode/execute.ts:6 from "@earendil-works/pi-agent-core", resolved to packages/agent/src/index.ts, re-exported by barrel packages/agent/src/index.ts:5 from packages/agent/src/types.ts
- E4 AgentToolResult imported at packages/coding-agent/src/extensions/codemode/execute.ts:6 from "@earendil-works/pi-agent-core", resolved to packages/agent/src/index.ts, re-exported by barrel packages/agent/src/index.ts:5 from packages/agent/src/types.ts
- E5 AgentTool fields read: execute (packages/coding-agent/src/extensions/codemode/execute.ts:530), outputSchema (packages/coding-agent/src/extensions/codemode/execute.ts:430)
- E5 AgentTool stoplisted reads: (none)
- E5 AgentToolCallOutcome fields read: durationMs (packages/coding-agent/src/extensions/codemode/execute.ts:490), isError (packages/coding-agent/src/extensions/codemode/execute.ts:432), toolCall (packages/coding-agent/src/extensions/codemode/execute.ts:489)
- E5 AgentToolCallOutcome stoplisted reads: result
- E5 AgentToolResult fields read: content (packages/coding-agent/src/extensions/codemode/execute.ts:76), isError (packages/coding-agent/src/extensions/codemode/execute.ts:432), structuredContent (packages/coding-agent/src/extensions/codemode/execute.ts:430), usage (packages/coding-agent/src/extensions/codemode/execute.ts:727)
- E5 AgentToolResult stoplisted reads: (none)
```
// packages/agent/src/types.ts:424
export interface AgentToolResult<T = JsonValue | undefined> {
	/** Text or image content returned to the model. */
	content: (TextContent | ImageContent)[];
	/** Arbitrary structured details for logs or UI rendering. */
	details: T;
	/**
	 * Machine-readable result matching the tool's `outputSchema`, for programmatic callers. Not sent
	 * to the model; `content` remains the model-facing result.
	 */
	structuredContent?: JsonValue;
	/** Usage from the final tool execution itself, if available. Not used for main LLM context accounting. */
	usage?: Usage;
```
```
// packages/coding-agent/src/extensions/codemode/execute.ts:530
		const store = ctx ? readCodemodeStore(ctx.sessionManager.getBranch()) : {};
		result = await sandbox.execute(code, { signal, store });
	} finally {
```
```
// packages/coding-agent/src/extensions/codemode/execute.ts:430
	const { result } = outcome;
	if (tool.outputSchema && result.structuredContent !== undefined) return result.structuredContent;
	const text = textOf(result);
```
```
// packages/coding-agent/src/extensions/codemode/execute.ts:490
			record.id = outcome.toolCall.id;
			record.durationMs = performance.now() - callStartedAt;
			if (outcome.isError) {
```
```
// packages/coding-agent/src/extensions/codemode/execute.ts:432
	const text = textOf(result);
	if (outcome.isError) throw new Error(text || `Tool "${tool.name}" failed`);
	return text;
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 6: code:types.ts->tools.ts
- E2 AgentToolResult declared at packages/agent/src/types.ts:424
- E4 AgentToolResult imported at packages/coding-agent/src/extensions/mcp/tools.ts:14 from "@earendil-works/pi-agent-core", resolved to packages/agent/src/index.ts, re-exported by barrel packages/agent/src/index.ts:5 from packages/agent/src/types.ts
- E5 AgentToolResult fields read: content (packages/coding-agent/src/extensions/mcp/tools.ts:274), details (packages/coding-agent/src/extensions/mcp/tools.ts:385), isError (packages/coding-agent/src/extensions/mcp/tools.ts:275)
- E5 AgentToolResult stoplisted reads: (none)
```
// packages/agent/src/types.ts:424
export interface AgentToolResult<T = JsonValue | undefined> {
	/** Text or image content returned to the model. */
	content: (TextContent | ImageContent)[];
	/** Arbitrary structured details for logs or UI rendering. */
	details: T;
	/**
	 * Machine-readable result matching the tool's `outputSchema`, for programmatic callers. Not sent
	 * to the model; `content` remains the model-facing result.
	 */
	structuredContent?: JsonValue;
	/** Usage from the final tool execution itself, if available. Not used for main LLM context accounting. */
	usage?: Usage;
```
```
// packages/coding-agent/src/extensions/mcp/tools.ts:274
	const converted: (TextContent | ImageContent)[] =
		result.content.length > 0 ? await toModelContent(server, result.content, options) : toLlmContent(result);
	if (result.isError && textOf(converted) === "") {
```
```
// packages/coding-agent/src/extensions/mcp/tools.ts:385
				);
				const fullOutputPath = (result.details as McpToolDetails | undefined)?.fullOutputPath;
				if (fullOutputPath) component.addChild(new Text(theme.fg("muted", `Full output: ${fullOutputPath}`), 0, 0));
```
```
// packages/coding-agent/src/extensions/mcp/tools.ts:275
		result.content.length > 0 ? await toModelContent(server, result.content, options) : toLlmContent(result);
	if (result.isError && textOf(converted) === "") {
		converted.push({ type: "text", text: `MCP tool ${server}/${tool} returned an error` });
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 7: code:types.ts->index.ts#2
- E2 OAuthCredentials declared at packages/ai/src/auth/types.ts:24
- E4 OAuthCredentials imported at packages/coding-agent/examples/extensions/custom-provider-anthropic/index.ts:26 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:24 from packages/ai/src/auth/types.ts
- E5 OAuthCredentials fields read: access (packages/coding-agent/examples/extensions/custom-provider-anthropic/index.ts:612), refresh (packages/coding-agent/examples/extensions/custom-provider-anthropic/index.ts:137)
- E5 OAuthCredentials stoplisted reads: (none)
```
// packages/ai/src/auth/types.ts:24
export interface OAuthCredentials {
	refresh: string;
	access: string;
	expires: number;
	[key: string]: unknown;
}

/** Stored canonical OAuth credential. */
export interface OAuthCredential extends OAuthCredentials {
	type: "oauth";
}

```
```
// packages/coding-agent/examples/extensions/custom-provider-anthropic/index.ts:612
			refreshToken: refreshAnthropicToken,
			getApiKey: (cred) => cred.access,
		},
```
```
// packages/coding-agent/examples/extensions/custom-provider-anthropic/index.ts:137
			client_id: CLIENT_ID,
			refresh_token: credentials.refresh,
		}),
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 8: code:types.ts->model-registry.ts
- E2 AuthResult declared at packages/ai/src/auth/types.ts:104
- E4 AuthResult imported at packages/coding-agent/src/core/model-registry.ts:1 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:24 from packages/ai/src/auth/types.ts
- E5 AuthResult fields read: auth (packages/coding-agent/src/core/model-registry.ts:101), env (packages/coding-agent/src/core/model-registry.ts:104)
- E5 AuthResult stoplisted reads: (none)
```
// packages/ai/src/auth/types.ts:104
export interface AuthResult {
	auth: ModelAuth;
	/** Provider-scoped environment/config values resolved from credentials and ambient context. */
	env?: ProviderEnv;
	/** Human-readable label for status UI: "ANTHROPIC_API_KEY", "OAuth", "~/.aws/credentials". */
	source?: string;
}

export interface AuthCheck {
	source?: string;
	type: "api_key" | "oauth";
}
```
```
// packages/coding-agent/src/core/model-registry.ts:101
				ok: true,
				apiKey: resolution.auth.apiKey,
				headers: resolution.auth.headers,
```
```
// packages/coding-agent/src/core/model-registry.ts:104
				...(resolution.auth.baseUrl ? { baseUrl: resolution.auth.baseUrl } : {}),
				env: resolution.env,
			};
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 9: code:types.ts->model-runtime.ts
- E2 AuthResult declared at packages/ai/src/auth/types.ts:104
- E4 AuthResult imported at packages/coding-agent/src/core/model-runtime.ts:2 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:24 from packages/ai/src/auth/types.ts
- E5 AuthResult fields read: auth (packages/coding-agent/src/core/model-runtime.ts:404), env (packages/coding-agent/src/core/model-runtime.ts:247), source (packages/coding-agent/src/core/model-runtime.ts:655)
- E5 AuthResult stoplisted reads: (none)
```
// packages/ai/src/auth/types.ts:104
export interface AuthResult {
	auth: ModelAuth;
	/** Provider-scoped environment/config values resolved from credentials and ambient context. */
	env?: ProviderEnv;
	/** Human-readable label for status UI: "ANTHROPIC_API_KEY", "OAuth", "~/.aws/credentials". */
	source?: string;
}

export interface AuthCheck {
	source?: string;
	type: "api_key" | "oauth";
}
```
```
// packages/coding-agent/src/core/model-runtime.ts:404
			const storedProviders = new Set(this.snapshot.storedProviders);
			const authByProvider = new Map(this.snapshot.auth);
			if (auth) {
```
```
// packages/coding-agent/src/core/model-runtime.ts:247
			providers,
			process.env.PI_OFFLINE === undefined,
		);
```
```
// packages/coding-agent/src/core/model-runtime.ts:655
		const check = this.snapshot.auth.get(providerId);
		return check ? { configured: true, source: "environment", label: check.source } : { configured: false };
	}
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 10: code:types.ts->provider-composer.ts
- E2 AuthResult declared at packages/ai/src/auth/types.ts:104
- E2 ModelAuth declared at packages/ai/src/auth/types.ts:7
- E4 AuthResult imported at packages/coding-agent/src/core/provider-composer.ts:1 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:24 from packages/ai/src/auth/types.ts
- E4 ModelAuth imported at packages/coding-agent/src/core/provider-composer.ts:1 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:24 from packages/ai/src/auth/types.ts
- E5 AuthResult fields read: auth (packages/coding-agent/src/core/provider-composer.ts:423), env (packages/coding-agent/src/core/provider-composer.ts:411), source (packages/coding-agent/src/core/provider-composer.ts:443)
- E5 AuthResult stoplisted reads: (none)
- E5 ModelAuth fields read: apiKey (packages/coding-agent/src/core/provider-composer.ts:316), baseUrl (packages/coding-agent/src/core/provider-composer.ts:222), headers (packages/coding-agent/src/core/provider-composer.ts:313)
- E5 ModelAuth stoplisted reads: (none)
```
// packages/ai/src/auth/types.ts:7
export interface ModelAuth {
	apiKey?: string;
	headers?: ProviderHeaders;
	baseUrl?: string;
}

/**
 * Stored api-key credential. `env` holds provider-scoped environment/config
 * values such as Cloudflare account/gateway ids.
 */
export interface ApiKeyCredential {
	type: "api_key";
```
```
// packages/coding-agent/src/core/provider-composer.ts:423
): ApiKeyAuth | undefined {
	const inherited = base?.auth.apiKey;
	const rawKey = configuredApiKey(config, extension);
```
```
// packages/coding-agent/src/core/provider-composer.ts:411
		if (env[name] !== undefined) continue;
		const value = await ctx.env(name);
		if (value !== undefined) env[name] = value;
```
```
// packages/coding-agent/src/core/provider-composer.ts:443
				const resolved = await inherited?.resolve(input);
				return resolved ? { type: "api_key", source: resolved.source } : undefined;
			}
```
```
// packages/coding-agent/src/core/provider-composer.ts:316
		!hasOverrides &&
		!config.apiKey &&
		!config.oauth &&
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 11: code:types.ts->provider.ts
- E2 AuthResult declared at packages/ai/src/auth/types.ts:104
- E4 AuthResult imported at packages/coding-agent/src/extensions/llama/provider.ts:1 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:24 from packages/ai/src/auth/types.ts
- E5 AuthResult fields read: env (packages/coding-agent/src/extensions/llama/provider.ts:27), source (packages/coding-agent/src/extensions/llama/provider.ts:44)
- E5 AuthResult stoplisted reads: (none)
```
// packages/ai/src/auth/types.ts:104
export interface AuthResult {
	auth: ModelAuth;
	/** Provider-scoped environment/config values resolved from credentials and ambient context. */
	env?: ProviderEnv;
	/** Human-readable label for status UI: "ANTHROPIC_API_KEY", "OAuth", "~/.aws/credentials". */
	source?: string;
}

export interface AuthCheck {
	source?: string;
	type: "api_key" | "oauth";
}
```
```
// packages/coding-agent/src/extensions/llama/provider.ts:27
function credentialServerUrl(credential: ApiKeyCredential | undefined): string | undefined {
	const value = credential?.env?.LLAMA_BASE_URL;
	return typeof value === "string" && value.trim() ? normalizeLlamaServerUrl(value) : undefined;
```
```
// packages/coding-agent/src/extensions/llama/provider.ts:44
	// Unloaded presets are routable only when llama.cpp router autoload can load them on first use.
	return routerAutoload && model.status.value === "unloaded" && !model.status.failed && model.source === "preset";
}
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 12: code:types.ts->utilities.ts
- E2 OAuthCredentials declared at packages/ai/src/auth/types.ts:24
- E4 OAuthCredentials imported at packages/coding-agent/test/utilities.ts:10 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:24 from packages/ai/src/auth/types.ts
- E5 OAuthCredentials fields read: expires (packages/coding-agent/test/utilities.ts:95), refresh (packages/coding-agent/test/utilities.ts:96)
- E5 OAuthCredentials stoplisted reads: (none)
```
// packages/ai/src/auth/types.ts:24
export interface OAuthCredentials {
	refresh: string;
	access: string;
	expires: number;
	[key: string]: unknown;
}

/** Stored canonical OAuth credential. */
export interface OAuthCredential extends OAuthCredentials {
	type: "oauth";
}

```
```
// packages/coding-agent/test/utilities.ts:95
		let credential = entry;
		if (Date.now() >= credential.expires) {
			credential = await oauth.refresh(credential, new AbortController().signal);
```
```
// packages/coding-agent/test/utilities.ts:96
		if (Date.now() >= credential.expires) {
			credential = await oauth.refresh(credential, new AbortController().signal);
			storage[provider] = credential;
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 13: code:models-store.ts->remote-catalog-provider.ts
- E2 ModelsStoreEntry declared at packages/ai/src/models-store.ts:3
- E4 ModelsStoreEntry imported at packages/coding-agent/src/core/remote-catalog-provider.ts:1 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:34 from packages/ai/src/models-store.ts
- E5 ModelsStoreEntry fields read: checkedAt (packages/coding-agent/src/core/remote-catalog-provider.ts:137), etag (packages/coding-agent/src/core/remote-catalog-provider.ts:146), lastModified (packages/coding-agent/src/core/remote-catalog-provider.ts:72), models (packages/coding-agent/src/core/remote-catalog-provider.ts:41)
- E5 ModelsStoreEntry stoplisted reads: (none)
```
// packages/ai/src/models-store.ts:3
export interface ModelsStoreEntry {
	/** Persisted models of every type. */
	models: readonly AnyModel[];
	/** Unix timestamp from the remote catalog's Last-Modified header. */
	lastModified?: number;
	/** Unix timestamp of the last completed remote check. */
	checkedAt?: number;
	/**
	 * Opaque validator from the remote catalog's ETag header, stored verbatim
	 * (quotes included) and echoed back as If-None-Match.
	 */
	etag?: string;
```
```
// packages/coding-agent/src/core/remote-catalog-provider.ts:137
				!context.force &&
				stored?.checkedAt !== undefined &&
				stored.lastModified !== undefined &&
```
```
// packages/coding-agent/src/core/remote-catalog-provider.ts:146
			// leave the overlay empty.
			const validator = stored && stored.models.length > 0 ? stored.etag : undefined;
			const response = await requestCatalog(provider.id, catalogBaseUrl, context.signal, validator);
```
```
// packages/coding-agent/src/core/remote-catalog-provider.ts:72
function remoteModels(entry: ModelsStoreEntry | undefined, localGeneratedAt: number | undefined): readonly AnyModel[] {
	return entry && isNewer(entry.lastModified, localGeneratedAt) ? entry.models : [];
}
```
```
// packages/coding-agent/src/core/remote-catalog-provider.ts:41
		? value
		: typeof value === "object" && value !== null && "models" in value && Array.isArray(value.models)
			? value.models
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 14: code:models.ts->model-registry.ts
- E2 Provider declared at packages/ai/src/models.ts:155
- E4 Provider imported at packages/coding-agent/src/core/model-registry.ts:1 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:33 from packages/ai/src/models.ts
- E5 Provider fields read: auth (packages/coding-agent/src/core/model-registry.ts:101), baseUrl (packages/coding-agent/src/core/model-registry.ts:103), headers (packages/coding-agent/src/core/model-registry.ts:97)
- E5 Provider stoplisted reads: name
```
// packages/ai/src/models.ts:155
export interface Provider<TApi extends Api = Api> {
	readonly id: string;
	readonly name: string;

	readonly baseUrl?: string;
	readonly headers?: ProviderHeaders;

	/**
	 * Required: at least one of `apiKey`/`oauth`. Every provider has auth
	 * semantics — even providers with only ambient credentials (env vars, AWS
	 * profiles, ADC files) and keyless local servers provide `apiKey` auth
	 * whose `resolve()` reports whether the provider is configured.
```
```
// packages/coding-agent/src/core/model-registry.ts:101
				ok: true,
				apiKey: resolution.auth.apiKey,
				headers: resolution.auth.headers,
```
```
// packages/coding-agent/src/core/model-registry.ts:103
				headers: resolution.auth.headers,
				...(resolution.auth.baseUrl ? { baseUrl: resolution.auth.baseUrl } : {}),
				env: resolution.env,
```
```
// packages/coding-agent/src/core/model-registry.ts:97
				}
				return { ok: true, headers: compatibility.headers };
			}
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 15: code:models.ts->model-runtime.ts
- E2 ModelsRefreshOptions declared at packages/ai/src/models.ts:103
- E2 ModelsRefreshResult declared at packages/ai/src/models.ts:112
- E2 Provider declared at packages/ai/src/models.ts:155
- E4 ModelsRefreshOptions imported at packages/coding-agent/src/core/model-runtime.ts:2 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:33 from packages/ai/src/models.ts
- E4 ModelsRefreshResult imported at packages/coding-agent/src/core/model-runtime.ts:2 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:33 from packages/ai/src/models.ts
- E4 Provider imported at packages/coding-agent/src/core/model-runtime.ts:2 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:33 from packages/ai/src/models.ts
- E5 ModelsRefreshOptions fields read: allowNetwork (packages/coding-agent/src/core/model-runtime.ts:859), providers (packages/coding-agent/src/core/model-runtime.ts:851), signal (packages/coding-agent/src/core/model-runtime.ts:256)
- E5 ModelsRefreshOptions stoplisted reads: (none)
- E5 ModelsRefreshResult fields read: aborted (packages/coding-agent/src/core/model-runtime.ts:381), errors (packages/coding-agent/src/core/model-runtime.ts:612)
- E5 ModelsRefreshResult stoplisted reads: (none)
- E5 Provider fields read: auth (packages/coding-agent/src/core/model-runtime.ts:404), baseUrl (packages/coding-agent/src/core/model-runtime.ts:275), headers (packages/coding-agent/src/core/model-runtime.ts:574)
- E5 Provider stoplisted reads: id, name
```
// packages/ai/src/models.ts:155
export interface Provider<TApi extends Api = Api> {
	readonly id: string;
	readonly name: string;

	readonly baseUrl?: string;
	readonly headers?: ProviderHeaders;

	/**
	 * Required: at least one of `apiKey`/`oauth`. Every provider has auth
	 * semantics — even providers with only ambient credentials (env vars, AWS
	 * profiles, ADC files) and keyless local servers provide `apiKey` auth
	 * whose `resolve()` reports whether the provider is configured.
```
```
// packages/coding-agent/src/core/model-runtime.ts:859
			...options,
			allowNetwork: options.allowNetwork ?? this.modelNetworkEnabled,
		};
```
```
// packages/coding-agent/src/core/model-runtime.ts:851
		this.configureRadiusProviders();
		if (options.providers) {
			for (const providerId of new Set(options.providers)) this.recomposeProvider(providerId);
```
```
// packages/coding-agent/src/core/model-runtime.ts:256
		const signal = controller
			? options.signal
				? AbortSignal.any([options.signal, controller.signal])
```
```
// packages/coding-agent/src/core/model-runtime.ts:381
		return this.runAvailabilityRefresh(seq, errorSeq, effectiveSignal).catch((error) => {
			if (errorSeq === this.availabilityErrorSeq && !effectiveSignal.aborted) {
				this.availabilityError = error instanceof Error ? error.message : String(error);
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 16: code:models.ts->provider-composer.ts
- E2 Provider declared at packages/ai/src/models.ts:155
- E2 RefreshModelsContext declared at packages/ai/src/models.ts:80
- E4 Provider imported at packages/coding-agent/src/core/provider-composer.ts:1 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:33 from packages/ai/src/models.ts
- E4 RefreshModelsContext imported at packages/coding-agent/src/core/provider-composer.ts:1 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:33 from packages/ai/src/models.ts
- E5 Provider fields read: auth (packages/coding-agent/src/core/provider-composer.ts:423), baseUrl (packages/coding-agent/src/core/provider-composer.ts:222), headers (packages/coding-agent/src/core/provider-composer.ts:313)
- E5 Provider stoplisted reads: id, name
- E5 RefreshModelsContext fields read: baseUrl (packages/coding-agent/src/core/provider-composer.ts:222), credential (packages/coding-agent/src/core/provider-composer.ts:439), signal (packages/coding-agent/src/core/provider-composer.ts:365)
- E5 RefreshModelsContext stoplisted reads: (none)
```
// packages/ai/src/models.ts:80
export interface RefreshModelsContext {
	/** Effective configured credential. OAuth credentials are refreshed before network access. */
	credential?: Credential;
	/** Immutable provider-scoped catalog snapshot captured before this refresh phase. */
	stored?: Readonly<ModelsStoreEntry>;
	/**
	 * Generation-checked publication. Persistence policy remains provider-owned;
	 * the update runs synchronously only after the selected persistence mutation.
	 */
	publish(publication: ModelsPublication): Promise<boolean>;
	/** False during offline/cache-only initialization. */
	allowNetwork: boolean;
```
```
// packages/coding-agent/src/core/provider-composer.ts:423
): ApiKeyAuth | undefined {
	const inherited = base?.auth.apiKey;
	const rawKey = configuredApiKey(config, extension);
```
```
// packages/coding-agent/src/core/provider-composer.ts:222
	}
	const baseUrl = definition.baseUrl ?? providerConfig.baseUrl ?? defaults?.baseUrl;
	if (!baseUrl) throw new Error(`Provider ${providerId}: "baseUrl" is required when defining custom models.`);
```
```
// packages/coding-agent/src/core/provider-composer.ts:313
		!config.baseUrl &&
		!config.headers &&
		!config.compat &&
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 17: code:models.ts->remote-catalog-provider.ts
- E2 RefreshModelsContext declared at packages/ai/src/models.ts:80
- E4 RefreshModelsContext imported at packages/coding-agent/src/core/remote-catalog-provider.ts:1 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:33 from packages/ai/src/models.ts
- E5 RefreshModelsContext fields read: allowNetwork (packages/coding-agent/src/core/remote-catalog-provider.ts:131), force (packages/coding-agent/src/core/remote-catalog-provider.ts:136), signal (packages/coding-agent/src/core/remote-catalog-provider.ts:131), stored (packages/coding-agent/src/core/remote-catalog-provider.ts:120)
- E5 RefreshModelsContext stoplisted reads: (none)
```
// packages/ai/src/models.ts:80
export interface RefreshModelsContext {
	/** Effective configured credential. OAuth credentials are refreshed before network access. */
	credential?: Credential;
	/** Immutable provider-scoped catalog snapshot captured before this refresh phase. */
	stored?: Readonly<ModelsStoreEntry>;
	/**
	 * Generation-checked publication. Persistence policy remains provider-owned;
	 * the update runs synchronously only after the selected persistence mutation.
	 */
	publish(publication: ModelsPublication): Promise<boolean>;
	/** False during offline/cache-only initialization. */
	allowNetwork: boolean;
```
```
// packages/coding-agent/src/core/remote-catalog-provider.ts:131
			}
			if (!context.allowNetwork || context.signal.aborted) return;
			// Skip the request if pi.dev was checked less than REMOTE_CATALOG_REFRESH_INTERVAL_MS ago.
```
```
// packages/coding-agent/src/core/remote-catalog-provider.ts:136
			if (
				!context.force &&
				stored?.checkedAt !== undefined &&
```
```
// packages/coding-agent/src/core/remote-catalog-provider.ts:120
		refreshModels: async (context) => {
			const stored = context.stored;
			const restored = remoteModels(stored, localGeneratedAt).filter((model) => model.provider === provider.id);
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 18: code:models.ts->provider.ts
- E2 RefreshModelsContext declared at packages/ai/src/models.ts:80
- E4 RefreshModelsContext imported at packages/coding-agent/src/extensions/llama/provider.ts:1 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:33 from packages/ai/src/models.ts
- E5 RefreshModelsContext fields read: allowNetwork (packages/coding-agent/src/extensions/llama/provider.ts:255), credential (packages/coding-agent/src/extensions/llama/provider.ts:255), signal (packages/coding-agent/src/extensions/llama/provider.ts:204), stored (packages/coding-agent/src/extensions/llama/provider.ts:233)
- E5 RefreshModelsContext stoplisted reads: (none)
```
// packages/ai/src/models.ts:80
export interface RefreshModelsContext {
	/** Effective configured credential. OAuth credentials are refreshed before network access. */
	credential?: Credential;
	/** Immutable provider-scoped catalog snapshot captured before this refresh phase. */
	stored?: Readonly<ModelsStoreEntry>;
	/**
	 * Generation-checked publication. Persistence policy remains provider-owned;
	 * the update runs synchronously only after the selected persistence mutation.
	 */
	publish(publication: ModelsPublication): Promise<boolean>;
	/** False during offline/cache-only initialization. */
	allowNetwork: boolean;
```
```
// packages/coding-agent/src/extensions/llama/provider.ts:255

			if (!context.allowNetwork || context.signal.aborted || context.credential?.type !== "api_key") return;
			const serverUrl = credentialServerUrl(context.credential);
```
```
// packages/coding-agent/src/extensions/llama/provider.ts:204
					).trim();
					await new LlamaClient(serverUrl, apiKey || undefined).list({ signal: interaction.signal });
					return {
```
```
// packages/coding-agent/src/extensions/llama/provider.ts:233
			const cachedContextWindows = new Map<string, number>();
			if (context.stored) {
				const stored = context.stored.models.filter((model) => model.provider === LLAMA_PROVIDER_ID);
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 19: code:models.ts->llama-extension.test.ts
- E2 ModelsPublication declared at packages/ai/src/models.ts:73
- E4 ModelsPublication imported at packages/coding-agent/test/llama-extension.test.ts:4 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:33 from packages/ai/src/models.ts
- E5 ModelsPublication fields read: persist (packages/coding-agent/test/llama-extension.test.ts:166), update (packages/coding-agent/test/llama-extension.test.ts:120)
- E5 ModelsPublication stoplisted reads: (none)
```
// packages/ai/src/models.ts:73
export interface ModelsPublication {
	/** Provider-selected persisted catalog. Omit to leave storage unchanged; null deletes it. */
	persist?: ModelsStoreEntry | null;
	/** Optional synchronous update of provider-private in-memory catalog state. */
	update?: () => void;
}

export interface RefreshModelsContext {
	/** Effective configured credential. OAuth credentials are refreshed before network access. */
	credential?: Credential;
	/** Immutable provider-scoped catalog snapshot captured before this refresh phase. */
	stored?: Readonly<ModelsStoreEntry>;
```
```
// packages/coding-agent/test/llama-extension.test.ts:166
		const publish = async (publication: ModelsPublication): Promise<boolean> => {
			if (publication.persist === null) cachedEntry = undefined;
			else if (publication.persist !== undefined) cachedEntry = structuredClone(publication.persist);
```
```
// packages/coding-agent/test/llama-extension.test.ts:120
			publish: async (publication) => {
				publication.update?.();
				return true;
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 20: code:models.ts->remote-catalog-provider.test.ts
- E2 ModelsPublication declared at packages/ai/src/models.ts:73
- E2 RefreshModelsContext declared at packages/ai/src/models.ts:80
- E4 ModelsPublication imported at packages/coding-agent/test/remote-catalog-provider.test.ts:1 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:33 from packages/ai/src/models.ts
- E4 RefreshModelsContext imported at packages/coding-agent/test/remote-catalog-provider.test.ts:1 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:33 from packages/ai/src/models.ts
- E5 ModelsPublication fields read: persist (packages/coding-agent/test/remote-catalog-provider.test.ts:63), update (packages/coding-agent/test/remote-catalog-provider.test.ts:65)
- E5 ModelsPublication stoplisted reads: (none)
- E5 RefreshModelsContext fields read: allowNetwork (packages/coding-agent/test/remote-catalog-provider.test.ts:72), force (packages/coding-agent/test/remote-catalog-provider.test.ts:73), signal (packages/coding-agent/test/remote-catalog-provider.test.ts:20)
- E5 RefreshModelsContext stoplisted reads: (none)
```
// packages/ai/src/models.ts:80
export interface RefreshModelsContext {
	/** Effective configured credential. OAuth credentials are refreshed before network access. */
	credential?: Credential;
	/** Immutable provider-scoped catalog snapshot captured before this refresh phase. */
	stored?: Readonly<ModelsStoreEntry>;
	/**
	 * Generation-checked publication. Persistence policy remains provider-owned;
	 * the update runs synchronously only after the selected persistence mutation.
	 */
	publish(publication: ModelsPublication): Promise<boolean>;
	/** False during offline/cache-only initialization. */
	allowNetwork: boolean;
```
```
// packages/coding-agent/test/remote-catalog-provider.test.ts:63
	const publish = async (publication: ModelsPublication): Promise<boolean> => {
		if (publication.persist === null) await store.delete(provider.id);
		else if (publication.persist !== undefined) await store.write(provider.id, publication.persist);
```
```
// packages/coding-agent/test/remote-catalog-provider.test.ts:65
		else if (publication.persist !== undefined) await store.write(provider.id, publication.persist);
		publication.update?.();
		return true;
```
```
// packages/coding-agent/test/remote-catalog-provider.test.ts:72
		publish,
		allowNetwork: overrides.allowNetwork ?? true,
		force: overrides.force,
```
```
// packages/coding-agent/test/remote-catalog-provider.test.ts:73
		allowNetwork: overrides.allowNetwork ?? true,
		force: overrides.force,
		signal: overrides.signal ?? neverAbortedSignal,
```
Judgment (TRUE / NOT TRUE, one sentence):

