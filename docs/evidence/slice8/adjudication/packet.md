# Property-route probe packet

### Egonex-AI__Understand-Anything 1: GraphEdge (understand-anything-plugin/packages/core/src/types.ts:70) read in understand-anything-plugin/packages/dashboard/src/components/DomainGraphView.tsx
- fields to check: source,target,weight
```
// understand-anything-plugin/packages/core/src/types.ts:70
export interface GraphEdge {
  source: string;
  target: string;
  type: EdgeType;
  direction: "forward" | "backward" | "bidirectional";
  description?: string;
  weight: number; // 0-1
}

// Layer (logical grouping)
export interface Layer {
  id: string;
  name: string;
  description: string;
```
```
// understand-anything-plugin/packages/dashboard/src/components/DomainGraphView.tsx:49
    if (edge.type === "contains_flow") {
      flowCountMap.set(edge.source, (flowCountMap.get(edge.source) ?? 0) + 1);
```
```
// understand-anything-plugin/packages/dashboard/src/components/DomainGraphView.tsx:75
    .map((e, i) => ({
      id: `cd-${i}-${e.source}-${e.target}`,
```
```
// understand-anything-plugin/packages/dashboard/src/components/DomainGraphView.tsx:111
  for (const edge of stepEdges) {
    stepOrderMap.set(edge.target, edge.weight);
```
Judgment (TRUE / NOT TRUE, one sentence):

### Egonex-AI__Understand-Anything 2: GraphNode (understand-anything-plugin/packages/core/src/types.ts:54) read in understand-anything-plugin/packages/dashboard/src/components/KnowledgeGraphView.tsx
- fields to check: complexity,summary,tags
```
// understand-anything-plugin/packages/core/src/types.ts:54
export interface GraphNode {
  id: string;
  type: NodeType;
  name: string;
  filePath?: string;
  lineRange?: [number, number];
  summary: string;
  tags: string[];
  complexity: "simple" | "moderate" | "complex";
  languageNotes?: string;
  domainMeta?: DomainMeta;
  knowledgeMeta?: KnowledgeMeta;
  figmaMeta?: FigmaMeta;
}
```
```
// understand-anything-plugin/packages/dashboard/src/components/KnowledgeGraphView.tsx:283
        summary: node.summary,
        complexity: node.complexity,
```
```
// understand-anything-plugin/packages/dashboard/src/components/KnowledgeGraphView.tsx:282
        nodeType: node.type,
        summary: node.summary,
```
```
// understand-anything-plugin/packages/dashboard/src/components/KnowledgeGraphView.tsx:295
        incomingCount: edgeCounts.get(node.id) ?? 0,
        tags: node.tags,
```
Judgment (TRUE / NOT TRUE, one sentence):

### Egonex-AI__Understand-Anything 3: GraphEdge (understand-anything-plugin/packages/core/src/types.ts:70) read in understand-anything-plugin/packages/dashboard/src/components/KnowledgeGraphView.tsx
- fields to check: source,target
```
// understand-anything-plugin/packages/core/src/types.ts:70
export interface GraphEdge {
  source: string;
  target: string;
  type: EdgeType;
  direction: "forward" | "backward" | "bidirectional";
  description?: string;
  weight: number; // 0-1
}

// Layer (logical grouping)
export interface Layer {
  id: string;
  name: string;
  description: string;
```
```
// understand-anything-plugin/packages/dashboard/src/components/KnowledgeGraphView.tsx:86
  for (const edge of graph.edges) {
    edgeCounts.set(edge.source, (edgeCounts.get(edge.source) ?? 0) + 1);
```
```
// understand-anything-plugin/packages/dashboard/src/components/KnowledgeGraphView.tsx:87
    edgeCounts.set(edge.source, (edgeCounts.get(edge.source) ?? 0) + 1);
    edgeCounts.set(edge.target, (edgeCounts.get(edge.target) ?? 0) + 1);
```
Judgment (TRUE / NOT TRUE, one sentence):

### Egonex-AI__Understand-Anything 4: KnowledgeMeta (understand-anything-plugin/packages/core/src/types.ts:24) read in understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx
- fields to check: content,wikilinks
```
// understand-anything-plugin/packages/core/src/types.ts:24
export interface KnowledgeMeta {
  wikilinks?: string[];
  backlinks?: string[];
  category?: string;
  content?: string;
}

// Optional domain metadata for domain/flow/step nodes
export interface DomainMeta {
  entities?: string[];
  businessRules?: string[];
  crossDomainInteractions?: string[];
  entryPoint?: string;
  entryType?: "http" | "cli" | "event" | "cron" | "manual";
```
```
// understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:142
      )}
      {meta?.content && (
```
```
// understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:104
      )}
      {meta?.wikilinks && meta.wikilinks.length > 0 && (
```
Judgment (TRUE / NOT TRUE, one sentence):

### Egonex-AI__Understand-Anything 5: DomainMeta (understand-anything-plugin/packages/core/src/types.ts:32) read in understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx
- fields to check: businessRules,crossDomainInteractions,entities,entryPoint
```
// understand-anything-plugin/packages/core/src/types.ts:32
export interface DomainMeta {
  entities?: string[];
  businessRules?: string[];
  crossDomainInteractions?: string[];
  entryPoint?: string;
  entryType?: "http" | "cli" | "event" | "cron" | "manual";
}

// Optional Figma metadata for page/screen/component/componentSet/instance/token nodes
export interface FigmaMeta {
  fileKey?: string;
  nodeId?: string;            // Figma node id, e.g. "1:23"
  figmaType?: string;         // FRAME | COMPONENT | COMPONENT_SET | INSTANCE | TEXT ...
  thumbnailUrl?: string;      // lazily filled from GET /v1/images
```
```
// understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:181
        ) : null}
        {Array.isArray(meta?.businessRules) && meta.businessRules.length > 0 ? (
```
```
// understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:191
        ) : null}
        {Array.isArray(meta?.crossDomainInteractions) && meta.crossDomainInteractions.length > 0 ? (
```
```
// understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:171
      <div className="space-y-3">
        {Array.isArray(meta?.entities) && meta.entities.length > 0 ? (
```
```
// understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:231
      <div className="space-y-3">
        {meta?.entryPoint ? (
```
Judgment (TRUE / NOT TRUE, one sentence):

### Egonex-AI__Understand-Anything 6: GraphEdge (understand-anything-plugin/packages/core/src/types.ts:70) read in understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx
- fields to check: source,target
```
// understand-anything-plugin/packages/core/src/types.ts:70
export interface GraphEdge {
  source: string;
  target: string;
  type: EdgeType;
  direction: "forward" | "backward" | "bidirectional";
  description?: string;
  weight: number; // 0-1
}

// Layer (logical grouping)
export interface Layer {
  id: string;
  name: string;
  description: string;
```
```
// understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:72
  const wikilinks = graph.edges
    .filter((e) => e.type === "related" && e.source === node.id)
```
```
// understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:73
    .filter((e) => e.type === "related" && e.source === node.id)
    .map((e) => graph.nodes.find((n) => n.id === e.target))
```
Judgment (TRUE / NOT TRUE, one sentence):

### Egonex-AI__Understand-Anything 7: ProjectMeta (understand-anything-plugin/packages/core/src/types.ts:97) read in understand-anything-plugin/src/context-builder.ts
- fields to check: frameworks,languages
```
// understand-anything-plugin/packages/core/src/types.ts:97
export interface ProjectMeta {
  name: string;
  languages: string[];
  frameworks: string[];
  description: string;
  analyzedAt: string;
  gitCommitHash: string;
}

// Root KnowledgeGraph
export interface KnowledgeGraph {
  version: string;
  kind?: "codebase" | "knowledge" | "design";
  project: ProjectMeta;
```
```
// understand-anything-plugin/src/context-builder.ts:74
    languages: graph.project.languages,
    frameworks: graph.project.frameworks,
```
```
// understand-anything-plugin/src/context-builder.ts:73
    projectDescription: graph.project.description,
    languages: graph.project.languages,
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 8: ModelAuth (packages/ai/src/auth/types.ts:7) read in packages/coding-agent/src/cli/auth-command.ts
- fields to check: apiKey,headers
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
	key?: string;
	env?: ProviderEnv;
```
```
// packages/coding-agent/src/cli/auth-command.ts:105
	}
	if (args.apiKey !== undefined || args.messages.length > 0 || args.fileArgs.length > 0) {
```
```
// packages/coding-agent/src/cli/auth-command.ts:122
	if (auth?.auth.apiKey) return auth.auth.apiKey;
	const authorization = Object.entries(auth?.auth.headers ?? {}).find(
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 9: ModelAuth (packages/ai/src/auth/types.ts:7) read in packages/coding-agent/src/core/agent-session.ts
- fields to check: apiKey,baseUrl,headers
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
	key?: string;
	env?: ProviderEnv;
```
```
// packages/coding-agent/src/core/agent-session.ts:552
		}
		if (result && (result.auth.apiKey || result.auth.headers)) {
```
```
// packages/coding-agent/src/core/agent-session.ts:553
		if (result && (result.auth.apiKey || result.auth.headers)) {
			const requestModel = result.auth.baseUrl ? { ...model, baseUrl: result.auth.baseUrl } : model;
```
```
// packages/coding-agent/src/core/agent-session.ts:552
		}
		if (result && (result.auth.apiKey || result.auth.headers)) {
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 10: ProviderAuth (packages/ai/src/auth/types.ts:252) read in packages/coding-agent/src/core/bug-report.ts
- fields to check: apiKey,oauth
```
// packages/ai/src/auth/types.ts:252
export interface ProviderAuth {
	apiKey?: ApiKeyAuth;
	oauth?: OAuthAuth;
}
```
```
// packages/coding-agent/src/core/bug-report.ts:113
	const authTypes: Array<"api_key" | "oauth"> = [];
	if (provider.auth.apiKey) authTypes.push("api_key");
```
```
// packages/coding-agent/src/core/bug-report.ts:114
	if (provider.auth.apiKey) authTypes.push("api_key");
	if (provider.auth.oauth) authTypes.push("oauth");
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 11: ProviderAuth (packages/ai/src/auth/types.ts:252) read in packages/coding-agent/src/core/model-runtime.ts
- fields to check: apiKey,oauth
```
// packages/ai/src/auth/types.ts:252
export interface ProviderAuth {
	apiKey?: ApiKeyAuth;
	oauth?: OAuthAuth;
}
```
```
// packages/coding-agent/src/core/model-runtime.ts:672
		const resolution = await this.getAuth(model, {
			apiKey: options?.apiKey,
```
```
// packages/coding-agent/src/core/model-runtime.ts:275
			const config = this.config.getProvider(providerId);
			if (config?.oauth !== "radius" || !config.baseUrl) continue;
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 12: ProviderAuth (packages/ai/src/auth/types.ts:252) read in packages/coding-agent/src/core/provider-composer.ts
- fields to check: apiKey,oauth
```
// packages/ai/src/auth/types.ts:252
export interface ProviderAuth {
	apiKey?: ApiKeyAuth;
	oauth?: OAuthAuth;
}
```
```
// packages/coding-agent/src/core/provider-composer.ts:316
		!hasOverrides &&
		!config.apiKey &&
```
```
// packages/coding-agent/src/core/provider-composer.ts:306
	if (!config) return [...baseModels];
	if (config.oauth && !config.baseUrl) {
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 13: AssistantMessage (packages/ai/src/types.ts:570) read in packages/coding-agent/test/suite/virtual-models.test.ts
- fields to check: errorMessage,stopReason
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
	diagnostics?: AssistantMessageDiagnostic[]; // Redacted provider/runtime diagnostics for failures and recoveries.
	usage: Usage;
```
```
// packages/coding-agent/test/suite/virtual-models.test.ts:93
		expect(reasons()).toEqual(["user", "retry", "continuation"]);
		expect(requests[1].failed?.message.errorMessage).toBe("overloaded_error");
```
```
// packages/coding-agent/test/suite/virtual-models.test.ts:146
		expect(requests[1].failed?.model.id).toBe("large");
		expect(requests[1].failed?.message.stopReason).toBe("length");
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 14: FauxProviderState (packages/ai/src/providers/faux.ts:103) read in packages/durable/test/harness-generation-recovery.test.ts
- fields to check: callCount,deferredFetchCount
```
// packages/ai/src/providers/faux.ts:103
export interface FauxProviderState {
	callCount: number;
	deferredFetchCount: number;
	cancelledDeferred: DeferredHandle[];
}

export type FauxResponseFactory = (
	context: TranscriptContext,
	options: SimpleStreamOptions | undefined,
	state: FauxProviderState,
	model: Model<string>,
) => AssistantMessage | Promise<AssistantMessage>;

export type FauxResponseStep = AssistantMessage | FauxResponseFactory;
```
```
// packages/durable/test/harness-generation-recovery.test.ts:196
		await new Promise((resolve) => setTimeout(resolve, 20));
		expect(setup.faux.state.callCount).toBe(1);
```
```
// packages/durable/test/harness-generation-recovery.test.ts:286
		expect(textOf(answer?.model?.[0])).toBe("deferred answer");
		expect(setup.faux.state.deferredFetchCount).toBe(1);
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 15: FauxProviderHandle (packages/ai/src/providers/faux.ts:146) read in packages/durable/test/harness-generation-recovery.test.ts
- fields to check: setResponses,state
```
// packages/ai/src/providers/faux.ts:146
export interface FauxProviderHandle {
	provider: Provider;
	api: string;
	models: [Model<string>, ...Model<string>[]];
	getModel(): Model<string>;
	getModel(modelId: string): Model<string> | undefined;
	state: FauxProviderState;
	setResponses: (responses: FauxResponseStep[]) => void;
	appendResponses: (responses: FauxResponseStep[]) => void;
	getPendingResponseCount: () => number;
}

function estimateTokens(text: string): number {
	return Math.ceil(text.length / 4);
```
```
// packages/durable/test/harness-generation-recovery.test.ts:55
		});
		setup.faux.setResponses([fauxAssistantMessage("answer")]);
```
```
// packages/durable/test/harness-generation-recovery.test.ts:38
	const record = await harness.getTask(id, context);
	return record?.state.status === "terminal" ? undefined : record?.state.checkpoint;
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 16: FauxProviderState (packages/ai/src/providers/faux.ts:103) read in packages/durable/test/harness-generation.test.ts
- fields to check: callCount,cancelledDeferred,deferredFetchCount
```
// packages/ai/src/providers/faux.ts:103
export interface FauxProviderState {
	callCount: number;
	deferredFetchCount: number;
	cancelledDeferred: DeferredHandle[];
}

export type FauxResponseFactory = (
	context: TranscriptContext,
	options: SimpleStreamOptions | undefined,
	state: FauxProviderState,
	model: Model<string>,
) => AssistantMessage | Promise<AssistantMessage>;

export type FauxResponseStep = AssistantMessage | FauxResponseFactory;
```
```
// packages/durable/test/harness-generation.test.ts:227
		expect(settled).toMatchObject({ status: "unanswered", reason: "model_error" });
		expect(setup.faux.state.callCount).toBe(1);
```
```
// packages/durable/test/harness-generation.test.ts:323
		expect(await submission.wait(context)).toMatchObject({ status: "unanswered", reason: "aborted" });
		expect(setup.faux.state.cancelledDeferred).toHaveLength(1);
```
```
// packages/durable/test/harness-generation.test.ts:280
		if (settled.status !== "done" || settled.type !== "input") throw new Error(`Unexpected ${settled.status}`);
		expect(setup.faux.state.deferredFetchCount).toBe(2);
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 17: FauxProviderHandle (packages/ai/src/providers/faux.ts:146) read in packages/durable/test/harness-generation.test.ts
- fields to check: getPendingResponseCount,setResponses,state
```
// packages/ai/src/providers/faux.ts:146
export interface FauxProviderHandle {
	provider: Provider;
	api: string;
	models: [Model<string>, ...Model<string>[]];
	getModel(): Model<string>;
	getModel(modelId: string): Model<string> | undefined;
	state: FauxProviderState;
	setResponses: (responses: FauxResponseStep[]) => void;
	appendResponses: (responses: FauxResponseStep[]) => void;
	getPendingResponseCount: () => number;
}

function estimateTokens(text: string): number {
	return Math.ceil(text.length / 4);
```
```
// packages/durable/test/harness-generation.test.ts:215
		expect((await allEntries(root)).map((entry) => entry.kind)).toEqual(["pi.user", "pi.assistant", "pi.assistant"]);
		expect(setup.faux.getPendingResponseCount()).toBe(1);
```
```
// packages/durable/test/harness-generation.test.ts:92
		addSection(setup.registry, "preamble", () => "You are helpful.", { tag: false });
		setup.faux.setResponses([fauxAssistantMessage("Hello there")]);
```
```
// packages/durable/test/harness-generation.test.ts:112
		expect(entries.map((entry) => entry.byTaskId)).toEqual([undefined, task!.id, task!.id]);
		expect(task?.state).toEqual({
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 18: FauxProviderHandle (packages/ai/src/providers/faux.ts:146) read in packages/durable/test/harness-inbox.test.ts
- fields to check: setResponses,state
```
// packages/ai/src/providers/faux.ts:146
export interface FauxProviderHandle {
	provider: Provider;
	api: string;
	models: [Model<string>, ...Model<string>[]];
	getModel(): Model<string>;
	getModel(modelId: string): Model<string> | undefined;
	state: FauxProviderState;
	setResponses: (responses: FauxResponseStep[]) => void;
	appendResponses: (responses: FauxResponseStep[]) => void;
	getPendingResponseCount: () => number;
}

function estimateTokens(text: string): number {
	return Math.ceil(text.length / 4);
```
```
// packages/durable/test/harness-inbox.test.ts:114
		const first = gated(answer("first"));
		setup.faux.setResponses([first.step, answer("second"), answer("third")]);
```
```
// packages/durable/test/harness-inbox.test.ts:170
		]);
		expect(setup.faux.state.callCount).toBe(2);
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 19: FauxProviderHandle (packages/ai/src/providers/faux.ts:146) read in packages/durable/test/harness-input-entries.test.ts
- fields to check: setResponses,state
```
// packages/ai/src/providers/faux.ts:146
export interface FauxProviderHandle {
	provider: Provider;
	api: string;
	models: [Model<string>, ...Model<string>[]];
	getModel(): Model<string>;
	getModel(modelId: string): Model<string> | undefined;
	state: FauxProviderState;
	setResponses: (responses: FauxResponseStep[]) => void;
	appendResponses: (responses: FauxResponseStep[]) => void;
	getPendingResponseCount: () => number;
}

function estimateTokens(text: string): number {
	return Math.ceil(text.length / 4);
```
```
// packages/durable/test/harness-input-entries.test.ts:103
		const requests: string[][] = [];
		setup.faux.setResponses([requestLog(requests, answer("hi ann"))]);
```
```
// packages/durable/test/harness-input-entries.test.ts:378
		expect(await harness.snapshot(LiveDoc, root.id, context)).toEqual({});
		expect(setup.faux.state.callCount).toBe(1);
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 20: FauxProviderHandle (packages/ai/src/providers/faux.ts:146) read in packages/durable/test/harness-tools.test.ts
- fields to check: setResponses,state
```
// packages/ai/src/providers/faux.ts:146
export interface FauxProviderHandle {
	provider: Provider;
	api: string;
	models: [Model<string>, ...Model<string>[]];
	getModel(): Model<string>;
	getModel(modelId: string): Model<string> | undefined;
	state: FauxProviderState;
	setResponses: (responses: FauxResponseStep[]) => void;
	appendResponses: (responses: FauxResponseStep[]) => void;
	getPendingResponseCount: () => number;
}

function estimateTokens(text: string): number {
	return Math.ceil(text.length / 4);
```
```
// packages/durable/test/harness-tools.test.ts:66
): Promise<{ harness: Harness; root: Conversation; entries: EntryRecord[]; status: string }> {
	setup.faux.setResponses(responses);
```
```
// packages/durable/test/harness-tools.test.ts:109
		// The second request sees the tool result right after its call.
		expect(setup.faux.state.callCount).toBe(2);
```
Judgment (TRUE / NOT TRUE, one sentence):

