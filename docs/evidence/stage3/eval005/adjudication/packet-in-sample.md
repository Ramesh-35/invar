### Egonex-AI__Understand-Anything 1: code:types.ts->DomainGraphView.tsx
- E2 GraphNode declared at understand-anything-plugin/packages/core/src/types.ts:54
- E2 KnowledgeGraph declared at understand-anything-plugin/packages/core/src/types.ts:107
- E4 GraphNode imported at understand-anything-plugin/packages/dashboard/src/components/DomainGraphView.tsx:23 from "@understand-anything/core/types", resolved to understand-anything-plugin/packages/core/src/types.ts
- E4 KnowledgeGraph imported at understand-anything-plugin/packages/dashboard/src/components/DomainGraphView.tsx:23 from "@understand-anything/core/types", resolved to understand-anything-plugin/packages/core/src/types.ts
- E5 GraphNode fields read: domainMeta (understand-anything-plugin/packages/dashboard/src/components/DomainGraphView.tsx:32), filePath (understand-anything-plugin/packages/dashboard/src/components/DomainGraphView.tsx:148), summary (understand-anything-plugin/packages/dashboard/src/components/DomainGraphView.tsx:57)
- E5 GraphNode stoplisted reads: id, name
- E5 KnowledgeGraph fields read: edges (understand-anything-plugin/packages/dashboard/src/components/DomainGraphView.tsx:47), nodes (understand-anything-plugin/packages/dashboard/src/components/DomainGraphView.tsx:43)
- E5 KnowledgeGraph stoplisted reads: (none)
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
```
```
// understand-anything-plugin/packages/core/src/types.ts:107
export interface KnowledgeGraph {
  version: string;
  kind?: "codebase" | "knowledge" | "design";
  project: ProjectMeta;
  nodes: GraphNode[];
  edges: GraphEdge[];
  layers: Layer[];
  tour: TourStep[];
}

// Theme configuration (for dashboard customization)
export interface ThemeConfig {
```
```
// understand-anything-plugin/packages/dashboard/src/components/DomainGraphView.tsx:32
function getDomainMeta(node: GraphNode) {
  return node.domainMeta;
}
```
```
// understand-anything-plugin/packages/dashboard/src/components/DomainGraphView.tsx:148
        summary: node.summary,
        filePath: node.filePath,
        stepId: node.id,
```
```
// understand-anything-plugin/packages/dashboard/src/components/DomainGraphView.tsx:57
      label: node.name,
      summary: node.summary,
      entities: meta?.entities as string[] | undefined,
```
```
// understand-anything-plugin/packages/dashboard/src/components/DomainGraphView.tsx:47
  const flowCountMap = new Map<string, number>();
  for (const edge of graph.edges) {
    if (edge.type === "contains_flow") {
```
Judgment (TRUE / NOT TRUE, one sentence):

### Egonex-AI__Understand-Anything 2: code:types.ts->GraphView.tsx
- E2 GraphNode declared at understand-anything-plugin/packages/core/src/types.ts:54
- E4 GraphNode imported at understand-anything-plugin/packages/dashboard/src/components/GraphView.tsx:27 from "@understand-anything/core/types", resolved to understand-anything-plugin/packages/core/src/types.ts
- E5 GraphNode fields read: complexity (understand-anything-plugin/packages/dashboard/src/components/GraphView.tsx:570), filePath (understand-anything-plugin/packages/dashboard/src/components/GraphView.tsx:567), summary (understand-anything-plugin/packages/dashboard/src/components/GraphView.tsx:569), tags (understand-anything-plugin/packages/dashboard/src/components/GraphView.tsx:571)
- E5 GraphNode stoplisted reads: id, name, type
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
```
```
// understand-anything-plugin/packages/dashboard/src/components/GraphView.tsx:570
          summary: node.summary,
          complexity: node.complexity,
          tags: node.tags,
```
```
// understand-anything-plugin/packages/dashboard/src/components/GraphView.tsx:567
        data: {
          label: node.name ?? node.filePath?.split("/").pop() ?? node.id,
          nodeType: node.type,
```
```
// understand-anything-plugin/packages/dashboard/src/components/GraphView.tsx:569
          nodeType: node.type,
          summary: node.summary,
          complexity: node.complexity,
```
```
// understand-anything-plugin/packages/dashboard/src/components/GraphView.tsx:571
          complexity: node.complexity,
          tags: node.tags,
          isHighlighted: false,
```
Judgment (TRUE / NOT TRUE, one sentence):

### Egonex-AI__Understand-Anything 3: code:types.ts->KnowledgeGraphView.tsx
- E2 KnowledgeGraph declared at understand-anything-plugin/packages/core/src/types.ts:107
- E4 KnowledgeGraph imported at understand-anything-plugin/packages/dashboard/src/components/KnowledgeGraphView.tsx:28 from "@understand-anything/core/types", resolved to understand-anything-plugin/packages/core/src/types.ts
- E5 KnowledgeGraph fields read: edges (understand-anything-plugin/packages/dashboard/src/components/KnowledgeGraphView.tsx:85), layers (understand-anything-plugin/packages/dashboard/src/components/KnowledgeGraphView.tsx:91), nodes (understand-anything-plugin/packages/dashboard/src/components/KnowledgeGraphView.tsx:100)
- E5 KnowledgeGraph stoplisted reads: (none)
```
// understand-anything-plugin/packages/core/src/types.ts:107
export interface KnowledgeGraph {
  version: string;
  kind?: "codebase" | "knowledge" | "design";
  project: ProjectMeta;
  nodes: GraphNode[];
  edges: GraphEdge[];
  layers: Layer[];
  tour: TourStep[];
}

// Theme configuration (for dashboard customization)
export interface ThemeConfig {
```
```
// understand-anything-plugin/packages/dashboard/src/components/KnowledgeGraphView.tsx:85
  const edgeCounts = new Map<string, number>();
  for (const edge of graph.edges) {
    edgeCounts.set(edge.source, (edgeCounts.get(edge.source) ?? 0) + 1);
```
```
// understand-anything-plugin/packages/dashboard/src/components/KnowledgeGraphView.tsx:91
  const communityMap = new Map<string, number>();
  graph.layers.forEach((layer, i) => {
    for (const nodeId of layer.nodeIds) {
```
```
// understand-anything-plugin/packages/dashboard/src/components/KnowledgeGraphView.tsx:100
    edgeCounts,
    nodes: graph.nodes.map((node) => ({
      id: node.id,
```
Judgment (TRUE / NOT TRUE, one sentence):

### Egonex-AI__Understand-Anything 4: code:types.ts->NodeInfo.tsx
- E2 GraphNode declared at understand-anything-plugin/packages/core/src/types.ts:54
- E2 KnowledgeGraph declared at understand-anything-plugin/packages/core/src/types.ts:107
- E4 GraphNode imported at understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:4 from "@understand-anything/core/types", resolved to understand-anything-plugin/packages/core/src/types.ts
- E4 KnowledgeGraph imported at understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:4 from "@understand-anything/core/types", resolved to understand-anything-plugin/packages/core/src/types.ts
- E5 GraphNode fields read: complexity (understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:332), domainMeta (understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:161), figmaMeta (understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:56), filePath (understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:261), knowledgeMeta (understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:68), languageNotes (understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:433), lineRange (understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:268), summary (understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:405), tags (understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:459)
- E5 GraphNode stoplisted reads: id, name, type
- E5 KnowledgeGraph fields read: edges (understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:71), nodes (understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:73)
- E5 KnowledgeGraph stoplisted reads: (none)
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
```
```
// understand-anything-plugin/packages/core/src/types.ts:107
export interface KnowledgeGraph {
  version: string;
  kind?: "codebase" | "knowledge" | "design";
  project: ProjectMeta;
  nodes: GraphNode[];
  edges: GraphEdge[];
  layers: Layer[];
  tour: TourStep[];
}

// Theme configuration (for dashboard customization)
export interface ThemeConfig {
```
```
// understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:332
  const complexityBadge =
    complexityBadgeColors[node.complexity] ?? complexityBadgeColors.simple;

```
```
// understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:161
  const { t } = useI18n();
  const meta = node.domainMeta;

```
```
// understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:56
function FigmaThumbnail({ node }: { node: GraphNode }) {
  const url = node.figmaMeta?.thumbnailUrl;
  if (!url) return null;
```
```
// understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:261
  if (node.type === "step") {
    if (!node.filePath) return null;
    return (
```
Judgment (TRUE / NOT TRUE, one sentence):

### Egonex-AI__Understand-Anything 5: code:types.ts->store.ts
- E2 KnowledgeGraph declared at understand-anything-plugin/packages/core/src/types.ts:107
- E4 KnowledgeGraph imported at understand-anything-plugin/packages/dashboard/src/store.ts:5 from "@understand-anything/core/types", resolved to understand-anything-plugin/packages/core/src/types.ts
- E5 KnowledgeGraph fields read: layers (understand-anything-plugin/packages/dashboard/src/store.ts:84), nodes (understand-anything-plugin/packages/dashboard/src/store.ts:81), tour (understand-anything-plugin/packages/dashboard/src/store.ts:244)
- E5 KnowledgeGraph stoplisted reads: (none)
```
// understand-anything-plugin/packages/core/src/types.ts:107
export interface KnowledgeGraph {
  version: string;
  kind?: "codebase" | "knowledge" | "design";
  project: ProjectMeta;
  nodes: GraphNode[];
  edges: GraphEdge[];
  layers: Layer[];
  tour: TourStep[];
}

// Theme configuration (for dashboard customization)
export interface ThemeConfig {
```
```
// understand-anything-plugin/packages/dashboard/src/store.ts:84
  const nodeIdToLayerIds = new Map<string, Set<string>>();
  for (const layer of graph.layers) {
    for (const nid of layer.nodeIds) {
```
```
// understand-anything-plugin/packages/dashboard/src/store.ts:81
  const nodesById = new Map<string, GraphNode>();
  for (const node of graph.nodes) nodesById.set(node.id, node);
  const nodeIdToLayerId = new Map<string, string>();
```
```
// understand-anything-plugin/packages/dashboard/src/store.ts:244
function getSortedTour(graph: KnowledgeGraph): TourStep[] {
  const tour = graph.tour ?? [];
  return [...tour].sort((a, b) => a.order - b.order);
```
Judgment (TRUE / NOT TRUE, one sentence):

### Egonex-AI__Understand-Anything 6: code:types.ts->edgeAggregation.ts
- E2 GraphEdge declared at understand-anything-plugin/packages/core/src/types.ts:70
- E2 KnowledgeGraph declared at understand-anything-plugin/packages/core/src/types.ts:107
- E4 GraphEdge imported at understand-anything-plugin/packages/dashboard/src/utils/edgeAggregation.ts:1 from "@understand-anything/core/types", resolved to understand-anything-plugin/packages/core/src/types.ts
- E4 KnowledgeGraph imported at understand-anything-plugin/packages/dashboard/src/utils/edgeAggregation.ts:1 from "@understand-anything/core/types", resolved to understand-anything-plugin/packages/core/src/types.ts
- E5 GraphEdge fields read: source (understand-anything-plugin/packages/dashboard/src/utils/edgeAggregation.ts:174), target (understand-anything-plugin/packages/dashboard/src/utils/edgeAggregation.ts:175)
- E5 GraphEdge stoplisted reads: type
- E5 KnowledgeGraph fields read: edges (understand-anything-plugin/packages/dashboard/src/utils/edgeAggregation.ts:37), layers (understand-anything-plugin/packages/dashboard/src/utils/edgeAggregation.ts:25)
- E5 KnowledgeGraph stoplisted reads: (none)
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
```
```
// understand-anything-plugin/packages/core/src/types.ts:107
export interface KnowledgeGraph {
  version: string;
  kind?: "codebase" | "knowledge" | "design";
  project: ProjectMeta;
  nodes: GraphNode[];
  edges: GraphEdge[];
  layers: Layer[];
  tour: TourStep[];
}

// Theme configuration (for dashboard customization)
export interface ThemeConfig {
```
```
// understand-anything-plugin/packages/dashboard/src/utils/edgeAggregation.ts:174
  for (const e of edges) {
    const sc = nodeToContainer.get(e.source);
    const tc = nodeToContainer.get(e.target);
```
```
// understand-anything-plugin/packages/dashboard/src/utils/edgeAggregation.ts:175
    const sc = nodeToContainer.get(e.source);
    const tc = nodeToContainer.get(e.target);
    if (!sc || !tc) continue;
```
```
// understand-anything-plugin/packages/dashboard/src/utils/edgeAggregation.ts:37

  for (const edge of graph.edges) {
    const sourceLayer = nodeToLayer.get(edge.source);
```
```
// understand-anything-plugin/packages/dashboard/src/utils/edgeAggregation.ts:25
  const nodeToLayer = new Map<string, string>();
  for (const layer of graph.layers) {
    for (const nodeId of layer.nodeIds) {
```
Judgment (TRUE / NOT TRUE, one sentence):

### Egonex-AI__Understand-Anything 7: code:types.ts->filters.ts
- E2 GraphEdge declared at understand-anything-plugin/packages/core/src/types.ts:70
- E4 GraphEdge imported at understand-anything-plugin/packages/dashboard/src/utils/filters.ts:1 from "@understand-anything/core/types", resolved to understand-anything-plugin/packages/core/src/types.ts
- E5 GraphEdge fields read: source (understand-anything-plugin/packages/dashboard/src/utils/filters.ts:64), target (understand-anything-plugin/packages/dashboard/src/utils/filters.ts:64)
- E5 GraphEdge stoplisted reads: type
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
```
```
// understand-anything-plugin/packages/dashboard/src/utils/filters.ts:64
    // Only keep edges between visible nodes
    if (!visibleNodeIds.has(edge.source) || !visibleNodeIds.has(edge.target)) {
      return false;
```
Judgment (TRUE / NOT TRUE, one sentence):

### Egonex-AI__Understand-Anything 8: code:types.ts->louvain.ts
- E2 GraphEdge declared at understand-anything-plugin/packages/core/src/types.ts:70
- E4 GraphEdge imported at understand-anything-plugin/packages/dashboard/src/utils/louvain.ts:3 from "@understand-anything/core/types", resolved to understand-anything-plugin/packages/core/src/types.ts
- E5 GraphEdge fields read: source (understand-anything-plugin/packages/dashboard/src/utils/louvain.ts:25), target (understand-anything-plugin/packages/dashboard/src/utils/louvain.ts:25)
- E5 GraphEdge stoplisted reads: (none)
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
```
```
// understand-anything-plugin/packages/dashboard/src/utils/louvain.ts:25
  for (const e of edges) {
    if (!ids.has(e.source) || !ids.has(e.target)) continue;
    if (e.source === e.target) continue;
```
Judgment (TRUE / NOT TRUE, one sentence):

### Egonex-AI__Understand-Anything 9: code:types.ts->context-builder.ts
- E2 KnowledgeGraph declared at understand-anything-plugin/packages/core/src/types.ts:107
- E4 KnowledgeGraph imported at understand-anything-plugin/src/context-builder.ts:2 from "@understand-anything/core", resolved to understand-anything-plugin/packages/core/src/index.ts, re-exported by barrel understand-anything-plugin/packages/core/src/index.ts:1 from understand-anything-plugin/packages/core/src/types.ts
- E5 KnowledgeGraph fields read: edges (understand-anything-plugin/src/context-builder.ts:41), layers (understand-anything-plugin/src/context-builder.ts:66), nodes (understand-anything-plugin/src/context-builder.ts:33), project (understand-anything-plugin/src/context-builder.ts:71)
- E5 KnowledgeGraph stoplisted reads: (none)
```
// understand-anything-plugin/packages/core/src/types.ts:107
export interface KnowledgeGraph {
  version: string;
  kind?: "codebase" | "knowledge" | "design";
  project: ProjectMeta;
  nodes: GraphNode[];
  edges: GraphEdge[];
  layers: Layer[];
  tour: TourStep[];
}

// Theme configuration (for dashboard customization)
export interface ThemeConfig {
```
```
// understand-anything-plugin/src/context-builder.ts:41
  const expandedIds = new Set(matchedIds);
  for (const edge of graph.edges) {
    if (matchedIds.has(edge.source)) {
```
```
// understand-anything-plugin/src/context-builder.ts:66
  // 4. Find layers containing any relevant node
  const relevantLayers = graph.layers.filter((layer) =>
    layer.nodeIds.some((id) => expandedIds.has(id)),
```
```
// understand-anything-plugin/src/context-builder.ts:33
  // 1. Use SearchEngine to find relevant nodes
  const engine = new SearchEngine(graph.nodes);
  const searchResults = engine.search(query, { limit });
```
```
// understand-anything-plugin/src/context-builder.ts:71
  return {
    projectName: graph.project.name,
    projectDescription: graph.project.description,
```
Judgment (TRUE / NOT TRUE, one sentence):

### Egonex-AI__Understand-Anything 10: code:types.ts->explain-builder.ts
- E2 GraphEdge declared at understand-anything-plugin/packages/core/src/types.ts:70
- E2 GraphNode declared at understand-anything-plugin/packages/core/src/types.ts:54
- E4 GraphEdge imported at understand-anything-plugin/src/explain-builder.ts:1 from "@understand-anything/core", resolved to understand-anything-plugin/packages/core/src/index.ts, re-exported by barrel understand-anything-plugin/packages/core/src/index.ts:1 from understand-anything-plugin/packages/core/src/types.ts
- E4 GraphNode imported at understand-anything-plugin/src/explain-builder.ts:1 from "@understand-anything/core", resolved to understand-anything-plugin/packages/core/src/index.ts, re-exported by barrel understand-anything-plugin/packages/core/src/index.ts:1 from understand-anything-plugin/packages/core/src/types.ts
- E5 GraphEdge fields read: source (understand-anything-plugin/src/explain-builder.ts:78), target (understand-anything-plugin/src/explain-builder.ts:78)
- E5 GraphEdge stoplisted reads: description, type
- E5 GraphNode fields read: filePath (understand-anything-plugin/src/explain-builder.ts:130), languageNotes (understand-anything-plugin/src/explain-builder.ts:180), lineRange (understand-anything-plugin/src/explain-builder.ts:132)
- E5 GraphNode stoplisted reads: id
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
```
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
```
```
// understand-anything-plugin/src/explain-builder.ts:78
  for (const edge of edges) {
    if (allRelatedIds.has(edge.source) || allRelatedIds.has(edge.target)) {
      relevantEdges.push(edge);
```
```
// understand-anything-plugin/src/explain-builder.ts:130
  );
  if (targetNode.filePath)
    lines.push(`**File:** \`${targetNode.filePath}\``);
```
```
// understand-anything-plugin/src/explain-builder.ts:180

  if (targetNode.languageNotes) {
    lines.push("## Language Notes");
```
```
// understand-anything-plugin/src/explain-builder.ts:132
    lines.push(`**File:** \`${targetNode.filePath}\``);
  if (targetNode.lineRange)
    lines.push(
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 1: code:types.ts->index.ts
- E2 AgentToolResult declared at packages/agent/src/types.ts:424
- E4 AgentToolResult imported at packages/coding-agent/examples/extensions/subagent/index.ts:19 from "@earendil-works/pi-agent-core", resolved to packages/agent/src/index.ts, re-exported by barrel packages/agent/src/index.ts:5 from packages/agent/src/types.ts
- E5 AgentToolResult fields read: content (packages/coding-agent/examples/extensions/subagent/index.ts:564), details (packages/coding-agent/examples/extensions/subagent/index.ts:560)
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
// packages/coding-agent/examples/extensions/subagent/index.ts:564
									onUpdate({
										content: partial.content,
										details: makeDetails("chain")(allResults),
```
```
// packages/coding-agent/examples/extensions/subagent/index.ts:560
								// Combine completed results with current streaming result
								const currentResult = partial.details?.results[0];
								if (currentResult) {
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 2: code:types.ts->nested-tool-calls.ts
- E2 AgentToolCallOutcome declared at packages/agent/src/types.ts:449
- E4 AgentToolCallOutcome imported at packages/coding-agent/src/core/nested-tool-calls.ts:11 from "@earendil-works/pi-agent-core", resolved to packages/agent/src/index.ts, re-exported by barrel packages/agent/src/index.ts:5 from packages/agent/src/types.ts
- E5 AgentToolCallOutcome fields read: durationMs (packages/coding-agent/src/core/nested-tool-calls.ts:246), isError (packages/coding-agent/src/core/nested-tool-calls.ts:237)
- E5 AgentToolCallOutcome stoplisted reads: result
```
// packages/agent/src/types.ts:449
export interface AgentToolCallOutcome {
	toolCall: AgentToolCall;
	result: AgentToolResult<any>;
	isError: boolean;
	/** Milliseconds `execute()` took, measured with a monotonic clock; absent when the tool did not run. */
	durationMs?: number;
}

/**
 * Callback used by tools to stream partial execution updates.
 *
 * The callback is scoped to the current `execute()` invocation. Calls made after
```
```
// packages/coding-agent/src/core/nested-tool-calls.ts:246
			isError: outcome.isError,
			...(outcome.durationMs === undefined ? {} : { durationMs: outcome.durationMs }),
			parentToolCallId: callerId,
```
```
// packages/coding-agent/src/core/nested-tool-calls.ts:237

		scope.recorder.finish(record, outcome.isError, textOf(outcome.result));
		// Nested results are not persisted, so their usage is only counted through the recorder.
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 3: code:types.ts->tool-definition-wrapper.ts
- E2 AgentTool declared at packages/agent/src/types.ts:466
- E4 AgentTool imported at packages/coding-agent/src/core/tools/tool-definition-wrapper.ts:1 from "@earendil-works/pi-agent-core", resolved to packages/agent/src/index.ts, re-exported by barrel packages/agent/src/index.ts:5 from packages/agent/src/types.ts
- E5 AgentTool fields read: execute (packages/coding-agent/src/core/tools/tool-definition-wrapper.ts:56), executionMode (packages/coding-agent/src/core/tools/tool-definition-wrapper.ts:55), outputSchema (packages/coding-agent/src/core/tools/tool-definition-wrapper.ts:52), prepareArguments (packages/coding-agent/src/core/tools/tool-definition-wrapper.ts:54)
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
// packages/coding-agent/src/core/tools/tool-definition-wrapper.ts:56
		executionMode: tool.executionMode,
		execute: async (toolCallId, params, signal, onUpdate) => tool.execute(toolCallId, params, signal, onUpdate),
	};
```
```
// packages/coding-agent/src/core/tools/tool-definition-wrapper.ts:55
		prepareArguments: tool.prepareArguments,
		executionMode: tool.executionMode,
		execute: async (toolCallId, params, signal, onUpdate) => tool.execute(toolCallId, params, signal, onUpdate),
```
```
// packages/coding-agent/src/core/tools/tool-definition-wrapper.ts:52
		parameters: tool.parameters as any,
		outputSchema: tool.outputSchema,
		constrainedSampling: tool.constrainedSampling,
```
```
// packages/coding-agent/src/core/tools/tool-definition-wrapper.ts:54
		constrainedSampling: tool.constrainedSampling,
		prepareArguments: tool.prepareArguments,
		executionMode: tool.executionMode,
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 4: code:types.ts->execute.ts
- E2 AgentToolCallOutcome declared at packages/agent/src/types.ts:449
- E2 AgentToolResult declared at packages/agent/src/types.ts:424
- E4 AgentToolCallOutcome imported at packages/coding-agent/src/extensions/codemode/execute.ts:6 from "@earendil-works/pi-agent-core", resolved to packages/agent/src/index.ts, re-exported by barrel packages/agent/src/index.ts:5 from packages/agent/src/types.ts
- E4 AgentToolResult imported at packages/coding-agent/src/extensions/codemode/execute.ts:6 from "@earendil-works/pi-agent-core", resolved to packages/agent/src/index.ts, re-exported by barrel packages/agent/src/index.ts:5 from packages/agent/src/types.ts
- E5 AgentToolCallOutcome fields read: isError (packages/coding-agent/src/extensions/codemode/execute.ts:432), toolCall (packages/coding-agent/src/extensions/codemode/execute.ts:489)
- E5 AgentToolCallOutcome stoplisted reads: result
- E5 AgentToolResult fields read: content (packages/coding-agent/src/extensions/codemode/execute.ts:76), structuredContent (packages/coding-agent/src/extensions/codemode/execute.ts:430), usage (packages/coding-agent/src/extensions/codemode/execute.ts:727)
- E5 AgentToolResult stoplisted reads: (none)
```
// packages/agent/src/types.ts:449
export interface AgentToolCallOutcome {
	toolCall: AgentToolCall;
	result: AgentToolResult<any>;
	isError: boolean;
	/** Milliseconds `execute()` took, measured with a monotonic clock; absent when the tool did not run. */
	durationMs?: number;
}

/**
 * Callback used by tools to stream partial execution updates.
 *
 * The callback is scoped to the current `execute()` invocation. Calls made after
```
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
// packages/coding-agent/src/extensions/codemode/execute.ts:432
	const text = textOf(result);
	if (outcome.isError) throw new Error(text || `Tool "${tool.name}" failed`);
	return text;
```
```
// packages/coding-agent/src/extensions/codemode/execute.ts:489
			const outcome = await ctx.executeTool(tool.name, args, { signal: callSignal });
			record.id = outcome.toolCall.id;
			record.durationMs = performance.now() - callStartedAt;
```
```
// packages/coding-agent/src/extensions/codemode/execute.ts:76
function textOf(result: AgentToolResult<unknown>): string {
	return (result.content ?? [])
		.filter((block): block is TextContent => block.type === "text")
```
```
// packages/coding-agent/src/extensions/codemode/execute.ts:430
	const { result } = outcome;
	if (tool.outputSchema && result.structuredContent !== undefined) return result.structuredContent;
	const text = textOf(result);
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 5: code:types.ts->agent-session.ts
- E2 AuthResult declared at packages/ai/src/auth/types.ts:104
- E4 AuthResult imported at packages/coding-agent/src/core/agent-session.ts:35 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:24 from packages/ai/src/auth/types.ts
- E5 AuthResult fields read: auth (packages/coding-agent/src/core/agent-session.ts:552), env (packages/coding-agent/src/core/agent-session.ts:558)
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
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 6: code:types.ts->model-runtime.ts
- E2 AuthResult declared at packages/ai/src/auth/types.ts:104
- E4 AuthResult imported at packages/coding-agent/src/core/model-runtime.ts:2 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:24 from packages/ai/src/auth/types.ts
- E5 AuthResult fields read: auth (packages/coding-agent/src/core/model-runtime.ts:574), env (packages/coding-agent/src/core/model-runtime.ts:568)
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
// packages/coding-agent/src/core/model-runtime.ts:574
				...resolution.auth,
				headers: mergeHeaders(resolution.auth.headers, configuredHeaders),
			},
```
```
// packages/coding-agent/src/core/model-runtime.ts:568
			this.extensionProviders.get(providerOrModel.provider),
			{ ...(resolution.env ?? {}), ...(overrides.env ?? {}) },
		);
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 7: code:types.ts->provider-composer.ts
- E2 AuthResult declared at packages/ai/src/auth/types.ts:104
- E2 ModelAuth declared at packages/ai/src/auth/types.ts:7
- E4 AuthResult imported at packages/coding-agent/src/core/provider-composer.ts:1 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:24 from packages/ai/src/auth/types.ts
- E4 ModelAuth imported at packages/coding-agent/src/core/provider-composer.ts:1 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:24 from packages/ai/src/auth/types.ts
- E5 AuthResult fields read: auth (packages/coding-agent/src/core/provider-composer.ts:478), env (packages/coding-agent/src/core/provider-composer.ts:475)
- E5 AuthResult stoplisted reads: (none)
- E5 ModelAuth fields read: apiKey (packages/coding-agent/src/core/provider-composer.ts:382), headers (packages/coding-agent/src/core/provider-composer.ts:380)
- E5 ModelAuth stoplisted reads: (none)
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
// packages/coding-agent/src/core/provider-composer.ts:478
			const headers = resolveHeadersOrThrow(rawHeaders, `provider "${providerId}"`, headerEnv);
			return { ...result, auth: withConfiguredAuth(result.auth, headers, authHeader) };
		},
```
```
// packages/coding-agent/src/core/provider-composer.ts:475
			if (!result) return undefined;
			const explicitEnv = { ...(input.credential?.env ?? {}), ...(result.env ?? {}) };
			const headerEnv = await configContextEnv(Object.values(rawHeaders ?? {}), input.ctx, explicitEnv);
```
```
// packages/coding-agent/src/core/provider-composer.ts:382
	if (authHeader) {
		if (!auth.apiKey) throw new Error("authHeader requires a resolved API key");
		mergedHeaders = { ...mergedHeaders, Authorization: `Bearer ${auth.apiKey}` };
```
```
// packages/coding-agent/src/core/provider-composer.ts:380
	let mergedHeaders: ProviderHeaders | undefined =
		auth.headers || headers ? { ...auth.headers, ...headers } : undefined;
	if (authHeader) {
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 8: code:models-store.ts->remote-catalog-provider.ts
- E2 ModelsStoreEntry declared at packages/ai/src/models-store.ts:3
- E4 ModelsStoreEntry imported at packages/coding-agent/src/core/remote-catalog-provider.ts:1 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:34 from packages/ai/src/models-store.ts
- E5 ModelsStoreEntry fields read: lastModified (packages/coding-agent/src/core/remote-catalog-provider.ts:72), models (packages/coding-agent/src/core/remote-catalog-provider.ts:72)
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
// packages/coding-agent/src/core/remote-catalog-provider.ts:72
function remoteModels(entry: ModelsStoreEntry | undefined, localGeneratedAt: number | undefined): readonly AnyModel[] {
	return entry && isNewer(entry.lastModified, localGeneratedAt) ? entry.models : [];
}
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 9: code:models.ts->bug-report.ts
- E2 Provider declared at packages/ai/src/models.ts:155
- E4 Provider imported at packages/coding-agent/src/core/bug-report.ts:4 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:33 from packages/ai/src/models.ts
- E5 Provider fields read: auth (packages/coding-agent/src/core/bug-report.ts:113), baseUrl (packages/coding-agent/src/core/bug-report.ts:118), headers (packages/coding-agent/src/core/bug-report.ts:119)
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
// packages/coding-agent/src/core/bug-report.ts:118
		name: provider.name,
		baseUrl: provider.baseUrl ? redactUrl(provider.baseUrl) : null,
		headerNames: Object.keys(provider.headers ?? {}).sort(),
```
```
// packages/coding-agent/src/core/bug-report.ts:119
		baseUrl: provider.baseUrl ? redactUrl(provider.baseUrl) : null,
		headerNames: Object.keys(provider.headers ?? {}).sort(),
		authTypes,
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 10: code:models.ts->model-runtime.ts
- E2 ModelsRefreshOptions declared at packages/ai/src/models.ts:103
- E2 ModelsRefreshResult declared at packages/ai/src/models.ts:112
- E4 ModelsRefreshOptions imported at packages/coding-agent/src/core/model-runtime.ts:2 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:33 from packages/ai/src/models.ts
- E4 ModelsRefreshResult imported at packages/coding-agent/src/core/model-runtime.ts:2 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:33 from packages/ai/src/models.ts
- E5 ModelsRefreshOptions fields read: allowNetwork (packages/coding-agent/src/core/model-runtime.ts:859), providers (packages/coding-agent/src/core/model-runtime.ts:851), signal (packages/coding-agent/src/core/model-runtime.ts:256)
- E5 ModelsRefreshOptions stoplisted reads: (none)
- E5 ModelsRefreshResult fields read: aborted (packages/coding-agent/src/core/model-runtime.ts:611), errors (packages/coding-agent/src/core/model-runtime.ts:612)
- E5 ModelsRefreshResult stoplisted reads: (none)
```
// packages/ai/src/models.ts:103
export interface ModelsRefreshOptions {
	allowNetwork?: boolean;
	/** Restrict refresh to these provider IDs. Unknown and static providers are ignored. */
	providers?: readonly string[];
	/** Bypass provider freshness checks and fetch immediately when network access is allowed. */
	force?: boolean;
	signal?: AbortSignal;
}

export interface ModelsRefreshResult {
	aborted: boolean;
	errors: ReadonlyMap<string, Error>;
```
```
// packages/ai/src/models.ts:112
export interface ModelsRefreshResult {
	aborted: boolean;
	errors: ReadonlyMap<string, Error>;
}

export interface ModelsRequestTransforms {
	/** Transform fully assembled model/auth/request headers before provider dispatch. */
	transformHeaders?: (headers: ProviderHeaders) => ProviderHeaders | Promise<ProviderHeaders>;
}

export type ModelsApiStreamOptions<TApi extends Api> = ApiStreamOptions<TApi> & ModelsRequestTransforms;
export type ModelsSimpleStreamOptions = SimpleStreamOptions & ModelsRequestTransforms;
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
// packages/coding-agent/src/core/model-runtime.ts:611
			const result = await this.models.refresh({ allowNetwork: false, providers: [providerId], signal });
			if (result.aborted) signal.throwIfAborted();
			const refreshError = result.errors.get(providerId);
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 11: code:models.ts->provider-composer.ts
- E2 Provider declared at packages/ai/src/models.ts:155
- E2 RefreshModelsContext declared at packages/ai/src/models.ts:80
- E4 Provider imported at packages/coding-agent/src/core/provider-composer.ts:1 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:33 from packages/ai/src/models.ts
- E4 RefreshModelsContext imported at packages/coding-agent/src/core/provider-composer.ts:1 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:33 from packages/ai/src/models.ts
- E5 Provider fields read: auth (packages/coding-agent/src/core/provider-composer.ts:423), baseUrl (packages/coding-agent/src/core/provider-composer.ts:608), headers (packages/coding-agent/src/core/provider-composer.ts:609)
- E5 Provider stoplisted reads: name
- E5 RefreshModelsContext fields read: credential (packages/coding-agent/src/core/provider-composer.ts:620), signal (packages/coding-agent/src/core/provider-composer.ts:619)
- E5 RefreshModelsContext stoplisted reads: (none)
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
// packages/coding-agent/src/core/provider-composer.ts:608
		name: extension?.name ?? config?.name ?? base?.name ?? extension?.oauth?.name ?? providerId,
		baseUrl: extension?.baseUrl ?? config?.baseUrl ?? base?.baseUrl,
		headers: base?.headers,
```
```
// packages/coding-agent/src/core/provider-composer.ts:609
		baseUrl: extension?.baseUrl ?? config?.baseUrl ?? base?.baseUrl,
		headers: base?.headers,
		auth: { ...(apiKey ? { apiKey } : {}), ...(oauth ? { oauth } : {}) },
```
```
// packages/coding-agent/src/core/provider-composer.ts:620
						if (context.signal.aborted) return;
						const oauthCredential = context.credential?.type === "oauth" ? context.credential : undefined;
						await context.publish({
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 12: code:models.ts->remote-catalog-provider.ts
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

### earendil-works__pi 13: code:models.ts->provider.ts
- E2 RefreshModelsContext declared at packages/ai/src/models.ts:80
- E4 RefreshModelsContext imported at packages/coding-agent/src/extensions/llama/provider.ts:1 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:33 from packages/ai/src/models.ts
- E5 RefreshModelsContext fields read: allowNetwork (packages/coding-agent/src/extensions/llama/provider.ts:255), credential (packages/coding-agent/src/extensions/llama/provider.ts:255), signal (packages/coding-agent/src/extensions/llama/provider.ts:255), stored (packages/coding-agent/src/extensions/llama/provider.ts:233)
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
// packages/coding-agent/src/extensions/llama/provider.ts:233
			const cachedContextWindows = new Map<string, number>();
			if (context.stored) {
				const stored = context.stored.models.filter((model) => model.provider === LLAMA_PROVIDER_ID);
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 14: code:models.ts->llama-extension.test.ts
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

### earendil-works__pi 15: code:models.ts->remote-catalog-provider.test.ts
- E2 ModelsPublication declared at packages/ai/src/models.ts:73
- E4 ModelsPublication imported at packages/coding-agent/test/remote-catalog-provider.test.ts:1 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:33 from packages/ai/src/models.ts
- E5 ModelsPublication fields read: persist (packages/coding-agent/test/remote-catalog-provider.test.ts:63), update (packages/coding-agent/test/remote-catalog-provider.test.ts:65)
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
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 16: code:faux.ts->harness.ts
- E2 FauxProviderRegistration declared at packages/ai/src/providers/faux.ts:134
- E4 FauxProviderRegistration imported at packages/coding-agent/test/suite/harness.ts:11 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:35 from packages/ai/src/providers/faux.ts
- E5 FauxProviderRegistration fields read: api (packages/coding-agent/test/suite/harness.ts:177), appendResponses (packages/coding-agent/test/suite/harness.ts:261), getPendingResponseCount (packages/coding-agent/test/suite/harness.ts:262), models (packages/coding-agent/test/suite/harness.ts:178), setResponses (packages/coding-agent/test/suite/harness.ts:155), unregister (packages/coding-agent/test/suite/harness.ts:270)
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
// packages/coding-agent/test/suite/harness.ts:177
			apiKey: "faux-key",
			api: fauxProvider.api,
			models: fauxProvider.models.map((registeredModel) => ({
```
```
// packages/coding-agent/test/suite/harness.ts:261
		setResponses: fauxProvider.setResponses,
		appendResponses: fauxProvider.appendResponses,
		getPendingResponseCount: fauxProvider.getPendingResponseCount,
```
```
// packages/coding-agent/test/suite/harness.ts:262
		appendResponses: fauxProvider.appendResponses,
		getPendingResponseCount: fauxProvider.getPendingResponseCount,
		events,
```
```
// packages/coding-agent/test/suite/harness.ts:178
			api: fauxProvider.api,
			models: fauxProvider.models.map((registeredModel) => ({
				id: registeredModel.id,
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 17: code:types.ts->agent.test.ts
- E2 AssistantMessage declared at packages/ai/src/types.ts:570
- E4 AssistantMessage imported at packages/agent/test/agent.test.ts:1 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E5 AssistantMessage fields read: content (packages/agent/test/agent.test.ts:815), role (packages/agent/test/agent.test.ts:172)
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
// packages/agent/test/agent.test.ts:815
			if (message.role !== "user") return false;
			if (typeof message.content === "string") return message.content === "Queued follow-up";
			return message.content.some((part) => part.type === "text" && part.text === "Queued follow-up");
```
```
// packages/agent/test/agent.test.ts:172
					context.messages.flatMap((message) =>
						message.role === "system"
							? [
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 18: code:types.ts->e2e.test.ts
- E2 AssistantMessage declared at packages/ai/src/types.ts:570
- E4 AssistantMessage imported at packages/agent/test/e2e.test.ts:1 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E5 AssistantMessage fields read: content (packages/agent/test/e2e.test.ts:27), role (packages/agent/test/e2e.test.ts:59)
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
// packages/agent/test/e2e.test.ts:27
function getTextContent(message: AssistantMessage | ToolResultMessage): string {
	return message.content
		.filter((block) => block.type === "text")
```
```
// packages/agent/test/e2e.test.ts:59
	const assistantMessage = agent.state.messages[2];
	if (assistantMessage.role !== "assistant") throw new Error("Expected assistant message");
	expect(getTextContent(assistantMessage)).toContain("4");
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 19: code:types.ts->agent-session.ts#2
- E2 AssistantMessage declared at packages/ai/src/types.ts:570
- E2 Model declared at packages/ai/src/types.ts:839
- E4 AssistantMessage imported at packages/coding-agent/src/core/agent-session.ts:35 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E4 Model imported at packages/coding-agent/src/core/agent-session.ts:35 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E5 AssistantMessage fields read: content (packages/coding-agent/src/core/agent-session.ts:1931), errorMessage (packages/coding-agent/src/core/agent-session.ts:1969), model (packages/coding-agent/src/core/agent-session.ts:618), provider (packages/coding-agent/src/core/agent-session.ts:618), role (packages/coding-agent/src/core/agent-session.ts:820), stopReason (packages/coding-agent/src/core/agent-session.ts:852), timestamp (packages/coding-agent/src/core/agent-session.ts:3002), usage (packages/coding-agent/src/core/agent-session.ts:1115)
- E5 AssistantMessage stoplisted reads: (none)
- E5 Model fields read: contextWindow (packages/coding-agent/src/core/agent-session.ts:769), maxTokens (packages/coding-agent/src/core/agent-session.ts:3039), reasoning (packages/coding-agent/src/core/agent-session.ts:2696)
- E5 Model stoplisted reads: (none)
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
// packages/coding-agent/src/core/agent-session.ts:1931
				// Untyped extensions can pass null/missing content; normalize at ingestion.
				content: msg.content ?? [],
				display: msg.display,
```
```
// packages/coding-agent/src/core/agent-session.ts:1969
				attempt: this._retryAttempt,
				finalError: message.errorMessage,
			});
```
```
// packages/coding-agent/src/core/agent-session.ts:618
		const model = this.model;
		if (model && isVirtualModel(model)) return this._modelRuntime.getPhysicalModel(message.provider, message.model);
		return model?.provider === message.provider && model.id === message.model ? model : undefined;
```
```
// packages/coding-agent/src/core/agent-session.ts:820
			// start a turn; extension messages can follow them, e.g. from before_agent_start.
			const lastResponse = context.messages.findLastIndex((message) => message.role === "assistant");
			const userTurn = context.messages.slice(lastResponse + 1).some((message) => message.role === "user");
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 20: code:types.ts->bug-report.ts
- E2 Model declared at packages/ai/src/types.ts:839
- E4 Model imported at packages/coding-agent/src/core/bug-report.ts:4 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E5 Model fields read: compat (packages/coding-agent/src/core/bug-report.ts:105), contextWindow (packages/coding-agent/src/core/bug-report.ts:102), maxTokens (packages/coding-agent/src/core/bug-report.ts:103), reasoning (packages/coding-agent/src/core/bug-report.ts:100), samplingParams (packages/coding-agent/src/core/bug-report.ts:104), thinkingLevelMap (packages/coding-agent/src/core/bug-report.ts:106)
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

### earendil-works__pi 21: code:types.ts->compaction.ts
- E2 AssistantMessage declared at packages/ai/src/types.ts:570
- E2 SystemMessage declared at packages/ai/src/types.ts:546
- E2 Usage declared at packages/ai/src/types.ts:451
- E4 AssistantMessage imported at packages/coding-agent/src/core/compaction/compaction.ts:18 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E4 SystemMessage imported at packages/coding-agent/src/core/compaction/compaction.ts:18 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E4 Usage imported at packages/coding-agent/src/core/compaction/compaction.ts:18 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E5 AssistantMessage fields read: content (packages/coding-agent/src/core/compaction/compaction.ts:321), stopReason (packages/coding-agent/src/core/compaction/compaction.ts:152), usage (packages/coding-agent/src/core/compaction/compaction.ts:154)
- E5 AssistantMessage stoplisted reads: (none)
- E5 SystemMessage fields read: content (packages/coding-agent/src/core/compaction/compaction.ts:304), sections (packages/coding-agent/src/core/compaction/compaction.ts:305), toolsAdded (packages/coding-agent/src/core/compaction/compaction.ts:310)
- E5 SystemMessage stoplisted reads: (none)
- E5 Usage fields read: cacheRead (packages/coding-agent/src/core/compaction/compaction.ts:141), cacheWrite (packages/coding-agent/src/core/compaction/compaction.ts:141), input (packages/coding-agent/src/core/compaction/compaction.ts:141), output (packages/coding-agent/src/core/compaction/compaction.ts:141), totalTokens (packages/coding-agent/src/core/compaction/compaction.ts:141)
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
// packages/coding-agent/src/core/compaction/compaction.ts:321
			const assistant = message as AssistantMessage;
			for (const block of assistant.content) {
				if (block.type === "text") {
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
```
// packages/coding-agent/src/core/compaction/compaction.ts:304
			const system = message as SystemMessage;
			chars = estimateTextAndImageContentChars(system.content);
			if (system.sections) {
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 22: code:types.ts->usage-totals.ts
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

### earendil-works__pi 23: code:types.ts->interactive-mode.ts
- E2 AssistantMessage declared at packages/ai/src/types.ts:570
- E2 Usage declared at packages/ai/src/types.ts:451
- E4 AssistantMessage imported at packages/coding-agent/src/modes/interactive/interactive-mode.ts:12 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E4 Usage imported at packages/coding-agent/src/modes/interactive/interactive-mode.ts:12 from "@earendil-works/pi-ai/compat", resolved to packages/ai/src/compat.ts, re-exported through packages/ai/src/compat.ts:27 -> packages/ai/src/index.ts:37 from packages/ai/src/types.ts
- E5 AssistantMessage fields read: content (packages/coding-agent/src/modes/interactive/interactive-mode.ts:3812), diagnostics (packages/coding-agent/src/modes/interactive/interactive-mode.ts:4150), errorMessage (packages/coding-agent/src/modes/interactive/interactive-mode.ts:2183), role (packages/coding-agent/src/modes/interactive/interactive-mode.ts:3810), stopReason (packages/coding-agent/src/modes/interactive/interactive-mode.ts:2182)
- E5 AssistantMessage stoplisted reads: (none)
- E5 Usage fields read: cacheRead (packages/coding-agent/src/modes/interactive/interactive-mode.ts:4139), cacheWrite (packages/coding-agent/src/modes/interactive/interactive-mode.ts:4139), cost (packages/coding-agent/src/modes/interactive/interactive-mode.ts:4140), input (packages/coding-agent/src/modes/interactive/interactive-mode.ts:4139), output (packages/coding-agent/src/modes/interactive/interactive-mode.ts:4139)
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
// packages/coding-agent/src/modes/interactive/interactive-mode.ts:3812
		const textBlocks =
			typeof message.content === "string"
				? [{ type: "text", text: message.content }]
```
```
// packages/coding-agent/src/modes/interactive/interactive-mode.ts:4150
		let count = 0;
		for (const diagnostic of message.diagnostics ?? []) {
			if (diagnostic.type !== "anthropic_input_transformations") continue;
```
```
// packages/coding-agent/src/modes/interactive/interactive-mode.ts:2183
		if (message.stopReason !== "error" || isRetryableAssistantError(message)) return;
		if (/\b(?:abort(?:ed)?|cancel(?:l?ed)?)\b/i.test(message.errorMessage ?? "")) return;
		if (this.maybeShowInstallChangeWarning()) return;
```
```
// packages/coding-agent/src/modes/interactive/interactive-mode.ts:3810
	private getUserMessageText(message: Message): string {
		if (message.role !== "user") return "";
		const textBlocks =
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 24: code:types.ts->agent-session-concurrent.test.ts
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

### earendil-works__pi 25: code:read.ts->index.ts
- E2 ReadOperations declared at packages/coding-agent/src/core/tools/read.ts:48
- E4 ReadOperations imported at packages/coding-agent/examples/extensions/gondolin/index.ts:25 from "@earendil-works/pi-coding-agent", resolved to packages/coding-agent/src/index.ts, re-exported through packages/coding-agent/src/index.ts:336 -> packages/coding-agent/src/core/tools/index.ts:56 from packages/coding-agent/src/core/tools/read.ts
- E5 ReadOperations fields read: access (packages/coding-agent/examples/extensions/gondolin/index.ts:118), readFile (packages/coding-agent/examples/extensions/gondolin/index.ts:116)
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
// packages/coding-agent/examples/extensions/gondolin/index.ts:118
		writeFile: writeOps.writeFile,
		access: readOps.access,
	};
```
```
// packages/coding-agent/examples/extensions/gondolin/index.ts:116
	return {
		readFile: readOps.readFile,
		writeFile: writeOps.writeFile,
```
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 1: code:editorInterface.ts->transformHandles.ts
- E2 EditorInterface declared at packages/common/src/editorInterface.ts:3
- E4 EditorInterface imported at packages/element/src/transformHandles.ts:1 from "@excalidraw/common", resolved to packages/common/src/index.ts, re-exported by barrel packages/common/src/index.ts:15 from packages/common/src/editorInterface.ts
- E5 EditorInterface fields read: formFactor (packages/element/src/transformHandles.ts:114), userAgent (packages/element/src/transformHandles.ts:115)
- E5 EditorInterface stoplisted reads: (none)
```
// packages/common/src/editorInterface.ts:3
export type EditorInterface = Readonly<{
  formFactor: "phone" | "tablet" | "desktop";
  desktopUIMode: "compact" | "full";
  userAgent: Readonly<{
    isMobileDevice: boolean;
    platform: "ios" | "android" | "other" | "unknown";
  }>;
  isTouchScreen: boolean;
  canFitSidebar: boolean;
  isLandscape: boolean;
}>;

```
```
// packages/element/src/transformHandles.ts:114
  if (
    editorInterface.formFactor === "phone" &&
    editorInterface.userAgent.isMobileDevice
```
```
// packages/element/src/transformHandles.ts:115
    editorInterface.formFactor === "phone" &&
    editorInterface.userAgent.isMobileDevice
  ) {
```
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 2: code:editorInterface.ts->App.tsx
- E2 EditorInterface declared at packages/common/src/editorInterface.ts:3
- E4 EditorInterface imported at packages/excalidraw/components/App.tsx:10 from "@excalidraw/common", resolved to packages/common/src/index.ts, re-exported by barrel packages/common/src/index.ts:15 from packages/common/src/editorInterface.ts
- E5 EditorInterface fields read: canFitSidebar (packages/excalidraw/components/App.tsx:4665), desktopUIMode (packages/excalidraw/components/App.tsx:3727), formFactor (packages/excalidraw/components/App.tsx:2375), isTouchScreen (packages/excalidraw/components/App.tsx:6567)
- E5 EditorInterface stoplisted reads: (none)
```
// packages/common/src/editorInterface.ts:3
export type EditorInterface = Readonly<{
  formFactor: "phone" | "tablet" | "desktop";
  desktopUIMode: "compact" | "full";
  userAgent: Readonly<{
    isMobileDevice: boolean;
    platform: "ios" | "android" | "other" | "unknown";
  }>;
  isTouchScreen: boolean;
  canFitSidebar: boolean;
  isLandscape: boolean;
}>;

```
```
// packages/excalidraw/components/App.tsx:4665
          this.state.openSidebar &&
          this.editorInterface.canFitSidebar &&
          editorJotaiStore.get(isSidebarDockedAtom)
```
```
// packages/excalidraw/components/App.tsx:3727
    const nextEditorInterface = updateObject(this.editorInterface, {
      desktopUIMode: storedDesktopUIMode ?? this.editorInterface.desktopUIMode,
      formFactor: this.getFormFactor(editorWidth, editorHeight),
```
```
// packages/excalidraw/components/App.tsx:2375
              this.state.openDialog?.name === "elementLinkSelector",
            "excalidraw--mobile": this.editorInterface.formFactor === "phone",
            "excalidraw--non-interactive": !this.isInteractionEnabled(),
```
```
// packages/excalidraw/components/App.tsx:6567
  ): boolean => {
    if (this.editorInterface.isTouchScreen) {
      const hitElement = this.getElementAtPosition(
```
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 3: code:renderElement.ts->staticScene.ts
- E2 ElementRenderState declared at packages/element/src/renderElement.ts:151
- E4 ElementRenderState imported at packages/excalidraw/renderer/staticScene.ts:30 from "@excalidraw/element", resolved to packages/element/src/index.ts, re-exported by barrel packages/element/src/index.ts:88 from packages/element/src/renderElement.ts
- E5 ElementRenderState fields read: offset (packages/excalidraw/renderer/staticScene.ts:212), opacity (packages/excalidraw/renderer/staticScene.ts:269)
- E5 ElementRenderState stoplisted reads: (none)
```
// packages/element/src/renderElement.ts:151
export type ElementRenderState = Readonly<{
  /** Alpha including frame opacity and pending erasure; selection dimming is separate. */
  opacity: number;
  offset: RenderPositionOffset;
}>;

/** Resolve visual state at the drawing boundary, preserving document cache keys. */
export const resolveElementRenderState = (
  element: ExcalidrawElement,
  elementsMap: ElementsMap,
  renderConfig: Pick<
    StaticCanvasRenderConfig,
```
```
// packages/excalidraw/renderer/staticScene.ts:212
      [
        x1 + renderState.offset.x,
        y1 + renderState.offset.y,
```
```
// packages/excalidraw/renderer/staticScene.ts:269
    }
    context.globalAlpha = renderState.opacity;
    context.drawImage(linkCanvas, x - centerX, y - centerY, width, height);
```
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 4: code:types.ts->DebugCanvas.tsx
- E2 FixedPointBinding declared at packages/element/src/types.ts:320
- E4 FixedPointBinding imported at excalidraw-app/components/DebugCanvas.tsx:34 from "@excalidraw/element/types", resolved to packages/element/src/types.ts
- E5 FixedPointBinding fields read: elementId (excalidraw-app/components/DebugCanvas.tsx:146), fixedPoint (excalidraw-app/components/DebugCanvas.tsx:140)
- E5 FixedPointBinding stoplisted reads: (none)
```
// packages/element/src/types.ts:320
export type FixedPointBinding = {
  elementId: ExcalidrawBindableElement["id"];

  // Represents the fixed point binding information in form of a vertical and
  // horizontal ratio (i.e. a percentage value in the 0.0-1.0 range). This ratio
  // gives the user selected fixed point by multiplying the bound element width
  // with fixedPoint[0] and the bound element height with fixedPoint[1] to get the
  // bound element-local point coordinate.
  fixedPoint: FixedPoint;

  // Determines whether the arrow remains outside the shape or is allowed to
  // go all the way inside the shape up to the exact fixed point.
```
```
// excalidraw-app/components/DebugCanvas.tsx:146
  const bindable = elementsMap.get(
    binding.elementId,
  ) as ExcalidrawBindableElement;
```
```
// excalidraw-app/components/DebugCanvas.tsx:140
) => {
  if (!binding.fixedPoint) {
    console.warn("Binding must have a fixedPoint");
```
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 5: code:types.ts->restore.ts
- E2 FixedPointBinding declared at packages/element/src/types.ts:320
- E4 FixedPointBinding imported at packages/excalidraw/data/restore.ts:89 from "@excalidraw/element/types", resolved to packages/element/src/types.ts
- E5 FixedPointBinding fields read: elementId (packages/excalidraw/data/restore.ts:357), fixedPoint (packages/excalidraw/data/restore.ts:340), mode (packages/excalidraw/data/restore.ts:341)
- E5 FixedPointBinding stoplisted reads: (none)
```
// packages/element/src/types.ts:320
export type FixedPointBinding = {
  elementId: ExcalidrawBindableElement["id"];

  // Represents the fixed point binding information in form of a vertical and
  // horizontal ratio (i.e. a percentage value in the 0.0-1.0 range). This ratio
  // gives the user selected fixed point by multiplying the bound element width
  // with fixedPoint[0] and the bound element height with fixedPoint[1] to get the
  // bound element-local point coordinate.
  fixedPoint: FixedPoint;

  // Determines whether the arrow remains outside the shape or is allowed to
  // go all the way inside the shape up to the exact fixed point.
```
```
// packages/excalidraw/data/restore.ts:357
      // (it's done in a separate pass)
      if (binding.elementId) {
        return {
```
```
// packages/excalidraw/data/restore.ts:340
        ...binding,
        fixedPoint: normalizeFixedPoint(binding.fixedPoint),
        mode: binding.mode || "orbit",
```
```
// packages/excalidraw/data/restore.ts:341
        fixedPoint: normalizeFixedPoint(binding.fixedPoint),
        mode: binding.mode || "orbit",
      };
```
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 6: code:visualdebug.ts->DebugCanvas.tsx
- E2 DebugElement declared at packages/element/src/visualdebug.ts:32
- E4 DebugElement imported at excalidraw-app/components/DebugCanvas.tsx:30 from "@excalidraw/element/visualdebug", resolved to packages/element/src/visualdebug.ts
- E5 DebugElement fields read: color (excalidraw-app/components/DebugCanvas.tsx:313), permanent (excalidraw-app/components/DebugCanvas.tsx:381)
- E5 DebugElement stoplisted reads: data
```
// packages/element/src/visualdebug.ts:32
export type DebugElement = {
  color: string;
  data: LineSegment<GlobalPoint> | Curve<GlobalPoint> | DebugPolygon;
  permanent: boolean;
};

export type DebugPolygon = {
  type: "polygon";
  points: GlobalPoint[];
  fill?: boolean;
  close?: boolean;
};
```
```
// excalidraw-app/components/DebugCanvas.tsx:313
          el.data as LineSegment<GlobalPoint>,
          el.color,
        );
```
```
// excalidraw-app/components/DebugCanvas.tsx:381
      window.visualDebug?.data.map((frame) =>
        frame.filter((el) => el.permanent),
      ) ?? [];
```
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 7: code:types.ts->index.ts
- E2 ImportedDataState declared at packages/excalidraw/data/types.ts:35
- E4 ImportedDataState imported at excalidraw-app/data/index.ts:17 from "@excalidraw/excalidraw/data/types", resolved to packages/excalidraw/data/types.ts
- E5 ImportedDataState fields read: appState (excalidraw-app/data/index.ts:198), elements (excalidraw-app/data/index.ts:197)
- E5 ImportedDataState stoplisted reads: (none)
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
// excalidraw-app/data/index.ts:198
    elements: data.elements || null,
    appState: data.appState || null,
  };
```
```
// excalidraw-app/data/index.ts:197
  return {
    elements: data.elements || null,
    appState: data.appState || null,
```
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 8: code:types.ts->renderElement.ts
- E2 StaticCanvasRenderConfig declared at packages/excalidraw/scene/types.ts:28
- E4 StaticCanvasRenderConfig imported at packages/element/src/renderElement.ts:43 from "@excalidraw/excalidraw/scene/types", resolved to packages/excalidraw/scene/types.ts
- E5 StaticCanvasRenderConfig fields read: imageCache (packages/element/src/renderElement.ts:100), isExporting (packages/element/src/renderElement.ts:990), theme (packages/element/src/renderElement.ts:449)
- E5 StaticCanvasRenderConfig stoplisted reads: (none)
```
// packages/excalidraw/scene/types.ts:28
export type StaticCanvasRenderConfig = {
  canvasBackgroundColor: AppState["viewBackgroundColor"];
  // extra options passed to the renderer
  // ---------------------------------------------------------------------------
  imageCache: AppClassProperties["imageCache"];
  renderGrid: boolean;
  /** whether to render link icons on elements with links (never rendered
   when exporting). @default true */
  renderLinks?: boolean;
  /** when exporting the behavior is slightly different (e.g. we can't use
   CSS filters), and we disable render optimizations for best output */
  isExporting: boolean;
```
```
// packages/element/src/renderElement.ts:100
  isInitializedImageElement(element) &&
  !renderConfig.imageCache.has(element.fileId);

```
```
// packages/element/src/renderElement.ts:990
  if (
    (renderConfig.isExporting || isFrameLikeElement(element)) &&
    (renderState.offset.x || renderState.offset.y)
```
```
// packages/element/src/renderElement.ts:449
        element.backgroundColor,
        renderConfig.theme === THEME.DARK,
      );
```
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 9: code:types.ts->CustomFooter.tsx
- E2 ExcalidrawImperativeAPI declared at packages/excalidraw/types.ts:1364
- E4 ExcalidrawImperativeAPI imported at examples/with-script-in-browser/components/CustomFooter.tsx:4 from "@excalidraw/excalidraw/types", resolved to packages/excalidraw/types.ts
- E5 ExcalidrawImperativeAPI fields read: setActiveTool (examples/with-script-in-browser/components/CustomFooter.tsx:44), setCursor (examples/with-script-in-browser/components/CustomFooter.tsx:64)
- E5 ExcalidrawImperativeAPI stoplisted reads: (none)
```
// packages/excalidraw/types.ts:1364
export interface ExcalidrawImperativeAPI {
  /** Whether the editor has been unmounted and the API is no longer usable. */
  isDestroyed: boolean;
  updateScene: InstanceType<typeof App>["updateScene"];
  applyDeltas: InstanceType<typeof App>["applyDeltas"];
  mutateElement: InstanceType<typeof App>["mutateElement"];
  updateLibrary: InstanceType<typeof Library>["updateLibrary"];
  resetScene: InstanceType<typeof App>["resetScene"];
  getSceneElementsIncludingDeleted: InstanceType<
    typeof App
  >["getSceneElementsIncludingDeleted"];
  getSceneElementsMapIncludingDeleted: InstanceType<
```
```
// examples/with-script-in-browser/components/CustomFooter.tsx:44
        onSelect={() => {
          excalidrawAPI?.setActiveTool({
            type: "custom",
```
```
// examples/with-script-in-browser/components/CustomFooter.tsx:64
          )}`;
          excalidrawAPI?.setCursor(`url(${url}), auto`);
        }}
```
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 10: code:types.ts->ExampleApp.tsx
- E2 ExcalidrawImperativeAPI declared at packages/excalidraw/types.ts:1364
- E4 ExcalidrawImperativeAPI imported at examples/with-script-in-browser/components/ExampleApp.tsx:17 from "@excalidraw/excalidraw/types", resolved to packages/excalidraw/types.ts
- E5 ExcalidrawImperativeAPI fields read: addFiles (examples/with-script-in-browser/components/ExampleApp.tsx:167), getAppState (examples/with-script-in-browser/components/ExampleApp.tsx:391), getFiles (examples/with-script-in-browser/components/ExampleApp.tsx:392), getSceneElements (examples/with-script-in-browser/components/ExampleApp.tsx:390), resetScene (examples/with-script-in-browser/components/ExampleApp.tsx:679), setActiveTool (examples/with-script-in-browser/components/ExampleApp.tsx:467), setViewport (examples/with-script-in-browser/components/ExampleApp.tsx:983), updateLibrary (examples/with-script-in-browser/components/ExampleApp.tsx:309), updateScene (examples/with-script-in-browser/components/ExampleApp.tsx:307)
- E5 ExcalidrawImperativeAPI stoplisted reads: (none)
```
// packages/excalidraw/types.ts:1364
export interface ExcalidrawImperativeAPI {
  /** Whether the editor has been unmounted and the API is no longer usable. */
  isDestroyed: boolean;
  updateScene: InstanceType<typeof App>["updateScene"];
  applyDeltas: InstanceType<typeof App>["applyDeltas"];
  mutateElement: InstanceType<typeof App>["mutateElement"];
  updateLibrary: InstanceType<typeof Library>["updateLibrary"];
  resetScene: InstanceType<typeof App>["resetScene"];
  getSceneElementsIncludingDeleted: InstanceType<
    typeof App
  >["getSceneElementsIncludingDeleted"];
  getSceneElementsMapIncludingDeleted: InstanceType<
```
```
// examples/with-script-in-browser/components/ExampleApp.tsx:167
        });
        excalidrawAPI.addFiles(imagesArray);
      };
```
```
// examples/with-script-in-browser/components/ExampleApp.tsx:391
      elements: excalidrawAPI.getSceneElements(),
      appState: excalidrawAPI.getAppState(),
      files: excalidrawAPI.getFiles(),
```
```
// examples/with-script-in-browser/components/ExampleApp.tsx:392
      appState: excalidrawAPI.getAppState(),
      files: excalidrawAPI.getFiles(),
      type,
```
```
// examples/with-script-in-browser/components/ExampleApp.tsx:390
    await exportToClipboard({
      elements: excalidrawAPI.getSceneElements(),
      appState: excalidrawAPI.getAppState(),
```
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 11: code:types.ts->App.tsx
- E2 AppState declared at packages/excalidraw/types.ts:329
- E2 ExcalidrawImperativeAPI declared at packages/excalidraw/types.ts:1364
- E4 AppState imported at excalidraw-app/App.tsx:71 from "@excalidraw/excalidraw/types", resolved to packages/excalidraw/types.ts
- E4 ExcalidrawImperativeAPI imported at excalidraw-app/App.tsx:71 from "@excalidraw/excalidraw/types", resolved to packages/excalidraw/types.ts
- E5 AppState fields read: exportBackground (excalidraw-app/App.tsx:784), viewBackgroundColor (excalidraw-app/App.tsx:785)
- E5 AppState stoplisted reads: (none)
- E5 ExcalidrawImperativeAPI fields read: addFiles (excalidraw-app/App.tsx:492), getAppState (excalidraw-app/App.tsx:346), getEditorInterface (excalidraw-app/App.tsx:1002), getFiles (excalidraw-app/App.tsx:626), getName (excalidraw-app/App.tsx:968), getSceneElements (excalidraw-app/App.tsx:697), getSceneElementsIncludingDeleted (excalidraw-app/App.tsx:354), refresh (excalidraw-app/App.tsx:1061), setViewport (excalidraw-app/App.tsx:1022), updateLibrary (excalidraw-app/App.tsx:616), updateScene (excalidraw-app/App.tsx:580)
- E5 ExcalidrawImperativeAPI stoplisted reads: (none)
```
// packages/excalidraw/types.ts:329
export interface AppState {
  contextMenu: {
    items: ContextMenuItems;
    top: number;
    left: number;
  } | null;
  showWelcomeScreen: boolean;
  isLoading: boolean;
  errorMessage: React.ReactNode;
  activeEmbeddable: {
    element: NonDeletedExcalidrawElement;
    state: "hover" | "active";
```
```
// packages/excalidraw/types.ts:1364
export interface ExcalidrawImperativeAPI {
  /** Whether the editor has been unmounted and the API is no longer usable. */
  isDestroyed: boolean;
  updateScene: InstanceType<typeof App>["updateScene"];
  applyDeltas: InstanceType<typeof App>["applyDeltas"];
  mutateElement: InstanceType<typeof App>["mutateElement"];
  updateLibrary: InstanceType<typeof Library>["updateLibrary"];
  resetScene: InstanceType<typeof App>["resetScene"];
  getSceneElementsIncludingDeleted: InstanceType<
    typeof App
  >["getSceneElementsIncludingDeleted"];
  getSceneElementsMapIncludingDeleted: InstanceType<
```
```
// excalidraw-app/App.tsx:784
          ...appState,
          viewBackgroundColor: appState.exportBackground
            ? appState.viewBackgroundColor
```
```
// excalidraw-app/App.tsx:785
          viewBackgroundColor: appState.exportBackground
            ? appState.viewBackgroundColor
            : getDefaultAppState().viewBackgroundColor,
```
```
// excalidraw-app/App.tsx:492
            .then(({ loadedFiles, erroredFiles }) => {
              excalidrawAPI.addFiles(loadedFiles);
              updateStaleImageStatuses({
```
```
// excalidraw-app/App.tsx:346
            },
            excalidrawAPI.getAppState(),
          ),
```
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 12: code:types.ts->Collab.tsx
- E2 ExcalidrawImperativeAPI declared at packages/excalidraw/types.ts:1364
- E4 ExcalidrawImperativeAPI imported at excalidraw-app/collab/Collab.tsx:45 from "@excalidraw/excalidraw/types", resolved to packages/excalidraw/types.ts
- E5 ExcalidrawImperativeAPI fields read: addFiles (excalidraw-app/collab/Collab.tsx:801), getAppState (excalidraw-app/collab/Collab.tsx:331), getSceneElements (excalidraw-app/collab/Collab.tsx:552), getSceneElementsIncludingDeleted (excalidraw-app/collab/Collab.tsx:373), onScrollChange (excalidraw-app/collab/Collab.tsx:227), onUserFollow (excalidraw-app/collab/Collab.tsx:219), resetScene (excalidraw-app/collab/Collab.tsx:550), updateScene (excalidraw-app/collab/Collab.tsx:406)
- E5 ExcalidrawImperativeAPI stoplisted reads: (none)
```
// packages/excalidraw/types.ts:1364
export interface ExcalidrawImperativeAPI {
  /** Whether the editor has been unmounted and the API is no longer usable. */
  isDestroyed: boolean;
  updateScene: InstanceType<typeof App>["updateScene"];
  applyDeltas: InstanceType<typeof App>["applyDeltas"];
  mutateElement: InstanceType<typeof App>["mutateElement"];
  updateLibrary: InstanceType<typeof Library>["updateLibrary"];
  resetScene: InstanceType<typeof App>["resetScene"];
  getSceneElementsIncludingDeleted: InstanceType<
    typeof App
  >["getSceneElementsIncludingDeleted"];
  getSceneElementsMapIncludingDeleted: InstanceType<
```
```
// excalidraw-app/collab/Collab.tsx:801

    this.excalidrawAPI.addFiles(loadedFiles);

```
```
// excalidraw-app/collab/Collab.tsx:331
        syncableElements,
        this.excalidrawAPI.getAppState(),
      );
```
```
// excalidraw-app/collab/Collab.tsx:552
    } else {
      const elements = this.excalidrawAPI.getSceneElements().map((element) => {
        if (isImageElement(element) && element.status === "saved") {
```
```
// excalidraw-app/collab/Collab.tsx:373
      getSyncableElements(
        this.excalidrawAPI.getSceneElementsIncludingDeleted(),
      ),
```
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 13: code:types.ts->AI.tsx
- E2 ExcalidrawImperativeAPI declared at packages/excalidraw/types.ts:1364
- E4 ExcalidrawImperativeAPI imported at excalidraw-app/components/AI.tsx:15 from "@excalidraw/excalidraw/types", resolved to packages/excalidraw/types.ts
- E5 ExcalidrawImperativeAPI fields read: getAppState (excalidraw-app/components/AI.tsx:28), getFiles (excalidraw-app/components/AI.tsx:45)
- E5 ExcalidrawImperativeAPI stoplisted reads: (none)
```
// packages/excalidraw/types.ts:1364
export interface ExcalidrawImperativeAPI {
  /** Whether the editor has been unmounted and the API is no longer usable. */
  isDestroyed: boolean;
  updateScene: InstanceType<typeof App>["updateScene"];
  applyDeltas: InstanceType<typeof App>["applyDeltas"];
  mutateElement: InstanceType<typeof App>["mutateElement"];
  updateLibrary: InstanceType<typeof Library>["updateLibrary"];
  resetScene: InstanceType<typeof App>["resetScene"];
  getSceneElementsIncludingDeleted: InstanceType<
    typeof App
  >["getSceneElementsIncludingDeleted"];
  getSceneElementsMapIncludingDeleted: InstanceType<
```
```
// excalidraw-app/components/AI.tsx:28
        generate={async ({ frame, children, onPartial }) => {
          const appState = excalidrawAPI.getAppState();

```
```
// excalidraw-app/components/AI.tsx:45
            exportingFrame: frame,
            files: excalidrawAPI.getFiles(),
            mimeType: MIME_TYPES.jpg,
```
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 14: code:types.ts->DebugCanvas.tsx#2
- E2 AppState declared at packages/excalidraw/types.ts:329
- E4 AppState imported at excalidraw-app/components/DebugCanvas.tsx:10 from "@excalidraw/excalidraw/types", resolved to packages/excalidraw/types.ts
- E5 AppState fields read: scrollX (excalidraw-app/components/DebugCanvas.tsx:355), scrollY (excalidraw-app/components/DebugCanvas.tsx:356), zoom (excalidraw-app/components/DebugCanvas.tsx:311)
- E5 AppState stoplisted reads: (none)
```
// packages/excalidraw/types.ts:329
export interface AppState {
  contextMenu: {
    items: ContextMenuItems;
    top: number;
    left: number;
  } | null;
  showWelcomeScreen: boolean;
  isLoading: boolean;
  errorMessage: React.ReactNode;
  activeEmbeddable: {
    element: NonDeletedExcalidrawElement;
    state: "hover" | "active";
```
```
// excalidraw-app/components/DebugCanvas.tsx:355
  context.translate(
    appState.scrollX * appState.zoom.value,
    appState.scrollY * appState.zoom.value,
```
```
// excalidraw-app/components/DebugCanvas.tsx:356
    appState.scrollX * appState.zoom.value,
    appState.scrollY * appState.zoom.value,
  );
```
```
// excalidraw-app/components/DebugCanvas.tsx:311
          context,
          appState.zoom.value,
          el.data as LineSegment<GlobalPoint>,
```
Judgment (TRUE / NOT TRUE, one sentence):

### puppeteer__puppeteer 1: code:ElementHandle.ts->elementhandle.test.ts
- E2 Point declared at packages/puppeteer-core/src/api/ElementHandle.ts:109
- E4 Point imported at test/src/elementhandle.test.ts:9 from "puppeteer-core/internal/api/ElementHandle.js", resolved to packages/puppeteer-core/src/api/ElementHandle.ts
- E5 Point fields read: x (test/src/elementhandle.test.ts:184), y (test/src/elementhandle.test.ts:185)
- E5 Point stoplisted reads: (none)
```
// packages/puppeteer-core/src/api/ElementHandle.ts:109
export interface Point {
  x: number;
  y: number;
}

/**
 * @public
 */
export interface ElementScreenshotOptions extends ScreenshotOptions {
  /**
   * @defaultValue `true`
   */
```
```
// test/src/elementhandle.test.ts:184
          {
            x: topLeft.x,
            y: topLeft.y,
```
```
// test/src/elementhandle.test.ts:185
            x: topLeft.x,
            y: topLeft.y,
          },
```
Judgment (TRUE / NOT TRUE, one sentence):

### puppeteer__puppeteer 2: code:Accessibility.ts->accessibility.test.ts
- E2 SerializedAXNode declared at packages/puppeteer-core/src/cdp/Accessibility.ts:18
- E4 SerializedAXNode imported at test/src/accessibility.test.ts:10 from "puppeteer-core/internal/cdp/Accessibility.js", resolved to packages/puppeteer-core/src/cdp/Accessibility.ts
- E5 SerializedAXNode fields read: children (test/src/accessibility.test.ts:905), focused (test/src/accessibility.test.ts:902)
- E5 SerializedAXNode stoplisted reads: (none)
```
// packages/puppeteer-core/src/cdp/Accessibility.ts:18
export interface SerializedAXNode {
  /**
   * The {@link https://www.w3.org/TR/wai-aria/#usage_intro | role} of the node.
   */
  role: string;
  /**
   * A human readable name for the node.
   */
  name?: string;
  /**
   * The current value of the node.
   */
```
```
// test/src/accessibility.test.ts:905
    }
    for (const child of node?.children || []) {
      const focusedChild = findFocusedNode(child);
```
```
// test/src/accessibility.test.ts:902
  ): SerializedAXNode | null {
    if (node?.focused) {
      return node;
```
Judgment (TRUE / NOT TRUE, one sentence):

### puppeteer__puppeteer 3: code:Configuration.ts->getConfiguration.ts
- E2 ChromeSettings declared at packages/puppeteer-core/src/common/Configuration.ts:87
- E2 Configuration declared at packages/puppeteer-core/src/common/Configuration.ts:26
- E4 ChromeSettings imported at packages/puppeteer/src/getConfiguration.ts:11 from "puppeteer-core", resolved to packages/puppeteer-core/src/puppeteer-core.ts, re-exported through packages/puppeteer-core/src/puppeteer-core.ts:10 -> packages/puppeteer-core/src/index.ts:7 -> packages/puppeteer-core/src/index-browser.ts:12 -> packages/puppeteer-core/src/common/common.ts:10 from packages/puppeteer-core/src/common/Configuration.ts
- E4 Configuration imported at packages/puppeteer/src/getConfiguration.ts:11 from "puppeteer-core", resolved to packages/puppeteer-core/src/puppeteer-core.ts, re-exported through packages/puppeteer-core/src/puppeteer-core.ts:10 -> packages/puppeteer-core/src/index.ts:7 -> packages/puppeteer-core/src/index-browser.ts:12 -> packages/puppeteer-core/src/common/common.ts:10 from packages/puppeteer-core/src/common/Configuration.ts
- E5 ChromeSettings fields read: downloadBaseUrl (packages/puppeteer/src/getConfiguration.ts:96), skipDownload (packages/puppeteer/src/getConfiguration.ts:103)
- E5 ChromeSettings stoplisted reads: version
- E5 Configuration fields read: cacheDirectory (packages/puppeteer/src/getConfiguration.ts:164), defaultBrowser (packages/puppeteer/src/getConfiguration.ts:137), executablePath (packages/puppeteer/src/getConfiguration.ts:141), logLevel (packages/puppeteer/src/getConfiguration.ts:132), skipDownload (packages/puppeteer/src/getConfiguration.ts:102), temporaryDirectory (packages/puppeteer/src/getConfiguration.ts:168)
- E5 Configuration stoplisted reads: (none)
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
// packages/puppeteer/src/getConfiguration.ts:96
    configuration[browser]?.downloadBaseUrl ??
    defaultConfig.downloadBaseUrl;

```
```
// packages/puppeteer/src/getConfiguration.ts:103
    configuration.skipDownload ??
    defaultConfig.skipDownload;

```
```
// packages/puppeteer/src/getConfiguration.ts:164
    process.env['PUPPETEER_CACHE_DIR'] ??
    configuration.cacheDirectory ??
    join(homedir(), '.cache', 'puppeteer');
```
```
// packages/puppeteer/src/getConfiguration.ts:137
  configuration.defaultBrowser = getDefaultBrowser(
    process.env['PUPPETEER_BROWSER'] ?? configuration.defaultBrowser,
  );
```
Judgment (TRUE / NOT TRUE, one sentence):

### puppeteer__puppeteer 4: code:Configuration.ts->install.ts
- E2 ChromeSettings declared at packages/puppeteer-core/src/common/Configuration.ts:87
- E4 ChromeSettings imported at packages/puppeteer/src/node/install.ts:14 from "puppeteer-core", resolved to packages/puppeteer-core/src/puppeteer-core.ts, re-exported through packages/puppeteer-core/src/puppeteer-core.ts:10 -> packages/puppeteer-core/src/index.ts:7 -> packages/puppeteer-core/src/index-browser.ts:12 -> packages/puppeteer-core/src/common/common.ts:10 from packages/puppeteer-core/src/common/Configuration.ts
- E5 ChromeSettings fields read: downloadBaseUrl (packages/puppeteer/src/node/install.ts:39), skipDownload (packages/puppeteer/src/node/install.ts:71)
- E5 ChromeSettings stoplisted reads: version
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

### puppeteer__puppeteer 5: code:LaunchOptions.ts->mocha-utils.ts
- E2 LaunchOptions declared at packages/puppeteer-core/src/node/LaunchOptions.ts:39
- E4 LaunchOptions imported at test/src/mocha-utils.ts:19 from "puppeteer-core/internal/node/LaunchOptions.js", resolved to packages/puppeteer-core/src/node/LaunchOptions.ts
- E5 LaunchOptions fields read: args (test/src/mocha-utils.ts:106), executablePath (test/src/mocha-utils.ts:117), extraPrefsFirefox (test/src/mocha-utils.ts:104)
- E5 LaunchOptions stoplisted reads: (none)
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
// test/src/mocha-utils.ts:106
  `domain1.test,domain2.test,domain3.test`;
defaultBrowserOptions.args!.push(
  `--host-resolver-rules=MAP domain1.test 127.0.0.1,MAP domain2.test 127.0.0.1,MAP domain3.test 127.0.0.1`,
```
```
// test/src/mocha-utils.ts:117

  if (defaultBrowserOptions.executablePath) {
    console.warn(
```
```
// test/src/mocha-utils.ts:104
// Required by tests and cannot be overridden by EXTRA_LAUNCH_OPTIONS.
defaultBrowserOptions.extraPrefsFirefox!['network.dns.localDomains'] =
  `domain1.test,domain2.test,domain3.test`;
```
Judgment (TRUE / NOT TRUE, one sentence):

