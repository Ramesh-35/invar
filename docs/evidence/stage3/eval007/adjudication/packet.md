# STAGE3-EVAL-007 adjudication packet

40 items. Judge only the shape named in each heading.

### Egonex-AI__Understand-Anything 1: code:schema.ts->App.tsx (judge only: ValidationResult)
- E2 ValidationResult declared at understand-anything-plugin/packages/core/src/schema.ts:492
- E4 ValidationResult not imported by name; resolved via return type of validateGraph() declared in understand-anything-plugin/packages/core/src/schema.ts:563, validateGraph imported at understand-anything-plugin/packages/dashboard/src/App.tsx:2 from "@understand-anything/core/schema"
- E5 ValidationResult fields read: fatal (understand-anything-plugin/packages/dashboard/src/App.tsx:182), issues (understand-anything-plugin/packages/dashboard/src/App.tsx:170), success (understand-anything-plugin/packages/dashboard/src/App.tsx:168)
- E5 ValidationResult stoplisted reads: data
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
// understand-anything-plugin/packages/core/src/schema.ts:563
export function validateGraph(data: unknown): ValidationResult {
  // Tier 4: Fatal — not even an object
  if (typeof data !== "object" || data === null) {
```
```
// understand-anything-plugin/packages/dashboard/src/App.tsx:182
            }
          }
        } else if (result.fatal) {
          console.error("Knowledge graph validation failed:", result.fatal);
```
```
// understand-anything-plugin/packages/dashboard/src/App.tsx:170
        if (result.success && result.data) {
          setGraph(result.data);
          setGraphIssues(result.issues);
          if ((data as Record<string, unknown>).kind === "knowledge") {
```
```
// understand-anything-plugin/packages/dashboard/src/App.tsx:168
      .then((data: unknown) => {
        const result = validateGraph(data);
        if (result.success && result.data) {
          setGraph(result.data);
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 2: code:faux.ts->agent-session-runtime-events.test.ts (judge only: FauxProviderRegistration)
- E2 FauxProviderRegistration declared at packages/ai/src/providers/faux.ts:134
- E4 FauxProviderRegistration not imported by name; resolved via return type of registerFauxProvider() declared in packages/ai/src/compat.ts:162, registerFauxProvider imported at packages/coding-agent/test/agent-session-runtime-events.test.ts:4 from "@earendil-works/pi-ai/compat"
- E5 FauxProviderRegistration fields read: setResponses (packages/coding-agent/test/agent-session-runtime-events.test.ts:43), unregister (packages/coding-agent/test/agent-session-runtime-events.test.ts:106)
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
// packages/ai/src/compat.ts:162
export function registerFauxProvider(options: RegisterFauxProviderOptions = {}): FauxProviderRegistration {
	const core = createFauxCore(options);
	const sourceId = `faux-provider-${Math.random().toString(36).slice(2, 10)}`;
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
			if (existsSync(tempDir)) {
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 3: code:faux.ts->jev-router-example.test.ts (judge only: FauxProviderHandle)
- E2 FauxProviderHandle declared at packages/ai/src/providers/faux.ts:146
- E4 FauxProviderHandle not imported by name; resolved via return type of fauxProvider() declared in packages/ai/src/providers/faux.ts:704, fauxProvider imported at packages/coding-agent/test/jev-router-example.test.ts:2 from "@earendil-works/pi-ai"
- E5 FauxProviderHandle fields read: appendResponses (packages/coding-agent/test/jev-router-example.test.ts:94), provider (packages/coding-agent/test/jev-router-example.test.ts:82)
- E5 FauxProviderHandle stoplisted reads: (none)
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
// packages/ai/src/providers/faux.ts:704
export function fauxProvider(options: RegisterFauxProviderOptions = {}): FauxProviderHandle {
	const core = createFauxCore(options);
	const provider = createProvider({
```
```
// packages/coding-agent/test/jev-router-example.test.ts:94
		await harness.session.setModel(runtime.getModel("jev", "auto")!);

		const respond = (...messages: AssistantMessage[]) => codex.appendResponses(messages);
		const dispatched = () =>
```
```
// packages/coding-agent/test/jev-router-example.test.ts:82
			extensionFactories: [
				(pi) => {
					pi.registerProvider(codex.provider);
					pi.registerProvider(typesafe);
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 4: code:faux.ts->agent-session-runtime.test.ts (judge only: FauxProviderRegistration)
- E2 FauxProviderRegistration declared at packages/ai/src/providers/faux.ts:134
- E4 FauxProviderRegistration not imported by name; resolved via return type of registerFauxProvider() declared in packages/ai/src/compat.ts:162, registerFauxProvider imported at packages/coding-agent/test/suite/agent-session-runtime.test.ts:4 from "@earendil-works/pi-ai/compat"
- E5 FauxProviderRegistration fields read: api (packages/coding-agent/test/suite/agent-session-runtime.test.ts:70), models (packages/coding-agent/test/suite/agent-session-runtime.test.ts:71), setResponses (packages/coding-agent/test/suite/agent-session-runtime.test.ts:54), unregister (packages/coding-agent/test/suite/agent-session-runtime.test.ts:116)
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
// packages/ai/src/compat.ts:162
export function registerFauxProvider(options: RegisterFauxProviderOptions = {}): FauxProviderRegistration {
	const core = createFauxCore(options);
	const sourceId = `faux-provider-${Math.random().toString(36).slice(2, 10)}`;
```
```
// packages/coding-agent/test/suite/agent-session-runtime.test.ts:70
							baseUrl: faux.getModel().baseUrl,
							apiKey: "faux-key",
							api: faux.api,
							models: faux.models.map((registeredModel) => ({
```
```
// packages/coding-agent/test/suite/agent-session-runtime.test.ts:71
							apiKey: "faux-key",
							api: faux.api,
							models: faux.models.map((registeredModel) => ({
								id: registeredModel.id,
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
			if (existsSync(tempDir)) {
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 5: code:faux.ts->2753-reload-stale-resource-settings.test.ts (judge only: FauxProviderRegistration)
- E2 FauxProviderRegistration declared at packages/ai/src/providers/faux.ts:134
- E4 FauxProviderRegistration not imported by name; resolved via return type of registerFauxProvider() declared in packages/ai/src/compat.ts:162, registerFauxProvider imported at packages/coding-agent/test/suite/regressions/2753-reload-stale-resource-settings.test.ts:4 from "@earendil-works/pi-ai/compat"
- E5 FauxProviderRegistration fields read: api (packages/coding-agent/test/suite/regressions/2753-reload-stale-resource-settings.test.ts:53), models (packages/coding-agent/test/suite/regressions/2753-reload-stale-resource-settings.test.ts:54), unregister (packages/coding-agent/test/suite/regressions/2753-reload-stale-resource-settings.test.ts:90)
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
// packages/ai/src/compat.ts:162
export function registerFauxProvider(options: RegisterFauxProviderOptions = {}): FauxProviderRegistration {
	const core = createFauxCore(options);
	const sourceId = `faux-provider-${Math.random().toString(36).slice(2, 10)}`;
```
```
// packages/coding-agent/test/suite/regressions/2753-reload-stale-resource-settings.test.ts:53
								baseUrl: faux.getModel().baseUrl,
								apiKey: "faux-key",
								api: faux.api,
								models: faux.models.map((registeredModel) => ({
```
```
// packages/coding-agent/test/suite/regressions/2753-reload-stale-resource-settings.test.ts:54
								apiKey: "faux-key",
								api: faux.api,
								models: faux.models.map((registeredModel) => ({
									id: registeredModel.id,
```
```
// packages/coding-agent/test/suite/regressions/2753-reload-stale-resource-settings.test.ts:90
		cleanups.push(() => {
			runtime.session.dispose();
			faux.unregister();
			if (existsSync(tempDir)) {
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 6: code:faux.ts->2860-replaced-session-context.test.ts (judge only: FauxProviderRegistration)
- E2 FauxProviderRegistration declared at packages/ai/src/providers/faux.ts:134
- E4 FauxProviderRegistration not imported by name; resolved via return type of registerFauxProvider() declared in packages/ai/src/compat.ts:162, registerFauxProvider imported at packages/coding-agent/test/suite/regressions/2860-replaced-session-context.test.ts:4 from "@earendil-works/pi-ai/compat"
- E5 FauxProviderRegistration fields read: api (packages/coding-agent/test/suite/regressions/2860-replaced-session-context.test.ts:66), models (packages/coding-agent/test/suite/regressions/2860-replaced-session-context.test.ts:67), setResponses (packages/coding-agent/test/suite/regressions/2860-replaced-session-context.test.ts:46), unregister (packages/coding-agent/test/suite/regressions/2860-replaced-session-context.test.ts:138)
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
// packages/ai/src/compat.ts:162
export function registerFauxProvider(options: RegisterFauxProviderOptions = {}): FauxProviderRegistration {
	const core = createFauxCore(options);
	const sourceId = `faux-provider-${Math.random().toString(36).slice(2, 10)}`;
```
```
// packages/coding-agent/test/suite/regressions/2860-replaced-session-context.test.ts:66
								baseUrl: faux.getModel().baseUrl,
								apiKey: "faux-key",
								api: faux.api,
								models: faux.models.map((registeredModel) => ({
```
```
// packages/coding-agent/test/suite/regressions/2860-replaced-session-context.test.ts:67
								apiKey: "faux-key",
								api: faux.api,
								models: faux.models.map((registeredModel) => ({
									id: registeredModel.id,
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
			if (existsSync(tempDir)) {
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 7: code:faux.ts->5596-missing-theme-export.test.ts (judge only: FauxProviderRegistration)
- E2 FauxProviderRegistration declared at packages/ai/src/providers/faux.ts:134
- E4 FauxProviderRegistration not imported by name; resolved via return type of registerFauxProvider() declared in packages/ai/src/compat.ts:162, registerFauxProvider imported at packages/coding-agent/test/suite/regressions/5596-missing-theme-export.test.ts:5 from "@earendil-works/pi-ai/compat"
- E5 FauxProviderRegistration fields read: api (packages/coding-agent/test/suite/regressions/5596-missing-theme-export.test.ts:40), models (packages/coding-agent/test/suite/regressions/5596-missing-theme-export.test.ts:41), setResponses (packages/coding-agent/test/suite/regressions/5596-missing-theme-export.test.ts:31), unregister (packages/coding-agent/test/suite/regressions/5596-missing-theme-export.test.ts:76)
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
// packages/ai/src/compat.ts:162
export function registerFauxProvider(options: RegisterFauxProviderOptions = {}): FauxProviderRegistration {
	const core = createFauxCore(options);
	const sourceId = `faux-provider-${Math.random().toString(36).slice(2, 10)}`;
```
```
// packages/coding-agent/test/suite/regressions/5596-missing-theme-export.test.ts:40
			baseUrl: model.baseUrl,
			apiKey: "faux-key",
			api: faux.api,
			models: faux.models.map((registeredModel) => ({
```
```
// packages/coding-agent/test/suite/regressions/5596-missing-theme-export.test.ts:41
			apiKey: "faux-key",
			api: faux.api,
			models: faux.models.map((registeredModel) => ({
				id: registeredModel.id,
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
			if (existsSync(tempDir)) {
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 8: code:faux.ts->8964-extension-provider-streaming.test.ts (judge only: FauxProviderHandle)
- E2 FauxProviderHandle declared at packages/ai/src/providers/faux.ts:146
- E4 FauxProviderHandle not imported by name; resolved via return type of fauxProvider() declared in packages/ai/src/providers/faux.ts:704, fauxProvider imported at packages/coding-agent/test/suite/regressions/8964-extension-provider-streaming.test.ts:1 from "@earendil-works/pi-ai"
- E5 FauxProviderHandle fields read: api (packages/coding-agent/test/suite/regressions/8964-extension-provider-streaming.test.ts:24), models (packages/coding-agent/test/suite/regressions/8964-extension-provider-streaming.test.ts:27), provider (packages/coding-agent/test/suite/regressions/8964-extension-provider-streaming.test.ts:23), setResponses (packages/coding-agent/test/suite/regressions/8964-extension-provider-streaming.test.ts:12)
- E5 FauxProviderHandle stoplisted reads: (none)
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
// packages/ai/src/providers/faux.ts:704
export function fauxProvider(options: RegisterFauxProviderOptions = {}): FauxProviderHandle {
	const core = createFauxCore(options);
	const provider = createProvider({
```
```
// packages/coding-agent/test/suite/regressions/8964-extension-provider-streaming.test.ts:24
				(pi) => {
					pi.registerProvider(faux.provider.id, {
						api: faux.api,
						baseUrl: faux.getModel().baseUrl,
```
```
// packages/coding-agent/test/suite/regressions/8964-extension-provider-streaming.test.ts:27
						baseUrl: faux.getModel().baseUrl,
						apiKey: "extension-key",
						models: faux.models,
						streamSimple: faux.provider.streamSimple,
```
```
// packages/coding-agent/test/suite/regressions/8964-extension-provider-streaming.test.ts:23
			extensionFactories: [
				(pi) => {
					pi.registerProvider(faux.provider.id, {
						api: faux.api,
```
```
// packages/coding-agent/test/suite/regressions/8964-extension-provider-streaming.test.ts:12
		const faux = fauxProvider({ provider: "extension-provider", api: "issue-8964-extension-api" });
		let receivedApiKey: string | undefined;
		faux.setResponses([
			(_context, options) => {
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 9: code:faux.ts->virtual-models.test.ts (judge only: FauxProviderHandle)
- E2 FauxProviderHandle declared at packages/ai/src/providers/faux.ts:146
- E4 FauxProviderHandle not imported by name; resolved via return type of fauxProvider() declared in packages/ai/src/providers/faux.ts:704, fauxProvider imported at packages/coding-agent/test/virtual-models.test.ts:4 from "@earendil-works/pi-ai"
- E5 FauxProviderHandle fields read: provider (packages/coding-agent/test/virtual-models.test.ts:33), setResponses (packages/coding-agent/test/virtual-models.test.ts:214)
- E5 FauxProviderHandle stoplisted reads: (none)
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
// packages/ai/src/providers/faux.ts:704
export function fauxProvider(options: RegisterFauxProviderOptions = {}): FauxProviderHandle {
	const core = createFauxCore(options);
	const provider = createProvider({
```
```
// packages/coding-agent/test/virtual-models.test.ts:33
		],
	});
	runtime.registerNativeProvider(faux.provider);
	const definition: VirtualModelDefinition = {
```
```
// packages/coding-agent/test/virtual-models.test.ts:214
		const { runtime, faux, virtual } = await createRuntime(requests);
		let maxTokens: number | undefined;
		faux.setResponses([
			(_context, options) => {
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 10: code:faux.ts->worker.ts (judge only: FauxProviderHandle)
- E2 FauxProviderHandle declared at packages/ai/src/providers/faux.ts:146
- E4 FauxProviderHandle not imported by name; resolved via return type of fauxProvider() declared in packages/ai/src/providers/faux.ts:704, fauxProvider imported at packages/durable/test-workers/worker.ts:5 from "@earendil-works/pi-ai"
- E5 FauxProviderHandle fields read: provider (packages/durable/test-workers/worker.ts:61), setResponses (packages/durable/test-workers/worker.ts:59)
- E5 FauxProviderHandle stoplisted reads: (none)
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
// packages/ai/src/providers/faux.ts:704
export function fauxProvider(options: RegisterFauxProviderOptions = {}): FauxProviderHandle {
	const core = createFauxCore(options);
	const provider = createProvider({
```
```
// packages/durable/test-workers/worker.ts:61
			faux.setResponses(Array.from({ length: 8 }, () => this.#step()));
			const models = createModels();
			models.setProvider(faux.provider);
			const registry = createRegistry();
```
```
// packages/durable/test-workers/worker.ts:59
			this.#offset = ctx.storage.kv.get<number>("offset") ?? 0;
			const faux = fauxProvider();
			faux.setResponses(Array.from({ length: 8 }, () => this.#step()));
			const models = createModels();
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 11: code:faux.ts->codemode-bench.ts (judge only: FauxProviderHandle)
- E2 FauxProviderHandle declared at packages/ai/src/providers/faux.ts:146
- E4 FauxProviderHandle not imported by name; resolved via return type of fauxProvider() declared in packages/ai/src/providers/faux.ts:704, fauxProvider imported at packages/durable/test/codemode-bench.ts:13 from "@earendil-works/pi-ai/providers/faux"
- E5 FauxProviderHandle fields read: provider (packages/durable/test/codemode-bench.ts:49), setResponses (packages/durable/test/codemode-bench.ts:61)
- E5 FauxProviderHandle stoplisted reads: (none)
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
// packages/ai/src/providers/faux.ts:704
export function fauxProvider(options: RegisterFauxProviderOptions = {}): FauxProviderHandle {
	const core = createFauxCore(options);
	const provider = createProvider({
```
```
// packages/durable/test/codemode-bench.ts:49
		const faux = fauxProvider();
		const models = createModels();
		models.setProvider(faux.provider);
		const sandbox = new CodemodeSandbox({ memoryLimitBytes: 256 * 1024 * 1024 });
```
```
// packages/durable/test/codemode-bench.ts:61
				? fauxToolCall("codemode", { code }, { id: "c1" })
				: fauxToolCall("plain", { n, shape }, { id: "c1" });
		faux.setResponses([
			fauxAssistantMessage([call], { stopReason: "toolUse" }),
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 12: code:faux.ts->11-extension-state.ts (judge only: FauxProviderHandle)
- E2 FauxProviderHandle declared at packages/ai/src/providers/faux.ts:146
- E4 FauxProviderHandle not imported by name; resolved via return type of fauxProvider() declared in packages/ai/src/providers/faux.ts:704, fauxProvider imported at packages/durable/test/examples/11-extension-state.ts:7 from "@earendil-works/pi-ai/providers/faux"
- E5 FauxProviderHandle fields read: provider (packages/durable/test/examples/11-extension-state.ts:59), setResponses (packages/durable/test/examples/11-extension-state.ts:65)
- E5 FauxProviderHandle stoplisted reads: (none)
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
// packages/ai/src/providers/faux.ts:704
export function fauxProvider(options: RegisterFauxProviderOptions = {}): FauxProviderHandle {
	const core = createFauxCore(options);
	const provider = createProvider({
```
```
// packages/durable/test/examples/11-extension-state.ts:59
const faux = fauxProvider();
const models = createModels();
models.setProvider(faux.provider);
const registry = createRegistry();
```
```
// packages/durable/test/examples/11-extension-state.ts:65
const root = await harness.root(context, { agent: { model: { provider: "faux", modelId: "faux-1" } } });

faux.setResponses([
	fauxAssistantMessage(fauxToolCall("todo", { item: "fix the build" }), { stopReason: "toolUse" }),
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 13: code:faux.ts->14-chat.ts (judge only: FauxProviderHandle)
- E2 FauxProviderHandle declared at packages/ai/src/providers/faux.ts:146
- E4 FauxProviderHandle not imported by name; resolved via return type of fauxProvider() declared in packages/ai/src/providers/faux.ts:704, fauxProvider imported at packages/durable/test/examples/14-chat.ts:6 from "@earendil-works/pi-ai/providers/faux"
- E5 FauxProviderHandle fields read: provider (packages/durable/test/examples/14-chat.ts:14), setResponses (packages/durable/test/examples/14-chat.ts:15)
- E5 FauxProviderHandle stoplisted reads: (none)
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
// packages/ai/src/providers/faux.ts:704
export function fauxProvider(options: RegisterFauxProviderOptions = {}): FauxProviderHandle {
	const core = createFauxCore(options);
	const provider = createProvider({
```
```
// packages/durable/test/examples/14-chat.ts:14
const faux = fauxProvider();
const models = createModels();
models.setProvider(faux.provider);
faux.setResponses([fauxAssistantMessage("Paris.")]);
```
```
// packages/durable/test/examples/14-chat.ts:15
const models = createModels();
models.setProvider(faux.provider);
faux.setResponses([fauxAssistantMessage("Paris.")]);

```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 14: code:faux.ts->15-system-prompt.ts (judge only: FauxProviderHandle)
- E2 FauxProviderHandle declared at packages/ai/src/providers/faux.ts:146
- E4 FauxProviderHandle not imported by name; resolved via return type of fauxProvider() declared in packages/ai/src/providers/faux.ts:704, fauxProvider imported at packages/durable/test/examples/15-system-prompt.ts:6 from "@earendil-works/pi-ai/providers/faux"
- E5 FauxProviderHandle fields read: provider (packages/durable/test/examples/15-system-prompt.ts:22), setResponses (packages/durable/test/examples/15-system-prompt.ts:23)
- E5 FauxProviderHandle stoplisted reads: (none)
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
// packages/ai/src/providers/faux.ts:704
export function fauxProvider(options: RegisterFauxProviderOptions = {}): FauxProviderHandle {
	const core = createFauxCore(options);
	const provider = createProvider({
```
```
// packages/durable/test/examples/15-system-prompt.ts:22
const faux = fauxProvider();
const models = createModels();
models.setProvider(faux.provider);
faux.setResponses([fauxAssistantMessage("Done."), fauxAssistantMessage("Done."), fauxAssistantMessage("Done.")]);
```
```
// packages/durable/test/examples/15-system-prompt.ts:23
const models = createModels();
models.setProvider(faux.provider);
faux.setResponses([fauxAssistantMessage("Done."), fauxAssistantMessage("Done."), fauxAssistantMessage("Done.")]);

```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 15: code:faux.ts->17-coding-tools.ts (judge only: FauxProviderHandle)
- E2 FauxProviderHandle declared at packages/ai/src/providers/faux.ts:146
- E4 FauxProviderHandle not imported by name; resolved via return type of fauxProvider() declared in packages/ai/src/providers/faux.ts:704, fauxProvider imported at packages/durable/test/examples/17-coding-tools.ts:12 from "@earendil-works/pi-ai/providers/faux"
- E5 FauxProviderHandle fields read: provider (packages/durable/test/examples/17-coding-tools.ts:33), setResponses (packages/durable/test/examples/17-coding-tools.ts:36)
- E5 FauxProviderHandle stoplisted reads: (none)
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
// packages/ai/src/providers/faux.ts:704
export function fauxProvider(options: RegisterFauxProviderOptions = {}): FauxProviderHandle {
	const core = createFauxCore(options);
	const provider = createProvider({
```
```
// packages/durable/test/examples/17-coding-tools.ts:33
const faux = fauxProvider();
const models = createModels();
models.setProvider(faux.provider);
const toolCall = (name: string, args: Parameters<typeof fauxToolCall>[1], id: string) =>
```
```
// packages/durable/test/examples/17-coding-tools.ts:36
const toolCall = (name: string, args: Parameters<typeof fauxToolCall>[1], id: string) =>
	fauxAssistantMessage([fauxToolCall(name, args, { id })], { stopReason: "toolUse" });
faux.setResponses([
	toolCall("read", { path: "notes.txt" }, "r"),
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 16: code:faux.ts->18-print.ts (judge only: FauxProviderHandle)
- E2 FauxProviderHandle declared at packages/ai/src/providers/faux.ts:146
- E4 FauxProviderHandle not imported by name; resolved via return type of fauxProvider() declared in packages/ai/src/providers/faux.ts:704, fauxProvider imported at packages/durable/test/examples/18-print.ts:8 from "@earendil-works/pi-ai/providers/faux"
- E5 FauxProviderHandle fields read: provider (packages/durable/test/examples/18-print.ts:23), setResponses (packages/durable/test/examples/18-print.ts:25)
- E5 FauxProviderHandle stoplisted reads: (none)
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
// packages/ai/src/providers/faux.ts:704
export function fauxProvider(options: RegisterFauxProviderOptions = {}): FauxProviderHandle {
	const core = createFauxCore(options);
	const provider = createProvider({
```
```
// packages/durable/test/examples/18-print.ts:23
} else {
	const faux = fauxProvider();
	models.setProvider(faux.provider);
	model = { provider: "faux", modelId: "faux-1" };
```
```
// packages/durable/test/examples/18-print.ts:25
	models.setProvider(faux.provider);
	model = { provider: "faux", modelId: "faux-1" };
	faux.setResponses([
		fauxAssistantMessage([fauxToolCall("bash", { command: "ls" }, { id: "call-1" })], { stopReason: "toolUse" }),
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 17: code:faux.ts->19-json.ts (judge only: FauxProviderHandle)
- E2 FauxProviderHandle declared at packages/ai/src/providers/faux.ts:146
- E4 FauxProviderHandle not imported by name; resolved via return type of fauxProvider() declared in packages/ai/src/providers/faux.ts:704, fauxProvider imported at packages/durable/test/examples/19-json.ts:14 from "@earendil-works/pi-ai/providers/faux"
- E5 FauxProviderHandle fields read: provider (packages/durable/test/examples/19-json.ts:60), setResponses (packages/durable/test/examples/19-json.ts:62)
- E5 FauxProviderHandle stoplisted reads: (none)
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
// packages/ai/src/providers/faux.ts:704
export function fauxProvider(options: RegisterFauxProviderOptions = {}): FauxProviderHandle {
	const core = createFauxCore(options);
	const provider = createProvider({
```
```
// packages/durable/test/examples/19-json.ts:60
} else {
	const faux = fauxProvider({ tokensPerSecond: 200 });
	models.setProvider(faux.provider);
	model = { provider: "faux", modelId: "faux-1" };
```
```
// packages/durable/test/examples/19-json.ts:62
	models.setProvider(faux.provider);
	model = { provider: "faux", modelId: "faux-1" };
	faux.setResponses([
		fauxAssistantMessage([fauxToolCall("bash", { command: "ls" }, { id: "call-1" })], { stopReason: "toolUse" }),
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 18: code:faux.ts->20-inbox.ts (judge only: FauxProviderHandle)
- E2 FauxProviderHandle declared at packages/ai/src/providers/faux.ts:146
- E4 FauxProviderHandle not imported by name; resolved via return type of fauxProvider() declared in packages/ai/src/providers/faux.ts:704, fauxProvider imported at packages/durable/test/examples/20-inbox.ts:7 from "@earendil-works/pi-ai/providers/faux"
- E5 FauxProviderHandle fields read: provider (packages/durable/test/examples/20-inbox.ts:24), setResponses (packages/durable/test/examples/20-inbox.ts:22)
- E5 FauxProviderHandle stoplisted reads: (none)
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
// packages/ai/src/providers/faux.ts:704
export function fauxProvider(options: RegisterFauxProviderOptions = {}): FauxProviderHandle {
	const core = createFauxCore(options);
	const provider = createProvider({
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
const models = createModels();
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 19: code:faux.ts->21-late-join.ts (judge only: FauxProviderHandle)
- E2 FauxProviderHandle declared at packages/ai/src/providers/faux.ts:146
- E4 FauxProviderHandle not imported by name; resolved via return type of fauxProvider() declared in packages/ai/src/providers/faux.ts:704, fauxProvider imported at packages/durable/test/examples/21-late-join.ts:8 from "@earendil-works/pi-ai/providers/faux"
- E5 FauxProviderHandle fields read: provider (packages/durable/test/examples/21-late-join.ts:50), setResponses (packages/durable/test/examples/21-late-join.ts:45)
- E5 FauxProviderHandle stoplisted reads: (none)
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
// packages/ai/src/providers/faux.ts:704
export function fauxProvider(options: RegisterFauxProviderOptions = {}): FauxProviderHandle {
	const core = createFauxCore(options);
	const provider = createProvider({
```
```
// packages/durable/test/examples/21-late-join.ts:50
]);
const models = createModels();
models.setProvider(faux.provider);
const harness = await Harness.open(new MemoryStorage(), { models, registry }, context);
```
```
// packages/durable/test/examples/21-late-join.ts:45
);
const faux = fauxProvider({ tokensPerSecond: 40, tokenSize: { min: 1, max: 1 } });
faux.setResponses([
	fauxAssistantMessage([fauxToolCall("count", {}, { id: "call-1" })], { stopReason: "toolUse" }),
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 20: code:faux.ts->22-subagent-foreground.ts (judge only: FauxProviderHandle)
- E2 FauxProviderHandle declared at packages/ai/src/providers/faux.ts:146
- E4 FauxProviderHandle not imported by name; resolved via return type of fauxProvider() declared in packages/ai/src/providers/faux.ts:704, fauxProvider imported at packages/durable/test/examples/22-subagent-foreground.ts:11 from "@earendil-works/pi-ai/providers/faux"
- E5 FauxProviderHandle fields read: provider (packages/durable/test/examples/22-subagent-foreground.ts:88), setResponses (packages/durable/test/examples/22-subagent-foreground.ts:91)
- E5 FauxProviderHandle stoplisted reads: (none)
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
// packages/ai/src/providers/faux.ts:704
export function fauxProvider(options: RegisterFauxProviderOptions = {}): FauxProviderHandle {
	const core = createFauxCore(options);
	const provider = createProvider({
```
```
// packages/durable/test/examples/22-subagent-foreground.ts:88
	// The parent delegates, the child answers, and the parent reports.
	const faux = fauxProvider();
	models.setProvider(faux.provider);
	model = { provider: "faux", modelId: "faux-1" };
```
```
// packages/durable/test/examples/22-subagent-foreground.ts:91
	model = { provider: "faux", modelId: "faux-1" };
	const delegate = fauxToolCall("subagent", { task: "Name three prime numbers." }, { id: "call-1" });
	faux.setResponses([
		fauxAssistantMessage([delegate], { stopReason: "toolUse" }),
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 21: code:faux.ts->23-subagent-background.ts (judge only: FauxProviderHandle)
- E2 FauxProviderHandle declared at packages/ai/src/providers/faux.ts:146
- E4 FauxProviderHandle not imported by name; resolved via return type of fauxProvider() declared in packages/ai/src/providers/faux.ts:704, fauxProvider imported at packages/durable/test/examples/23-subagent-background.ts:14 from "@earendil-works/pi-ai/providers/faux"
- E5 FauxProviderHandle fields read: provider (packages/durable/test/examples/23-subagent-background.ts:223), setResponses (packages/durable/test/examples/23-subagent-background.ts:254)
- E5 FauxProviderHandle stoplisted reads: (none)
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
// packages/ai/src/providers/faux.ts:704
export function fauxProvider(options: RegisterFauxProviderOptions = {}): FauxProviderHandle {
	const core = createFauxCore(options);
	const provider = createProvider({
```
```
// packages/durable/test/examples/23-subagent-background.ts:223
	// It streams its answers at 50 tokens per second, like a slow real model.
	const faux = fauxProvider({ tokensPerSecond: 50 });
	models.setProvider(faux.provider);
	model = { provider: "faux", modelId: "faux-1" };
```
```
// packages/durable/test/examples/23-subagent-background.ts:254
	};
	// More responses than the script needs; each request takes the next one.
	faux.setResponses(Array.from({ length: 40 }, () => route));
}
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 22: code:faux.ts->25-compaction.ts (judge only: FauxProviderHandle)
- E2 FauxProviderHandle declared at packages/ai/src/providers/faux.ts:146
- E4 FauxProviderHandle not imported by name; resolved via return type of fauxProvider() declared in packages/ai/src/providers/faux.ts:704, fauxProvider imported at packages/durable/test/examples/25-compaction.ts:7 from "@earendil-works/pi-ai/providers/faux"
- E5 FauxProviderHandle fields read: provider (packages/durable/test/examples/25-compaction.ts:44), setResponses (packages/durable/test/examples/25-compaction.ts:42)
- E5 FauxProviderHandle stoplisted reads: (none)
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
// packages/ai/src/providers/faux.ts:704
export function fauxProvider(options: RegisterFauxProviderOptions = {}): FauxProviderHandle {
	const core = createFauxCore(options);
	const provider = createProvider({
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
const models = createModels();
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 23: code:faux.ts->26-coding-agent.ts (judge only: FauxProviderHandle)
- E2 FauxProviderHandle declared at packages/ai/src/providers/faux.ts:146
- E4 FauxProviderHandle not imported by name; resolved via return type of fauxProvider() declared in packages/ai/src/providers/faux.ts:704, fauxProvider imported at packages/durable/test/examples/26-coding-agent.ts:11 from "@earendil-works/pi-ai/providers/faux"
- E5 FauxProviderHandle fields read: provider (packages/durable/test/examples/26-coding-agent.ts:58), setResponses (packages/durable/test/examples/26-coding-agent.ts:60)
- E5 FauxProviderHandle stoplisted reads: (none)
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
// packages/ai/src/providers/faux.ts:704
export function fauxProvider(options: RegisterFauxProviderOptions = {}): FauxProviderHandle {
	const core = createFauxCore(options);
	const provider = createProvider({
```
```
// packages/durable/test/examples/26-coding-agent.ts:58
const faux = fauxProvider();
const models = createModels();
models.setProvider(faux.provider);
const pwd = () => fauxAssistantMessage(fauxToolCall("bash", { command: "pwd" }), { stopReason: "toolUse" });
```
```
// packages/durable/test/examples/26-coding-agent.ts:60
models.setProvider(faux.provider);
const pwd = () => fauxAssistantMessage(fauxToolCall("bash", { command: "pwd" }), { stopReason: "toolUse" });
faux.setResponses([pwd(), fauxAssistantMessage("Done."), pwd(), fauxAssistantMessage("Done.")]);

```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 24: code:faux.ts->27-plan-mode.ts (judge only: FauxProviderHandle)
- E2 FauxProviderHandle declared at packages/ai/src/providers/faux.ts:146
- E4 FauxProviderHandle not imported by name; resolved via return type of fauxProvider() declared in packages/ai/src/providers/faux.ts:704, fauxProvider imported at packages/durable/test/examples/27-plan-mode.ts:11 from "@earendil-works/pi-ai/providers/faux"
- E5 FauxProviderHandle fields read: provider (packages/durable/test/examples/27-plan-mode.ts:71), setResponses (packages/durable/test/examples/27-plan-mode.ts:72)
- E5 FauxProviderHandle stoplisted reads: (none)
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
// packages/ai/src/providers/faux.ts:704
export function fauxProvider(options: RegisterFauxProviderOptions = {}): FauxProviderHandle {
	const core = createFauxCore(options);
	const provider = createProvider({
```
```
// packages/durable/test/examples/27-plan-mode.ts:71
const faux = fauxProvider();
const models = createModels();
models.setProvider(faux.provider);
faux.setResponses([
```
```
// packages/durable/test/examples/27-plan-mode.ts:72
const models = createModels();
models.setProvider(faux.provider);
faux.setResponses([
	fauxAssistantMessage(fauxToolCall("read", { path: "server.ts" }), { stopReason: "toolUse" }),
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 25: code:faux.ts->28-reviewer.ts (judge only: FauxProviderHandle)
- E2 FauxProviderHandle declared at packages/ai/src/providers/faux.ts:146
- E4 FauxProviderHandle not imported by name; resolved via return type of fauxProvider() declared in packages/ai/src/providers/faux.ts:704, fauxProvider imported at packages/durable/test/examples/28-reviewer.ts:11 from "@earendil-works/pi-ai/providers/faux"
- E5 FauxProviderHandle fields read: provider (packages/durable/test/examples/28-reviewer.ts:54), setResponses (packages/durable/test/examples/28-reviewer.ts:55)
- E5 FauxProviderHandle stoplisted reads: (none)
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
// packages/ai/src/providers/faux.ts:704
export function fauxProvider(options: RegisterFauxProviderOptions = {}): FauxProviderHandle {
	const core = createFauxCore(options);
	const provider = createProvider({
```
```
// packages/durable/test/examples/28-reviewer.ts:54
const faux = fauxProvider({ models: [{ id: "big" }, { id: "small" }] });
const models = createModels();
models.setProvider(faux.provider);
faux.setResponses([
```
```
// packages/durable/test/examples/28-reviewer.ts:55
const models = createModels();
models.setProvider(faux.provider);
faux.setResponses([
	fauxAssistantMessage(fauxToolCall("read", { path: "user.ts" }), { stopReason: "toolUse" }),
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 26: code:faux.ts->29-sandbox-per-conversation.ts (judge only: FauxProviderHandle)
- E2 FauxProviderHandle declared at packages/ai/src/providers/faux.ts:146
- E4 FauxProviderHandle not imported by name; resolved via return type of fauxProvider() declared in packages/ai/src/providers/faux.ts:704, fauxProvider imported at packages/durable/test/examples/29-sandbox-per-conversation.ts:11 from "@earendil-works/pi-ai/providers/faux"
- E5 FauxProviderHandle fields read: provider (packages/durable/test/examples/29-sandbox-per-conversation.ts:30), setResponses (packages/durable/test/examples/29-sandbox-per-conversation.ts:33)
- E5 FauxProviderHandle stoplisted reads: (none)
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
// packages/ai/src/providers/faux.ts:704
export function fauxProvider(options: RegisterFauxProviderOptions = {}): FauxProviderHandle {
	const core = createFauxCore(options);
	const provider = createProvider({
```
```
// packages/durable/test/examples/29-sandbox-per-conversation.ts:30
const faux = fauxProvider();
const models = createModels();
models.setProvider(faux.provider);
const note = (text: string) =>
```
```
// packages/durable/test/examples/29-sandbox-per-conversation.ts:33
const note = (text: string) =>
	fauxAssistantMessage(fauxToolCall("write", { path: "note.txt", content: text }), { stopReason: "toolUse" });
faux.setResponses([
	note("from alice"),
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 27: code:faux.ts->30-tool-override.ts (judge only: FauxProviderHandle)
- E2 FauxProviderHandle declared at packages/ai/src/providers/faux.ts:146
- E4 FauxProviderHandle not imported by name; resolved via return type of fauxProvider() declared in packages/ai/src/providers/faux.ts:704, fauxProvider imported at packages/durable/test/examples/30-tool-override.ts:11 from "@earendil-works/pi-ai/providers/faux"
- E5 FauxProviderHandle fields read: provider (packages/durable/test/examples/30-tool-override.ts:61), setResponses (packages/durable/test/examples/30-tool-override.ts:66)
- E5 FauxProviderHandle stoplisted reads: (none)
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
// packages/ai/src/providers/faux.ts:704
export function fauxProvider(options: RegisterFauxProviderOptions = {}): FauxProviderHandle {
	const core = createFauxCore(options);
	const provider = createProvider({
```
```
// packages/durable/test/examples/30-tool-override.ts:61
const faux = fauxProvider();
const models = createModels();
models.setProvider(faux.provider);
const probe = () =>
```
```
// packages/durable/test/examples/30-tool-override.ts:66
		stopReason: "toolUse",
	});
faux.setResponses([probe(), fauxAssistantMessage("Done."), probe(), fauxAssistantMessage("Done.")]);

```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 28: code:faux.ts->31-reload-and-restart.ts (judge only: FauxProviderHandle)
- E2 FauxProviderHandle declared at packages/ai/src/providers/faux.ts:146
- E4 FauxProviderHandle not imported by name; resolved via return type of fauxProvider() declared in packages/ai/src/providers/faux.ts:704, fauxProvider imported at packages/durable/test/examples/31-reload-and-restart.ts:11 from "@earendil-works/pi-ai/providers/faux"
- E5 FauxProviderHandle fields read: provider (packages/durable/test/examples/31-reload-and-restart.ts:40), setResponses (packages/durable/test/examples/31-reload-and-restart.ts:42)
- E5 FauxProviderHandle stoplisted reads: (none)
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
// packages/ai/src/providers/faux.ts:704
export function fauxProvider(options: RegisterFauxProviderOptions = {}): FauxProviderHandle {
	const core = createFauxCore(options);
	const provider = createProvider({
```
```
// packages/durable/test/examples/31-reload-and-restart.ts:40
const faux = fauxProvider();
const models = createModels();
models.setProvider(faux.provider);
const callVersion = () => fauxAssistantMessage(fauxToolCall("version", {}), { stopReason: "toolUse" });
```
```
// packages/durable/test/examples/31-reload-and-restart.ts:42
models.setProvider(faux.provider);
const callVersion = () => fauxAssistantMessage(fauxToolCall("version", {}), { stopReason: "toolUse" });
faux.setResponses([callVersion(), fauxAssistantMessage("Done."), callVersion(), fauxAssistantMessage("Done.")]);

```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 29: code:faux.ts->harness-host-inputs.test.ts (judge only: FauxProviderHandle)
- E2 FauxProviderHandle declared at packages/ai/src/providers/faux.ts:146
- E4 FauxProviderHandle not imported by name; resolved via return type of fauxProvider() declared in packages/ai/src/providers/faux.ts:704, fauxProvider imported at packages/durable/test/harness-host-inputs.test.ts:1 from "@earendil-works/pi-ai"
- E5 FauxProviderHandle fields read: provider (packages/durable/test/harness-host-inputs.test.ts:31), setResponses (packages/durable/test/harness-host-inputs.test.ts:29)
- E5 FauxProviderHandle stoplisted reads: (none)
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
// packages/ai/src/providers/faux.ts:704
export function fauxProvider(options: RegisterFauxProviderOptions = {}): FauxProviderHandle {
	const core = createFauxCore(options);
	const provider = createProvider({
```
```
// packages/durable/test/harness-host-inputs.test.ts:31
	faux.setResponses(Array.from({ length: 10 }, () => fauxAssistantMessage([fauxText(text)])));
	const models = createModels();
	models.setProvider(faux.provider);
	return models;
```
```
// packages/durable/test/harness-host-inputs.test.ts:29
function answering(text: string): Models {
	const faux = fauxProvider();
	faux.setResponses(Array.from({ length: 10 }, () => fauxAssistantMessage([fauxText(text)])));
	const models = createModels();
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 30: code:faux.ts->nested-images-bench.ts (judge only: FauxProviderHandle)
- E2 FauxProviderHandle declared at packages/ai/src/providers/faux.ts:146
- E4 FauxProviderHandle not imported by name; resolved via return type of fauxProvider() declared in packages/ai/src/providers/faux.ts:704, fauxProvider imported at packages/durable/test/nested-images-bench.ts:20 from "@earendil-works/pi-ai/providers/faux"
- E5 FauxProviderHandle fields read: provider (packages/durable/test/nested-images-bench.ts:174), setResponses (packages/durable/test/nested-images-bench.ts:201)
- E5 FauxProviderHandle stoplisted reads: (none)
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
// packages/ai/src/providers/faux.ts:704
export function fauxProvider(options: RegisterFauxProviderOptions = {}): FauxProviderHandle {
	const core = createFauxCore(options);
	const provider = createProvider({
```
```
// packages/durable/test/nested-images-bench.ts:174
		const faux = fauxProvider();
		const models = createModels();
		models.setProvider(faux.provider);
		let peakHeap = 0;
```
```
// packages/durable/test/nested-images-bench.ts:201
		const harness = await Harness.open(storage, { models, registry }, context);
		const root = await harness.root(context, { agent: { model: { provider: "faux", modelId: "faux-1" } } });
		faux.setResponses([
			fauxAssistantMessage([fauxToolCall("script", { count: scenario.count }, { id: "s" })], {
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 31: code:faux.ts->nested-tools-bench.ts (judge only: FauxProviderHandle)
- E2 FauxProviderHandle declared at packages/ai/src/providers/faux.ts:146
- E4 FauxProviderHandle not imported by name; resolved via return type of fauxProvider() declared in packages/ai/src/providers/faux.ts:704, fauxProvider imported at packages/durable/test/nested-tools-bench.ts:17 from "@earendil-works/pi-ai/providers/faux"
- E5 FauxProviderHandle fields read: provider (packages/durable/test/nested-tools-bench.ts:99), setResponses (packages/durable/test/nested-tools-bench.ts:147)
- E5 FauxProviderHandle stoplisted reads: (none)
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
// packages/ai/src/providers/faux.ts:704
export function fauxProvider(options: RegisterFauxProviderOptions = {}): FauxProviderHandle {
	const core = createFauxCore(options);
	const provider = createProvider({
```
```
// packages/durable/test/nested-tools-bench.ts:99
		const faux = fauxProvider();
		const models = createModels();
		models.setProvider(faux.provider);
		const registry = createRegistry();
```
```
// packages/durable/test/nested-tools-bench.ts:147
		const baseHeap = heapAfterGc();
		for (let run = 0; run < scenario.runs; run++) {
			faux.setResponses([
				fauxAssistantMessage(
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 32: code:faux.ts->scheduler-fanout-bench.ts (judge only: FauxProviderHandle)
- E2 FauxProviderHandle declared at packages/ai/src/providers/faux.ts:146
- E4 FauxProviderHandle not imported by name; resolved via return type of fauxProvider() declared in packages/ai/src/providers/faux.ts:704, fauxProvider imported at packages/durable/test/scheduler-fanout-bench.ts:10 from "@earendil-works/pi-ai/providers/faux"
- E5 FauxProviderHandle fields read: provider (packages/durable/test/scheduler-fanout-bench.ts:18), setResponses (packages/durable/test/scheduler-fanout-bench.ts:50)
- E5 FauxProviderHandle stoplisted reads: (none)
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
// packages/ai/src/providers/faux.ts:704
export function fauxProvider(options: RegisterFauxProviderOptions = {}): FauxProviderHandle {
	const core = createFauxCore(options);
	const provider = createProvider({
```
```
// packages/durable/test/scheduler-fanout-bench.ts:18
	const faux = fauxProvider();
	const models = createModels();
	models.setProvider(faux.provider);
	const registry = createRegistry();
```
```
// packages/durable/test/scheduler-fanout-bench.ts:50
	const harness = await Harness.open(storage, { models, registry }, context);
	const root = await harness.root(context, { agent: { model: { provider: "faux", modelId: "faux-1" } } });
	faux.setResponses([
		fauxAssistantMessage([fauxToolCall("fan", { count: calls }, { id: "fan" })], { stopReason: "toolUse" }),
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 33: code:faux.ts->tool-output-bench.ts (judge only: FauxProviderHandle)
- E2 FauxProviderHandle declared at packages/ai/src/providers/faux.ts:146
- E4 FauxProviderHandle not imported by name; resolved via return type of fauxProvider() declared in packages/ai/src/providers/faux.ts:704, fauxProvider imported at packages/durable/test/tool-output-bench.ts:18 from "@earendil-works/pi-ai/providers/faux"
- E5 FauxProviderHandle fields read: provider (packages/durable/test/tool-output-bench.ts:136), setResponses (packages/durable/test/tool-output-bench.ts:186)
- E5 FauxProviderHandle stoplisted reads: (none)
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
// packages/ai/src/providers/faux.ts:704
export function fauxProvider(options: RegisterFauxProviderOptions = {}): FauxProviderHandle {
	const core = createFauxCore(options);
	const provider = createProvider({
```
```
// packages/durable/test/tool-output-bench.ts:136
		const faux = fauxProvider();
		const models = createModels();
		models.setProvider(faux.provider);
		const registry = createRegistry();
```
```
// packages/durable/test/tool-output-bench.ts:186
				? [fauxToolCall("bash", { command: `cat ${BIG_FILE}` }, { id: "c0" })]
				: Array.from({ length: toolCount }, (_, index) => fauxToolCall("emit", {}, { id: `c${index}` }));
		faux.setResponses([
			fauxAssistantMessage(call, { stopReason: "toolUse" }),
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 34: code:transcript.ts->agent-loop.ts (judge only: ToolStateChanges)
- E2 ToolStateChanges declared at packages/ai/src/utils/transcript.ts:144
- E4 ToolStateChanges imported at packages/agent/src/agent-loop.ts:6 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:44 from packages/ai/src/utils/transcript.ts; receiver resolved via return type of getToolStateChanges() declared in packages/ai/src/utils/transcript.ts:150
- E5 ToolStateChanges fields read: toolsAdded (packages/agent/src/agent-loop.ts:351), toolsRemoved (packages/agent/src/agent-loop.ts:351)
- E5 ToolStateChanges stoplisted reads: (none)
```
// packages/ai/src/utils/transcript.ts:144
export interface ToolStateChanges {
	toolsAdded: Tool[];
	toolsRemoved: ToolReference[];
}

/** Compare two complete tool states. A changed definition is a removal followed by an addition. */
export function getToolStateChanges(previous: readonly Tool[], current: readonly Tool[]): ToolStateChanges {
	const previousTools = new Map(previous.map((tool) => [tool.name, tool]));
	const currentTools = new Map(current.map((tool) => [tool.name, tool]));
	return {
		toolsAdded: current
			.filter((tool) => {
```
```
// packages/ai/src/utils/transcript.ts:150
export function getToolStateChanges(previous: readonly Tool[], current: readonly Tool[]): ToolStateChanges {
	const previousTools = new Map(previous.map((tool) => [tool.name, tool]));
	const currentTools = new Map(current.map((tool) => [tool.name, tool]));
```
```
// packages/agent/src/agent-loop.ts:351
		(context.tools ?? []).map(toToolDeclaration),
	);
	const unchanged = changes.toolsAdded.length === 0 && changes.toolsRemoved.length === 0;

```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 35: code:output.ts->tool.ts (judge only: OutputSlice)
- E2 OutputSlice declared at packages/chord/src/output.ts:13
- E4 OutputSlice not imported by name; resolved via return type of boundOutput() declared in packages/chord/src/output.ts:49, boundOutput imported at packages/durable/src/harness/tool.ts:4 from "@earendil-works/chord/output"
- E5 OutputSlice fields read: droppedBytes (packages/durable/src/harness/tool.ts:555), droppedLines (packages/durable/src/harness/tool.ts:919), text (packages/durable/src/harness/tool.ts:917)
- E5 OutputSlice stoplisted reads: (none)
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
// packages/chord/src/output.ts:49
export function boundOutput(text: string, limits: OutputLimits): OutputSlice {
	const bytes = encoder.encode(text);
	const [from, to] = limits.retain === "head" ? headRange(bytes, limits) : tailRange(bytes, limits);
```
```
// packages/durable/src/harness/tool.ts:555
	}
	const bounded = boundContent(final.output ?? [], reported.limits);
	if (bounded.droppedBytes > 0) harness.push(truncated(bounded, reported.limits.retain));
	const broken = structuredOutputError(tool, final);
```
```
// packages/durable/src/harness/tool.ts:919
		else if (item === keep) result.push({ ...item, text: bounded.text });
	}
	return { content: result, droppedBytes: bounded.droppedBytes, droppedLines: bounded.droppedLines };
}
```
```
// packages/durable/src/harness/tool.ts:917
	for (const item of content) {
		if (item.type !== "text") result.push(item);
		else if (item === keep) result.push({ ...item, text: bounded.text });
	}
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 36: code:truncate.ts->index.ts (judge only: TruncationResult)
- E2 TruncationResult declared at packages/coding-agent/src/core/tools/truncate.ts:15
- E4 TruncationResult not imported by name; resolved via return type of truncateHead() declared in packages/coding-agent/src/core/tools/truncate.ts:78, truncateHead imported at packages/coding-agent/examples/extensions/gondolin/index.ts:25 from "@earendil-works/pi-coding-agent"
- E5 TruncationResult fields read: content (packages/coding-agent/examples/extensions/gondolin/index.ts:293), truncated (packages/coding-agent/examples/extensions/gondolin/index.ts:303)
- E5 TruncationResult stoplisted reads: (none)
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
// packages/coding-agent/src/core/tools/truncate.ts:78
export function truncateHead(content: string, options: TruncationOptions = {}): TruncationResult {
	const maxLines = options.maxLines ?? DEFAULT_MAX_LINES;
	const maxBytes = options.maxBytes ?? DEFAULT_MAX_BYTES;
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
		details.truncation = truncation;
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 37: code:colors.ts->system-theme.ts (judge only: RgbColorValue)
- E2 RgbColorValue declared at packages/tui/src/colors.ts:9
- E4 RgbColorValue not imported by name; resolved via return type of okhslColor() declared in packages/tui/src/colors.ts:107, okhslColor imported at packages/coding-agent/src/modes/interactive/theme/system-theme.ts:22 from "@earendil-works/pi-tui"
- E5 RgbColorValue fields read: b (packages/coding-agent/src/modes/interactive/theme/system-theme.ts:392), g (packages/coding-agent/src/modes/interactive/theme/system-theme.ts:392), r (packages/coding-agent/src/modes/interactive/theme/system-theme.ts:392)
- E5 RgbColorValue stoplisted reads: (none)
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
// packages/tui/src/colors.ts:107
export function okhslColor(h: number, s: number, l: number): RgbColorValue {
	requireFinite(h, "h");
	requireFinite(s, "s");
```
```
// packages/coding-agent/src/modes/interactive/theme/system-theme.ts:392
/** OKLab lightness of an sRGB color, 0-1. */
function oklabLightness(color: RgbColor): number {
	return colorToOklch(rgbColor(color.r, color.g, color.b)).l;
}
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 38: code:fuzzy.ts->session-selector-search.ts (judge only: FuzzyMatch)
- E2 FuzzyMatch declared at packages/tui/src/fuzzy.ts:7
- E4 FuzzyMatch not imported by name; resolved via return type of fuzzyMatch() declared in packages/tui/src/fuzzy.ts:12, fuzzyMatch imported at packages/coding-agent/src/modes/interactive/components/session-selector-search.ts:1 from "@earendil-works/pi-tui"
- E5 FuzzyMatch fields read: matches (packages/coding-agent/src/modes/interactive/components/session-selector-search.ts:149), score (packages/coding-agent/src/modes/interactive/components/session-selector-search.ts:150)
- E5 FuzzyMatch stoplisted reads: (none)
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
// packages/tui/src/fuzzy.ts:12
export function fuzzyMatch(query: string, text: string): FuzzyMatch {
	const queryLower = query.toLowerCase();
	const textLower = text.toLowerCase();
```
```
// packages/coding-agent/src/modes/interactive/components/session-selector-search.ts:149

		const m = fuzzyMatch(token.value, text);
		if (!m.matches) return { matches: false, score: 0 };
		totalScore += m.score;
```
```
// packages/coding-agent/src/modes/interactive/components/session-selector-search.ts:150
		const m = fuzzyMatch(token.value, text);
		if (!m.matches) return { matches: false, score: 0 };
		totalScore += m.score;
	}
```
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 39: code:stickyNote.ts->restore.ts (judge only: StickyNoteLayout)
- E2 StickyNoteLayout declared at packages/element/src/stickyNote.ts:548
- E4 StickyNoteLayout not imported by name; resolved via return type of getStickyNoteLayout() declared in packages/element/src/stickyNote.ts:669, getStickyNoteLayout imported at packages/excalidraw/data/restore.ts:53 from "@excalidraw/element"
- E5 StickyNoteLayout fields read: container (packages/excalidraw/data/restore.ts:948), text (packages/excalidraw/data/restore.ts:949)
- E5 StickyNoteLayout stoplisted reads: (none)
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
// packages/element/src/stickyNote.ts:669
export const getStickyNoteLayout = (
  container: ExcalidrawStickyNoteElement,
  textElement: ExcalidrawTextElement | null,
```
```
// packages/excalidraw/data/restore.ts:948
      const textElement = getBoundTextElement(element, elementsMap);
      const layout = getStickyNoteLayout(element, textElement);
      Object.assign(element, layout.container);
      if (textElement && layout.text) {
```
```
// packages/excalidraw/data/restore.ts:949
      const layout = getStickyNoteLayout(element, textElement);
      Object.assign(element, layout.container);
      if (textElement && layout.text) {
        Object.assign(textElement, layout.text);
```
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 40: code:stickyNote.ts->stickyNotes.test.tsx (judge only: StickyNoteLayout)
- E2 StickyNoteLayout declared at packages/element/src/stickyNote.ts:548
- E4 StickyNoteLayout not imported by name; resolved via return type of getStickyNoteLayout() declared in packages/element/src/stickyNote.ts:669, getStickyNoteLayout imported at packages/excalidraw/tests/stickyNotes.test.tsx:13 from "@excalidraw/element"
- E5 StickyNoteLayout fields read: container (packages/excalidraw/tests/stickyNotes.test.tsx:264), text (packages/excalidraw/tests/stickyNotes.test.tsx:265)
- E5 StickyNoteLayout stoplisted reads: (none)
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
// packages/element/src/stickyNote.ts:669
export const getStickyNoteLayout = (
  container: ExcalidrawStickyNoteElement,
  textElement: ExcalidrawTextElement | null,
```
```
// packages/excalidraw/tests/stickyNotes.test.tsx:264
    expect(note.height).toBeLessThan(grownHeight);
    const layout = getStickyNoteLayout(note, label);
    expect(note.height).toBeCloseTo(layout.container.height);
    expect(label.fontSize).toBe(layout.text!.fontSize);
```
```
// packages/excalidraw/tests/stickyNotes.test.tsx:265
    const layout = getStickyNoteLayout(note, label);
    expect(note.height).toBeCloseTo(layout.container.height);
    expect(label.fontSize).toBe(layout.text!.fontSize);
    expect(label.y).toBeCloseTo(layout.text!.y);
```
Judgment (TRUE / NOT TRUE, one sentence):

