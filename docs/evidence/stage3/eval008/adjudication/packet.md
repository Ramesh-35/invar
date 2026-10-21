# STAGE3-EVAL-008 adjudication packet

34 items. Judge only the shape named in each heading.

### Egonex-AI__Understand-Anything 1: code:types.ts->DomainGraphView.tsx (judge only: GraphEdge)
- E2 GraphEdge declared at understand-anything-plugin/packages/core/src/types.ts:70
- E4 GraphEdge not imported by name; resolved via property KnowledgeGraph.edges declared in understand-anything-plugin/packages/core/src/types.ts:112, KnowledgeGraph imported at understand-anything-plugin/packages/dashboard/src/components/DomainGraphView.tsx:23 from "@understand-anything/core/types"
- E5 GraphEdge fields read: source (understand-anything-plugin/packages/dashboard/src/components/DomainGraphView.tsx:49), target (understand-anything-plugin/packages/dashboard/src/components/DomainGraphView.tsx:111), weight (understand-anything-plugin/packages/dashboard/src/components/DomainGraphView.tsx:111)
- E5 GraphEdge stoplisted reads: description, type
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
// understand-anything-plugin/packages/core/src/types.ts:112
  edges: GraphEdge[];
  layers: Layer[];
  tour: TourStep[];
```
```
// understand-anything-plugin/packages/dashboard/src/components/DomainGraphView.tsx:49
  for (const edge of graph.edges) {
    if (edge.type === "contains_flow") {
      flowCountMap.set(edge.source, (flowCountMap.get(edge.source) ?? 0) + 1);
    }
```
```
// understand-anything-plugin/packages/dashboard/src/components/DomainGraphView.tsx:111
  const stepOrderMap = new Map<string, number>();
  for (const edge of stepEdges) {
    stepOrderMap.set(edge.target, edge.weight);
  }
```
Judgment (TRUE / NOT TRUE, one sentence):

### Egonex-AI__Understand-Anything 2: code:types.ts->GraphView.tsx (judge only: GraphEdge)
- E2 GraphEdge declared at understand-anything-plugin/packages/core/src/types.ts:70
- E4 GraphEdge imported at understand-anything-plugin/packages/dashboard/src/components/GraphView.tsx:27 from "@understand-anything/core/types", resolved to understand-anything-plugin/packages/core/src/types.ts; receiver resolved via property LayerDetailTopology.intraContainer declared in understand-anything-plugin/packages/dashboard/src/components/GraphView.tsx:370
- E5 GraphEdge fields read: source (understand-anything-plugin/packages/dashboard/src/components/GraphView.tsx:479), target (understand-anything-plugin/packages/dashboard/src/components/GraphView.tsx:479)
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
// understand-anything-plugin/packages/dashboard/src/components/GraphView.tsx:370
  intraContainer: GraphEdge[];
}

```
```
// understand-anything-plugin/packages/dashboard/src/components/GraphView.tsx:479

    let filteredGraphEdges = graph.edges.filter(
      (e) => filteredNodeIds.has(e.source) && filteredNodeIds.has(e.target),
    );
```
Judgment (TRUE / NOT TRUE, one sentence):

### Egonex-AI__Understand-Anything 3: code:types.ts->KnowledgeGraphView.tsx (judge only: GraphEdge)
- E2 GraphEdge declared at understand-anything-plugin/packages/core/src/types.ts:70
- E4 GraphEdge not imported by name; resolved via property KnowledgeGraph.edges declared in understand-anything-plugin/packages/core/src/types.ts:112, KnowledgeGraph imported at understand-anything-plugin/packages/dashboard/src/components/KnowledgeGraphView.tsx:28 from "@understand-anything/core/types"
- E5 GraphEdge fields read: source (understand-anything-plugin/packages/dashboard/src/components/KnowledgeGraphView.tsx:86), target (understand-anything-plugin/packages/dashboard/src/components/KnowledgeGraphView.tsx:87)
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
// understand-anything-plugin/packages/core/src/types.ts:112
  edges: GraphEdge[];
  layers: Layer[];
  tour: TourStep[];
```
```
// understand-anything-plugin/packages/dashboard/src/components/KnowledgeGraphView.tsx:86
  const edgeCounts = new Map<string, number>();
  for (const edge of graph.edges) {
    edgeCounts.set(edge.source, (edgeCounts.get(edge.source) ?? 0) + 1);
    edgeCounts.set(edge.target, (edgeCounts.get(edge.target) ?? 0) + 1);
```
```
// understand-anything-plugin/packages/dashboard/src/components/KnowledgeGraphView.tsx:87
  for (const edge of graph.edges) {
    edgeCounts.set(edge.source, (edgeCounts.get(edge.source) ?? 0) + 1);
    edgeCounts.set(edge.target, (edgeCounts.get(edge.target) ?? 0) + 1);
  }
```
Judgment (TRUE / NOT TRUE, one sentence):

### Egonex-AI__Understand-Anything 4: code:types.ts->KnowledgeGraphView.tsx (judge only: GraphNode)
- E2 GraphNode declared at understand-anything-plugin/packages/core/src/types.ts:54
- E4 GraphNode not imported by name; resolved via property KnowledgeGraph.nodes declared in understand-anything-plugin/packages/core/src/types.ts:111, KnowledgeGraph imported at understand-anything-plugin/packages/dashboard/src/components/KnowledgeGraphView.tsx:28 from "@understand-anything/core/types"
- E5 GraphNode fields read: complexity (understand-anything-plugin/packages/dashboard/src/components/KnowledgeGraphView.tsx:283), summary (understand-anything-plugin/packages/dashboard/src/components/KnowledgeGraphView.tsx:282), tags (understand-anything-plugin/packages/dashboard/src/components/KnowledgeGraphView.tsx:295)
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
// understand-anything-plugin/packages/core/src/types.ts:111
  nodes: GraphNode[];
  edges: GraphEdge[];
  layers: Layer[];
```
```
// understand-anything-plugin/packages/dashboard/src/components/KnowledgeGraphView.tsx:283
        nodeType: node.type,
        summary: node.summary,
        complexity: node.complexity,
        isHighlighted,
```
```
// understand-anything-plugin/packages/dashboard/src/components/KnowledgeGraphView.tsx:282
        label: node.name,
        nodeType: node.type,
        summary: node.summary,
        complexity: node.complexity,
```
```
// understand-anything-plugin/packages/dashboard/src/components/KnowledgeGraphView.tsx:295
        onNodeClick,
        incomingCount: edgeCounts.get(node.id) ?? 0,
        tags: node.tags,
      };
```
Judgment (TRUE / NOT TRUE, one sentence):

### Egonex-AI__Understand-Anything 5: code:types.ts->NodeInfo.tsx (judge only: DomainMeta)
- E2 DomainMeta declared at understand-anything-plugin/packages/core/src/types.ts:32
- E4 DomainMeta not imported by name; resolved via property GraphNode.domainMeta declared in understand-anything-plugin/packages/core/src/types.ts:64, GraphNode imported at understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:4 from "@understand-anything/core/types"
- E5 DomainMeta fields read: businessRules (understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:181), crossDomainInteractions (understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:191), entities (understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:171), entryPoint (understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:231)
- E5 DomainMeta stoplisted reads: (none)
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
```
```
// understand-anything-plugin/packages/core/src/types.ts:64
  domainMeta?: DomainMeta;
  knowledgeMeta?: KnowledgeMeta;
  figmaMeta?: FigmaMeta;
```
```
// understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:181
          </div>
        ) : null}
        {Array.isArray(meta?.businessRules) && meta.businessRules.length > 0 ? (
          <div>
```
```
// understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:191
          </div>
        ) : null}
        {Array.isArray(meta?.crossDomainInteractions) && meta.crossDomainInteractions.length > 0 ? (
          <div>
```
```
// understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:171
    return (
      <div className="space-y-3">
        {Array.isArray(meta?.entities) && meta.entities.length > 0 ? (
          <div>
```
```
// understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:231
    return (
      <div className="space-y-3">
        {meta?.entryPoint ? (
          <div>
```
Judgment (TRUE / NOT TRUE, one sentence):

### Egonex-AI__Understand-Anything 6: code:types.ts->NodeInfo.tsx (judge only: GraphEdge)
- E2 GraphEdge declared at understand-anything-plugin/packages/core/src/types.ts:70
- E4 GraphEdge not imported by name; resolved via property KnowledgeGraph.edges declared in understand-anything-plugin/packages/core/src/types.ts:112, KnowledgeGraph imported at understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:4 from "@understand-anything/core/types"
- E5 GraphEdge fields read: source (understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:72), target (understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:73)
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
// understand-anything-plugin/packages/core/src/types.ts:112
  edges: GraphEdge[];
  layers: Layer[];
  tour: TourStep[];
```
```
// understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:72
  // Wikilinks (outgoing related edges)
  const wikilinks = graph.edges
    .filter((e) => e.type === "related" && e.source === node.id)
    .map((e) => graph.nodes.find((n) => n.id === e.target))
```
```
// understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:73
  const wikilinks = graph.edges
    .filter((e) => e.type === "related" && e.source === node.id)
    .map((e) => graph.nodes.find((n) => n.id === e.target))
    .filter((n): n is GraphNode => n !== undefined);
```
Judgment (TRUE / NOT TRUE, one sentence):

### Egonex-AI__Understand-Anything 7: code:types.ts->NodeInfo.tsx (judge only: KnowledgeMeta)
- E2 KnowledgeMeta declared at understand-anything-plugin/packages/core/src/types.ts:24
- E4 KnowledgeMeta not imported by name; resolved via property GraphNode.knowledgeMeta declared in understand-anything-plugin/packages/core/src/types.ts:65, GraphNode imported at understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:4 from "@understand-anything/core/types"
- E5 KnowledgeMeta fields read: content (understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:142), wikilinks (understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:104)
- E5 KnowledgeMeta stoplisted reads: (none)
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
```
```
// understand-anything-plugin/packages/core/src/types.ts:65
  knowledgeMeta?: KnowledgeMeta;
  figmaMeta?: FigmaMeta;
}
```
```
// understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:142
        </div>
      )}
      {meta?.content && (
        <div>
```
```
// understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:104
        </div>
      )}
      {meta?.wikilinks && meta.wikilinks.length > 0 && (
        <div>
```
Judgment (TRUE / NOT TRUE, one sentence):

### Egonex-AI__Understand-Anything 8: code:types.ts->context-builder.ts (judge only: GraphEdge)
- E2 GraphEdge declared at understand-anything-plugin/packages/core/src/types.ts:70
- E4 GraphEdge imported at understand-anything-plugin/src/context-builder.ts:2 from "@understand-anything/core", resolved to understand-anything-plugin/packages/core/src/index.ts, re-exported by barrel understand-anything-plugin/packages/core/src/index.ts:1 from understand-anything-plugin/packages/core/src/types.ts; receiver resolved via property ChatContext.relevantEdges declared in understand-anything-plugin/src/context-builder.ts:15 and KnowledgeGraph.edges declared in understand-anything-plugin/packages/core/src/types.ts:112
- E5 GraphEdge fields read: source (understand-anything-plugin/src/context-builder.ts:42), target (understand-anything-plugin/src/context-builder.ts:43)
- E5 GraphEdge stoplisted reads: description
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
// understand-anything-plugin/src/context-builder.ts:15
  relevantEdges: GraphEdge[];
  relevantLayers: Layer[];
  query: string;
```
```
// understand-anything-plugin/packages/core/src/types.ts:112
  edges: GraphEdge[];
  layers: Layer[];
  tour: TourStep[];
```
```
// understand-anything-plugin/src/context-builder.ts:42
  const expandedIds = new Set(matchedIds);
  for (const edge of graph.edges) {
    if (matchedIds.has(edge.source)) {
      expandedIds.add(edge.target);
```
```
// understand-anything-plugin/src/context-builder.ts:43
  for (const edge of graph.edges) {
    if (matchedIds.has(edge.source)) {
      expandedIds.add(edge.target);
    }
```
Judgment (TRUE / NOT TRUE, one sentence):

### Egonex-AI__Understand-Anything 9: code:types.ts->context-builder.ts (judge only: GraphNode)
- E2 GraphNode declared at understand-anything-plugin/packages/core/src/types.ts:54
- E4 GraphNode imported at understand-anything-plugin/src/context-builder.ts:2 from "@understand-anything/core", resolved to understand-anything-plugin/packages/core/src/index.ts, re-exported by barrel understand-anything-plugin/packages/core/src/index.ts:1 from understand-anything-plugin/packages/core/src/types.ts; receiver resolved via property ChatContext.relevantNodes declared in understand-anything-plugin/src/context-builder.ts:14
- E5 GraphNode fields read: filePath (understand-anything-plugin/src/context-builder.ts:114), languageNotes (understand-anything-plugin/src/context-builder.ts:122), tags (understand-anything-plugin/src/context-builder.ts:119)
- E5 GraphNode stoplisted reads: id
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
// understand-anything-plugin/src/context-builder.ts:14
  relevantNodes: GraphNode[];
  relevantEdges: GraphEdge[];
  relevantLayers: Layer[];
```
```
// understand-anything-plugin/src/context-builder.ts:114
    for (const node of context.relevantNodes) {
      lines.push(`### ${node.name} (${node.type})`);
      if (node.filePath) {
        lines.push(`- **File:** ${node.filePath}`);
```
```
// understand-anything-plugin/src/context-builder.ts:122
        lines.push(`- **Tags:** ${node.tags.join(", ")}`);
      }
      if (node.languageNotes) {
        lines.push(`- **Language Notes:** ${node.languageNotes}`);
```
```
// understand-anything-plugin/src/context-builder.ts:119
      lines.push(`- **Complexity:** ${node.complexity}`);
      lines.push(`- **Summary:** ${node.summary}`);
      if (node.tags.length > 0) {
        lines.push(`- **Tags:** ${node.tags.join(", ")}`);
```
Judgment (TRUE / NOT TRUE, one sentence):

### Egonex-AI__Understand-Anything 10: code:types.ts->context-builder.ts (judge only: ProjectMeta)
- E2 ProjectMeta declared at understand-anything-plugin/packages/core/src/types.ts:97
- E4 ProjectMeta not imported by name; resolved via property KnowledgeGraph.project declared in understand-anything-plugin/packages/core/src/types.ts:110, KnowledgeGraph imported at understand-anything-plugin/src/context-builder.ts:2 from "@understand-anything/core"
- E5 ProjectMeta fields read: frameworks (understand-anything-plugin/src/context-builder.ts:74), languages (understand-anything-plugin/src/context-builder.ts:73)
- E5 ProjectMeta stoplisted reads: description, name
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
```
```
// understand-anything-plugin/packages/core/src/types.ts:110
  project: ProjectMeta;
  nodes: GraphNode[];
  edges: GraphEdge[];
```
```
// understand-anything-plugin/src/context-builder.ts:74
    projectDescription: graph.project.description,
    languages: graph.project.languages,
    frameworks: graph.project.frameworks,
    relevantNodes,
```
```
// understand-anything-plugin/src/context-builder.ts:73
    projectName: graph.project.name,
    projectDescription: graph.project.description,
    languages: graph.project.languages,
    frameworks: graph.project.frameworks,
```
Judgment (TRUE / NOT TRUE, one sentence):

### Egonex-AI__Understand-Anything 11: code:types.ts->diff-analyzer.ts (judge only: GraphEdge)
- E2 GraphEdge declared at understand-anything-plugin/packages/core/src/types.ts:70
- E4 GraphEdge imported at understand-anything-plugin/src/diff-analyzer.ts:1 from "@understand-anything/core", resolved to understand-anything-plugin/packages/core/src/index.ts, re-exported by barrel understand-anything-plugin/packages/core/src/index.ts:1 from understand-anything-plugin/packages/core/src/types.ts; receiver resolved via property DiffContext.impactedEdges declared in understand-anything-plugin/src/diff-analyzer.ts:13
- E5 GraphEdge fields read: source (understand-anything-plugin/src/diff-analyzer.ts:46), target (understand-anything-plugin/src/diff-analyzer.ts:47)
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
// understand-anything-plugin/src/diff-analyzer.ts:13
  impactedEdges: GraphEdge[];
  affectedLayers: Layer[];
  unmappedFiles: string[];
```
```
// understand-anything-plugin/src/diff-analyzer.ts:46
  // Also include "contains" children of changed file nodes
  for (const edge of edges) {
    if (edge.type === "contains" && changedNodeIds.has(edge.source)) {
      changedNodeIds.add(edge.target);
```
```
// understand-anything-plugin/src/diff-analyzer.ts:47
  for (const edge of edges) {
    if (edge.type === "contains" && changedNodeIds.has(edge.source)) {
      changedNodeIds.add(edge.target);
    }
```
Judgment (TRUE / NOT TRUE, one sentence):

### Egonex-AI__Understand-Anything 12: code:types.ts->diff-analyzer.ts (judge only: GraphNode)
- E2 GraphNode declared at understand-anything-plugin/packages/core/src/types.ts:54
- E4 GraphNode imported at understand-anything-plugin/src/diff-analyzer.ts:1 from "@understand-anything/core", resolved to understand-anything-plugin/packages/core/src/index.ts, re-exported by barrel understand-anything-plugin/packages/core/src/index.ts:1 from understand-anything-plugin/packages/core/src/types.ts; receiver resolved via property DiffContext.affectedNodes declared in understand-anything-plugin/src/diff-analyzer.ts:12 and DiffContext.changedNodes declared in understand-anything-plugin/src/diff-analyzer.ts:11
- E5 GraphNode fields read: complexity (understand-anything-plugin/src/diff-analyzer.ts:161), filePath (understand-anything-plugin/src/diff-analyzer.ts:34)
- E5 GraphNode stoplisted reads: id
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
// understand-anything-plugin/src/diff-analyzer.ts:12
  affectedNodes: GraphNode[];
  impactedEdges: GraphEdge[];
  affectedLayers: Layer[];
```
```
// understand-anything-plugin/src/diff-analyzer.ts:11
  changedNodes: GraphNode[];
  affectedNodes: GraphNode[];
  impactedEdges: GraphEdge[];
```
```
// understand-anything-plugin/src/diff-analyzer.ts:161
  lines.push("");
  const complexChanges = ctx.changedNodes.filter(
    (n) => n.complexity === "complex",
  );
```
```
// understand-anything-plugin/src/diff-analyzer.ts:34
    let mapped = false;
    for (const node of nodes) {
      if (node.filePath === file) {
        changedNodeIds.add(node.id);
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 13: code:types.ts->nested-tool-calls.ts (judge only: AgentToolResult)
- E2 AgentToolResult declared at packages/agent/src/types.ts:424
- E4 AgentToolResult imported at packages/coding-agent/src/core/nested-tool-calls.ts:11 from "@earendil-works/pi-agent-core", resolved to packages/agent/src/index.ts, re-exported by barrel packages/agent/src/index.ts:5 from packages/agent/src/types.ts; receiver resolved via property AgentToolCallOutcome.result declared in packages/agent/src/types.ts:451
- E5 AgentToolResult fields read: content (packages/coding-agent/src/core/nested-tool-calls.ts:155), usage (packages/coding-agent/src/core/nested-tool-calls.ts:239)
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
// packages/agent/src/types.ts:451
	result: AgentToolResult<any>;
	isError: boolean;
	/** Milliseconds `execute()` took, measured with a monotonic clock; absent when the tool did not run. */
```
```
// packages/coding-agent/src/core/nested-tool-calls.ts:155

function textOf(result: AgentToolResult<unknown>): string {
	return (result.content ?? [])
		.filter((block): block is TextContent => block.type === "text")
```
```
// packages/coding-agent/src/core/nested-tool-calls.ts:239
		scope.recorder.finish(record, outcome.isError, textOf(outcome.result));
		// Nested results are not persisted, so their usage is only counted through the recorder.
		if (outcome.result.usage) scope.recorder.addUsage(outcome.result.usage);
		await this.host.emit({
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 14: code:types.ts->auth-command.ts (judge only: ModelAuth)
- E2 ModelAuth declared at packages/ai/src/auth/types.ts:7
- E4 ModelAuth not imported by name; resolved via property AuthResult.auth declared in packages/ai/src/auth/types.ts:105, AuthResult imported at packages/coding-agent/src/cli/auth-command.ts:1 from "@earendil-works/pi-ai"
- E5 ModelAuth fields read: apiKey (packages/coding-agent/src/cli/auth-command.ts:121), headers (packages/coding-agent/src/cli/auth-command.ts:122)
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
// packages/ai/src/auth/types.ts:105
	auth: ModelAuth;
	/** Provider-scoped environment/config values resolved from credentials and ambient context. */
	env?: ProviderEnv;
```
```
// packages/coding-agent/src/cli/auth-command.ts:121

export function getAuthCredential(auth: AuthResult | undefined): string | undefined {
	if (auth?.auth.apiKey) return auth.auth.apiKey;
	const authorization = Object.entries(auth?.auth.headers ?? {}).find(
```
```
// packages/coding-agent/src/cli/auth-command.ts:122
export function getAuthCredential(auth: AuthResult | undefined): string | undefined {
	if (auth?.auth.apiKey) return auth.auth.apiKey;
	const authorization = Object.entries(auth?.auth.headers ?? {}).find(
		([name]) => name.toLowerCase() === "authorization",
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 15: code:types.ts->agent-session.ts (judge only: ModelAuth)
- E2 ModelAuth declared at packages/ai/src/auth/types.ts:7
- E4 ModelAuth not imported by name; resolved via property AuthResult.auth declared in packages/ai/src/auth/types.ts:105, AuthResult imported at packages/coding-agent/src/core/agent-session.ts:35 from "@earendil-works/pi-ai/compat"
- E5 ModelAuth fields read: apiKey (packages/coding-agent/src/core/agent-session.ts:552), baseUrl (packages/coding-agent/src/core/agent-session.ts:553), headers (packages/coding-agent/src/core/agent-session.ts:552)
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
// packages/ai/src/auth/types.ts:105
	auth: ModelAuth;
	/** Provider-scoped environment/config values resolved from credentials and ambient context. */
	env?: ProviderEnv;
```
```
// packages/coding-agent/src/core/agent-session.ts:552
			throw error;
		}
		if (result && (result.auth.apiKey || result.auth.headers)) {
			const requestModel = result.auth.baseUrl ? { ...model, baseUrl: result.auth.baseUrl } : model;
```
```
// packages/coding-agent/src/core/agent-session.ts:553
		}
		if (result && (result.auth.apiKey || result.auth.headers)) {
			const requestModel = result.auth.baseUrl ? { ...model, baseUrl: result.auth.baseUrl } : model;
			return {
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 16: code:types.ts->bug-report.ts#2 (judge only: ProviderAuth)
- E2 ProviderAuth declared at packages/ai/src/auth/types.ts:252
- E4 ProviderAuth not imported by name; resolved via property Provider.auth declared in packages/ai/src/models.ts:169, Provider imported at packages/coding-agent/src/core/bug-report.ts:4 from "@earendil-works/pi-ai/compat"
- E5 ProviderAuth fields read: apiKey (packages/coding-agent/src/core/bug-report.ts:113), oauth (packages/coding-agent/src/core/bug-report.ts:114)
- E5 ProviderAuth stoplisted reads: (none)
```
// packages/ai/src/auth/types.ts:252
export interface ProviderAuth {
	apiKey?: ApiKeyAuth;
	oauth?: OAuthAuth;
}
```
```
// packages/ai/src/models.ts:169
	readonly auth: ProviderAuth;

	/**
```
```
// packages/coding-agent/src/core/bug-report.ts:113
function describeProvider(modelRuntime: ModelRuntime, provider: Provider) {
	const authTypes: Array<"api_key" | "oauth"> = [];
	if (provider.auth.apiKey) authTypes.push("api_key");
	if (provider.auth.oauth) authTypes.push("oauth");
```
```
// packages/coding-agent/src/core/bug-report.ts:114
	const authTypes: Array<"api_key" | "oauth"> = [];
	if (provider.auth.apiKey) authTypes.push("api_key");
	if (provider.auth.oauth) authTypes.push("oauth");
	return {
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 17: code:types.ts->model-runtime.ts (judge only: ModelAuth)
- E2 ModelAuth declared at packages/ai/src/auth/types.ts:7
- E4 ModelAuth not imported by name; resolved via property AuthResult.auth declared in packages/ai/src/auth/types.ts:105, AuthResult imported at packages/coding-agent/src/core/model-runtime.ts:2 from "@earendil-works/pi-ai"
- E5 ModelAuth fields read: apiKey (packages/coding-agent/src/core/model-runtime.ts:692), baseUrl (packages/coding-agent/src/core/model-runtime.ts:686), headers (packages/coding-agent/src/core/model-runtime.ts:574)
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
// packages/ai/src/auth/types.ts:105
	auth: ModelAuth;
	/** Provider-scoped environment/config values resolved from credentials and ambient context. */
	env?: ProviderEnv;
```
```
// packages/coding-agent/src/core/model-runtime.ts:692
			options: {
				...providerOptions,
				apiKey: providerOptions.apiKey ?? resolution.auth.apiKey,
				headers,
```
```
// packages/coding-agent/src/core/model-runtime.ts:686
				? { ...(resolution.env ?? {}), ...(providerOptions.env ?? {}) }
				: undefined;
		const requestModel: TModel = resolution.auth.baseUrl ? { ...model, baseUrl: resolution.auth.baseUrl } : model;
		return {
```
```
// packages/coding-agent/src/core/model-runtime.ts:574
			auth: {
				...resolution.auth,
				headers: mergeHeaders(resolution.auth.headers, configuredHeaders),
			},
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 18: code:types.ts->model-runtime.ts (judge only: ProviderAuth)
- E2 ProviderAuth declared at packages/ai/src/auth/types.ts:252
- E4 ProviderAuth not imported by name; resolved via property Provider.auth declared in packages/ai/src/models.ts:169, Provider imported at packages/coding-agent/src/core/model-runtime.ts:2 from "@earendil-works/pi-ai"
- E5 ProviderAuth fields read: apiKey (packages/coding-agent/src/core/model-runtime.ts:900), oauth (packages/coding-agent/src/core/model-runtime.ts:900)
- E5 ProviderAuth stoplisted reads: (none)
```
// packages/ai/src/auth/types.ts:252
export interface ProviderAuth {
	apiKey?: ApiKeyAuth;
	oauth?: OAuthAuth;
}
```
```
// packages/ai/src/models.ts:169
	readonly auth: ProviderAuth;

	/**
```
```
// packages/coding-agent/src/core/model-runtime.ts:900
			provider.id,
			configuredRequestAuthStatus(this.config.getProvider(provider.id), undefined),
			provider.auth.oauth && !provider.auth.apiKey ? "oauth" : "api_key",
		);
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 19: code:types.ts->provider-composer.ts (judge only: ProviderAuth)
- E2 ProviderAuth declared at packages/ai/src/auth/types.ts:252
- E4 ProviderAuth not imported by name; resolved via property Provider.auth declared in packages/ai/src/models.ts:169, Provider imported at packages/coding-agent/src/core/provider-composer.ts:1 from "@earendil-works/pi-ai"
- E5 ProviderAuth fields read: apiKey (packages/coding-agent/src/core/provider-composer.ts:423), oauth (packages/coding-agent/src/core/provider-composer.ts:425)
- E5 ProviderAuth stoplisted reads: (none)
```
// packages/ai/src/auth/types.ts:252
export interface ProviderAuth {
	apiKey?: ApiKeyAuth;
	oauth?: OAuthAuth;
}
```
```
// packages/ai/src/models.ts:169
	readonly auth: ProviderAuth;

	/**
```
```
// packages/coding-agent/src/core/provider-composer.ts:423
	extension: ProviderConfigInput | undefined,
): ApiKeyAuth | undefined {
	const inherited = base?.auth.apiKey;
	const rawKey = configuredApiKey(config, extension);
```
```
// packages/coding-agent/src/core/provider-composer.ts:425
	const inherited = base?.auth.apiKey;
	const rawKey = configuredApiKey(config, extension);
	const oauth = extension?.oauth ?? base?.auth.oauth;
	// OAuth-only providers get no fabricated API-key login method.
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 20: code:faux.ts->harness-generation-recovery.test.ts (judge only: FauxProviderHandle)
- E2 FauxProviderHandle declared at packages/ai/src/providers/faux.ts:146
- E4 FauxProviderHandle not imported by name; resolved via property ChatSetup.faux declared in packages/durable/test/chat-support.ts:27, ChatSetup imported at packages/durable/test/harness-generation-recovery.test.ts:9 from "./chat-support.ts"
- E5 FauxProviderHandle fields read: setResponses (packages/durable/test/harness-generation-recovery.test.ts:55), state (packages/durable/test/harness-generation-recovery.test.ts:196)
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
// packages/durable/test/chat-support.ts:27
	readonly faux: FauxProviderHandle;
	readonly models: Models;
	readonly registry: Registry;
```
```
// packages/durable/test/harness-generation-recovery.test.ts:55
			return "p";
		});
		setup.faux.setResponses([fauxAssistantMessage("answer")]);
		let opened = await open(path, setup);
```
```
// packages/durable/test/harness-generation-recovery.test.ts:196
		opened.harness.resume();
		await new Promise((resolve) => setTimeout(resolve, 20));
		expect(setup.faux.state.callCount).toBe(1);
		expect(await opened.harness.snapshot(LiveDoc, opened.root.id, context)).toEqual({
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 21: code:faux.ts->harness-generation-recovery.test.ts (judge only: FauxProviderState)
- E2 FauxProviderState declared at packages/ai/src/providers/faux.ts:103
- E4 FauxProviderState not imported by name; resolved via property ChatSetup.faux.state declared in packages/ai/src/providers/faux.ts:152, ChatSetup imported at packages/durable/test/harness-generation-recovery.test.ts:9 from "./chat-support.ts"
- E5 FauxProviderState fields read: callCount (packages/durable/test/harness-generation-recovery.test.ts:196), deferredFetchCount (packages/durable/test/harness-generation-recovery.test.ts:286)
- E5 FauxProviderState stoplisted reads: (none)
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
```
```
// packages/ai/src/providers/faux.ts:152
	state: FauxProviderState;
	setResponses: (responses: FauxResponseStep[]) => void;
	appendResponses: (responses: FauxResponseStep[]) => void;
```
```
// packages/durable/test/harness-generation-recovery.test.ts:196
		opened.harness.resume();
		await new Promise((resolve) => setTimeout(resolve, 20));
		expect(setup.faux.state.callCount).toBe(1);
		expect(await opened.harness.snapshot(LiveDoc, opened.root.id, context)).toEqual({
```
```
// packages/durable/test/harness-generation-recovery.test.ts:286
		const answer = await opened.root.commit((tx) => tx.entry(AssistantEntry, settled.answer), context);
		expect(textOf(answer?.model?.[0])).toBe("deferred answer");
		expect(setup.faux.state.deferredFetchCount).toBe(1);
		await opened.harness.close(context);
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 22: code:faux.ts->harness-generation.test.ts (judge only: FauxProviderHandle)
- E2 FauxProviderHandle declared at packages/ai/src/providers/faux.ts:146
- E4 FauxProviderHandle not imported by name; resolved via property ChatSetup.faux declared in packages/durable/test/chat-support.ts:27, ChatSetup imported at packages/durable/test/harness-generation.test.ts:31 from "./chat-support.ts"
- E5 FauxProviderHandle fields read: getPendingResponseCount (packages/durable/test/harness-generation.test.ts:215), setResponses (packages/durable/test/harness-generation.test.ts:92), state (packages/durable/test/harness-generation.test.ts:227)
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
// packages/durable/test/chat-support.ts:27
	readonly faux: FauxProviderHandle;
	readonly models: Models;
	readonly registry: Registry;
```
```
// packages/durable/test/harness-generation.test.ts:215
		expect(settled).toMatchObject({ status: "unanswered", reason: "model_error", detail: "503 Service Unavailable" });
		expect((await allEntries(root)).map((entry) => entry.kind)).toEqual(["pi.user", "pi.assistant", "pi.assistant"]);
		expect(setup.faux.getPendingResponseCount()).toBe(1);
		await harness.close(context);
```
```
// packages/durable/test/harness-generation.test.ts:92
		const setup = chatSetup();
		addSection(setup.registry, "preamble", () => "You are helpful.", { tag: false });
		setup.faux.setResponses([fauxAssistantMessage("Hello there")]);
		const { harness, root } = await openChat(new MemoryStorage(), setup);
```
```
// packages/durable/test/harness-generation.test.ts:227
		const settled = await (await root.submit({ type: "input", content: "hi" }, context)).wait(context);
		expect(settled).toMatchObject({ status: "unanswered", reason: "model_error" });
		expect(setup.faux.state.callCount).toBe(1);
		await harness.close(context);
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 23: code:faux.ts->harness-generation.test.ts (judge only: FauxProviderState)
- E2 FauxProviderState declared at packages/ai/src/providers/faux.ts:103
- E4 FauxProviderState not imported by name; resolved via property ChatSetup.faux.state declared in packages/ai/src/providers/faux.ts:152, ChatSetup imported at packages/durable/test/harness-generation.test.ts:31 from "./chat-support.ts"
- E5 FauxProviderState fields read: callCount (packages/durable/test/harness-generation.test.ts:227), cancelledDeferred (packages/durable/test/harness-generation.test.ts:323), deferredFetchCount (packages/durable/test/harness-generation.test.ts:280)
- E5 FauxProviderState stoplisted reads: (none)
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
```
```
// packages/ai/src/providers/faux.ts:152
	state: FauxProviderState;
	setResponses: (responses: FauxResponseStep[]) => void;
	appendResponses: (responses: FauxResponseStep[]) => void;
```
```
// packages/durable/test/harness-generation.test.ts:227
		const settled = await (await root.submit({ type: "input", content: "hi" }, context)).wait(context);
		expect(settled).toMatchObject({ status: "unanswered", reason: "model_error" });
		expect(setup.faux.state.callCount).toBe(1);
		await harness.close(context);
```
```
// packages/durable/test/harness-generation.test.ts:323
		await harness.abortTask(taskId, context);
		expect(await submission.wait(context)).toMatchObject({ status: "unanswered", reason: "aborted" });
		expect(setup.faux.state.cancelledDeferred).toHaveLength(1);
		expect(await live(harness, root)).toEqual({});
```
```
// packages/durable/test/harness-generation.test.ts:280
		const settled = await (await root.submit({ type: "input", content: "hi" }, context)).wait(context);
		if (settled.status !== "done" || settled.type !== "input") throw new Error(`Unexpected ${settled.status}`);
		expect(setup.faux.state.deferredFetchCount).toBe(2);
		const pollTimes = values.flatMap((value) =>
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 24: code:faux.ts->harness-inbox.test.ts (judge only: FauxProviderHandle)
- E2 FauxProviderHandle declared at packages/ai/src/providers/faux.ts:146
- E4 FauxProviderHandle not imported by name; resolved via property ChatSetup.faux declared in packages/durable/test/chat-support.ts:27, ChatSetup imported at packages/durable/test/harness-inbox.test.ts:36 from "./chat-support.ts"
- E5 FauxProviderHandle fields read: setResponses (packages/durable/test/harness-inbox.test.ts:114), state (packages/durable/test/harness-inbox.test.ts:170)
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
// packages/durable/test/chat-support.ts:27
	readonly faux: FauxProviderHandle;
	readonly models: Models;
	readonly registry: Registry;
```
```
// packages/durable/test/harness-inbox.test.ts:114
		const setup = chatSetup();
		const first = gated(answer("first"));
		setup.faux.setResponses([first.step, answer("second"), answer("third")]);
		const { harness, root } = await openChat(new MemoryStorage(), setup);
```
```
// packages/durable/test/harness-inbox.test.ts:170
			"pi.assistant:both",
		]);
		expect(setup.faux.state.callCount).toBe(2);
		await harness.close(context);
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 25: code:faux.ts->harness-input-entries.test.ts (judge only: FauxProviderHandle)
- E2 FauxProviderHandle declared at packages/ai/src/providers/faux.ts:146
- E4 FauxProviderHandle not imported by name; resolved via property ChatSetup.faux declared in packages/durable/test/chat-support.ts:27, ChatSetup imported at packages/durable/test/harness-input-entries.test.ts:29 from "./chat-support.ts"
- E5 FauxProviderHandle fields read: setResponses (packages/durable/test/harness-input-entries.test.ts:103), state (packages/durable/test/harness-input-entries.test.ts:378)
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
// packages/durable/test/chat-support.ts:27
	readonly faux: FauxProviderHandle;
	readonly models: Models;
	readonly registry: Registry;
```
```
// packages/durable/test/harness-input-entries.test.ts:103
		setup.now = () => 42;
		const requests: string[][] = [];
		setup.faux.setResponses([requestLog(requests, answer("hi ann"))]);
		const { harness, root } = await openChat(new MemoryStorage(), setup);
```
```
// packages/durable/test/harness-input-entries.test.ts:378
		expect((await inboxItems(harness, root)).map((item) => item.id)).toEqual([f.id]);
		expect(await harness.snapshot(LiveDoc, root.id, context)).toEqual({});
		expect(setup.faux.state.callCount).toBe(1);
		// The next input places the waiting mention first.
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 26: code:faux.ts->harness-tools.test.ts (judge only: FauxProviderHandle)
- E2 FauxProviderHandle declared at packages/ai/src/providers/faux.ts:146
- E4 FauxProviderHandle not imported by name; resolved via property ChatSetup.faux declared in packages/durable/test/chat-support.ts:27, ChatSetup imported at packages/durable/test/harness-tools.test.ts:37 from "./chat-support.ts"
- E5 FauxProviderHandle fields read: setResponses (packages/durable/test/harness-tools.test.ts:66), state (packages/durable/test/harness-tools.test.ts:109)
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
// packages/durable/test/chat-support.ts:27
	readonly faux: FauxProviderHandle;
	readonly models: Models;
	readonly registry: Registry;
```
```
// packages/durable/test/harness-tools.test.ts:66
	options: Parameters<typeof openChat>[2] = {},
): Promise<{ harness: Harness; root: Conversation; entries: EntryRecord[]; status: string }> {
	setup.faux.setResponses(responses);
	const { harness, root } = await openChat(new MemoryStorage(), setup, options);
```
```
// packages/durable/test/harness-tools.test.ts:109
		expect(entries[3]!.byTaskId).toBeDefined();
		// The second request sees the tool result right after its call.
		expect(setup.faux.state.callCount).toBe(2);
		expect(await harness.snapshot(LiveDoc, root.id, context)).toEqual({});
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 27: code:types.ts->custom-footer.ts (judge only: Usage)
- E2 Usage declared at packages/ai/src/types.ts:451
- E4 Usage not imported by name; resolved via property AssistantMessage.usage declared in packages/ai/src/types.ts:583, AssistantMessage imported at packages/coding-agent/examples/extensions/custom-footer.ts:11 from "@earendil-works/pi-ai"
- E5 Usage fields read: cost (packages/coding-agent/examples/extensions/custom-footer.ts:40), input (packages/coding-agent/examples/extensions/custom-footer.ts:38), output (packages/coding-agent/examples/extensions/custom-footer.ts:39)
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
// packages/ai/src/types.ts:583
	usage: Usage;
	stopReason: StopReason;
	deferred?: DeferredHandle;
```
```
// packages/coding-agent/examples/extensions/custom-footer.ts:40
									input += m.usage.input;
									output += m.usage.output;
									cost += m.usage.cost.total;
								}
```
```
// packages/coding-agent/examples/extensions/custom-footer.ts:38
								if (e.type === "message" && e.message.role === "assistant") {
									const m = e.message as AssistantMessage;
									input += m.usage.input;
									output += m.usage.output;
```
```
// packages/coding-agent/examples/extensions/custom-footer.ts:39
									const m = e.message as AssistantMessage;
									input += m.usage.input;
									output += m.usage.output;
									cost += m.usage.cost.total;
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 28: code:types.ts->index.ts#2 (judge only: Usage)
- E2 Usage declared at packages/ai/src/types.ts:451
- E4 Usage not imported by name; resolved via property AssistantMessage.usage declared in packages/ai/src/types.ts:583, AssistantMessage imported at packages/coding-agent/examples/extensions/custom-provider-anthropic/index.ts:26 from "@earendil-works/pi-ai"
- E5 Usage fields read: cacheRead (packages/coding-agent/examples/extensions/custom-provider-anthropic/index.ts:467), cacheWrite (packages/coding-agent/examples/extensions/custom-provider-anthropic/index.ts:467), input (packages/coding-agent/examples/extensions/custom-provider-anthropic/index.ts:467), output (packages/coding-agent/examples/extensions/custom-provider-anthropic/index.ts:467)
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
// packages/ai/src/types.ts:583
	usage: Usage;
	stopReason: StopReason;
	deferred?: DeferredHandle;
```
```
// packages/coding-agent/examples/extensions/custom-provider-anthropic/index.ts:467
					output.usage.cacheWrite = (event.message.usage as any).cache_creation_input_tokens || 0;
					output.usage.totalTokens =
						output.usage.input + output.usage.output + output.usage.cacheRead + output.usage.cacheWrite;
					calculateCost(model, output.usage);
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 29: code:types.ts->cache-stats.ts (judge only: Usage)
- E2 Usage declared at packages/ai/src/types.ts:451
- E4 Usage not imported by name; resolved via property AssistantMessage.usage declared in packages/ai/src/types.ts:583, AssistantMessage imported at packages/coding-agent/src/core/cache-stats.ts:1 from "@earendil-works/pi-ai"
- E5 Usage fields read: cacheRead (packages/coding-agent/src/core/cache-stats.ts:62), cacheWrite (packages/coding-agent/src/core/cache-stats.ts:62), cost (packages/coding-agent/src/core/cache-stats.ts:78), input (packages/coding-agent/src/core/cache-stats.ts:62)
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
// packages/ai/src/types.ts:583
	usage: Usage;
	stopReason: StopReason;
	deferred?: DeferredHandle;
```
```
// packages/coding-agent/src/core/cache-stats.ts:62
): CacheMiss | undefined {
	const usage = message.usage;
	const promptTokens = usage.input + usage.cacheRead + usage.cacheWrite;
	// A zero-cache turn only counts when cache activity was reported before:
```
```
// packages/coding-agent/src/core/cache-stats.ts:78
	// from this message's own cost breakdown.
	const paidTokens = usage.input + usage.cacheWrite;
	const paidPerToken = paidTokens > 0 ? (usage.cost.input + usage.cost.cacheWrite) / paidTokens : 0;
	const readPerToken =
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 30: code:types.ts->virtual-models.test.ts (judge only: AssistantMessage)
- E2 AssistantMessage declared at packages/ai/src/types.ts:570
- E4 AssistantMessage not imported by name; resolved via property ModelRouteRequest.failed.message declared in packages/coding-agent/src/core/virtual-models.ts:64, ModelRouteRequest imported at packages/coding-agent/test/suite/virtual-models.test.ts:6 from "../../src/core/virtual-models.ts"
- E5 AssistantMessage fields read: errorMessage (packages/coding-agent/test/suite/virtual-models.test.ts:93), stopReason (packages/coding-agent/test/suite/virtual-models.test.ts:146)
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
// packages/coding-agent/src/core/virtual-models.ts:64
	failed?: { model: Model<Api>; thinkingLevel?: ModelThinkingLevel; message: AssistantMessage };
	/** Router state last returned on this session branch. Undefined before the first state and for `direct` requests. */
	state?: TState;
```
```
// packages/coding-agent/test/suite/virtual-models.test.ts:93

		expect(reasons()).toEqual(["user", "retry", "continuation"]);
		expect(requests[1].failed?.message.errorMessage).toBe("overloaded_error");
		expect(requests[2].previous?.model.id).toBe("large");
```
```
// packages/coding-agent/test/suite/virtual-models.test.ts:146
		expect(reasons()).toEqual(["user", "retry"]);
		expect(requests[1].failed?.model.id).toBe("large");
		expect(requests[1].failed?.message.stopReason).toBe("length");
	});
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 31: code:types.ts->test-harness.test.ts (judge only: Usage)
- E2 Usage declared at packages/ai/src/types.ts:451
- E4 Usage not imported by name; resolved via property AssistantMessage.usage declared in packages/ai/src/types.ts:583, AssistantMessage imported at packages/coding-agent/test/test-harness.test.ts:7 from "@earendil-works/pi-ai"
- E5 Usage fields read: input (packages/coding-agent/test/test-harness.test.ts:131), output (packages/coding-agent/test/test-harness.test.ts:132)
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
// packages/ai/src/types.ts:583
	usage: Usage;
	stopReason: StopReason;
	deferred?: DeferredHandle;
```
```
// packages/coding-agent/test/test-harness.test.ts:131

		const msg = harness.session.messages.find((m): m is AssistantMessage => m.role === "assistant")!;
		expect(msg.usage.input).toBe(100000);
		expect(msg.usage.output).toBe(5000);
```
```
// packages/coding-agent/test/test-harness.test.ts:132
		const msg = harness.session.messages.find((m): m is AssistantMessage => m.role === "assistant")!;
		expect(msg.usage.input).toBe(100000);
		expect(msg.usage.output).toBe(5000);
	});
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 32: code:types.ts->harness-inbox.test.ts (judge only: Usage)
- E2 Usage declared at packages/ai/src/types.ts:451
- E4 Usage imported at packages/durable/test/harness-inbox.test.ts:5 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:37 from packages/ai/src/types.ts; receiver resolved via property AssistantMessage.usage declared in packages/ai/src/types.ts:583
- E5 Usage fields read: input (packages/durable/test/harness-inbox.test.ts:800), output (packages/durable/test/harness-inbox.test.ts:801), totalTokens (packages/durable/test/harness-inbox.test.ts:802)
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
// packages/ai/src/types.ts:583
	usage: Usage;
	stopReason: StopReason;
	deferred?: DeferredHandle;
```
```
// packages/durable/test/harness-inbox.test.ts:800
		const expected = assistants.reduce(
			(sum, message) => ({
				input: sum.input + message.usage.input,
				output: sum.output + message.usage.output,
```
```
// packages/durable/test/harness-inbox.test.ts:801
			(sum, message) => ({
				input: sum.input + message.usage.input,
				output: sum.output + message.usage.output,
				totalTokens: sum.totalTokens + message.usage.totalTokens,
```
```
// packages/durable/test/harness-inbox.test.ts:802
				input: sum.input + message.usage.input,
				output: sum.output + message.usage.output,
				totalTokens: sum.totalTokens + message.usage.totalTokens,
			}),
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 33: code:types.ts->system-order-cache-e2e.test.ts (judge only: Usage)
- E2 Usage declared at packages/ai/src/types.ts:451
- E4 Usage not imported by name; resolved via property AssistantMessage.usage declared in packages/ai/src/types.ts:583, AssistantMessage imported at packages/durable/test/system-order-cache-e2e.test.ts:1 from "@earendil-works/pi-ai"
- E5 Usage fields read: cacheRead (packages/durable/test/system-order-cache-e2e.test.ts:122), cacheWrite (packages/durable/test/system-order-cache-e2e.test.ts:122), input (packages/durable/test/system-order-cache-e2e.test.ts:122)
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
// packages/ai/src/types.ts:583
	usage: Usage;
	stopReason: StopReason;
	deferred?: DeferredHandle;
```
```
// packages/durable/test/system-order-cache-e2e.test.ts:122
		};
		const before = await ask("READY");
		expect(before.usage.input + before.usage.cacheRead + before.usage.cacheWrite).toBeGreaterThanOrEqual(4_000);
		await root.configure({ tools: [first, second] }, context);
```
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 34: code:renderElement.ts->staticScene.ts (judge only: RenderPositionOffset)
- E2 RenderPositionOffset declared at packages/element/src/renderElement.ts:118
- E4 RenderPositionOffset not imported by name; resolved via property ElementRenderState.offset declared in packages/element/src/renderElement.ts:154, ElementRenderState imported at packages/excalidraw/renderer/staticScene.ts:30 from "@excalidraw/element"
- E5 RenderPositionOffset fields read: x (packages/excalidraw/renderer/staticScene.ts:212), y (packages/excalidraw/renderer/staticScene.ts:213)
- E5 RenderPositionOffset stoplisted reads: (none)
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
// packages/element/src/renderElement.ts:154
  offset: RenderPositionOffset;
}>;

```
```
// packages/excalidraw/renderer/staticScene.ts:212
    const [x, y, width, height] = getLinkHandleFromCoords(
      [
        x1 + renderState.offset.x,
        y1 + renderState.offset.y,
```
```
// packages/excalidraw/renderer/staticScene.ts:213
      [
        x1 + renderState.offset.x,
        y1 + renderState.offset.y,
        x2 + renderState.offset.x,
```
Judgment (TRUE / NOT TRUE, one sentence):

