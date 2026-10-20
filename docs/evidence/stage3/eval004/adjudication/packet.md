## earendil-works__pi: 26 new proposal(s), all adjudicated

### earendil-works__pi 1: code:types.ts->index.ts#3
- E2 OAuthCredentials declared at packages/ai/src/auth/types.ts:24
- E4 OAuthCredentials imported at packages/coding-agent/examples/extensions/custom-provider-gitlab-duo/index.ts:12 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:24 from packages/ai/src/auth/types.ts
- E5 OAuthCredentials fields read: access (packages/coding-agent/examples/extensions/custom-provider-gitlab-duo/index.ts:401), refresh (packages/coding-agent/examples/extensions/custom-provider-gitlab-duo/index.ts:284)
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
// packages/coding-agent/examples/extensions/custom-provider-gitlab-duo/index.ts:401
			refreshToken: refreshGitLabToken,
			getApiKey: (cred) => cred.access,
		},
```
```
// packages/coding-agent/examples/extensions/custom-provider-gitlab-duo/index.ts:284
			grant_type: "refresh_token",
			refresh_token: credentials.refresh,
		}).toString(),
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 2: code:types.ts->agent-session.ts#2
- E2 AuthResult declared at packages/ai/src/auth/types.ts:104
- E4 AuthResult imported at packages/coding-agent/src/core/agent-session.ts:35 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:24 from packages/ai/src/auth/types.ts
- E5 AuthResult fields read: auth (packages/coding-agent/src/core/agent-session.ts:552), env (packages/coding-agent/src/core/agent-session.ts:558), source (packages/coding-agent/src/core/agent-session.ts:2086)
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
// packages/coding-agent/src/core/agent-session.ts:552
		}
		if (result && (result.auth.apiKey || result.auth.headers)) {
			const requestModel = result.auth.baseUrl ? { ...model, baseUrl: result.auth.baseUrl } : model;
```
```
// packages/coding-agent/src/core/agent-session.ts:558
				headers: withoutDeletedHeaders(result.auth.headers),
				env: result.env,
			};
```
```
// packages/coding-agent/src/core/agent-session.ts:2086
			options?.images,
			options?.source ?? "interactive",
			this.isStreaming ? options?.streamingBehavior : undefined,
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 3: code:models.ts->bug-report.ts
- E2 Provider declared at packages/ai/src/models.ts:155
- E4 Provider imported at packages/coding-agent/src/core/bug-report.ts:4 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:33 from packages/ai/src/models.ts
- E5 Provider fields read: auth (packages/coding-agent/src/core/bug-report.ts:113), baseUrl (packages/coding-agent/src/core/bug-report.ts:99), headers (packages/coding-agent/src/core/bug-report.ts:107)
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
// packages/coding-agent/src/core/bug-report.ts:113
	const authTypes: Array<"api_key" | "oauth"> = [];
	if (provider.auth.apiKey) authTypes.push("api_key");
	if (provider.auth.oauth) authTypes.push("oauth");
```
```
// packages/coding-agent/src/core/bug-report.ts:99
		api: model.api,
		baseUrl: redactUrl(model.baseUrl),
		reasoning: model.reasoning,
```
```
// packages/coding-agent/src/core/bug-report.ts:107
		thinkingLevelMap: model.thinkingLevelMap ?? null,
		headerNames: Object.keys(model.headers ?? {}).sort(),
	};
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 4: code:faux.ts->e2e.test.ts
- E2 FauxProviderRegistration declared at packages/ai/src/providers/faux.ts:134
- E4 FauxProviderRegistration imported at packages/agent/test/e2e.test.ts:1 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:35 from packages/ai/src/providers/faux.ts
- E5 FauxProviderRegistration fields read: api (packages/agent/test/e2e.test.ts:297), setResponses (packages/agent/test/e2e.test.ts:189), state (packages/agent/test/e2e.test.ts:52), unregister (packages/agent/test/e2e.test.ts:35)
- E5 FauxProviderRegistration stoplisted reads: (none)
```
// packages/ai/src/providers/faux.ts:134
export interface FauxProviderRegistration {
	api: string;
	models: [Model<string>, ...Model<string>[]];
	getModel(): Model<string>;
	getModel(modelId: string): Model<string> | undefined;
	state: FauxProviderState;
	setResponses: (responses: FauxResponseStep[]) => void;
	appendResponses: (responses: FauxResponseStep[]) => void;
	getPendingResponseCount: () => number;
	unregister: () => void;
}

```
```
// packages/agent/test/e2e.test.ts:297
				content: [{ type: "text", text: "Hello" }],
				api: model.api,
				provider: model.provider,
```
```
// packages/agent/test/e2e.test.ts:189
		const faux = createFauxRegistration();
		faux.setResponses([fauxAssistantMessage("4")]);
		await basicPrompt(faux.getModel());
```
```
// packages/agent/test/e2e.test.ts:52

	expect(agent.state.isStreaming).toBe(false);
	expect(agent.state.messages.length).toBe(3);
```
```
// packages/agent/test/e2e.test.ts:35
	while (registrations.length > 0) {
		registrations.pop()?.unregister();
	}
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 5: code:faux.ts->harness.ts
- E2 FauxModelDefinition declared at packages/ai/src/providers/faux.ts:41
- E2 FauxProviderRegistration declared at packages/ai/src/providers/faux.ts:134
- E4 FauxModelDefinition imported at packages/coding-agent/test/suite/harness.ts:11 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:35 from packages/ai/src/providers/faux.ts
- E4 FauxProviderRegistration imported at packages/coding-agent/test/suite/harness.ts:11 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:35 from packages/ai/src/providers/faux.ts
- E5 FauxModelDefinition fields read: contextWindow (packages/coding-agent/test/suite/harness.ts:186), cost (packages/coding-agent/test/suite/harness.ts:185), input (packages/coding-agent/test/suite/harness.ts:183), inputLimits (packages/coding-agent/test/suite/harness.ts:184), maxTokens (packages/coding-agent/test/suite/harness.ts:187), reasoning (packages/coding-agent/test/suite/harness.ts:182)
- E5 FauxModelDefinition stoplisted reads: id, name
- E5 FauxProviderRegistration fields read: api (packages/coding-agent/test/suite/harness.ts:177), appendResponses (packages/coding-agent/test/suite/harness.ts:261), getPendingResponseCount (packages/coding-agent/test/suite/harness.ts:262), models (packages/coding-agent/test/suite/harness.ts:153), setResponses (packages/coding-agent/test/suite/harness.ts:155), unregister (packages/coding-agent/test/suite/harness.ts:270)
- E5 FauxProviderRegistration stoplisted reads: (none)
```
// packages/ai/src/providers/faux.ts:41
export interface FauxModelDefinition {
	id: string;
	name?: string;
	reasoning?: boolean;
	input?: ("text" | "image")[];
	inputLimits?: Model<string>["inputLimits"];
	cost?: { input: number; output: number; cacheRead: number; cacheWrite: number };
	contextWindow?: number;
	maxTokens?: number;
}

export type FauxContentBlock = TextContent | ThinkingContent | ToolCall;
```
```
// packages/ai/src/providers/faux.ts:134
export interface FauxProviderRegistration {
	api: string;
	models: [Model<string>, ...Model<string>[]];
	getModel(): Model<string>;
	getModel(modelId: string): Model<string> | undefined;
	state: FauxProviderState;
	setResponses: (responses: FauxResponseStep[]) => void;
	appendResponses: (responses: FauxResponseStep[]) => void;
	getPendingResponseCount: () => number;
	unregister: () => void;
}

```
```
// packages/coding-agent/test/suite/harness.ts:186
				cost: registeredModel.cost,
				contextWindow: registeredModel.contextWindow,
				maxTokens: registeredModel.maxTokens,
```
```
// packages/coding-agent/test/suite/harness.ts:185
				inputLimits: registeredModel.inputLimits,
				cost: registeredModel.cost,
				contextWindow: registeredModel.contextWindow,
```
```
// packages/coding-agent/test/suite/harness.ts:183
				reasoning: registeredModel.reasoning,
				input: registeredModel.input,
				inputLimits: registeredModel.inputLimits,
```
```
// packages/coding-agent/test/suite/harness.ts:184
				input: registeredModel.input,
				inputLimits: registeredModel.inputLimits,
				cost: registeredModel.cost,
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 6: code:types.ts->agent.test.ts
- E2 AssistantMessage declared at packages/ai/src/types.ts:570
- E2 UserMessage declared at packages/ai/src/types.ts:564
- E4 AssistantMessage imported at packages/agent/test/agent.test.ts:1 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E4 UserMessage imported at packages/agent/test/agent.test.ts:1 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E5 AssistantMessage fields read: content (packages/agent/test/agent.test.ts:44), errorMessage (packages/agent/test/agent.test.ts:124), model (packages/agent/test/agent.test.ts:117), role (packages/agent/test/agent.test.ts:157), stopReason (packages/agent/test/agent.test.ts:356), thinkingLevel (packages/agent/test/agent.test.ts:118)
- E5 AssistantMessage stoplisted reads: (none)
- E5 UserMessage fields read: content (packages/agent/test/agent.test.ts:44), role (packages/agent/test/agent.test.ts:157)
- E5 UserMessage stoplisted reads: (none)
```
// packages/ai/src/types.ts:570
export interface AssistantMessage {
	role: "assistant";
	content: (TextContent | ThinkingContent | ToolCall)[];
	api: Api;
	provider: ProviderId;
	model: string;
	responseModel?: string; // Concrete model reported by the provider when different from the requested `model`
	responseId?: string; // Provider-specific response/message identifier when the upstream API exposes one
	/** Exact provider-native effort level used for this response. Absent for legacy or unmanaged responses. */
	providerThinkingLevel?: string;
	/** Pi thinking level the agent loop requested for this response. Absent outside the agent loop and for legacy responses. */
	thinkingLevel?: ModelThinkingLevel;
```
```
// packages/ai/src/types.ts:564
export interface UserMessage {
	role: "user";
	content: string | (TextContent | ImageContent)[];
	timestamp: number; // Unix timestamp in milliseconds
}

export interface AssistantMessage {
	role: "assistant";
	content: (TextContent | ThinkingContent | ToolCall)[];
	api: Api;
	provider: ProviderId;
	model: string;
```
```
// packages/agent/test/agent.test.ts:44

type ToolCallContent = Extract<AssistantMessage["content"][number], { type: "toolCall" }>;

```
```
// packages/agent/test/agent.test.ts:124
		expect(agent.state.pendingToolCalls).toEqual(new Set());
		expect(agent.state.errorMessage).toBeUndefined();
	});
```
```
// packages/agent/test/agent.test.ts:117
		expect(agent.state).toBeDefined();
		expect(agent.state.model).toBeDefined();
		expect(agent.state.thinkingLevel).toBe("off");
```
```
// packages/agent/test/agent.test.ts:157
		const initial = agent.state.messages[0];
		expect(initial?.role).toBe("system");
		if (initial?.role !== "system") throw new Error("expected initial system message");
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 7: code:types.ts->e2e.test.ts
- E2 AssistantMessage declared at packages/ai/src/types.ts:570
- E2 UserMessage declared at packages/ai/src/types.ts:564
- E4 AssistantMessage imported at packages/agent/test/e2e.test.ts:1 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E4 UserMessage imported at packages/agent/test/e2e.test.ts:1 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E5 AssistantMessage fields read: api (packages/agent/test/e2e.test.ts:297), content (packages/agent/test/e2e.test.ts:27), errorMessage (packages/agent/test/e2e.test.ts:127), provider (packages/agent/test/e2e.test.ts:298), role (packages/agent/test/e2e.test.ts:54), stopReason (packages/agent/test/e2e.test.ts:126)
- E5 AssistantMessage stoplisted reads: (none)
- E5 UserMessage fields read: content (packages/agent/test/e2e.test.ts:27), role (packages/agent/test/e2e.test.ts:54)
- E5 UserMessage stoplisted reads: (none)
```
// packages/ai/src/types.ts:570
export interface AssistantMessage {
	role: "assistant";
	content: (TextContent | ThinkingContent | ToolCall)[];
	api: Api;
	provider: ProviderId;
	model: string;
	responseModel?: string; // Concrete model reported by the provider when different from the requested `model`
	responseId?: string; // Provider-specific response/message identifier when the upstream API exposes one
	/** Exact provider-native effort level used for this response. Absent for legacy or unmanaged responses. */
	providerThinkingLevel?: string;
	/** Pi thinking level the agent loop requested for this response. Absent outside the agent loop and for legacy responses. */
	thinkingLevel?: ModelThinkingLevel;
```
```
// packages/ai/src/types.ts:564
export interface UserMessage {
	role: "user";
	content: string | (TextContent | ImageContent)[];
	timestamp: number; // Unix timestamp in milliseconds
}

export interface AssistantMessage {
	role: "assistant";
	content: (TextContent | ThinkingContent | ToolCall)[];
	api: Api;
	provider: ProviderId;
	model: string;
```
```
// packages/agent/test/e2e.test.ts:297
				content: [{ type: "text", text: "Hello" }],
				api: model.api,
				provider: model.provider,
```
```
// packages/agent/test/e2e.test.ts:27
function getTextContent(message: AssistantMessage | ToolResultMessage): string {
	return message.content
		.filter((block) => block.type === "text")
```
```
// packages/agent/test/e2e.test.ts:127
	expect(lastMessage.stopReason).toBe("aborted");
	expect(lastMessage.errorMessage).toBeDefined();
	expect(agent.state.errorMessage).toBe(lastMessage.errorMessage);
```
```
// packages/agent/test/e2e.test.ts:298
				api: model.api,
				provider: model.provider,
				model: model.id,
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 8: code:types.ts->test.ts
- E2 Model declared at packages/ai/src/types.ts:839
- E4 Model imported at packages/coding-agent/examples/extensions/custom-provider-gitlab-duo/test.ts:11 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E5 Model fields read: contextWindow (packages/coding-agent/examples/extensions/custom-provider-gitlab-duo/test.ts:56), maxTokens (packages/coding-agent/examples/extensions/custom-provider-gitlab-duo/test.ts:57), reasoning (packages/coding-agent/examples/extensions/custom-provider-gitlab-duo/test.ts:53)
- E5 Model stoplisted reads: type
```
// packages/ai/src/types.ts:839
export interface Model<TApi extends Api> extends BaseModel<TApi> {
	/**
	 * Optional: chat is the default model type, so models without `type` are chat
	 * models. Narrow mixed model lists with `isModelType()` instead of comparing
	 * `type` directly.
	 */
	type?: "chat";
	reasoning: boolean;
	/**
	 * Maps pi thinking levels to provider/model-specific values.
	 * Missing keys use provider defaults. null marks a level as unsupported.
	 */
```
```
// packages/coding-agent/examples/extensions/custom-provider-gitlab-duo/test.ts:56
		cost: cfg.cost,
		contextWindow: cfg.contextWindow,
		maxTokens: cfg.maxTokens,
```
```
// packages/coding-agent/examples/extensions/custom-provider-gitlab-duo/test.ts:57
		contextWindow: cfg.contextWindow,
		maxTokens: cfg.maxTokens,
	};
```
```
// packages/coding-agent/examples/extensions/custom-provider-gitlab-duo/test.ts:53
		baseUrl: cfg.baseUrl,
		reasoning: cfg.reasoning,
		input: cfg.input,
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 9: code:types.ts->agent-session.ts#3
- E2 AssistantMessage declared at packages/ai/src/types.ts:570
- E2 Model declared at packages/ai/src/types.ts:839
- E2 SystemMessage declared at packages/ai/src/types.ts:546
- E2 Usage declared at packages/ai/src/types.ts:451
- E4 AssistantMessage imported at packages/coding-agent/src/core/agent-session.ts:35 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E4 Model imported at packages/coding-agent/src/core/agent-session.ts:35 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E4 SystemMessage imported at packages/coding-agent/src/core/agent-session.ts:35 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E4 Usage imported at packages/coding-agent/src/core/agent-session.ts:35 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E5 AssistantMessage fields read: content (packages/coding-agent/src/core/agent-session.ts:697), durationMs (packages/coding-agent/src/core/agent-session.ts:1362), errorMessage (packages/coding-agent/src/core/agent-session.ts:1969), model (packages/coding-agent/src/core/agent-session.ts:617), provider (packages/coding-agent/src/core/agent-session.ts:548), role (packages/coding-agent/src/core/agent-session.ts:820), stopReason (packages/coding-agent/src/core/agent-session.ts:852), thinkingLevel (packages/coding-agent/src/core/agent-session.ts:587), timestamp (packages/coding-agent/src/core/agent-session.ts:1803), usage (packages/coding-agent/src/core/agent-session.ts:701)
- E5 AssistantMessage stoplisted reads: (none)
- E5 Model fields read: contextWindow (packages/coding-agent/src/core/agent-session.ts:769), maxTokens (packages/coding-agent/src/core/agent-session.ts:3039), reasoning (packages/coding-agent/src/core/agent-session.ts:2696)
- E5 Model stoplisted reads: type
- E5 SystemMessage fields read: content (packages/coding-agent/src/core/agent-session.ts:697), role (packages/coding-agent/src/core/agent-session.ts:820), sections (packages/coding-agent/src/core/agent-session.ts:1751), timestamp (packages/coding-agent/src/core/agent-session.ts:1803), toolsAdded (packages/coding-agent/src/core/agent-session.ts:1779), toolsRemoved (packages/coding-agent/src/core/agent-session.ts:1779)
- E5 SystemMessage stoplisted reads: (none)
- E5 Usage fields read: cacheRead (packages/coding-agent/src/core/agent-session.ts:4258), cacheWrite (packages/coding-agent/src/core/agent-session.ts:4259), cost (packages/coding-agent/src/core/agent-session.ts:4262), input (packages/coding-agent/src/core/agent-session.ts:4256), output (packages/coding-agent/src/core/agent-session.ts:3913), reasoning (packages/coding-agent/src/core/agent-session.ts:2696)
- E5 Usage stoplisted reads: (none)
```
// packages/ai/src/types.ts:570
export interface AssistantMessage {
	role: "assistant";
	content: (TextContent | ThinkingContent | ToolCall)[];
	api: Api;
	provider: ProviderId;
	model: string;
	responseModel?: string; // Concrete model reported by the provider when different from the requested `model`
	responseId?: string; // Provider-specific response/message identifier when the upstream API exposes one
	/** Exact provider-native effort level used for this response. Absent for legacy or unmanaged responses. */
	providerThinkingLevel?: string;
	/** Pi thinking level the agent loop requested for this response. Absent outside the agent loop and for legacy responses. */
	thinkingLevel?: ModelThinkingLevel;
```
```
// packages/ai/src/types.ts:839
export interface Model<TApi extends Api> extends BaseModel<TApi> {
	/**
	 * Optional: chat is the default model type, so models without `type` are chat
	 * models. Narrow mixed model lists with `isModelType()` instead of comparing
	 * `type` directly.
	 */
	type?: "chat";
	reasoning: boolean;
	/**
	 * Maps pi thinking levels to provider/model-specific values.
	 * Missing keys use provider defaults. null marks a level as unsupported.
	 */
```
```
// packages/ai/src/types.ts:546
export interface SystemMessage {
	role: "system";
	/** Instruction text. On the leading message this is the base prompt; later, additional instructions. */
	content: string | TextContent[];
	/**
	 * Named, ordered prompt sections rendered verbatim after `content`. The leading message
	 * declares them; later messages replace sections by name, and `null` removes one. Keep
	 * each section self-delimiting (a tag, a heading) so the model can relate an update to
	 * the original. Avoid integer-like names; JSON objects reorder those.
	 */
	sections?: Record<string, string | null>;
	/** Complete definitions of tools that become available at this point. */
```
```
// packages/ai/src/types.ts:451
export interface Usage {
	input: number;
	output: number;
	cacheRead: number;
	cacheWrite: number;
	/** Subset of `cacheWrite` written with 1h retention. Only Anthropic reports this split. */
	cacheWrite1h?: number;
	/**
	 * Reasoning/thinking tokens, when the provider reports them. This is a subset of
	 * `output`: `output` already includes these tokens. Set to a number (possibly 0) by
	 * providers that expose a reasoning breakdown; left undefined by providers that don't.
	 */
```
```
// packages/coding-agent/src/core/agent-session.ts:697
					input: args as Record<string, unknown>,
					content: result.content,
					details: result.details,
```
```
// packages/coding-agent/src/core/agent-session.ts:1362
				isError: event.isError,
				...(event.durationMs === undefined ? {} : { durationMs: event.durationMs }),
			};
```
```
// packages/coding-agent/src/core/agent-session.ts:1969
				attempt: this._retryAttempt,
				finalError: message.errorMessage,
			});
```
```
// packages/coding-agent/src/core/agent-session.ts:617
	private _modelForMessage(message: AssistantMessage): Model<any> | undefined {
		const model = this.model;
		if (model && isVirtualModel(model)) return this._modelRuntime.getPhysicalModel(message.provider, message.model);
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 10: code:types.ts->bug-report.ts
- E2 Model declared at packages/ai/src/types.ts:839
- E4 Model imported at packages/coding-agent/src/core/bug-report.ts:4 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E5 Model fields read: compat (packages/coding-agent/src/core/bug-report.ts:105), contextWindow (packages/coding-agent/src/core/bug-report.ts:102), maxTokens (packages/coding-agent/src/core/bug-report.ts:103), reasoning (packages/coding-agent/src/core/bug-report.ts:100), samplingParams (packages/coding-agent/src/core/bug-report.ts:104), thinkingLevelMap (packages/coding-agent/src/core/bug-report.ts:106)
- E5 Model stoplisted reads: type
```
// packages/ai/src/types.ts:839
export interface Model<TApi extends Api> extends BaseModel<TApi> {
	/**
	 * Optional: chat is the default model type, so models without `type` are chat
	 * models. Narrow mixed model lists with `isModelType()` instead of comparing
	 * `type` directly.
	 */
	type?: "chat";
	reasoning: boolean;
	/**
	 * Maps pi thinking levels to provider/model-specific values.
	 * Missing keys use provider defaults. null marks a level as unsupported.
	 */
```
```
// packages/coding-agent/src/core/bug-report.ts:105
		samplingParams: model.samplingParams ? redactJsonValue(model.samplingParams) : null,
		compat: model.compat ? redactJsonValue(model.compat) : null,
		thinkingLevelMap: model.thinkingLevelMap ?? null,
```
```
// packages/coding-agent/src/core/bug-report.ts:102
		input: model.input,
		contextWindow: model.contextWindow,
		maxTokens: model.maxTokens,
```
```
// packages/coding-agent/src/core/bug-report.ts:103
		contextWindow: model.contextWindow,
		maxTokens: model.maxTokens,
		samplingParams: model.samplingParams ? redactJsonValue(model.samplingParams) : null,
```
```
// packages/coding-agent/src/core/bug-report.ts:100
		baseUrl: redactUrl(model.baseUrl),
		reasoning: model.reasoning,
		input: model.input,
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 11: code:types.ts->compaction.ts
- E2 AssistantMessage declared at packages/ai/src/types.ts:570
- E2 SystemMessage declared at packages/ai/src/types.ts:546
- E2 Usage declared at packages/ai/src/types.ts:451
- E4 AssistantMessage imported at packages/coding-agent/src/core/compaction/compaction.ts:18 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E4 SystemMessage imported at packages/coding-agent/src/core/compaction/compaction.ts:18 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E4 Usage imported at packages/coding-agent/src/core/compaction/compaction.ts:18 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E5 AssistantMessage fields read: content (packages/coding-agent/src/core/compaction/compaction.ts:304), role (packages/coding-agent/src/core/compaction/compaction.ts:101), stopReason (packages/coding-agent/src/core/compaction/compaction.ts:152), usage (packages/coding-agent/src/core/compaction/compaction.ts:154)
- E5 AssistantMessage stoplisted reads: (none)
- E5 SystemMessage fields read: content (packages/coding-agent/src/core/compaction/compaction.ts:304), role (packages/coding-agent/src/core/compaction/compaction.ts:101), sections (packages/coding-agent/src/core/compaction/compaction.ts:305), toolsAdded (packages/coding-agent/src/core/compaction/compaction.ts:310)
- E5 SystemMessage stoplisted reads: (none)
- E5 Usage fields read: cacheRead (packages/coding-agent/src/core/compaction/compaction.ts:141), cacheWrite (packages/coding-agent/src/core/compaction/compaction.ts:141), input (packages/coding-agent/src/core/compaction/compaction.ts:141), output (packages/coding-agent/src/core/compaction/compaction.ts:141), reasoning (packages/coding-agent/src/core/compaction/compaction.ts:605), totalTokens (packages/coding-agent/src/core/compaction/compaction.ts:141)
- E5 Usage stoplisted reads: (none)
```
// packages/ai/src/types.ts:570
export interface AssistantMessage {
	role: "assistant";
	content: (TextContent | ThinkingContent | ToolCall)[];
	api: Api;
	provider: ProviderId;
	model: string;
	responseModel?: string; // Concrete model reported by the provider when different from the requested `model`
	responseId?: string; // Provider-specific response/message identifier when the upstream API exposes one
	/** Exact provider-native effort level used for this response. Absent for legacy or unmanaged responses. */
	providerThinkingLevel?: string;
	/** Pi thinking level the agent loop requested for this response. Absent outside the agent loop and for legacy responses. */
	thinkingLevel?: ModelThinkingLevel;
```
```
// packages/ai/src/types.ts:546
export interface SystemMessage {
	role: "system";
	/** Instruction text. On the leading message this is the base prompt; later, additional instructions. */
	content: string | TextContent[];
	/**
	 * Named, ordered prompt sections rendered verbatim after `content`. The leading message
	 * declares them; later messages replace sections by name, and `null` removes one. Keep
	 * each section self-delimiting (a tag, a heading) so the model can relate an update to
	 * the original. Avoid integer-like names; JSON objects reorder those.
	 */
	sections?: Record<string, string | null>;
	/** Complete definitions of tools that become available at this point. */
```
```
// packages/ai/src/types.ts:451
export interface Usage {
	input: number;
	output: number;
	cacheRead: number;
	cacheWrite: number;
	/** Subset of `cacheWrite` written with 1h retention. Only Anthropic reports this split. */
	cacheWrite1h?: number;
	/**
	 * Reasoning/thinking tokens, when the provider reports them. This is a subset of
	 * `output`: `output` already includes these tokens. Set to a number (possibly 0) by
	 * providers that expose a reasoning breakdown; left undefined by providers that don't.
	 */
```
```
// packages/coding-agent/src/core/compaction/compaction.ts:304
			const system = message as SystemMessage;
			chars = estimateTextAndImageContentChars(system.content);
			if (system.sections) {
```
```
// packages/coding-agent/src/core/compaction/compaction.ts:101
	// System messages are prompt state, not conversation; the compaction entry carries their replay.
	return entry.messages.filter((message) => message.role !== "system");
}
```
```
// packages/coding-agent/src/core/compaction/compaction.ts:152
		if (
			assistantMsg.stopReason !== "aborted" &&
			assistantMsg.stopReason !== "error" &&
```
```
// packages/coding-agent/src/core/compaction/compaction.ts:154
			assistantMsg.stopReason !== "error" &&
			assistantMsg.usage &&
			calculateContextTokens(assistantMsg.usage) > 0
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 12: code:types.ts->usage-totals.ts
- E2 Usage declared at packages/ai/src/types.ts:451
- E4 Usage imported at packages/coding-agent/src/core/usage-totals.ts:1 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E5 Usage fields read: cacheRead (packages/coding-agent/src/core/usage-totals.ts:25), cacheWrite (packages/coding-agent/src/core/usage-totals.ts:26), cacheWrite1h (packages/coding-agent/src/core/usage-totals.ts:37), cost (packages/coding-agent/src/core/usage-totals.ts:27), input (packages/coding-agent/src/core/usage-totals.ts:23), output (packages/coding-agent/src/core/usage-totals.ts:24), reasoning (packages/coding-agent/src/core/usage-totals.ts:40), totalTokens (packages/coding-agent/src/core/usage-totals.ts:43)
- E5 Usage stoplisted reads: (none)
```
// packages/ai/src/types.ts:451
export interface Usage {
	input: number;
	output: number;
	cacheRead: number;
	cacheWrite: number;
	/** Subset of `cacheWrite` written with 1h retention. Only Anthropic reports this split. */
	cacheWrite1h?: number;
	/**
	 * Reasoning/thinking tokens, when the provider reports them. This is a subset of
	 * `output`: `output` already includes these tokens. Set to a number (possibly 0) by
	 * providers that expose a reasoning breakdown; left undefined by providers that don't.
	 */
```
```
// packages/coding-agent/src/core/usage-totals.ts:25
	totals.output += usage.output;
	totals.cacheRead += usage.cacheRead;
	totals.cacheWrite += usage.cacheWrite;
```
```
// packages/coding-agent/src/core/usage-totals.ts:26
	totals.cacheRead += usage.cacheRead;
	totals.cacheWrite += usage.cacheWrite;
	totals.cost += usage.cost.total;
```
```
// packages/coding-agent/src/core/usage-totals.ts:37
		cacheWrite: first.cacheWrite + second.cacheWrite,
		...(first.cacheWrite1h !== undefined || second.cacheWrite1h !== undefined
			? { cacheWrite1h: (first.cacheWrite1h ?? 0) + (second.cacheWrite1h ?? 0) }
```
```
// packages/coding-agent/src/core/usage-totals.ts:27
	totals.cacheWrite += usage.cacheWrite;
	totals.cost += usage.cost.total;
}
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 13: code:types.ts->interactive-mode.ts
- E2 AssistantMessage declared at packages/ai/src/types.ts:570
- E2 Usage declared at packages/ai/src/types.ts:451
- E4 AssistantMessage imported at packages/coding-agent/src/modes/interactive/interactive-mode.ts:12 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E4 Usage imported at packages/coding-agent/src/modes/interactive/interactive-mode.ts:12 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E5 AssistantMessage fields read: api (packages/coding-agent/src/modes/interactive/interactive-mode.ts:329), content (packages/coding-agent/src/modes/interactive/interactive-mode.ts:3458), diagnostics (packages/coding-agent/src/modes/interactive/interactive-mode.ts:1915), durationMs (packages/coding-agent/src/modes/interactive/interactive-mode.ts:3648), errorMessage (packages/coding-agent/src/modes/interactive/interactive-mode.ts:2183), model (packages/coding-agent/src/modes/interactive/interactive-mode.ts:955), provider (packages/coding-agent/src/modes/interactive/interactive-mode.ts:329), role (packages/coding-agent/src/modes/interactive/interactive-mode.ts:3503), stopReason (packages/coding-agent/src/modes/interactive/interactive-mode.ts:2182), thinkingLevel (packages/coding-agent/src/modes/interactive/interactive-mode.ts:954), timestamp (packages/coding-agent/src/modes/interactive/interactive-mode.ts:3461), usage (packages/coding-agent/src/modes/interactive/interactive-mode.ts:3477)
- E5 AssistantMessage stoplisted reads: (none)
- E5 Usage fields read: cacheRead (packages/coding-agent/src/modes/interactive/interactive-mode.ts:4139), cacheWrite (packages/coding-agent/src/modes/interactive/interactive-mode.ts:4139), cost (packages/coding-agent/src/modes/interactive/interactive-mode.ts:4140), input (packages/coding-agent/src/modes/interactive/interactive-mode.ts:2627), output (packages/coding-agent/src/modes/interactive/interactive-mode.ts:3891), reasoning (packages/coding-agent/src/modes/interactive/interactive-mode.ts:4562)
- E5 Usage stoplisted reads: (none)
```
// packages/ai/src/types.ts:570
export interface AssistantMessage {
	role: "assistant";
	content: (TextContent | ThinkingContent | ToolCall)[];
	api: Api;
	provider: ProviderId;
	model: string;
	responseModel?: string; // Concrete model reported by the provider when different from the requested `model`
	responseId?: string; // Provider-specific response/message identifier when the upstream API exposes one
	/** Exact provider-native effort level used for this response. Absent for legacy or unmanaged responses. */
	providerThinkingLevel?: string;
	/** Pi thinking level the agent loop requested for this response. Absent outside the agent loop and for legacy responses. */
	thinkingLevel?: ModelThinkingLevel;
```
```
// packages/ai/src/types.ts:451
export interface Usage {
	input: number;
	output: number;
	cacheRead: number;
	cacheWrite: number;
	/** Subset of `cacheWrite` written with 1h retention. Only Anthropic reports this split. */
	cacheWrite1h?: number;
	/**
	 * Reasoning/thinking tokens, when the provider reports them. This is a subset of
	 * `output`: `output` already includes these tokens. Set to a number (possibly 0) by
	 * providers that expose a reasoning breakdown; left undefined by providers that don't.
	 */
```
```
// packages/coding-agent/src/modes/interactive/interactive-mode.ts:329
function isUnknownModel(model: Model<any> | undefined): boolean {
	return !!model && model.provider === "unknown" && model.id === "unknown" && model.api === "unknown";
}
```
```
// packages/coding-agent/src/modes/interactive/interactive-mode.ts:3458
							event.entry.customType,
							event.entry.content,
							event.entry.display,
```
```
// packages/coding-agent/src/modes/interactive/interactive-mode.ts:1915
		if (showDiagnostics) {
			const skillDiagnostics = skillsResult.diagnostics;
			if (skillDiagnostics.length > 0) {
```
```
// packages/coding-agent/src/modes/interactive/interactive-mode.ts:3648
				if (component) {
					component.updateResult({ ...event.result, isError: event.isError, durationMs: event.durationMs });
					this.pendingTools.delete(event.toolCallId);
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 14: code:types.ts->agent-session-concurrent.test.ts
- E2 AssistantMessage declared at packages/ai/src/types.ts:570
- E4 AssistantMessage imported at packages/coding-agent/test/agent-session-concurrent.test.ts:10 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E5 AssistantMessage fields read: content (packages/coding-agent/test/agent-session-concurrent.test.ts:202), role (packages/coding-agent/test/agent-session-concurrent.test.ts:200)
- E5 AssistantMessage stoplisted reads: (none)
```
// packages/ai/src/types.ts:570
export interface AssistantMessage {
	role: "assistant";
	content: (TextContent | ThinkingContent | ToolCall)[];
	api: Api;
	provider: ProviderId;
	model: string;
	responseModel?: string; // Concrete model reported by the provider when different from the requested `model`
	responseId?: string; // Provider-specific response/message identifier when the upstream API exposes one
	/** Exact provider-native effort level used for this response. Absent for legacy or unmanaged responses. */
	providerThinkingLevel?: string;
	/** Pi thinking level the agent loop requested for this response. Absent outside the agent loop and for legacy responses. */
	thinkingLevel?: ModelThinkingLevel;
```
```
// packages/coding-agent/test/agent-session-concurrent.test.ts:202
						.map((message) => {
							if (typeof message.content === "string") {
								return message.content;
```
```
// packages/coding-agent/test/agent-session-concurrent.test.ts:200
					const userTexts = context.messages
						.filter((message) => message.role === "user")
						.map((message) => {
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 15: code:types.ts->agent-session-stats.test.ts
- E2 AssistantMessage declared at packages/ai/src/types.ts:570
- E2 Usage declared at packages/ai/src/types.ts:451
- E4 AssistantMessage imported at packages/coding-agent/test/agent-session-stats.test.ts:2 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E4 Usage imported at packages/coding-agent/test/agent-session-stats.test.ts:2 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E5 AssistantMessage fields read: api (packages/coding-agent/test/agent-session-stats.test.ts:41), provider (packages/coding-agent/test/agent-session-stats.test.ts:42)
- E5 AssistantMessage stoplisted reads: (none)
- E5 Usage fields read: cost (packages/coding-agent/test/agent-session-stats.test.ts:182), input (packages/coding-agent/test/agent-session-stats.test.ts:133)
- E5 Usage stoplisted reads: (none)
```
// packages/ai/src/types.ts:570
export interface AssistantMessage {
	role: "assistant";
	content: (TextContent | ThinkingContent | ToolCall)[];
	api: Api;
	provider: ProviderId;
	model: string;
	responseModel?: string; // Concrete model reported by the provider when different from the requested `model`
	responseId?: string; // Provider-specific response/message identifier when the upstream API exposes one
	/** Exact provider-native effort level used for this response. Absent for legacy or unmanaged responses. */
	providerThinkingLevel?: string;
	/** Pi thinking level the agent loop requested for this response. Absent outside the agent loop and for legacy responses. */
	thinkingLevel?: ModelThinkingLevel;
```
```
// packages/ai/src/types.ts:451
export interface Usage {
	input: number;
	output: number;
	cacheRead: number;
	cacheWrite: number;
	/** Subset of `cacheWrite` written with 1h retention. Only Anthropic reports this split. */
	cacheWrite1h?: number;
	/**
	 * Reasoning/thinking tokens, when the provider reports them. This is a subset of
	 * `output`: `output` already includes these tokens. Set to a number (possibly 0) by
	 * providers that expose a reasoning breakdown; left undefined by providers that don't.
	 */
```
```
// packages/coding-agent/test/agent-session-stats.test.ts:41
		content: [{ type: "text", text }],
		api: model.api,
		provider: model.provider,
```
```
// packages/coding-agent/test/agent-session-stats.test.ts:42
		api: model.api,
		provider: model.provider,
		model: model.id,
```
```
// packages/coding-agent/test/agent-session-stats.test.ts:182
			expect(stats.tokens).toEqual({ input: 10, output: 20, cacheRead: 30, cacheWrite: 40, total: 100 });
			expect(stats.cost).toBe(1);
		} finally {
```
```
// packages/coding-agent/test/agent-session-stats.test.ts:133
			// Totals cover ALL entries, including history compacted away (180k + 195k).
			expect(stats.tokens.input).toBe(375_000);
			expect(stats.contextUsage).toBeDefined();
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 16: code:types.ts->compaction.test.ts
- E2 AssistantMessage declared at packages/ai/src/types.ts:570
- E4 AssistantMessage imported at packages/coding-agent/test/compaction.test.ts:2 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E5 AssistantMessage fields read: content (packages/coding-agent/test/compaction.test.ts:158), model (packages/coding-agent/test/compaction.test.ts:426), role (packages/coding-agent/test/compaction.test.ts:156), thinkingLevel (packages/coding-agent/test/compaction.test.ts:425)
- E5 AssistantMessage stoplisted reads: (none)
```
// packages/ai/src/types.ts:570
export interface AssistantMessage {
	role: "assistant";
	content: (TextContent | ThinkingContent | ToolCall)[];
	api: Api;
	provider: ProviderId;
	model: string;
	responseModel?: string; // Concrete model reported by the provider when different from the requested `model`
	responseId?: string; // Provider-specific response/message identifier when the upstream API exposes one
	/** Exact provider-native effort level used for this response. Absent for legacy or unmanaged responses. */
	providerThinkingLevel?: string;
	/** Pi thinking level the agent loop requested for this response. Absent outside the agent loop and for legacy responses. */
	thinkingLevel?: ModelThinkingLevel;
```
```
// packages/coding-agent/test/compaction.test.ts:158
				case "user":
					return typeof message.content === "string"
						? message.content
```
```
// packages/coding-agent/test/compaction.test.ts:426
		expect(loaded.thinkingLevel).toBe("off");
		expect(loaded.model).toEqual({ provider: "anthropic", modelId: "claude-sonnet-4-5" });
	});
```
```
// packages/coding-agent/test/compaction.test.ts:156
		.map((message) => {
			switch (message.role) {
				case "user":
```
```
// packages/coding-agent/test/compaction.test.ts:425
		expect(loaded.messages.length).toBe(4);
		expect(loaded.thinkingLevel).toBe("off");
		expect(loaded.model).toEqual({ provider: "anthropic", modelId: "claude-sonnet-4-5" });
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 17: code:types.ts->model-registry.test.ts
- E2 Model declared at packages/ai/src/types.ts:839
- E4 Model imported at packages/coding-agent/test/model-registry.test.ts:5 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E5 Model fields read: compat (packages/coding-agent/test/model-registry.test.ts:369), promptCache (packages/coding-agent/test/model-registry.test.ts:829), samplingParams (packages/coding-agent/test/model-registry.test.ts:791), samplingParamsByThinkingLevel (packages/coding-agent/test/model-registry.test.ts:792), thinkingLevelMap (packages/coding-agent/test/model-registry.test.ts:462)
- E5 Model stoplisted reads: (none)
```
// packages/ai/src/types.ts:839
export interface Model<TApi extends Api> extends BaseModel<TApi> {
	/**
	 * Optional: chat is the default model type, so models without `type` are chat
	 * models. Narrow mixed model lists with `isModelType()` instead of comparing
	 * `type` directly.
	 */
	type?: "chat";
	reasoning: boolean;
	/**
	 * Maps pi thinking levels to provider/model-specific values.
	 * Missing keys use provider defaults. null marks a level as unsupported.
	 */
```
```
// packages/coding-agent/test/model-registry.test.ts:369
			const registry = await createModelRegistry(authStorage, modelsJsonPath);
			const compat = registry.find("demo", "demo-model")?.compat as OpenAICompletionsCompat | undefined;

```
```
// packages/coding-agent/test/model-registry.test.ts:829
			expect(registry.getError()).toBeUndefined();
			expect(openrouter.find((m) => m.id === "custom/cached-model")?.promptCache).toEqual({ short: 120 });
			expect(openrouter.find((m) => m.id === "anthropic/claude-sonnet-4.5")?.promptCache).toEqual({ short: 300 });
```
```
// packages/coding-agent/test/model-registry.test.ts:791
			const custom = models.find((m) => m.id === "custom/sampling-model");
			expect(custom?.samplingParams).toEqual({ temperature: 1, top_p: 0.95, top_k: 0 });
			expect(custom?.samplingParamsByThinkingLevel).toEqual({
```
```
// packages/coding-agent/test/model-registry.test.ts:792
			expect(custom?.samplingParams).toEqual({ temperature: 1, top_p: 0.95, top_k: 0 });
			expect(custom?.samplingParamsByThinkingLevel).toEqual({
				low: { temperature: 0.5, top_p: 0.95, top_k: 20 },
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 18: code:types.ts->sdk-codex-cache-probe-tool-loop.ts
- E2 AssistantMessage declared at packages/ai/src/types.ts:570
- E4 AssistantMessage imported at packages/coding-agent/test/sdk-codex-cache-probe-tool-loop.ts:16 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E5 AssistantMessage fields read: content (packages/coding-agent/test/sdk-codex-cache-probe-tool-loop.ts:240), role (packages/coding-agent/test/sdk-codex-cache-probe-tool-loop.ts:354), stopReason (packages/coding-agent/test/sdk-codex-cache-probe-tool-loop.ts:63), usage (packages/coding-agent/test/sdk-codex-cache-probe-tool-loop.ts:62)
- E5 AssistantMessage stoplisted reads: (none)
```
// packages/ai/src/types.ts:570
export interface AssistantMessage {
	role: "assistant";
	content: (TextContent | ThinkingContent | ToolCall)[];
	api: Api;
	provider: ProviderId;
	model: string;
	responseModel?: string; // Concrete model reported by the provider when different from the requested `model`
	responseId?: string; // Provider-specific response/message identifier when the upstream API exposes one
	/** Exact provider-native effort level used for this response. Absent for legacy or unmanaged responses. */
	providerThinkingLevel?: string;
	/** Pi thinking level the agent loop requested for this response. Absent outside the agent loop and for legacy responses. */
	thinkingLevel?: ModelThinkingLevel;
```
```
// packages/coding-agent/test/sdk-codex-cache-probe-tool-loop.ts:240
function getAssistantText(message: AssistantMessage): string {
	return message.content
		.filter((block): block is Extract<AssistantMessage["content"][number], { type: "text" }> => block.type === "text")
```
```
// packages/coding-agent/test/sdk-codex-cache-probe-tool-loop.ts:354
		const assistantMessages = newMessages.filter((message): message is AssistantMessage =>
			Boolean(message && typeof message === "object" && (message as { role?: unknown }).role === "assistant"),
		);
```
```
// packages/coding-agent/test/sdk-codex-cache-probe-tool-loop.ts:63
	usage: AssistantMessage["usage"];
	stopReason: AssistantMessage["stopReason"];
	text: string;
```
```
// packages/coding-agent/test/sdk-codex-cache-probe-tool-loop.ts:62
	elapsedMs: number;
	usage: AssistantMessage["usage"];
	stopReason: AssistantMessage["stopReason"];
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 19: code:types.ts->agent-session-mcp.test.ts
- E2 SystemMessage declared at packages/ai/src/types.ts:546
- E4 SystemMessage imported at packages/coding-agent/test/suite/agent-session-mcp.test.ts:3 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E5 SystemMessage fields read: content (packages/coding-agent/test/suite/agent-session-mcp.test.ts:370), role (packages/coding-agent/test/suite/agent-session-mcp.test.ts:212), sections (packages/coding-agent/test/suite/agent-session-mcp.test.ts:220), toolsAdded (packages/coding-agent/test/suite/agent-session-mcp.test.ts:213), toolsRemoved (packages/coding-agent/test/suite/agent-session-mcp.test.ts:1222)
- E5 SystemMessage stoplisted reads: (none)
```
// packages/ai/src/types.ts:546
export interface SystemMessage {
	role: "system";
	/** Instruction text. On the leading message this is the base prompt; later, additional instructions. */
	content: string | TextContent[];
	/**
	 * Named, ordered prompt sections rendered verbatim after `content`. The leading message
	 * declares them; later messages replace sections by name, and `null` removes one. Keep
	 * each section self-delimiting (a tag, a heading) so the model can relate an update to
	 * the original. Avoid integer-like names; JSON objects reorder those.
	 */
	sections?: Record<string, string | null>;
	/** Complete definitions of tools that become available at this point. */
```
```
// packages/coding-agent/test/suite/agent-session-mcp.test.ts:370
		const savedPath = /^\[Image saved to (\S+\.png) \(image\/png, \d+B\)\]$/.exec(
			(result.content[1] as { text: string }).text,
		)?.[1];
```
```
// packages/coding-agent/test/suite/agent-session-mcp.test.ts:212
		return harness.session.messages
			.filter((message): message is SystemMessage => message.role === "system")
			.flatMap((message) => (message.toolsAdded ?? []).map((tool) => tool.name));
```
```
// packages/coding-agent/test/suite/agent-session-mcp.test.ts:220
		for (const message of harness.session.messages) {
			if (message.role === "system" && message.sections && MCP_SERVERS_SECTION in message.sections) {
				section = message.sections[MCP_SERVERS_SECTION];
```
```
// packages/coding-agent/test/suite/agent-session-mcp.test.ts:213
			.filter((message): message is SystemMessage => message.role === "system")
			.flatMap((message) => (message.toolsAdded ?? []).map((tool) => tool.name));
	}
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 20: code:types.ts->harness.ts
- E2 Model declared at packages/ai/src/types.ts:839
- E4 Model imported at packages/coding-agent/test/suite/harness.ts:11 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E5 Model fields read: contextWindow (packages/coding-agent/test/suite/harness.ts:186), maxTokens (packages/coding-agent/test/suite/harness.ts:187), reasoning (packages/coding-agent/test/suite/harness.ts:182)
- E5 Model stoplisted reads: type
```
// packages/ai/src/types.ts:839
export interface Model<TApi extends Api> extends BaseModel<TApi> {
	/**
	 * Optional: chat is the default model type, so models without `type` are chat
	 * models. Narrow mixed model lists with `isModelType()` instead of comparing
	 * `type` directly.
	 */
	type?: "chat";
	reasoning: boolean;
	/**
	 * Maps pi thinking levels to provider/model-specific values.
	 * Missing keys use provider defaults. null marks a level as unsupported.
	 */
```
```
// packages/coding-agent/test/suite/harness.ts:186
				cost: registeredModel.cost,
				contextWindow: registeredModel.contextWindow,
				maxTokens: registeredModel.maxTokens,
```
```
// packages/coding-agent/test/suite/harness.ts:187
				contextWindow: registeredModel.contextWindow,
				maxTokens: registeredModel.maxTokens,
				baseUrl: registeredModel.baseUrl,
```
```
// packages/coding-agent/test/suite/harness.ts:182
				api: registeredModel.api,
				reasoning: registeredModel.reasoning,
				input: registeredModel.input,
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 21: code:agent-session-services.ts->tui.docs.eval.ts
- E2 AgentSessionServices declared at packages/coding-agent/src/core/agent-session-services.ts:73
- E4 AgentSessionServices imported at packages/evals/evals/tui.docs.eval.ts:4 from "@earendil-works/pi-coding-agent", resolved to packages/coding-agent/src/index.ts, re-exported through packages/coding-agent/src/index.ts:250 -> packages/coding-agent/src/core/sdk.ts:120 -> packages/coding-agent/src/core/agent-session-runtime.ts:440 from packages/coding-agent/src/core/agent-session-services.ts
- E5 AgentSessionServices fields read: cwd (packages/evals/evals/tui.docs.eval.ts:128), modelRuntime (packages/evals/evals/tui.docs.eval.ts:120), resourceLoader (packages/evals/evals/tui.docs.eval.ts:122), settingsManager (packages/evals/evals/tui.docs.eval.ts:121)
- E5 AgentSessionServices stoplisted reads: (none)
```
// packages/coding-agent/src/core/agent-session-services.ts:73
export interface AgentSessionServices {
	cwd: string;
	agentDir: string;
	modelRuntime: ModelRuntime;
	settingsManager: SettingsManager;
	resourceLoader: ResourceLoader;
	diagnostics: AgentSessionRuntimeDiagnostic[];
}

function applyExtensionFlagValues(
	resourceLoader: ResourceLoader,
	extensionFlagValues: Map<string, boolean | string> | undefined,
```
```
// packages/evals/evals/tui.docs.eval.ts:128
			services,
			sessionManager: SessionManager.inMemory(services.cwd),
			model,
```
```
// packages/evals/evals/tui.docs.eval.ts:120
		agentDir,
		modelRuntime: session.modelRuntime,
		settingsManager: session.settingsManager,
```
```
// packages/evals/evals/tui.docs.eval.ts:122
		settingsManager: session.settingsManager,
		resourceLoader: session.resourceLoader,
		diagnostics: [],
```
```
// packages/evals/evals/tui.docs.eval.ts:121
		modelRuntime: session.modelRuntime,
		settingsManager: session.settingsManager,
		resourceLoader: session.resourceLoader,
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 22: code:types.ts->index.ts#6
- E2 ExtensionContext declared at packages/coding-agent/src/core/extensions/types.ts:325
- E4 ExtensionContext imported at packages/coding-agent/examples/extensions/gondolin/index.ts:24 from "@earendil-works/pi-coding-agent", resolved to packages/coding-agent/src/index.ts, re-exported through packages/coding-agent/src/index.ts:54 -> packages/coding-agent/src/core/extensions/index.ts:22 from packages/coding-agent/src/core/extensions/types.ts
- E5 ExtensionContext fields read: cwd (packages/coding-agent/examples/extensions/gondolin/index.ts:366), signal (packages/coding-agent/examples/extensions/gondolin/index.ts:346), ui (packages/coding-agent/examples/extensions/gondolin/index.ts:380)
- E5 ExtensionContext stoplisted reads: (none)
```
// packages/coding-agent/src/core/extensions/types.ts:325
export interface ExtensionContext {
	/** UI methods for user interaction */
	ui: ExtensionUIContext;
	/** Current run mode. Use "tui" to guard terminal-only UI such as custom components. */
	mode: ExtensionMode;
	/** Whether dialog-capable UI is available (true in TUI and RPC modes) */
	hasUI: boolean;
	/** Current working directory */
	cwd: string;
	/** Session manager (read-only) */
	sessionManager: ReadonlySessionManager;
	/** Model registry for API key resolution */
```
```
// packages/coding-agent/examples/extensions/gondolin/index.ts:366
export default function (pi: ExtensionAPI) {
	const localCwd = process.cwd();
	const localRead = createReadTool(localCwd);
```
```
// packages/coding-agent/examples/extensions/gondolin/index.ts:346
					env: sanitizeEnv(env),
					signal: controller.signal,
					stdout: "pipe",
```
```
// packages/coding-agent/examples/extensions/gondolin/index.ts:380
	async function startVm(ctx?: ExtensionContext): Promise<VM> {
		ctx?.ui.setStatus("gondolin", ctx.ui.theme.fg("accent", `Gondolin: starting ${GUEST_WORKSPACE}`));
		const created = await VM.create({
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 23: code:edit.ts->index.ts
- E2 EditOperations declared at packages/coding-agent/src/core/tools/edit.ts:83
- E4 EditOperations imported at packages/coding-agent/examples/extensions/gondolin/index.ts:25 from "@earendil-works/pi-coding-agent", resolved to packages/coding-agent/src/index.ts, re-exported through packages/coding-agent/src/index.ts:336 -> packages/coding-agent/src/core/tools/index.ts:12 from packages/coding-agent/src/core/tools/edit.ts
- E5 EditOperations fields read: access (packages/coding-agent/examples/extensions/gondolin/index.ts:88), readFile (packages/coding-agent/examples/extensions/gondolin/index.ts:86), writeFile (packages/coding-agent/examples/extensions/gondolin/index.ts:104)
- E5 EditOperations stoplisted reads: (none)
```
// packages/coding-agent/src/core/tools/edit.ts:83
export interface EditOperations {
	/** Read file contents as a Buffer */
	readFile: (absolutePath: string) => Promise<Buffer>;
	/** Write content to a file */
	writeFile: (absolutePath: string, content: string) => Promise<void>;
	/** Check if file is readable and writable (throw if not) */
	access: (absolutePath: string) => Promise<void>;
}

const defaultEditOperations: EditOperations = {
	readFile: (path) => fsReadFile(path),
	writeFile: (path, content) => fsWriteFile(path, content, "utf-8"),
```
```
// packages/coding-agent/examples/extensions/gondolin/index.ts:88
		access: async (filePath) => {
			await vm.fs.access(toGuestPath(localCwd, filePath));
		},
```
```
// packages/coding-agent/examples/extensions/gondolin/index.ts:86
	return {
		readFile: async (filePath) => vm.fs.readFile(toGuestPath(localCwd, filePath)),
		access: async (filePath) => {
```
```
// packages/coding-agent/examples/extensions/gondolin/index.ts:104
		writeFile: async (filePath, content) => {
			await vm.fs.writeFile(toGuestPath(localCwd, filePath), content, { encoding: "utf8" });
		},
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 24: code:grep.ts->index.ts
- E2 GrepToolDetails declared at packages/coding-agent/src/core/tools/grep.ts:43
- E4 GrepToolDetails imported at packages/coding-agent/examples/extensions/gondolin/index.ts:25 from "@earendil-works/pi-coding-agent", resolved to packages/coding-agent/src/index.ts, re-exported through packages/coding-agent/src/index.ts:336 -> packages/coding-agent/src/core/tools/index.ts:29 from packages/coding-agent/src/core/tools/grep.ts
- E5 GrepToolDetails fields read: linesTruncated (packages/coding-agent/examples/extensions/gondolin/index.ts:300), matchLimitReached (packages/coding-agent/examples/extensions/gondolin/index.ts:296), truncation (packages/coding-agent/examples/extensions/gondolin/index.ts:304)
- E5 GrepToolDetails stoplisted reads: (none)
```
// packages/coding-agent/src/core/tools/grep.ts:43
export interface GrepToolDetails {
	truncation?: TruncationResult;
	matchLimitReached?: number;
	linesTruncated?: boolean;
}

/**
 * Pluggable operations for the grep tool.
 * Override these to delegate search to remote systems (for example SSH).
 */
export interface GrepOperations {
	/** Check if path is a directory. Throws if path does not exist. */
```
```
// packages/coding-agent/examples/extensions/gondolin/index.ts:300
	if (linesTruncated) {
		details.linesTruncated = true;
		notices.push("long lines truncated");
```
```
// packages/coding-agent/examples/extensions/gondolin/index.ts:296
	if (matchLimitReached) {
		details.matchLimitReached = effectiveLimit;
		notices.push(`${effectiveLimit} matches limit reached`);
```
```
// packages/coding-agent/examples/extensions/gondolin/index.ts:304
	if (truncation.truncated) {
		details.truncation = truncation;
		notices.push(`${formatSize(DEFAULT_MAX_BYTES)} limit reached`);
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 25: code:read.ts->index.ts
- E2 ReadOperations declared at packages/coding-agent/src/core/tools/read.ts:48
- E4 ReadOperations imported at packages/coding-agent/examples/extensions/gondolin/index.ts:25 from "@earendil-works/pi-coding-agent", resolved to packages/coding-agent/src/index.ts, re-exported through packages/coding-agent/src/index.ts:336 -> packages/coding-agent/src/core/tools/index.ts:56 from packages/coding-agent/src/core/tools/read.ts
- E5 ReadOperations fields read: access (packages/coding-agent/examples/extensions/gondolin/index.ts:88), readFile (packages/coding-agent/examples/extensions/gondolin/index.ts:86)
- E5 ReadOperations stoplisted reads: (none)
```
// packages/coding-agent/src/core/tools/read.ts:48
export interface ReadOperations {
	/** Read file contents as a Buffer */
	readFile: (absolutePath: string) => Promise<Buffer>;
	/** Check if file is readable (throw if not) */
	access: (absolutePath: string) => Promise<void>;
	/** Detect image MIME type, return null or undefined for non-images */
	detectImageMimeType?: (absolutePath: string) => Promise<string | null | undefined>;
}

const defaultReadOperations: ReadOperations = {
	readFile: (path) => fsReadFile(path),
	access: (path) => fsAccess(path, constants.R_OK),
```
```
// packages/coding-agent/examples/extensions/gondolin/index.ts:88
		access: async (filePath) => {
			await vm.fs.access(toGuestPath(localCwd, filePath));
		},
```
```
// packages/coding-agent/examples/extensions/gondolin/index.ts:86
	return {
		readFile: async (filePath) => vm.fs.readFile(toGuestPath(localCwd, filePath)),
		access: async (filePath) => {
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 26: code:write.ts->index.ts
- E2 WriteOperations declared at packages/coding-agent/src/core/tools/write.ts:27
- E4 WriteOperations imported at packages/coding-agent/examples/extensions/gondolin/index.ts:25 from "@earendil-works/pi-coding-agent", resolved to packages/coding-agent/src/index.ts, re-exported through packages/coding-agent/src/index.ts:336 -> packages/coding-agent/src/core/tools/index.ts:74 from packages/coding-agent/src/core/tools/write.ts
- E5 WriteOperations fields read: mkdir (packages/coding-agent/examples/extensions/gondolin/index.ts:107), writeFile (packages/coding-agent/examples/extensions/gondolin/index.ts:104)
- E5 WriteOperations stoplisted reads: (none)
```
// packages/coding-agent/src/core/tools/write.ts:27
export interface WriteOperations {
	/** Write content to a file */
	writeFile: (absolutePath: string, content: string) => Promise<void>;
	/** Create directory recursively */
	mkdir: (dir: string) => Promise<void>;
}

const defaultWriteOperations: WriteOperations = {
	writeFile: (path, content) => fsWriteFile(path, content, "utf-8"),
	mkdir: (dir) => fsMkdir(dir, { recursive: true }).then(() => {}),
};

```
```
// packages/coding-agent/examples/extensions/gondolin/index.ts:107
		mkdir: async (dirPath) => {
			await vm.fs.mkdir(toGuestPath(localCwd, dirPath), { recursive: true });
		},
```
```
// packages/coding-agent/examples/extensions/gondolin/index.ts:104
		writeFile: async (filePath, content) => {
			await vm.fs.writeFile(toGuestPath(localCwd, filePath), content, { encoding: "utf8" });
		},
```
Judgment (TRUE / NOT TRUE, one sentence):

## earendil-works__pi: 1 existing proposal(s) with new shapes (reported separately)

### earendil-works__pi G1: code:types.ts->agent-session-codemode.test.ts (judge only: Usage)
- E2 Usage declared at packages/ai/src/types.ts:451
- E4 Usage imported at packages/coding-agent/test/suite/agent-session-codemode.test.ts:16 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E5 Usage fields read: cost (packages/coding-agent/test/suite/agent-session-codemode.test.ts:330), input (packages/coding-agent/test/suite/agent-session-codemode.test.ts:252)
- E5 Usage stoplisted reads: (none)
```
// packages/ai/src/types.ts:451
export interface Usage {
	input: number;
	output: number;
	cacheRead: number;
	cacheWrite: number;
	/** Subset of `cacheWrite` written with 1h retention. Only Anthropic reports this split. */
	cacheWrite1h?: number;
	/**
	 * Reasoning/thinking tokens, when the provider reports them. This is a subset of
	 * `output`: `output` already includes these tokens. Set to a number (possibly 0) by
	 * providers that expose a reasoning breakdown; left undefined by providers that don't.
	 */
```
```
// packages/coding-agent/test/suite/agent-session-codemode.test.ts:330
		).toEqual(result.usage);
		expect(harness.session.getSessionStats().cost).toBe(0.5);
	});
```
```
// packages/coding-agent/test/suite/agent-session-codemode.test.ts:252
				pi.on("tool_call", (event) => {
					if (event.toolName === "echo" && (event.input as { text: string }).text === "forbidden") {
						return { block: true, reason: "echo of forbidden text is blocked" };
```
Judgment (TRUE / NOT TRUE, one sentence):

## puppeteer__puppeteer: 3 new proposal(s), all adjudicated

### puppeteer__puppeteer 1: code:Issue.ts->webmcp.test.ts
- E2 Issue declared at packages/puppeteer-core/src/api/Issue.ts:14
- E4 Issue imported at test/src/cdp/webmcp.test.ts:10 from "puppeteer", resolved to packages/puppeteer/src/puppeteer.ts, re-exported through packages/puppeteer/src/puppeteer.ts:7 -> packages/puppeteer-core/src/puppeteer-core.ts:10 -> packages/puppeteer-core/src/index.ts:7 -> packages/puppeteer-core/src/index-browser.ts:10 -> packages/puppeteer-core/src/api/api.ts:20 from packages/puppeteer-core/src/api/Issue.ts
- E5 Issue fields read: code (test/src/cdp/webmcp.test.ts:777), details (test/src/cdp/webmcp.test.ts:778)
- E5 Issue stoplisted reads: (none)
```
// packages/puppeteer-core/src/api/Issue.ts:14
export interface Issue {
  /**
   * The code of the issue.
   */
  code: string;

  /**
   * The details of the issue.
   */
  details: Protocol.Audits.InspectorIssueDetails;
}
```
```
// test/src/cdp/webmcp.test.ts:777
    expect(issue).toBeTruthy();
    expect(issue.code).toBe('GenericIssue');
    expect(issue.details.genericIssueDetails).toBeTruthy();
```
```
// test/src/cdp/webmcp.test.ts:778
    expect(issue.code).toBe('GenericIssue');
    expect(issue.details.genericIssueDetails).toBeTruthy();
    expect(issue.details.genericIssueDetails!.errorType).toBe(
```
Judgment (TRUE / NOT TRUE, one sentence):

### puppeteer__puppeteer 2: code:Configuration.ts->getConfiguration.ts
- E2 ChromeHeadlessShellSettings declared at packages/puppeteer-core/src/common/Configuration.ts:127
- E2 ChromeSettings declared at packages/puppeteer-core/src/common/Configuration.ts:87
- E2 Configuration declared at packages/puppeteer-core/src/common/Configuration.ts:26
- E2 FirefoxSettings declared at packages/puppeteer-core/src/common/Configuration.ts:167
- E4 ChromeHeadlessShellSettings imported at packages/puppeteer/src/getConfiguration.ts:11 from "puppeteer-core", resolved to packages/puppeteer-core/src/puppeteer-core.ts, re-exported through packages/puppeteer-core/src/puppeteer-core.ts:10 -> packages/puppeteer-core/src/index.ts:7 -> packages/puppeteer-core/src/index-browser.ts:12 -> packages/puppeteer-core/src/common/common.ts:10 from packages/puppeteer-core/src/common/Configuration.ts
- E4 ChromeSettings imported at packages/puppeteer/src/getConfiguration.ts:11 from "puppeteer-core", resolved to packages/puppeteer-core/src/puppeteer-core.ts, re-exported through packages/puppeteer-core/src/puppeteer-core.ts:10 -> packages/puppeteer-core/src/index.ts:7 -> packages/puppeteer-core/src/index-browser.ts:12 -> packages/puppeteer-core/src/common/common.ts:10 from packages/puppeteer-core/src/common/Configuration.ts
- E4 Configuration imported at packages/puppeteer/src/getConfiguration.ts:11 from "puppeteer-core", resolved to packages/puppeteer-core/src/puppeteer-core.ts, re-exported through packages/puppeteer-core/src/puppeteer-core.ts:10 -> packages/puppeteer-core/src/index.ts:7 -> packages/puppeteer-core/src/index-browser.ts:12 -> packages/puppeteer-core/src/common/common.ts:10 from packages/puppeteer-core/src/common/Configuration.ts
- E4 FirefoxSettings imported at packages/puppeteer/src/getConfiguration.ts:11 from "puppeteer-core", resolved to packages/puppeteer-core/src/puppeteer-core.ts, re-exported through packages/puppeteer-core/src/puppeteer-core.ts:10 -> packages/puppeteer-core/src/index.ts:7 -> packages/puppeteer-core/src/index-browser.ts:12 -> packages/puppeteer-core/src/common/common.ts:10 from packages/puppeteer-core/src/common/Configuration.ts
- E5 ChromeHeadlessShellSettings fields read: downloadBaseUrl (packages/puppeteer/src/getConfiguration.ts:93), skipDownload (packages/puppeteer/src/getConfiguration.ts:98)
- E5 ChromeHeadlessShellSettings stoplisted reads: version
- E5 ChromeSettings fields read: downloadBaseUrl (packages/puppeteer/src/getConfiguration.ts:93), skipDownload (packages/puppeteer/src/getConfiguration.ts:98)
- E5 ChromeSettings stoplisted reads: version
- E5 Configuration fields read: cacheDirectory (packages/puppeteer/src/getConfiguration.ts:162), chrome (packages/puppeteer/src/getConfiguration.ts:153), defaultBrowser (packages/puppeteer/src/getConfiguration.ts:136), executablePath (packages/puppeteer/src/getConfiguration.ts:140), experiments (packages/puppeteer/src/getConfiguration.ts:170), firefox (packages/puppeteer/src/getConfiguration.ts:158), logLevel (packages/puppeteer/src/getConfiguration.ts:131), skipDownload (packages/puppeteer/src/getConfiguration.ts:98), temporaryDirectory (packages/puppeteer/src/getConfiguration.ts:167)
- E5 Configuration stoplisted reads: (none)
- E5 FirefoxSettings fields read: downloadBaseUrl (packages/puppeteer/src/getConfiguration.ts:93), skipDownload (packages/puppeteer/src/getConfiguration.ts:98)
- E5 FirefoxSettings stoplisted reads: version
```
// packages/puppeteer-core/src/common/Configuration.ts:127
export interface ChromeHeadlessShellSettings {
  /**
   * Tells Puppeteer to not download the browser during installation.
   *
   * Can be overridden by `PUPPETEER_CHROME_HEADLESS_SHELL_SKIP_DOWNLOAD`
   * or `PUPPETEER_SKIP_CHROME_HEADLESS_SHELL_DOWNLOAD`.
   *
   * @defaultValue false
   */
  skipDownload?: boolean;
  /**
   * Specifies the URL prefix that is used to download the browser.
```
```
// packages/puppeteer-core/src/common/Configuration.ts:87
export interface ChromeSettings {
  /**
   * Tells Puppeteer to not download the browser during installation.
   *
   * Can be overridden by `PUPPETEER_CHROME_SKIP_DOWNLOAD`.
   *
   * @defaultValue false
   */
  skipDownload?: boolean;
  /**
   * Specifies the URL prefix that is used to download the browser.
   *
```
```
// packages/puppeteer-core/src/common/Configuration.ts:26
export interface Configuration {
  /**
   * Defines the directory to be used by Puppeteer for caching.
   *
   * Can be overridden by `PUPPETEER_CACHE_DIR`.
   *
   * @defaultValue `path.join(os.homedir(), '.cache', 'puppeteer')`
   */
  cacheDirectory?: string;
  /**
   * Specifies an executable path to be used in
   * {@link PuppeteerNode.launch | puppeteer.launch}.
```
```
// packages/puppeteer-core/src/common/Configuration.ts:167
export interface FirefoxSettings {
  /**
   * Tells Puppeteer to not download the browser during installation.
   *
   * Can be overridden by `PUPPETEER_FIREFOX_SKIP_DOWNLOAD`.
   *
   * @defaultValue true
   */
  skipDownload?: boolean;
  /**
   * Specifies the URL prefix that is used to download the browser.
   *
```
```
// packages/puppeteer/src/getConfiguration.ts:93

  browserSetting.downloadBaseUrl =
    process.env[`PUPPETEER_${browserEnvName}_DOWNLOAD_BASE_URL`] ??
```
```
// packages/puppeteer/src/getConfiguration.ts:98

  browserSetting.skipDownload =
    getBooleanEnvVar(`PUPPETEER_${browserEnvName}_SKIP_DOWNLOAD`) ??
```
```
// packages/puppeteer/src/getConfiguration.ts:162

  configuration.cacheDirectory =
    process.env['PUPPETEER_CACHE_DIR'] ??
```
```
// packages/puppeteer/src/getConfiguration.ts:153
  // Prepare variables used in browser downloading
  configuration.chrome = getBrowserSetting('chrome', configuration);
  configuration['chrome-headless-shell'] = getBrowserSetting(
```
Judgment (TRUE / NOT TRUE, one sentence):

### puppeteer__puppeteer 3: code:Configuration.ts->install.ts
- E2 ChromeHeadlessShellSettings declared at packages/puppeteer-core/src/common/Configuration.ts:127
- E2 ChromeSettings declared at packages/puppeteer-core/src/common/Configuration.ts:87
- E2 FirefoxSettings declared at packages/puppeteer-core/src/common/Configuration.ts:167
- E4 ChromeHeadlessShellSettings imported at packages/puppeteer/src/node/install.ts:14 from "puppeteer-core", resolved to packages/puppeteer-core/src/puppeteer-core.ts, re-exported through packages/puppeteer-core/src/puppeteer-core.ts:10 -> packages/puppeteer-core/src/index.ts:7 -> packages/puppeteer-core/src/index-browser.ts:12 -> packages/puppeteer-core/src/common/common.ts:10 from packages/puppeteer-core/src/common/Configuration.ts
- E4 ChromeSettings imported at packages/puppeteer/src/node/install.ts:14 from "puppeteer-core", resolved to packages/puppeteer-core/src/puppeteer-core.ts, re-exported through packages/puppeteer-core/src/puppeteer-core.ts:10 -> packages/puppeteer-core/src/index.ts:7 -> packages/puppeteer-core/src/index-browser.ts:12 -> packages/puppeteer-core/src/common/common.ts:10 from packages/puppeteer-core/src/common/Configuration.ts
- E4 FirefoxSettings imported at packages/puppeteer/src/node/install.ts:14 from "puppeteer-core", resolved to packages/puppeteer-core/src/puppeteer-core.ts, re-exported through packages/puppeteer-core/src/puppeteer-core.ts:10 -> packages/puppeteer-core/src/index.ts:7 -> packages/puppeteer-core/src/index-browser.ts:12 -> packages/puppeteer-core/src/common/common.ts:10 from packages/puppeteer-core/src/common/Configuration.ts
- E5 ChromeHeadlessShellSettings fields read: downloadBaseUrl (packages/puppeteer/src/node/install.ts:39), skipDownload (packages/puppeteer/src/node/install.ts:71)
- E5 ChromeHeadlessShellSettings stoplisted reads: version
- E5 ChromeSettings fields read: downloadBaseUrl (packages/puppeteer/src/node/install.ts:39), skipDownload (packages/puppeteer/src/node/install.ts:71)
- E5 ChromeSettings stoplisted reads: version
- E5 FirefoxSettings fields read: downloadBaseUrl (packages/puppeteer/src/node/install.ts:39), skipDownload (packages/puppeteer/src/node/install.ts:71)
- E5 FirefoxSettings stoplisted reads: version
```
// packages/puppeteer-core/src/common/Configuration.ts:127
export interface ChromeHeadlessShellSettings {
  /**
   * Tells Puppeteer to not download the browser during installation.
   *
   * Can be overridden by `PUPPETEER_CHROME_HEADLESS_SHELL_SKIP_DOWNLOAD`
   * or `PUPPETEER_SKIP_CHROME_HEADLESS_SHELL_DOWNLOAD`.
   *
   * @defaultValue false
   */
  skipDownload?: boolean;
  /**
   * Specifies the URL prefix that is used to download the browser.
```
```
// packages/puppeteer-core/src/common/Configuration.ts:87
export interface ChromeSettings {
  /**
   * Tells Puppeteer to not download the browser during installation.
   *
   * Can be overridden by `PUPPETEER_CHROME_SKIP_DOWNLOAD`.
   *
   * @defaultValue false
   */
  skipDownload?: boolean;
  /**
   * Specifies the URL prefix that is used to download the browser.
   *
```
```
// packages/puppeteer-core/src/common/Configuration.ts:167
export interface FirefoxSettings {
  /**
   * Tells Puppeteer to not download the browser during installation.
   *
   * Can be overridden by `PUPPETEER_FIREFOX_SKIP_DOWNLOAD`.
   *
   * @defaultValue true
   */
  skipDownload?: boolean;
  /**
   * Specifies the URL prefix that is used to download the browser.
   *
```
```
// packages/puppeteer/src/node/install.ts:39
    configuration?.version || PUPPETEER_REVISIONS[browser] || 'latest';
  const baseUrl = configuration?.downloadBaseUrl;
  const buildId = await resolveBuildId(browser, platform, unresolvedBuildId);
```
```
// packages/puppeteer/src/node/install.ts:71
  const configuration = await getConfiguration();
  if (configuration.skipDownload) {
    logPolitely('**INFO** Skipping downloading browsers as instructed.');
```
Judgment (TRUE / NOT TRUE, one sentence):

