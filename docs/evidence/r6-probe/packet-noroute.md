### earendil-works__pi 1: ModelAuth (packages/ai/src/auth/types.ts) read in packages/coding-agent/src/core/bug-report.ts
- fields claimed: apiKey, baseUrl, headers
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
// packages/coding-agent/src/core/bug-report.ts:113
function describeProvider(modelRuntime: ModelRuntime, provider: Provider) {
	const authTypes: Array<"api_key" | "oauth"> = [];
	if (provider.auth.apiKey) authTypes.push("api_key");
```
```
// packages/coding-agent/src/core/bug-report.ts:99
		name: model.name,
		api: model.api,
		baseUrl: redactUrl(model.baseUrl),
```
```
// packages/coding-agent/src/core/bug-report.ts:107
		compat: model.compat ? redactJsonValue(model.compat) : null,
		thinkingLevelMap: model.thinkingLevelMap ?? null,
		headerNames: Object.keys(model.headers ?? {}).sort(),
```
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 2: RangeProps (packages/excalidraw/components/Range.tsx) read in packages/element/src/renderElement.ts
- fields claimed: max, min
```
// packages/excalidraw/components/Range.tsx:5
export type RangeProps = {
  label: React.ReactNode;
  value: number;
  onChange: (value: number) => void;
  min?: number;
  max?: number;
  step?: number;
  minLabel?: React.ReactNode;
  hasCommonValue?: boolean;
  testId?: string;
};

```
```
// packages/element/src/renderElement.ts:797
    // generously covers the arrow's blit at any rotation
    const outerHalf =
      Math.max(distance(x1, x2), distance(y1, y2)) * devicePixelRatio +
```
```
// packages/element/src/renderElement.ts:257
    height * scale > WIDTH_HEIGHT_LIMIT
  ) {
    scale = Math.min(WIDTH_HEIGHT_LIMIT / width, WIDTH_HEIGHT_LIMIT / height);
```
Judgment (TRUE / NOT TRUE, one sentence):

### puppeteer__puppeteer 3: LaunchOptions (packages/puppeteer-core/src/node/LaunchOptions.ts) read in test/src/cdp/pipe.test.ts
- fields claimed: args, timeout
```
// packages/puppeteer-core/src/node/LaunchOptions.ts:39
export interface LaunchOptions extends ConnectOptions {
  /**
   * If specified for Chrome, looks for a regular Chrome installation at a known
   * system location instead of using the bundled Chrome binary.
   */
  channel?: ChromeReleaseChannel;
  /**
   * Path to a browser executable to use instead of the bundled browser. Note
   * that Puppeteer is only guaranteed to work with the bundled browser, so use
   * this setting at your own risk.
   *
   * @remarks
```
```
// test/src/cdp/pipe.test.ts:28
    const {defaultBrowserOptions} = await getTestState({skipLaunch: true});
    const options = Object.assign({}, defaultBrowserOptions);
    options.args = ['--remote-debugging-pipe'].concat(options.args || []);
```
```
// test/src/cdp/pipe.test.ts:49
        Deferred.create({
          message: `Failed in after Hook`,
          timeout: this.timeout() - 1000,
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 4: ServerCapabilities (packages/mcp/src/protocol/types.ts) read in packages/coding-agent/test/mcp-extension.test.ts
- fields claimed: resources, tools
```
// packages/mcp/src/protocol/types.ts:30
export interface ServerCapabilities {
	experimental?: Record<string, unknown>;
	logging?: Record<string, unknown>;
	prompts?: { listChanged?: boolean };
	resources?: { subscribe?: boolean; listChanged?: boolean };
	tools?: { listChanged?: boolean };
	completions?: Record<string, unknown>;
}

export interface InitializeParams {
	protocolVersion: string;
	capabilities: ClientCapabilities;
```
```
// packages/coding-agent/test/mcp-extension.test.ts:462
								capabilities: {
									...(options.noTools ? { prompts: {} } : { tools: {} }),
									...(options.resources ? { resources: {} } : {}),
```
```
// packages/coding-agent/test/mcp-extension.test.ts:115
		// Overrides keep `enabled: true`, since it replaces the global value.
		updateMcpServerConfig(project, "tools", { enabled: true });
		expect(JSON.parse(readFileSync(project, "utf8")).mcpServers.tools).toEqual({ enabled: true });
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 5: BoundaryState (packages/coding-agent/src/core/extensions/types.ts) read in packages/coding-agent/examples/extensions/gondolin/index.ts
- fields claimed: context, entries
```
// packages/coding-agent/src/core/extensions/types.ts:1015
export interface BoundaryState {
	entries: SessionBoundaryDraft[];
	continue: boolean;
	context: BoundaryContextPreview;
	outcome: AgentActivityOutcome;
}

export interface BoundaryResult {
	entries?: SessionBoundaryDraft[];
	continue?: boolean;
}

```
```
// packages/coding-agent/examples/extensions/gondolin/index.ts:249
	const rootIsDirectory = rootStat.isDirectory();
	const matcher = createLineMatcher(params.pattern, params.literal, params.ignoreCase);
	const contextLines = params.context && params.context > 0 ? params.context : 0;
```
```
// packages/coding-agent/examples/extensions/gondolin/index.ts:318
	if (!env) return undefined;
	const result: Record<string, string> = {};
	for (const [key, value] of Object.entries(env)) {
```
Judgment (TRUE / NOT TRUE, one sentence):

### microsoft__playwright 6: ScreencastClient (packages/playwright-core/src/server/screencast.ts) read in packages/playwright/src/index.ts
- fields claimed: dispose, fps, size
```
// packages/playwright-core/src/server/screencast.ts:28
export type ScreencastClient = {
  onFrame: (frame: types.ScreencastFrame) => Promise<void> | void;
  gracefulClose?: () => Promise<void> | void;
  dispose: () => void;
  size?: types.Size;
  quality?: number;
  fps?: number;
};

type AnnotatePosition = 'top-left' | 'top' | 'top-right' | 'bottom-left' | 'bottom' | 'bottom-right';

type ActionOptions = {
```
```
// packages/playwright/src/index.ts:196
    const hook = (test.info() as TestInfoImpl)._currentHookType();
    if (hook === 'beforeAll') {
      await request.dispose({ reason: [
```
```
// packages/playwright/src/index.ts:421
          dir: tracing().artifactsDir(),
          size: typeof video === 'string' ? undefined : video.size,
          fps: typeof video === 'string' ? undefined : video.fps,
```
```
// packages/playwright/src/index.ts:420
        recordVideo: {
          dir: tracing().artifactsDir(),
          size: typeof video === 'string' ? undefined : video.size,
```
Judgment (TRUE / NOT TRUE, one sentence):

### microsoft__playwright 7: BrowserNewContextParams (packages/playwright-core/src/server/channels.d.ts) read in packages/playwright/src/index.ts
- fields claimed: acceptDownloads, baseURL, bypassCSP, clientCertificates, colorScheme, contrast, deviceScaleFactor, extraHTTPHeaders, forcedColors, geolocation, hasTouch, httpCredentials, ignoreHTTPSErrors, isMobile, javaScriptEnabled, locale, offline, permissions, proxy, reducedMotion, screen, serviceWorkers, storageState, timezoneId, userAgent, viewport
```
// packages/playwright-core/src/server/channels.d.ts:880
export type BrowserNewContextParams = {
  noDefaultViewport?: boolean,
  viewport?: {
    width: number,
    height: number,
  },
  screen?: {
    width: number,
    height: number,
  },
  ignoreHTTPSErrors?: boolean,
  clientCertificates?: {
```
```
// packages/playwright/src/index.ts:264
  }, { scope: 'worker', timeout: 0 }],

  acceptDownloads: [({ contextOptions }, use) => use(contextOptions.acceptDownloads ?? true), { option: true, box: true }],
```
```
// packages/playwright/src/index.ts:374
      options.viewport = viewport;
    if (baseURL !== undefined)
      options.baseURL = baseURL;
```
```
// packages/playwright/src/index.ts:265

  acceptDownloads: [({ contextOptions }, use) => use(contextOptions.acceptDownloads ?? true), { option: true, box: true }],
  bypassCSP: [({ contextOptions }, use) => use(contextOptions.bypassCSP ?? false), { option: true, box: true }],
```
```
// packages/playwright/src/index.ts:284
  screen: [({ contextOptions }, use) => use(contextOptions.screen), { option: true, box: true }],
  storageState: [({ contextOptions }, use) => use(contextOptions.storageState), { option: true, box: true }],
  clientCertificates: [({ contextOptions }, use) => use(contextOptions.clientCertificates), { option: true, box: true }],
```
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 8: ExportedDataState (packages/excalidraw/data/types.ts) read in excalidraw-app/data/index.ts
- fields claimed: appState, elements
```
// packages/excalidraw/data/types.ts:14
export interface ExportedDataState {
  type: string;
  version: number;
  source: string;
  elements: readonly ExcalidrawElement[];
  appState: ReturnType<typeof cleanAppStateForExport>;
  files: BinaryFiles | undefined;
}

/**
 * Map of legacy AppState keys, with values of:
 *  [<legacy type>, <new AppState proeprty>]
```
```
// excalidraw-app/data/index.ts:198
  return {
    elements: data.elements || null,
    appState: data.appState || null,
```
```
// excalidraw-app/data/index.ts:197

  return {
    elements: data.elements || null,
```
Judgment (TRUE / NOT TRUE, one sentence):

### microsoft__playwright 9: StackSession (packages/playwright-core/src/server/localUtils.ts) read in packages/playwright/src/worker/testTracing.ts
- fields claimed: file, live
```
// packages/playwright-core/src/server/localUtils.ts:36
export type StackSession = {
  file: string;
  writer: Promise<void>;
  tmpDir: string | undefined;
  callStacks: channels.ClientSideCallMetadata[];
  live?: boolean;
};

export async function zip(progress: Progress, stackSessions: Map<string, StackSession>, params: channels.LocalUtilsZipParams): Promise<void> {
  const promise = new ManualPromise<void>();
  const zipFile = new yazl.ZipFile();
  (zipFile as any as EventEmitter).on('error', error => promise.reject(error));
```
```
// packages/playwright/src/worker/testTracing.ts:120
      // Note that trace name must start with testId for live tracing to work.
      this._liveTraceFile = { file: path.join(this._tracesDir, `${this._testInfo.testId}-test.trace`), fs: new SerializedFS() };
      this._liveTraceFile.fs.mkdir(path.dirname(this._liveTraceFile.file));
```
```
// packages/playwright/src/worker/testTracing.ts:117
    }

    if (!this._liveTraceFile && this._options.live) {
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 10: AssistantMessage (packages/ai/src/types.ts) read in packages/coding-agent/test/suite/virtual-models.test.ts
- fields claimed: errorMessage, model, role, stopReason, thinkingLevel
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
// packages/coding-agent/test/suite/virtual-models.test.ts:93

		expect(reasons()).toEqual(["user", "retry", "continuation"]);
		expect(requests[1].failed?.message.errorMessage).toBe("overloaded_error");
```
```
// packages/coding-agent/test/suite/virtual-models.test.ts:25
	const sticky = request.failed ?? request.previous;
	if (request.reason !== "user" && sticky) {
		return { model: sticky.model, thinkingLevel: sticky.thinkingLevel ?? "high" };
```
```
// packages/coding-agent/test/suite/virtual-models.test.ts:75
		const dispatched = () =>
			harness.session.messages.flatMap((message) =>
				message.role === "assistant" ? [`${message.provider}/${message.model}:${message.thinkingLevel}`] : [],
```
```
// packages/coding-agent/test/suite/virtual-models.test.ts:146
		expect(reasons()).toEqual(["user", "retry"]);
		expect(requests[1].failed?.model.id).toBe("large");
		expect(requests[1].failed?.message.stopReason).toBe("length");
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 11: FauxModelDefinition (packages/ai/src/providers/faux.ts) read in packages/coding-agent/test/cache-stats.test.ts
- fields claimed: cost, input
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
// packages/coding-agent/test/cache-stats.test.ts:38
			cacheWrite: options.cacheWrite ?? 0,
			totalTokens: 0,
			cost: { ...zeroCost, ...options.cost },
```
```
// packages/coding-agent/test/cache-stats.test.ts:33
		model: options.model ?? "test-model",
		usage: {
			input: options.input ?? 0,
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 12: ProviderRequestOptions (packages/ai/src/types.ts) read in packages/coding-agent/examples/extensions/custom-provider-gitlab-duo/index.ts
- fields claimed: apiKey, headers
```
// packages/ai/src/types.ts:157
export interface ProviderRequestOptions<TModel = Model<Api>> {
	signal?: AbortSignal;
	/** Explicit parent context for telemetry produced by this logical request. */
	telemetryContext?: TelemetryContext;
	apiKey?: string;
	/**
	 * Optional fetch implementation for provider HTTP requests.
	 * Defaults to `globalThis.fetch`. Provider adapters that cannot inject a custom implementation may reject it.
	 * This does not affect WebSocket transports.
	 */
	fetch?: FetchFunction;
	/**
```
```
// packages/coding-agent/examples/extensions/custom-provider-gitlab-duo/index.ts:316
	(async () => {
		try {
			const gitlabAccessToken = options?.apiKey;
```
```
// packages/coding-agent/examples/extensions/custom-provider-gitlab-duo/index.ts:206

	const data = (await response.json()) as { token: string; headers: Record<string, string> };
	cachedDirectAccess = { token: data.token, headers: data.headers, expiresAt: now + DIRECT_ACCESS_TTL };
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 13: CombinedAbortSignal (packages/ai/src/utils/abort-signals.ts) read in packages/coding-agent/test/suite/regressions/9340-9777-auto-compaction-cancellation.test.ts
- fields claimed: cleanup, signal
```
// packages/ai/src/utils/abort-signals.ts:1
export interface CombinedAbortSignal {
	signal?: AbortSignal;
	cleanup: () => void;
}

export function combineAbortSignals(signals: readonly (AbortSignal | undefined)[]): CombinedAbortSignal {
	const activeSignals = signals.filter((signal): signal is AbortSignal => signal !== undefined);
	if (activeSignals.length === 0) {
		return { cleanup: () => {} };
	}
	if (activeSignals.length === 1) {
		return { signal: activeSignals[0], cleanup: () => {} };
```
```
// packages/coding-agent/test/suite/regressions/9340-9777-auto-compaction-cancellation.test.ts:43
	afterEach(() => {
		vi.restoreAllMocks();
		while (harnesses.length > 0) harnesses.pop()?.cleanup();
```
```
// packages/coding-agent/test/suite/regressions/9340-9777-auto-compaction-cancellation.test.ts:88
		let authSignal: AbortSignal | undefined;
		vi.spyOn(harness.session.modelRuntime, "getAuth").mockImplementation(async (_model, options) => {
			authSignal = options?.signal;
```
Judgment (TRUE / NOT TRUE, one sentence):

### microsoft__playwright 14: BrowserTypeLaunchParams (packages/playwright-core/src/server/channels.d.ts) read in packages/playwright/types/test.d.ts
- fields claimed: channel, proxy
```
// packages/playwright-core/src/server/channels.d.ts:1813
export type BrowserTypeLaunchParams = {
  channel?: string,
  executablePath?: string,
  args?: string[],
  ignoreAllDefaultArgs?: boolean,
  ignoreDefaultArgs?: string[],
  handleSIGINT?: boolean,
  handleSIGTERM?: boolean,
  handleSIGHUP?: boolean,
  env?: NameValue[],
  headless?: boolean,
  proxy?: {
```
```
// packages/playwright/types/test.d.ts:7423
   * [browserType.launch([options])](https://playwright.dev/docs/api/class-browsertype#browser-type-launch). Specific
   * options [testOptions.headless](https://playwright.dev/docs/api/class-testoptions#test-options-headless) and
   * [testOptions.channel](https://playwright.dev/docs/api/class-testoptions#test-options-channel) take priority over
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 15: Provider (packages/ai/src/models.ts) read in packages/coding-agent/test/model-runtime-images.test.ts
- fields claimed: auth, baseUrl, headers
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
// packages/coding-agent/test/model-runtime-images.test.ts:309
		const chatAuth = await runtime.getAuth(chat);
		const imageAuth = await runtime.getAuth(image);
		expect(imageAuth?.auth.apiKey).toBe("sk-or");
```
```
// packages/coding-agent/test/model-runtime-images.test.ts:138
			["chat", "extension-chat"],
		]);
		expect(runtime.getModel("mixed", "extension-chat")?.baseUrl).toBe("https://chat-proxy.test/v1");
```
```
// packages/coding-agent/test/model-runtime-images.test.ts:155
				"test-images": {
					generateImages: async (model, _context, options) => {
						observed.push({ apiKey: options?.apiKey, headers: options?.headers });
```
Judgment (TRUE / NOT TRUE, one sentence):

### microsoft__playwright 16: BrowserNewContextForReuseOptions (packages/playwright-core/src/client/channels.d.ts) read in packages/playwright/src/index.ts
- fields claimed: acceptDownloads, baseURL, bypassCSP, clientCertificates, colorScheme, contrast, deviceScaleFactor, extraHTTPHeaders, forcedColors, geolocation, hasTouch, httpCredentials, ignoreHTTPSErrors, isMobile, javaScriptEnabled, locale, offline, permissions, proxy, reducedMotion, screen, serviceWorkers, storageState, timezoneId, userAgent, viewport
```
// packages/playwright-core/src/client/channels.d.ts:1110
export type BrowserNewContextForReuseOptions = {
  noDefaultViewport?: boolean,
  viewport?: {
    width: number,
    height: number,
  },
  screen?: {
    width: number,
    height: number,
  },
  ignoreHTTPSErrors?: boolean,
  clientCertificates?: {
```
```
// packages/playwright/src/index.ts:264
  }, { scope: 'worker', timeout: 0 }],

  acceptDownloads: [({ contextOptions }, use) => use(contextOptions.acceptDownloads ?? true), { option: true, box: true }],
```
```
// packages/playwright/src/index.ts:374
      options.viewport = viewport;
    if (baseURL !== undefined)
      options.baseURL = baseURL;
```
```
// packages/playwright/src/index.ts:265

  acceptDownloads: [({ contextOptions }, use) => use(contextOptions.acceptDownloads ?? true), { option: true, box: true }],
  bypassCSP: [({ contextOptions }, use) => use(contextOptions.bypassCSP ?? false), { option: true, box: true }],
```
```
// packages/playwright/src/index.ts:284
  screen: [({ contextOptions }, use) => use(contextOptions.screen), { option: true, box: true }],
  storageState: [({ contextOptions }, use) => use(contextOptions.storageState), { option: true, box: true }],
  clientCertificates: [({ contextOptions }, use) => use(contextOptions.clientCertificates), { option: true, box: true }],
```
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 17: RectangleBox (packages/element/src/bounds.ts) read in packages/excalidraw/components/App.textTool.ts
- fields claimed: x, y
```
// packages/element/src/bounds.ts:68
export type RectangleBox = {
  x: number;
  y: number;
  width: number;
  height: number;
  angle: number;
};

type MaybeQuadraticSolution = [number | null, number | null] | false;

export type SceneBounds = readonly [
  sceneX: number,
```
```
// packages/excalidraw/components/App.textTool.ts:288
      return;
    }
    const sceneX = pointerDownState.origin.x;
```
```
// packages/excalidraw/components/App.textTool.ts:289
    }
    const sceneX = pointerDownState.origin.x;
    const sceneY = pointerDownState.origin.y;
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 18: FauxProviderHandle (packages/ai/src/providers/faux.ts) read in packages/durable/test/harness-nested-tools-restart.test.ts
- fields claimed: setResponses, state
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
// packages/durable/test/harness-nested-tools-restart.test.ts:138
			}),
		);
		setup.faux.setResponses([call("batch"), DONE]);
```
```
// packages/durable/test/harness-nested-tools-restart.test.ts:154
		for (const task of [started, gate]) {
			expect(task).toMatchObject({ abandonOnRestart: true, abortRequested: true, abortReason: "restart" });
			expect(task!.state).toMatchObject({ status: "terminal", outcome: { status: "aborted" } });
```
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 19: ImportedDataState (packages/excalidraw/data/types.ts) read in excalidraw-app/CustomStats.tsx
- fields claimed: appState, elements
```
// packages/excalidraw/data/types.ts:35
export interface ImportedDataState {
  type?: string;
  version?: number;
  source?: string;
  elements?: readonly ExcalidrawElement[] | null;
  appState?: Readonly<
    Partial<
      AppState & {
        [T in keyof LegacyAppState]: LegacyAppState[T][0];
      }
    >
  > | null;
```
```
// excalidraw-app/CustomStats.tsx:46
      setStorageSizes(sizes);
    });
  }, [props.elements, props.appState]);
```
```
// excalidraw-app/CustomStats.tsx:46
      setStorageSizes(sizes);
    });
  }, [props.elements, props.appState]);
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 20: ClassifierResult (packages/ai/src/types.ts) read in packages/durable/test/examples/26-coding-agent.ts
- fields claimed: model, provider
```
// packages/ai/src/types.ts:718
export interface ClassifierResult {
	api: ClassifierApi;
	provider: ProviderId;
	model: string;
	answers: Record<string, ClassifierAnswer>;
	/** Token usage and its cost at the model's catalog price, when the service reports token counts. */
	usage?: Usage;
	stopReason: ClassifierStopReason;
	errorMessage?: string;
	timestamp: number; // Unix timestamp in milliseconds
}

```
```
// packages/durable/test/examples/26-coding-agent.ts:67
const lastBashOutput = async () => {
	const page = await root.entries({}, 10, undefined, context);
	const result = page.items.find((entry) => ToolResultEntry.is(entry))?.model?.[0] as ToolResultMessage;
```
```
// packages/durable/test/examples/26-coding-agent.ts:58
const faux = fauxProvider();
const models = createModels();
models.setProvider(faux.provider);
```
Judgment (TRUE / NOT TRUE, one sentence):

### microsoft__playwright 21: CDPSessionSendParams (packages/playwright-core/src/server/channels.d.ts) read in packages/playwright/src/common/process.ts
- fields claimed: method, params
```
// packages/playwright-core/src/server/channels.d.ts:5053
export type CDPSessionSendParams = {
  method: string,
  params?: any,
};
export type CDPSessionSendOptions = {
  params?: any,
};
export type CDPSessionSendResult = {
  result: any,
};
export type CDPSessionDetachParams = {};
export type CDPSessionDetachOptions = {};
```
```
// packages/playwright/src/common/process.ts:74

  process.on('message', async (message: any) => {
    if (message.method === '__init__') {
```
```
// packages/playwright/src/common/process.ts:75
  process.on('message', async (message: any) => {
    if (message.method === '__init__') {
      const { processParams, runnerParams } = message.params as { processParams: ProcessInitParams, runnerParams: any };
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 22: GrammarToolInputJsonBuffer (packages/ai/src/api/constrained-sampling.ts) read in packages/durable/test/harness-events.test.ts
- fields claimed: closed, input
```
// packages/ai/src/api/constrained-sampling.ts:152
export interface GrammarToolInputJsonBuffer {
	input: string;
	started: boolean;
	closed: boolean;
}

export function getGrammarToolInput(
	toolName: string,
	arguments_: Record<string, unknown>,
	inputProperty: string,
): string {
	const input = arguments_[inputProperty];
```
```
// packages/durable/test/harness-events.test.ts:596
		const stream: AgentEventStream = await watchEvents(harness, root.id, context);
		await harness.close(context);
		expect(await stream.closed).toEqual({ reason: "session_closed" });
```
```
// packages/durable/test/harness-events.test.ts:553
		// A usage-only change sends the usage and no changes.
		await change((live) => {
			live.generation!.message!.usage.input = 42;
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 23: AssistantImages (packages/ai/src/types.ts) read in packages/coding-agent/src/core/cache-stats.ts
- fields claimed: model, provider, timestamp, usage
```
// packages/ai/src/types.ts:655
export interface AssistantImages {
	api: ImageApi;
	provider: ProviderId;
	model: string;
	output: ImagesOutputContent[];
	responseId?: string;
	usage?: Usage;
	stopReason: ImagesStopReason;
	errorMessage?: string;
	timestamp: number; // Unix timestamp in milliseconds
}

```
```
// packages/coding-agent/src/core/cache-stats.ts:82
		usage.cacheRead > 0
			? usage.cost.cacheRead / usage.cacheRead
			: (models.getModel(message.provider, message.model)?.cost.cacheRead ?? 0) / 1_000_000;
```
```
// packages/coding-agent/src/core/cache-stats.ts:82
		usage.cacheRead > 0
			? usage.cost.cacheRead / usage.cacheRead
			: (models.getModel(message.provider, message.model)?.cost.cacheRead ?? 0) / 1_000_000;
```
```
// packages/coding-agent/src/core/cache-stats.ts:87
		missedTokens,
		missedCost: missedTokens * Math.max(0, paidPerToken - readPerToken),
		idleMs: Math.max(0, message.timestamp - prev.timestamp),
```
```
// packages/coding-agent/src/core/cache-stats.ts:61
	models: ModelPriceSource,
): CacheMiss | undefined {
	const usage = message.usage;
```
Judgment (TRUE / NOT TRUE, one sentence):

### microsoft__playwright 24: PageScreencastFrameEvent (packages/playwright-core/src/server/channels.d.ts) read in packages/dashboard/src/dashboardModel.ts
- fields claimed: viewportHeight, viewportWidth
```
// packages/playwright-core/src/server/channels.d.ts:4082
export type PageScreencastFrameEvent = {
  frameId: number,
  data: Binary,
  timestamp: number,
  viewportWidth: number,
  viewportHeight: number,
};
export type PageWebSocketRouteEvent = {
  webSocketRoute: WebSocketRouteChannel,
};
export type PageWebSocketEvent = {
  webSocket: WebSocketChannel,
```
```
// packages/dashboard/src/dashboardModel.ts:334
        url: frame.url,
        viewportWidth: frame.viewportWidth,
        viewportHeight: frame.viewportHeight,
```
```
// packages/dashboard/src/dashboardModel.ts:333
        title: frame.title,
        url: frame.url,
        viewportWidth: frame.viewportWidth,
```
Judgment (TRUE / NOT TRUE, one sentence):

### puppeteer__puppeteer 25: BidiBrowserOptions (packages/puppeteer-core/src/bidi/Browser.ts) read in test/src/launcher.test.ts
- fields claimed: connection, process
```
// packages/puppeteer-core/src/bidi/Browser.ts:46
export interface BidiBrowserOptions {
  process?: ChildProcess;
  closeCallback?: BrowserCloseCallback;
  connection: BidiConnection;
  cdpConnection?: CdpConnection;
  defaultViewport: Viewport | null;
  acceptInsecureCerts?: boolean;
  capabilities?: SupportedWebDriverCapabilities;
  networkEnabled: boolean;
  issuesEnabled: boolean;
  logger: Logger;
}
```
```
// test/src/launcher.test.ts:124
        try {
          const browserProcess = browser.process()!;
          const connection = (browser as BidiBrowser).connection;
```
```
// test/src/launcher.test.ts:123
        const {browser, close} = await launch({});
        try {
          const browserProcess = browser.process()!;
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 26: FauxModelDefinition (packages/ai/src/providers/faux.ts) read in packages/coding-agent/test/agent-session-stats.test.ts
- fields claimed: contextWindow, cost, input
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
// packages/coding-agent/test/agent-session-stats.test.ts:112
			expect(stats.contextUsage).toEqual(session.getContextUsage());
			expect(stats.contextUsage?.tokens).toBe(200);
			expect(stats.contextUsage?.contextWindow).toBe(model.contextWindow);
```
```
// packages/coding-agent/test/agent-session-stats.test.ts:182
			const stats = session.getSessionStats();
			expect(stats.tokens).toEqual({ input: 10, output: 20, cacheRead: 30, cacheWrite: 40, total: 100 });
			expect(stats.cost).toBe(1);
```
```
// packages/coding-agent/test/agent-session-stats.test.ts:133
			const stats = session.getSessionStats();
			// Totals cover ALL entries, including history compacted away (180k + 195k).
			expect(stats.tokens.input).toBe(375_000);
```
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 27: ImageCrop (packages/element/src/types.ts) read in packages/excalidraw/components/Stats/Position.tsx
- fields claimed: height, naturalHeight, naturalWidth, width, x, y
```
// packages/element/src/types.ts:152
export type ImageCrop = {
  x: number;
  y: number;
  width: number;
  height: number;
  naturalWidth: number;
  naturalHeight: number;
};

export type ExcalidrawImageElement = _ExcalidrawElementBase &
  Readonly<{
    type: "image";
```
```
// packages/excalidraw/components/Stats/Position.tsx:46
  const [cx, cy] = [
    origElement.x + origElement.width / 2,
    origElement.y + origElement.height / 2,
```
```
// packages/excalidraw/components/Stats/Position.tsx:98
          ...crop,
          y: clamp(
            nextValue * (crop.naturalHeight / uncroppedHeight),
```
```
// packages/excalidraw/components/Stats/Position.tsx:71
      if (property === "x") {
        const nextValueInNatural =
          nextValue * (crop.naturalWidth / uncroppedWidth);
```
```
// packages/excalidraw/components/Stats/Position.tsx:45
  const origElement = originalElements[0];
  const [cx, cy] = [
    origElement.x + origElement.width / 2,
```
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 28: RenderPositionOffset (packages/element/src/renderElement.ts) read in packages/excalidraw/components/SearchMenu.tsx
- fields claimed: x, y
```
// packages/element/src/renderElement.ts:118
export type RenderPositionOffset = Readonly<{ x: number; y: number }>;

const ZERO_RENDER_OFFSET: RenderPositionOffset = { x: 0, y: 0 };

/** Bound labels follow their container; a label's own offset is ignored. */
export const getElementRenderOffset = (
  element: ExcalidrawElement,
  elementsMap: ElementsMap,
  overrides: ElementRenderOverrides | undefined,
): RenderPositionOffset | undefined => {
  const container = isTextElement(element)
    ? getContainerElement(element, elementsMap)
```
```
// packages/excalidraw/components/SearchMenu.tsx:203
        const matchAsElement = newTextElement({
          text: match.searchQuery,
          x: match.element.x + (match.matchedLines[0]?.offsetX ?? 0),
```
```
// packages/excalidraw/components/SearchMenu.tsx:204
          text: match.searchQuery,
          x: match.element.x + (match.matchedLines[0]?.offsetX ?? 0),
          y: match.element.y + (match.matchedLines[0]?.offsetY ?? 0),
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 29: AiGatewayPricing (packages/ai/scripts/ai-gateway-pricing.ts) read in packages/durable/test/harness-compaction.test.ts
- fields claimed: input, output
```
// packages/ai/scripts/ai-gateway-pricing.ts:11
export interface AiGatewayPricing {
	input?: string | number;
	output?: string | number;
	input_cache_read?: string | number;
	input_cache_write?: string | number;
	input_tiers?: AiGatewayPriceTier[];
	output_tiers?: AiGatewayPriceTier[];
	input_cache_read_tiers?: AiGatewayPriceTier[];
	input_cache_write_tiers?: AiGatewayPriceTier[];
}

const RATE_FIELDS = [
```
```
// packages/durable/test/harness-compaction.test.ts:419
		// The summarizer's spend is in the ledger, and nothing counts it again later.
		const usage = (await chat.harness.snapshot(UsageDoc, chat.root.id, context))!.models["faux/faux-1"]!;
		expect(usage.input).toBeGreaterThan(usageBefore.input);
```
```
// packages/durable/test/harness-compaction.test.ts:1725
		const usage = (await chat.harness.snapshot(UsageDoc, chat.root.id, context))!.models["faux/faux-1"]!;
		expect(usage.input).toBeGreaterThan(usageBefore.input);
		expect(usage.output - usageBefore.output).toBe(2);
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 30: ToolStateChanges (packages/ai/src/utils/transcript.ts) read in packages/agent/test/agent.test.ts
- fields claimed: toolsAdded, toolsRemoved
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
// packages/agent/test/agent.test.ts:160
		if (initial?.role !== "system") throw new Error("expected initial system message");
		expect(initial.content).toBe("You are helpful.");
		expect(initial.toolsAdded?.map((value) => value.name)).toEqual(["echo"]);
```
```
// packages/agent/test/agent.test.ts:175
							? [
									`+${(message.toolsAdded ?? []).map((tool) => tool.name).join(",")}`,
									`-${(message.toolsRemoved ?? []).map((tool) => tool.name).join(",")}`,
```
Judgment (TRUE / NOT TRUE, one sentence):

