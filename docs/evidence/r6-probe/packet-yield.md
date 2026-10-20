### Egonex-AI__Understand-Anything 1: ValidationResult (understand-anything-plugin/packages/core/src/schema.ts) read in understand-anything-plugin/packages/dashboard/src/App.tsx
- fields claimed: fatal, issues, success
```
// understand-anything-plugin/packages/core/src/schema.ts:492
export interface ValidationResult {
  success: boolean;
  data?: z.infer<typeof KnowledgeGraphSchema>;
  /** @deprecated Use issues/fatal instead */
  errors?: string[];
  issues: GraphIssue[];
  fatal?: string;
}

function buildInvalidCollectionIssue(name: string): GraphIssue {
  return {
    level: "fatal",
```
```
// understand-anything-plugin/packages/dashboard/src/App.tsx:182
            }
          }
        } else if (result.fatal) {
```
```
// understand-anything-plugin/packages/dashboard/src/App.tsx:170
        if (result.success && result.data) {
          setGraph(result.data);
          setGraphIssues(result.issues);
```
```
// understand-anything-plugin/packages/dashboard/src/App.tsx:168
      .then((data: unknown) => {
        const result = validateGraph(data);
        if (result.success && result.data) {
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 2: TruncationResult (packages/coding-agent/src/core/tools/truncate.ts) read in packages/coding-agent/examples/extensions/gondolin/index.ts
- fields claimed: content, truncated
```
// packages/coding-agent/src/core/tools/truncate.ts:15
export interface TruncationResult {
	/** The truncated content */
	content: string;
	/** Whether truncation occurred */
	truncated: boolean;
	/** Which limit was hit: "lines", "bytes", or null if not truncated */
	truncatedBy: "lines" | "bytes" | null;
	/** Total number of lines in the original content */
	totalLines: number;
	/** Total number of bytes in the original content */
	totalBytes: number;
	/** Number of complete lines in the truncated output */
```
```
// packages/coding-agent/examples/extensions/gondolin/index.ts:293
	const truncation = truncateHead(rawOutput, { maxLines: Number.MAX_SAFE_INTEGER });
	const notices: string[] = [];
	let output = truncation.content;
```
```
// packages/coding-agent/examples/extensions/gondolin/index.ts:303
		notices.push("long lines truncated");
	}
	if (truncation.truncated) {
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 3: FuzzyMatch (packages/tui/src/fuzzy.ts) read in packages/coding-agent/src/modes/interactive/components/session-selector-search.ts
- fields claimed: matches, score
```
// packages/tui/src/fuzzy.ts:7
export interface FuzzyMatch {
	matches: boolean;
	score: number;
}

export function fuzzyMatch(query: string, text: string): FuzzyMatch {
	const queryLower = query.toLowerCase();
	const textLower = text.toLowerCase();

	const matchQuery = (normalizedQuery: string): FuzzyMatch => {
		if (normalizedQuery.length === 0) {
			return { matches: true, score: 0 };
```
```
// packages/coding-agent/src/modes/interactive/components/session-selector-search.ts:149

		const m = fuzzyMatch(token.value, text);
		if (!m.matches) return { matches: false, score: 0 };
```
```
// packages/coding-agent/src/modes/interactive/components/session-selector-search.ts:150
		const m = fuzzyMatch(token.value, text);
		if (!m.matches) return { matches: false, score: 0 };
		totalScore += m.score;
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 4: RgbColorValue (packages/tui/src/colors.ts) read in packages/coding-agent/src/modes/interactive/theme/system-theme.ts
- fields claimed: b, g, r
```
// packages/tui/src/colors.ts:9
export interface RgbColorValue {
	readonly kind: "rgb";
	readonly r: number;
	readonly g: number;
	readonly b: number;
}

export interface OklchColorValue {
	readonly kind: "oklch";
	readonly l: number;
	readonly c: number;
	readonly h: number;
```
```
// packages/coding-agent/src/modes/interactive/theme/system-theme.ts:392
/** OKLab lightness of an sRGB color, 0-1. */
function oklabLightness(color: RgbColor): number {
	return colorToOklch(rgbColor(color.r, color.g, color.b)).l;
```
```
// packages/coding-agent/src/modes/interactive/theme/system-theme.ts:378
	/** Saturation multiplier from 0 (grayscale) to 1. The first frame renders in grayscale until colors arrive. */
	saturation?: number;
	/** Appearance when the terminal did not report its background, e.g. from its light/dark report or COLORFGBG. */
```
```
// packages/coding-agent/src/modes/interactive/theme/system-theme.ts:392
/** OKLab lightness of an sRGB color, 0-1. */
function oklabLightness(color: RgbColor): number {
	return colorToOklch(rgbColor(color.r, color.g, color.b)).l;
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 5: FauxProviderRegistration (packages/ai/src/providers/faux.ts) read in packages/coding-agent/test/agent-session-runtime-events.test.ts
- fields claimed: setResponses, unregister
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
// packages/coding-agent/test/agent-session-runtime-events.test.ts:43

		const faux = registerFauxProvider();
		faux.setResponses([fauxAssistantMessage("one"), fauxAssistantMessage("two"), fauxAssistantMessage("three")]);
```
```
// packages/coding-agent/test/agent-session-runtime-events.test.ts:106
		cleanups.push(async () => {
			await runtimeHost.dispose();
			faux.unregister();
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 6: FauxProviderHandle (packages/ai/src/providers/faux.ts) read in packages/coding-agent/test/jev-router-example.test.ts
- fields claimed: appendResponses, provider
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

```
```
// packages/coding-agent/test/jev-router-example.test.ts:94
		await harness.session.setModel(runtime.getModel("jev", "auto")!);

		const respond = (...messages: AssistantMessage[]) => codex.appendResponses(messages);
```
```
// packages/coding-agent/test/jev-router-example.test.ts:62
					classify: async (model) => ({
						api: model.api,
						provider: model.provider,
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 7: FauxProviderRegistration (packages/ai/src/providers/faux.ts) read in packages/coding-agent/test/suite/agent-session-runtime.test.ts
- fields claimed: api, models, setResponses, unregister
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
// packages/coding-agent/test/suite/agent-session-runtime.test.ts:70
							baseUrl: faux.getModel().baseUrl,
							apiKey: "faux-key",
							api: faux.api,
```
```
// packages/coding-agent/test/suite/agent-session-runtime.test.ts:71
							apiKey: "faux-key",
							api: faux.api,
							models: faux.models.map((registeredModel) => ({
```
```
// packages/coding-agent/test/suite/agent-session-runtime.test.ts:54
			],
		});
		faux.setResponses([fauxAssistantMessage("one"), fauxAssistantMessage("two"), fauxAssistantMessage("three")]);
```
```
// packages/coding-agent/test/suite/agent-session-runtime.test.ts:116
		cleanups.push(async () => {
			await runtime.dispose();
			faux.unregister();
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 8: FauxProviderRegistration (packages/ai/src/providers/faux.ts) read in packages/coding-agent/test/suite/regressions/2753-reload-stale-resource-settings.test.ts
- fields claimed: api, models, unregister
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
// packages/coding-agent/test/suite/regressions/2753-reload-stale-resource-settings.test.ts:53
								baseUrl: faux.getModel().baseUrl,
								apiKey: "faux-key",
								api: faux.api,
```
```
// packages/coding-agent/test/suite/regressions/2753-reload-stale-resource-settings.test.ts:54
								apiKey: "faux-key",
								api: faux.api,
								models: faux.models.map((registeredModel) => ({
```
```
// packages/coding-agent/test/suite/regressions/2753-reload-stale-resource-settings.test.ts:90
		cleanups.push(() => {
			runtime.session.dispose();
			faux.unregister();
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 9: FauxProviderRegistration (packages/ai/src/providers/faux.ts) read in packages/coding-agent/test/suite/regressions/2860-replaced-session-context.test.ts
- fields claimed: api, models, setResponses, unregister
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
// packages/coding-agent/test/suite/regressions/2860-replaced-session-context.test.ts:66
								baseUrl: faux.getModel().baseUrl,
								apiKey: "faux-key",
								api: faux.api,
```
```
// packages/coding-agent/test/suite/regressions/2860-replaced-session-context.test.ts:67
								apiKey: "faux-key",
								api: faux.api,
								models: faux.models.map((registeredModel) => ({
```
```
// packages/coding-agent/test/suite/regressions/2860-replaced-session-context.test.ts:46
			models: [{ id: "faux-1", reasoning: false }],
		});
		faux.setResponses(responses.map((response) => fauxAssistantMessage(response)));
```
```
// packages/coding-agent/test/suite/regressions/2860-replaced-session-context.test.ts:138
		cleanups.push(async () => {
			await runtime.dispose();
			faux.unregister();
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 10: FauxProviderRegistration (packages/ai/src/providers/faux.ts) read in packages/coding-agent/test/suite/regressions/5596-missing-theme-export.test.ts
- fields claimed: api, models, setResponses, unregister
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
// packages/coding-agent/test/suite/regressions/5596-missing-theme-export.test.ts:40
			baseUrl: model.baseUrl,
			apiKey: "faux-key",
			api: faux.api,
```
```
// packages/coding-agent/test/suite/regressions/5596-missing-theme-export.test.ts:41
			apiKey: "faux-key",
			api: faux.api,
			models: faux.models.map((registeredModel) => ({
```
```
// packages/coding-agent/test/suite/regressions/5596-missing-theme-export.test.ts:31
			models: [{ id: "faux-1", reasoning: false }],
		});
		faux.setResponses([fauxAssistantMessage("hello")]);
```
```
// packages/coding-agent/test/suite/regressions/5596-missing-theme-export.test.ts:76
		cleanups.push(() => {
			session.dispose();
			faux.unregister();
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 11: FauxProviderHandle (packages/ai/src/providers/faux.ts) read in packages/coding-agent/test/suite/regressions/8964-extension-provider-streaming.test.ts
- fields claimed: api, models, provider, setResponses
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

```
```
// packages/coding-agent/test/suite/regressions/8964-extension-provider-streaming.test.ts:24
				(pi) => {
					pi.registerProvider(faux.provider.id, {
						api: faux.api,
```
```
// packages/coding-agent/test/suite/regressions/8964-extension-provider-streaming.test.ts:27
						baseUrl: faux.getModel().baseUrl,
						apiKey: "extension-key",
						models: faux.models,
```
```
// packages/coding-agent/test/suite/regressions/8964-extension-provider-streaming.test.ts:23
			extensionFactories: [
				(pi) => {
					pi.registerProvider(faux.provider.id, {
```
```
// packages/coding-agent/test/suite/regressions/8964-extension-provider-streaming.test.ts:12
		const faux = fauxProvider({ provider: "extension-provider", api: "issue-8964-extension-api" });
		let receivedApiKey: string | undefined;
		faux.setResponses([
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 12: FauxProviderHandle (packages/ai/src/providers/faux.ts) read in packages/coding-agent/test/virtual-models.test.ts
- fields claimed: provider, setResponses
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

```
```
// packages/coding-agent/test/virtual-models.test.ts:33
		],
	});
	runtime.registerNativeProvider(faux.provider);
```
```
// packages/coding-agent/test/virtual-models.test.ts:214
		const { runtime, faux, virtual } = await createRuntime(requests);
		let maxTokens: number | undefined;
		faux.setResponses([
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 13: OutputSlice (packages/chord/src/output.ts) read in packages/durable/src/harness/tool.ts
- fields claimed: droppedBytes, droppedLines, text
```
// packages/chord/src/output.ts:13
export type OutputSlice = {
	readonly text: string;
	readonly bytes: number;
	readonly droppedBytes: number;
	readonly droppedLines: number;
};

/**
 * Output a producer omitted before a chunk, measured on the decoded text: every U+FFFD counts as three bytes, and no
 * sanitizing is applied.
 */
export interface OutputSkip {
```
```
// packages/durable/src/harness/tool.ts:333
			assertLive();
			const retained = reported.output.snapshot();
			return { text: retained.text, truncated: retained.droppedBytes > 0 };
```
```
// packages/durable/src/harness/tool.ts:499
				if ((slot.output ?? "") !== snapshot.text) slot.output = snapshot.text;
				if (snapshot.droppedBytes > 0) slot.droppedBytes = snapshot.droppedBytes;
				if (snapshot.droppedLines > 0) slot.droppedLines = snapshot.droppedLines;
```
```
// packages/durable/src/harness/tool.ts:333
			assertLive();
			const retained = reported.output.snapshot();
			return { text: retained.text, truncated: retained.droppedBytes > 0 };
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 14: FauxProviderHandle (packages/ai/src/providers/faux.ts) read in packages/durable/test-workers/worker.ts
- fields claimed: provider, setResponses
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

```
```
// packages/durable/test-workers/worker.ts:61
			faux.setResponses(Array.from({ length: 8 }, () => this.#step()));
			const models = createModels();
			models.setProvider(faux.provider);
```
```
// packages/durable/test-workers/worker.ts:59
			this.#offset = ctx.storage.kv.get<number>("offset") ?? 0;
			const faux = fauxProvider();
			faux.setResponses(Array.from({ length: 8 }, () => this.#step()));
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 15: FauxProviderHandle (packages/ai/src/providers/faux.ts) read in packages/durable/test/codemode-bench.ts
- fields claimed: provider, setResponses
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

```
```
// packages/durable/test/codemode-bench.ts:49
		const faux = fauxProvider();
		const models = createModels();
		models.setProvider(faux.provider);
```
```
// packages/durable/test/codemode-bench.ts:61
				? fauxToolCall("codemode", { code }, { id: "c1" })
				: fauxToolCall("plain", { n, shape }, { id: "c1" });
		faux.setResponses([
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 16: FauxProviderHandle (packages/ai/src/providers/faux.ts) read in packages/durable/test/examples/11-extension-state.ts
- fields claimed: provider, setResponses
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

```
```
// packages/durable/test/examples/11-extension-state.ts:59
const faux = fauxProvider();
const models = createModels();
models.setProvider(faux.provider);
```
```
// packages/durable/test/examples/11-extension-state.ts:65
const root = await harness.root(context, { agent: { model: { provider: "faux", modelId: "faux-1" } } });

faux.setResponses([
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 17: FauxProviderHandle (packages/ai/src/providers/faux.ts) read in packages/durable/test/examples/14-chat.ts
- fields claimed: provider, setResponses
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

```
```
// packages/durable/test/examples/14-chat.ts:14
const faux = fauxProvider();
const models = createModels();
models.setProvider(faux.provider);
```
```
// packages/durable/test/examples/14-chat.ts:15
const models = createModels();
models.setProvider(faux.provider);
faux.setResponses([fauxAssistantMessage("Paris.")]);
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 18: FauxProviderHandle (packages/ai/src/providers/faux.ts) read in packages/durable/test/examples/15-system-prompt.ts
- fields claimed: provider, setResponses
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

```
```
// packages/durable/test/examples/15-system-prompt.ts:22
const faux = fauxProvider();
const models = createModels();
models.setProvider(faux.provider);
```
```
// packages/durable/test/examples/15-system-prompt.ts:23
const models = createModels();
models.setProvider(faux.provider);
faux.setResponses([fauxAssistantMessage("Done."), fauxAssistantMessage("Done."), fauxAssistantMessage("Done.")]);
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 19: FauxProviderHandle (packages/ai/src/providers/faux.ts) read in packages/durable/test/examples/17-coding-tools.ts
- fields claimed: provider, setResponses
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

```
```
// packages/durable/test/examples/17-coding-tools.ts:33
const faux = fauxProvider();
const models = createModels();
models.setProvider(faux.provider);
```
```
// packages/durable/test/examples/17-coding-tools.ts:36
const toolCall = (name: string, args: Parameters<typeof fauxToolCall>[1], id: string) =>
	fauxAssistantMessage([fauxToolCall(name, args, { id })], { stopReason: "toolUse" });
faux.setResponses([
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 20: FauxProviderHandle (packages/ai/src/providers/faux.ts) read in packages/durable/test/examples/18-print.ts
- fields claimed: provider, setResponses
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

```
```
// packages/durable/test/examples/18-print.ts:23
} else {
	const faux = fauxProvider();
	models.setProvider(faux.provider);
```
```
// packages/durable/test/examples/18-print.ts:25
	models.setProvider(faux.provider);
	model = { provider: "faux", modelId: "faux-1" };
	faux.setResponses([
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 21: FauxProviderHandle (packages/ai/src/providers/faux.ts) read in packages/durable/test/examples/19-json.ts
- fields claimed: provider, setResponses
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

```
```
// packages/durable/test/examples/19-json.ts:60
} else {
	const faux = fauxProvider({ tokensPerSecond: 200 });
	models.setProvider(faux.provider);
```
```
// packages/durable/test/examples/19-json.ts:62
	models.setProvider(faux.provider);
	model = { provider: "faux", modelId: "faux-1" };
	faux.setResponses([
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 22: FauxProviderHandle (packages/ai/src/providers/faux.ts) read in packages/durable/test/examples/20-inbox.ts
- fields claimed: provider, setResponses
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

```
```
// packages/durable/test/examples/20-inbox.ts:24
faux.setResponses([slow, fauxAssistantMessage("Answer to the follow-up and the steer.")]);
const models = createModels();
models.setProvider(faux.provider);
```
```
// packages/durable/test/examples/20-inbox.ts:22
};
const faux = fauxProvider();
faux.setResponses([slow, fauxAssistantMessage("Answer to the follow-up and the steer.")]);
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 23: FauxProviderHandle (packages/ai/src/providers/faux.ts) read in packages/durable/test/examples/21-late-join.ts
- fields claimed: provider, setResponses
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

```
```
// packages/durable/test/examples/21-late-join.ts:50
]);
const models = createModels();
models.setProvider(faux.provider);
```
```
// packages/durable/test/examples/21-late-join.ts:45
);
const faux = fauxProvider({ tokensPerSecond: 40, tokenSize: { min: 1, max: 1 } });
faux.setResponses([
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 24: FauxProviderHandle (packages/ai/src/providers/faux.ts) read in packages/durable/test/examples/22-subagent-foreground.ts
- fields claimed: provider, setResponses
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

```
```
// packages/durable/test/examples/22-subagent-foreground.ts:88
	// The parent delegates, the child answers, and the parent reports.
	const faux = fauxProvider();
	models.setProvider(faux.provider);
```
```
// packages/durable/test/examples/22-subagent-foreground.ts:91
	model = { provider: "faux", modelId: "faux-1" };
	const delegate = fauxToolCall("subagent", { task: "Name three prime numbers." }, { id: "call-1" });
	faux.setResponses([
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 25: FauxProviderHandle (packages/ai/src/providers/faux.ts) read in packages/durable/test/examples/23-subagent-background.ts
- fields claimed: provider, setResponses
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

```
```
// packages/durable/test/examples/23-subagent-background.ts:223
	// It streams its answers at 50 tokens per second, like a slow real model.
	const faux = fauxProvider({ tokensPerSecond: 50 });
	models.setProvider(faux.provider);
```
```
// packages/durable/test/examples/23-subagent-background.ts:254
	};
	// More responses than the script needs; each request takes the next one.
	faux.setResponses(Array.from({ length: 40 }, () => route));
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 26: FauxProviderHandle (packages/ai/src/providers/faux.ts) read in packages/durable/test/examples/25-compaction.ts
- fields claimed: provider, setResponses
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

```
```
// packages/durable/test/examples/25-compaction.ts:44
faux.setResponses(Array.from({ length: 100 }, () => respond));
const models = createModels();
models.setProvider(faux.provider);
```
```
// packages/durable/test/examples/25-compaction.ts:42
}
const faux = fauxProvider({ models: [{ id: "tiny", contextWindow: 3000, maxTokens: 1000 }] });
faux.setResponses(Array.from({ length: 100 }, () => respond));
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 27: FauxProviderHandle (packages/ai/src/providers/faux.ts) read in packages/durable/test/examples/26-coding-agent.ts
- fields claimed: provider, setResponses
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

```
```
// packages/durable/test/examples/26-coding-agent.ts:58
const faux = fauxProvider();
const models = createModels();
models.setProvider(faux.provider);
```
```
// packages/durable/test/examples/26-coding-agent.ts:60
models.setProvider(faux.provider);
const pwd = () => fauxAssistantMessage(fauxToolCall("bash", { command: "pwd" }), { stopReason: "toolUse" });
faux.setResponses([pwd(), fauxAssistantMessage("Done."), pwd(), fauxAssistantMessage("Done.")]);
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 28: FauxProviderHandle (packages/ai/src/providers/faux.ts) read in packages/durable/test/examples/27-plan-mode.ts
- fields claimed: provider, setResponses
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

```
```
// packages/durable/test/examples/27-plan-mode.ts:71
const faux = fauxProvider();
const models = createModels();
models.setProvider(faux.provider);
```
```
// packages/durable/test/examples/27-plan-mode.ts:72
const models = createModels();
models.setProvider(faux.provider);
faux.setResponses([
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 29: FauxProviderHandle (packages/ai/src/providers/faux.ts) read in packages/durable/test/examples/28-reviewer.ts
- fields claimed: provider, setResponses
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

```
```
// packages/durable/test/examples/28-reviewer.ts:54
const faux = fauxProvider({ models: [{ id: "big" }, { id: "small" }] });
const models = createModels();
models.setProvider(faux.provider);
```
```
// packages/durable/test/examples/28-reviewer.ts:55
const models = createModels();
models.setProvider(faux.provider);
faux.setResponses([
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 30: FauxProviderHandle (packages/ai/src/providers/faux.ts) read in packages/durable/test/examples/29-sandbox-per-conversation.ts
- fields claimed: provider, setResponses
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

```
```
// packages/durable/test/examples/29-sandbox-per-conversation.ts:30
const faux = fauxProvider();
const models = createModels();
models.setProvider(faux.provider);
```
```
// packages/durable/test/examples/29-sandbox-per-conversation.ts:33
const note = (text: string) =>
	fauxAssistantMessage(fauxToolCall("write", { path: "note.txt", content: text }), { stopReason: "toolUse" });
faux.setResponses([
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 31: FauxProviderHandle (packages/ai/src/providers/faux.ts) read in packages/durable/test/examples/30-tool-override.ts
- fields claimed: provider, setResponses
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

```
```
// packages/durable/test/examples/30-tool-override.ts:61
const faux = fauxProvider();
const models = createModels();
models.setProvider(faux.provider);
```
```
// packages/durable/test/examples/30-tool-override.ts:66
		stopReason: "toolUse",
	});
faux.setResponses([probe(), fauxAssistantMessage("Done."), probe(), fauxAssistantMessage("Done.")]);
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 32: FauxProviderHandle (packages/ai/src/providers/faux.ts) read in packages/durable/test/examples/31-reload-and-restart.ts
- fields claimed: provider, setResponses
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

```
```
// packages/durable/test/examples/31-reload-and-restart.ts:40
const faux = fauxProvider();
const models = createModels();
models.setProvider(faux.provider);
```
```
// packages/durable/test/examples/31-reload-and-restart.ts:42
models.setProvider(faux.provider);
const callVersion = () => fauxAssistantMessage(fauxToolCall("version", {}), { stopReason: "toolUse" });
faux.setResponses([callVersion(), fauxAssistantMessage("Done."), callVersion(), fauxAssistantMessage("Done.")]);
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 33: FauxProviderHandle (packages/ai/src/providers/faux.ts) read in packages/durable/test/harness-host-inputs.test.ts
- fields claimed: provider, setResponses
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

```
```
// packages/durable/test/harness-host-inputs.test.ts:31
	faux.setResponses(Array.from({ length: 10 }, () => fauxAssistantMessage([fauxText(text)])));
	const models = createModels();
	models.setProvider(faux.provider);
```
```
// packages/durable/test/harness-host-inputs.test.ts:29
function answering(text: string): Models {
	const faux = fauxProvider();
	faux.setResponses(Array.from({ length: 10 }, () => fauxAssistantMessage([fauxText(text)])));
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 34: FauxProviderHandle (packages/ai/src/providers/faux.ts) read in packages/durable/test/nested-images-bench.ts
- fields claimed: provider, setResponses
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

```
```
// packages/durable/test/nested-images-bench.ts:174
		const faux = fauxProvider();
		const models = createModels();
		models.setProvider(faux.provider);
```
```
// packages/durable/test/nested-images-bench.ts:201
		const harness = await Harness.open(storage, { models, registry }, context);
		const root = await harness.root(context, { agent: { model: { provider: "faux", modelId: "faux-1" } } });
		faux.setResponses([
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 35: FauxProviderHandle (packages/ai/src/providers/faux.ts) read in packages/durable/test/nested-tools-bench.ts
- fields claimed: provider, setResponses
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

```
```
// packages/durable/test/nested-tools-bench.ts:99
		const faux = fauxProvider();
		const models = createModels();
		models.setProvider(faux.provider);
```
```
// packages/durable/test/nested-tools-bench.ts:147
		const baseHeap = heapAfterGc();
		for (let run = 0; run < scenario.runs; run++) {
			faux.setResponses([
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 36: FauxProviderHandle (packages/ai/src/providers/faux.ts) read in packages/durable/test/scheduler-fanout-bench.ts
- fields claimed: provider, setResponses
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

```
```
// packages/durable/test/scheduler-fanout-bench.ts:18
	const faux = fauxProvider();
	const models = createModels();
	models.setProvider(faux.provider);
```
```
// packages/durable/test/scheduler-fanout-bench.ts:50
	const harness = await Harness.open(storage, { models, registry }, context);
	const root = await harness.root(context, { agent: { model: { provider: "faux", modelId: "faux-1" } } });
	faux.setResponses([
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 37: FauxProviderHandle (packages/ai/src/providers/faux.ts) read in packages/durable/test/tool-output-bench.ts
- fields claimed: provider, setResponses
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

```
```
// packages/durable/test/tool-output-bench.ts:136
		const faux = fauxProvider();
		const models = createModels();
		models.setProvider(faux.provider);
```
```
// packages/durable/test/tool-output-bench.ts:186
				? [fauxToolCall("bash", { command: `cat ${BIG_FILE}` }, { id: "c0" })]
				: Array.from({ length: toolCount }, (_, index) => fauxToolCall("emit", {}, { id: `c${index}` }));
		faux.setResponses([
```
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 38: StickyNoteLayout (packages/element/src/stickyNote.ts) read in packages/excalidraw/data/restore.ts
- fields claimed: container, text
```
// packages/element/src/stickyNote.ts:548
export type StickyNoteLayout = {
  container: Pick<
    ExcalidrawStickyNoteElement,
    "x" | "y" | "width" | "height" | "baseHeight"
  >;
  text: Pick<
    ExcalidrawTextElement,
    | "text"
    | "fontSize"
    | "baseFontSize"
    | "width"
    | "height"
```
```
// packages/excalidraw/data/restore.ts:948
      const textElement = getBoundTextElement(element, elementsMap);
      const layout = getStickyNoteLayout(element, textElement);
      Object.assign(element, layout.container);
```
```
// packages/excalidraw/data/restore.ts:568
        fontSize = DEFAULT_FONT_SIZE;
      }
      const text = (typeof element.text === "string" && element.text) || "";
```
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 39: StickyNoteLayout (packages/element/src/stickyNote.ts) read in packages/excalidraw/tests/stickyNotes.test.tsx
- fields claimed: container, text
```
// packages/element/src/stickyNote.ts:548
export type StickyNoteLayout = {
  container: Pick<
    ExcalidrawStickyNoteElement,
    "x" | "y" | "width" | "height" | "baseHeight"
  >;
  text: Pick<
    ExcalidrawTextElement,
    | "text"
    | "fontSize"
    | "baseFontSize"
    | "width"
    | "height"
```
```
// packages/excalidraw/tests/stickyNotes.test.tsx:264
    expect(note.height).toBeLessThan(grownHeight);
    const layout = getStickyNoteLayout(note, label);
    expect(note.height).toBeCloseTo(layout.container.height);
```
```
// packages/excalidraw/tests/stickyNotes.test.tsx:265
    const layout = getStickyNoteLayout(note, label);
    expect(note.height).toBeCloseTo(layout.container.height);
    expect(label.fontSize).toBe(layout.text!.fontSize);
```
Judgment (TRUE / NOT TRUE, one sentence):

