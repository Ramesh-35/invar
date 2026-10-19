## Egonex-AI__Understand-Anything: 13 proposal(s), first 13 adjudicated

### Egonex-AI__Understand-Anything 1: code:types.ts->DomainGraphView.tsx
- E2 GraphNode declared at understand-anything-plugin/packages/core/src/types.ts:54
- E2 KnowledgeGraph declared at understand-anything-plugin/packages/core/src/types.ts:107
- E4 GraphNode imported at understand-anything-plugin/packages/dashboard/src/components/DomainGraphView.tsx:23 from "@understand-anything/core/types", resolved to understand-anything-plugin/packages/core/src/types.ts
- E4 KnowledgeGraph imported at understand-anything-plugin/packages/dashboard/src/components/DomainGraphView.tsx:23 from "@understand-anything/core/types", resolved to understand-anything-plugin/packages/core/src/types.ts
- E5 GraphNode fields read: domainMeta (understand-anything-plugin/packages/dashboard/src/components/DomainGraphView.tsx:32), filePath (understand-anything-plugin/packages/dashboard/src/components/DomainGraphView.tsx:148), summary (understand-anything-plugin/packages/dashboard/src/components/DomainGraphView.tsx:57)
- E5 GraphNode stoplisted reads: id, name, type
- E5 KnowledgeGraph fields read: edges (understand-anything-plugin/packages/dashboard/src/components/DomainGraphView.tsx:47), nodes (understand-anything-plugin/packages/dashboard/src/components/DomainGraphView.tsx:43)
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

### Egonex-AI__Understand-Anything 2: code:types.ts->ExportMenu.tsx
- E2 KnowledgeGraph declared at understand-anything-plugin/packages/core/src/types.ts:107
- E4 KnowledgeGraph imported at understand-anything-plugin/packages/dashboard/src/components/ExportMenu.tsx:4 from "@understand-anything/core/types", resolved to understand-anything-plugin/packages/core/src/types.ts
- E5 KnowledgeGraph fields read: edges (understand-anything-plugin/packages/dashboard/src/components/ExportMenu.tsx:196), nodes (understand-anything-plugin/packages/dashboard/src/components/ExportMenu.tsx:190)
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
// understand-anything-plugin/packages/dashboard/src/components/ExportMenu.tsx:196

      let filteredGraphEdges = graph.edges.filter(
        (e) => filteredNodeIds.has(e.source) && filteredNodeIds.has(e.target)
```
```
// understand-anything-plugin/packages/dashboard/src/components/ExportMenu.tsx:190
      let filteredGraphNodes = persona === "non-technical"
        ? graph.nodes.filter((n) => !subFileTypes.has(n.type))
        : graph.nodes;
```
Judgment (TRUE / NOT TRUE, one sentence):

### Egonex-AI__Understand-Anything 3: code:types.ts->GraphView.tsx
- E2 GraphEdge declared at understand-anything-plugin/packages/core/src/types.ts:70
- E2 GraphNode declared at understand-anything-plugin/packages/core/src/types.ts:54
- E2 KnowledgeGraph declared at understand-anything-plugin/packages/core/src/types.ts:107
- E4 GraphEdge imported at understand-anything-plugin/packages/dashboard/src/components/GraphView.tsx:27 from "@understand-anything/core/types", resolved to understand-anything-plugin/packages/core/src/types.ts
- E4 GraphNode imported at understand-anything-plugin/packages/dashboard/src/components/GraphView.tsx:27 from "@understand-anything/core/types", resolved to understand-anything-plugin/packages/core/src/types.ts
- E4 KnowledgeGraph imported at understand-anything-plugin/packages/dashboard/src/components/GraphView.tsx:27 from "@understand-anything/core/types", resolved to understand-anything-plugin/packages/core/src/types.ts
- E5 GraphEdge fields read: source (understand-anything-plugin/packages/dashboard/src/components/GraphView.tsx:443), target (understand-anything-plugin/packages/dashboard/src/components/GraphView.tsx:444)
- E5 GraphEdge stoplisted reads: description, type
- E5 GraphNode fields read: complexity (understand-anything-plugin/packages/dashboard/src/components/GraphView.tsx:570), filePath (understand-anything-plugin/packages/dashboard/src/components/GraphView.tsx:567), summary (understand-anything-plugin/packages/dashboard/src/components/GraphView.tsx:569), tags (understand-anything-plugin/packages/dashboard/src/components/GraphView.tsx:571)
- E5 GraphNode stoplisted reads: id, name, type
- E5 KnowledgeGraph fields read: edges (understand-anything-plugin/packages/dashboard/src/components/GraphView.tsx:366), layers (understand-anything-plugin/packages/dashboard/src/components/GraphView.tsx:253), nodes (understand-anything-plugin/packages/dashboard/src/components/GraphView.tsx:458)
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
// understand-anything-plugin/packages/dashboard/src/components/GraphView.tsx:443
      for (const edge of graph.edges) {
        if (edge.type === "contains" && layerNodeIds.has(edge.source)) {
          const child = nodesById.get(edge.target);
```
```
// understand-anything-plugin/packages/dashboard/src/components/GraphView.tsx:444
        if (edge.type === "contains" && layerNodeIds.has(edge.source)) {
          const child = nodesById.get(edge.target);
          if (!child) continue;
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
Judgment (TRUE / NOT TRUE, one sentence):

### Egonex-AI__Understand-Anything 4: code:types.ts->KnowledgeGraphView.tsx
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

### Egonex-AI__Understand-Anything 5: code:types.ts->NodeInfo.tsx
- E2 GraphNode declared at understand-anything-plugin/packages/core/src/types.ts:54
- E2 KnowledgeGraph declared at understand-anything-plugin/packages/core/src/types.ts:107
- E4 GraphNode imported at understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:4 from "@understand-anything/core/types", resolved to understand-anything-plugin/packages/core/src/types.ts
- E4 KnowledgeGraph imported at understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:4 from "@understand-anything/core/types", resolved to understand-anything-plugin/packages/core/src/types.ts
- E5 GraphNode fields read: complexity (understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:332), domainMeta (understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:161), figmaMeta (understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:56), filePath (understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:261), knowledgeMeta (understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:68), languageNotes (understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:433), lineRange (understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:268), summary (understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:405), tags (understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:459)
- E5 GraphNode stoplisted reads: id, name, type
- E5 KnowledgeGraph fields read: edges (understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:71), nodes (understand-anything-plugin/packages/dashboard/src/components/NodeInfo.tsx:73)
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

### Egonex-AI__Understand-Anything 6: code:types.ts->store.ts
- E2 KnowledgeGraph declared at understand-anything-plugin/packages/core/src/types.ts:107
- E2 TourStep declared at understand-anything-plugin/packages/core/src/types.ts:88
- E4 KnowledgeGraph imported at understand-anything-plugin/packages/dashboard/src/store.ts:5 from "@understand-anything/core/types", resolved to understand-anything-plugin/packages/core/src/types.ts
- E4 TourStep imported at understand-anything-plugin/packages/dashboard/src/store.ts:5 from "@understand-anything/core/types", resolved to understand-anything-plugin/packages/core/src/types.ts
- E5 KnowledgeGraph fields read: layers (understand-anything-plugin/packages/dashboard/src/store.ts:84), nodes (understand-anything-plugin/packages/dashboard/src/store.ts:81), tour (understand-anything-plugin/packages/dashboard/src/store.ts:244)
- E5 KnowledgeGraph stoplisted reads: (none)
- E5 TourStep fields read: nodeIds (understand-anything-plugin/packages/dashboard/src/store.ts:85), order (understand-anything-plugin/packages/dashboard/src/store.ts:245)
- E5 TourStep stoplisted reads: (none)
```
// understand-anything-plugin/packages/core/src/types.ts:88
export interface TourStep {
  order: number;
  title: string;
  description: string;
  nodeIds: string[];
  languageLesson?: string;
}

// ProjectMeta
export interface ProjectMeta {
  name: string;
  languages: string[];
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
```
// understand-anything-plugin/packages/dashboard/src/store.ts:85
  for (const layer of graph.layers) {
    for (const nid of layer.nodeIds) {
      if (!nodeIdToLayerId.has(nid)) nodeIdToLayerId.set(nid, layer.id);
```
Judgment (TRUE / NOT TRUE, one sentence):

### Egonex-AI__Understand-Anything 7: code:types.ts->edgeAggregation.ts
- E2 GraphEdge declared at understand-anything-plugin/packages/core/src/types.ts:70
- E2 KnowledgeGraph declared at understand-anything-plugin/packages/core/src/types.ts:107
- E4 GraphEdge imported at understand-anything-plugin/packages/dashboard/src/utils/edgeAggregation.ts:1 from "@understand-anything/core/types", resolved to understand-anything-plugin/packages/core/src/types.ts
- E4 KnowledgeGraph imported at understand-anything-plugin/packages/dashboard/src/utils/edgeAggregation.ts:1 from "@understand-anything/core/types", resolved to understand-anything-plugin/packages/core/src/types.ts
- E5 GraphEdge fields read: source (understand-anything-plugin/packages/dashboard/src/utils/edgeAggregation.ts:38), target (understand-anything-plugin/packages/dashboard/src/utils/edgeAggregation.ts:39)
- E5 GraphEdge stoplisted reads: type
- E5 KnowledgeGraph fields read: edges (understand-anything-plugin/packages/dashboard/src/utils/edgeAggregation.ts:37), layers (understand-anything-plugin/packages/dashboard/src/utils/edgeAggregation.ts:25)
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
// understand-anything-plugin/packages/dashboard/src/utils/edgeAggregation.ts:38
  for (const edge of graph.edges) {
    const sourceLayer = nodeToLayer.get(edge.source);
    const targetLayer = nodeToLayer.get(edge.target);
```
```
// understand-anything-plugin/packages/dashboard/src/utils/edgeAggregation.ts:39
    const sourceLayer = nodeToLayer.get(edge.source);
    const targetLayer = nodeToLayer.get(edge.target);
    if (!sourceLayer || !targetLayer) continue;
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

### Egonex-AI__Understand-Anything 8: code:types.ts->filters.ts
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

### Egonex-AI__Understand-Anything 9: code:types.ts->louvain.ts
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

### Egonex-AI__Understand-Anything 10: code:types.ts->context-builder.test.ts
- E2 GraphEdge declared at understand-anything-plugin/packages/core/src/types.ts:70
- E4 GraphEdge imported at understand-anything-plugin/src/__tests__/context-builder.test.ts:3 from "@understand-anything/core", resolved to understand-anything-plugin/packages/core/src/index.ts, re-exported by barrel understand-anything-plugin/packages/core/src/index.ts:1 from understand-anything-plugin/packages/core/src/types.ts
- E5 GraphEdge fields read: source (understand-anything-plugin/src/__tests__/context-builder.test.ts:163), target (understand-anything-plugin/src/__tests__/context-builder.test.ts:163)
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
// understand-anything-plugin/src/__tests__/context-builder.test.ts:163
    const hasAuthToUser = ctx.relevantEdges.some(
      (e) => e.source === "auth-ctrl" && e.target === "user-model",
    );
```
Judgment (TRUE / NOT TRUE, one sentence):

### Egonex-AI__Understand-Anything 11: code:types.ts->context-builder.ts
- E2 GraphEdge declared at understand-anything-plugin/packages/core/src/types.ts:70
- E2 GraphNode declared at understand-anything-plugin/packages/core/src/types.ts:54
- E2 KnowledgeGraph declared at understand-anything-plugin/packages/core/src/types.ts:107
- E4 GraphEdge imported at understand-anything-plugin/src/context-builder.ts:2 from "@understand-anything/core", resolved to understand-anything-plugin/packages/core/src/index.ts, re-exported by barrel understand-anything-plugin/packages/core/src/index.ts:1 from understand-anything-plugin/packages/core/src/types.ts
- E4 GraphNode imported at understand-anything-plugin/src/context-builder.ts:2 from "@understand-anything/core", resolved to understand-anything-plugin/packages/core/src/index.ts, re-exported by barrel understand-anything-plugin/packages/core/src/index.ts:1 from understand-anything-plugin/packages/core/src/types.ts
- E4 KnowledgeGraph imported at understand-anything-plugin/src/context-builder.ts:2 from "@understand-anything/core", resolved to understand-anything-plugin/packages/core/src/index.ts, re-exported by barrel understand-anything-plugin/packages/core/src/index.ts:1 from understand-anything-plugin/packages/core/src/types.ts
- E5 GraphEdge fields read: source (understand-anything-plugin/src/context-builder.ts:42), target (understand-anything-plugin/src/context-builder.ts:43)
- E5 GraphEdge stoplisted reads: description
- E5 GraphNode fields read: filePath (understand-anything-plugin/src/context-builder.ts:114), languageNotes (understand-anything-plugin/src/context-builder.ts:122), tags (understand-anything-plugin/src/context-builder.ts:119)
- E5 GraphNode stoplisted reads: id, name
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
// understand-anything-plugin/src/context-builder.ts:42
  for (const edge of graph.edges) {
    if (matchedIds.has(edge.source)) {
      expandedIds.add(edge.target);
```
```
// understand-anything-plugin/src/context-builder.ts:43
    if (matchedIds.has(edge.source)) {
      expandedIds.add(edge.target);
    }
```
```
// understand-anything-plugin/src/context-builder.ts:114
      lines.push(`### ${node.name} (${node.type})`);
      if (node.filePath) {
        lines.push(`- **File:** ${node.filePath}`);
```
```
// understand-anything-plugin/src/context-builder.ts:122
      }
      if (node.languageNotes) {
        lines.push(`- **Language Notes:** ${node.languageNotes}`);
```
Judgment (TRUE / NOT TRUE, one sentence):

### Egonex-AI__Understand-Anything 12: code:types.ts->diff-analyzer.ts
- E2 GraphEdge declared at understand-anything-plugin/packages/core/src/types.ts:70
- E2 GraphNode declared at understand-anything-plugin/packages/core/src/types.ts:54
- E4 GraphEdge imported at understand-anything-plugin/src/diff-analyzer.ts:1 from "@understand-anything/core", resolved to understand-anything-plugin/packages/core/src/index.ts, re-exported by barrel understand-anything-plugin/packages/core/src/index.ts:1 from understand-anything-plugin/packages/core/src/types.ts
- E4 GraphNode imported at understand-anything-plugin/src/diff-analyzer.ts:1 from "@understand-anything/core", resolved to understand-anything-plugin/packages/core/src/index.ts, re-exported by barrel understand-anything-plugin/packages/core/src/index.ts:1 from understand-anything-plugin/packages/core/src/types.ts
- E5 GraphEdge fields read: source (understand-anything-plugin/src/diff-analyzer.ts:46), target (understand-anything-plugin/src/diff-analyzer.ts:47)
- E5 GraphEdge stoplisted reads: type
- E5 GraphNode fields read: complexity (understand-anything-plugin/src/diff-analyzer.ts:161), filePath (understand-anything-plugin/src/diff-analyzer.ts:34)
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
// understand-anything-plugin/src/diff-analyzer.ts:46
  for (const edge of edges) {
    if (edge.type === "contains" && changedNodeIds.has(edge.source)) {
      changedNodeIds.add(edge.target);
```
```
// understand-anything-plugin/src/diff-analyzer.ts:47
    if (edge.type === "contains" && changedNodeIds.has(edge.source)) {
      changedNodeIds.add(edge.target);
    }
```
```
// understand-anything-plugin/src/diff-analyzer.ts:161
  const complexChanges = ctx.changedNodes.filter(
    (n) => n.complexity === "complex",
  );
```
```
// understand-anything-plugin/src/diff-analyzer.ts:34
    for (const node of nodes) {
      if (node.filePath === file) {
        changedNodeIds.add(node.id);
```
Judgment (TRUE / NOT TRUE, one sentence):

### Egonex-AI__Understand-Anything 13: code:types.ts->explain-builder.ts
- E2 GraphEdge declared at understand-anything-plugin/packages/core/src/types.ts:70
- E2 GraphNode declared at understand-anything-plugin/packages/core/src/types.ts:54
- E4 GraphEdge imported at understand-anything-plugin/src/explain-builder.ts:1 from "@understand-anything/core", resolved to understand-anything-plugin/packages/core/src/index.ts, re-exported by barrel understand-anything-plugin/packages/core/src/index.ts:1 from understand-anything-plugin/packages/core/src/types.ts
- E4 GraphNode imported at understand-anything-plugin/src/explain-builder.ts:1 from "@understand-anything/core", resolved to understand-anything-plugin/packages/core/src/index.ts, re-exported by barrel understand-anything-plugin/packages/core/src/index.ts:1 from understand-anything-plugin/packages/core/src/types.ts
- E5 GraphEdge fields read: source (understand-anything-plugin/src/explain-builder.ts:62), target (understand-anything-plugin/src/explain-builder.ts:63)
- E5 GraphEdge stoplisted reads: description, type
- E5 GraphNode fields read: filePath (understand-anything-plugin/src/explain-builder.ts:37), languageNotes (understand-anything-plugin/src/explain-builder.ts:180), lineRange (understand-anything-plugin/src/explain-builder.ts:132)
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
// understand-anything-plugin/src/explain-builder.ts:62
      (e) =>
        e.source === targetNode!.id &&
        e.target === n.id &&
```
```
// understand-anything-plugin/src/explain-builder.ts:63
        e.source === targetNode!.id &&
        e.target === n.id &&
        e.type === "contains",
```
```
// understand-anything-plugin/src/explain-builder.ts:37
      nodes.find(
        (n) => n.filePath === filePath && n.name === funcName,
      ) ?? null;
```
```
// understand-anything-plugin/src/explain-builder.ts:180

  if (targetNode.languageNotes) {
    lines.push("## Language Notes");
```
Judgment (TRUE / NOT TRUE, one sentence):

