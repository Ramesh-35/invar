# STAGE3-EVAL-001 phase 2 adjudication packet (9 proposals)

Rubric D: TRUE iff renaming or removing one side without the other is a bug
a reviewer would flag; UNSURE scores NOT TRUE. One line per judgment.

## thedotmack__claude-mem proposal 1: strings:MemoryItemSchema@const:observationsKnownColumns
overlap: concepts createdAtEpoch facts filesModified filesRead narrative subtitle text
producer src/core/schemas/memory-item.ts (MemoryItemSchema):
```
export const MemoryItemSchema = z.object({
  id: z.string().min(1),
  projectId: z.string().min(1),
  serverSessionId: z.string().min(1).nullable().default(null),
  legacyObservationId: z.number().int().positive().nullable().default(null),
  kind: MemoryItemKindSchema,
  type: z.string().min(1),
  title: z.string().min(1).nullable().default(null),
  subtitle: z.string().min(1).nullable().default(null),
  text: z.string().nullable().default(null),
  narrative: z.string().nullable().default(null),
  facts: z.array(z.string()).default([]),
```
consumer src/services/sqlite/SessionStore.ts (const:observationsKnownColumns, scope same-package):
```
    const observationsKnownColumns = [
      'id', 'memory_session_id', 'project', 'text', 'type', 'title', 'subtitle',
      'facts', 'narrative', 'concepts', 'files_read', 'files_modified',
      'prompt_number', 'discovery_tokens', 'created_at', 'created_at_epoch',
    ];
    const observationsNewSQL = `
      CREATE TABLE observations_new (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        memory_session_id TEXT NOT NULL,
        project TEXT NOT NULL,
```

## thedotmack__claude-mem proposal 2: strings:MemoryItemSchema@const:OBSERVATION_JSON_FIELDS
overlap: concepts facts filesModified filesRead
producer src/core/schemas/memory-item.ts (MemoryItemSchema):
```
export const MemoryItemSchema = z.object({
  id: z.string().min(1),
  projectId: z.string().min(1),
  serverSessionId: z.string().min(1).nullable().default(null),
  legacyObservationId: z.number().int().positive().nullable().default(null),
  kind: MemoryItemKindSchema,
  type: z.string().min(1),
  title: z.string().min(1).nullable().default(null),
  subtitle: z.string().min(1).nullable().default(null),
  text: z.string().nullable().default(null),
  narrative: z.string().nullable().default(null),
  facts: z.array(z.string()).default([]),
```
consumer src/services/worker/http/routes/DataRoutes.ts (const:OBSERVATION_JSON_FIELDS, scope same-package):
```
const OBSERVATION_JSON_FIELDS = ['facts', 'concepts', 'files_read', 'files_modified'] as const;
const SUMMARY_JSON_FIELDS = ['files_read', 'files_edited'] as const;

const jsonStringifyFields = (fields: readonly string[]) =>
  (value: unknown): unknown => {
    if (!value || typeof value !== 'object' || Array.isArray(value)) return value;
    const record = value as Record<string, unknown>;
    let normalized: Record<string, unknown> | undefined;
    for (const field of fields) {
      const fieldValue = record[field];
```

## thedotmack__claude-mem proposal 3: strings:MemoryItemSchema@const:indexedFields
overlap: concepts facts narrative subtitle text
producer src/core/schemas/memory-item.ts (MemoryItemSchema):
```
export const MemoryItemSchema = z.object({
  id: z.string().min(1),
  projectId: z.string().min(1),
  serverSessionId: z.string().min(1).nullable().default(null),
  legacyObservationId: z.number().int().positive().nullable().default(null),
  kind: MemoryItemKindSchema,
  type: z.string().min(1),
  title: z.string().min(1).nullable().default(null),
  subtitle: z.string().min(1).nullable().default(null),
  text: z.string().nullable().default(null),
  narrative: z.string().nullable().default(null),
  facts: z.array(z.string()).default([]),
```
consumer src/storage/sqlite/memory-items.ts (const:indexedFields, scope import):
```
    const indexedFields = ['title', 'subtitle', 'text', 'narrative', 'facts', 'concepts'];
    for (const term of substrings) {
      const alternatives = [...new Set([term.literal, term.literal.normalize('NFKC')])];
      const clauses = alternatives.flatMap(literal => indexedFields.map(field => {
        parameters.push(literal);
        return `instr(lower(COALESCE(memory_items.${field}, '')), lower(?)) > 0`;
      }));
      if (term.words) {
        clauses.push(ftsCondition);
        parameters.push(projectId, term.words);
```

## diegosouzapw__OmniRoute proposal 1: strings:pricingFieldsSchema@const:PRICING_FIELDS
overlap: cache_creation cached input output reasoning
producer src/shared/validation/schemas/pricing.ts (pricingFieldsSchema):
```
export const pricingFieldsSchema = z
  .object({
    input: z.number().min(0).optional(),
    output: z.number().min(0).optional(),
    cached: z.number().min(0).optional(),
    reasoning: z.number().min(0).optional(),
    cache_creation: z.number().min(0).optional(),
    // Long-context tier carried by default rows (e.g. OpenAI >272K, xAI >=200K) and echoed
    // back by the dashboard's GET → PATCH round-trip (#12494).
    long_context: z
      .object({
        threshold: z.number().positive(),
```
consumer src/app/(dashboard)/dashboard/settings/components/PricingTab.tsx (const:PRICING_FIELDS, scope same-package):
```
const PRICING_FIELDS = ["input", "output", "cached", "reasoning", "cache_creation"] as const;
const FIELD_LABEL_KEYS: Record<(typeof PRICING_FIELDS)[number], string> = {
  input: "input",
  output: "output",
  cached: "cached",
  reasoning: "reasoning",
  cache_creation: "cacheCreation",
};

type PricingField = (typeof PRICING_FIELDS)[number];
```

## diegosouzapw__OmniRoute proposal 2: strings:updateProviderConnectionSchema@const:CONNECTION_COLUMNS
overlap: errorCode isActive lastErrorAt lastErrorType priority rateLimitedUntil testStatus
producer src/shared/validation/schemas/provider.ts (updateProviderConnectionSchema):
```
export const updateProviderConnectionSchema = z
  .object({
    name: z.string().max(200).optional(),
    // #6562: `priority` is auto-incremented on connection creation
    // (src/lib/db/providers.ts::createProviderConnection — `MAX(priority)+1` per
    // provider, unbounded) and the edit UI always round-trips the connection's
    // current priority unchanged. Providers whose accounts are commonly rotated
    // in bulk (e.g. Codex OAuth import-bulk, up to 50 entries per call, repeatable)
    // routinely exceed 100 connections, so a `max(100)` UI-only ceiling here
    // rejected re-saving an already-valid, already-persisted priority with
    // "Invalid request" on every edit. Bounded well above any realistic
    // connection count (still rejects garbage input) rather than removed.
```
consumer src/app/api/resilience/connections/route.ts (const:CONNECTION_COLUMNS, scope same-package):
```
const CONNECTION_COLUMNS: string[] = [
  "id",
  "provider",
  "name",
  "auth_type",
  "priority",
  "is_active",
  "test_status",
  "error_code",
  "last_error_type",
```

## diegosouzapw__OmniRoute proposal 3: strings:pricingFieldsSchema@const:PRICING_FIELDS#2
overlap: cache_creation cached input output
producer src/shared/validation/schemas/pricing.ts (pricingFieldsSchema):
```
export const pricingFieldsSchema = z
  .object({
    input: z.number().min(0).optional(),
    output: z.number().min(0).optional(),
    cached: z.number().min(0).optional(),
    reasoning: z.number().min(0).optional(),
    cache_creation: z.number().min(0).optional(),
    // Long-context tier carried by default rows (e.g. OpenAI >272K, xAI >=200K) and echoed
    // back by the dashboard's GET → PATCH round-trip (#12494).
    long_context: z
      .object({
        threshold: z.number().positive(),
```
consumer src/lib/catalogUserPricing.ts (const:PRICING_FIELDS, scope same-package):
```
const PRICING_FIELDS = ["input", "output", "cached", "cache_creation"] as const;

/** Bulk-load the user `pricing` namespace once (sync, no per-entry SQLite). */
export function readUserPricingSync(): UserPricingByProvider {
  const out: UserPricingByProvider = {};
  try {
    const rows = getDbInstance()
      .prepare("SELECT key, value FROM key_value WHERE namespace = 'pricing'")
      .all() as Array<{ key?: unknown; value?: unknown }>;
    for (const row of rows) {
```

## diegosouzapw__OmniRoute proposal 4: strings:updateProviderConnectionSchema@const:optionalFields
overlap: apiKey defaultModel errorCode globalPriority group healthCheckInterval lastError lastErrorAt lastErrorSource lastErrorType lastTested maxConcurrent perKeyProxyEnabled projectId proxyEnabled quotaVisible quotaWindowThresholds rateLimitOverrides rateLimitedUntil testStatus
producer src/shared/validation/schemas/provider.ts (updateProviderConnectionSchema):
```
export const updateProviderConnectionSchema = z
  .object({
    name: z.string().max(200).optional(),
    // #6562: `priority` is auto-incremented on connection creation
    // (src/lib/db/providers.ts::createProviderConnection — `MAX(priority)+1` per
    // provider, unbounded) and the edit UI always round-trips the connection's
    // current priority unchanged. Providers whose accounts are commonly rotated
    // in bulk (e.g. Codex OAuth import-bulk, up to 50 entries per call, repeatable)
    // routinely exceed 100 connections, so a `max(100)` UI-only ceiling here
    // rejected re-saving an already-valid, already-persisted priority with
    // "Invalid request" on every edit. Bounded well above any realistic
    // connection count (still rejects garbage input) rather than removed.
```
consumer src/lib/db/providers.ts (const:optionalFields, scope same-package):
```
  const optionalFields = [
    "displayName",
    "email",
    "globalPriority",
    "defaultModel",
    "accessToken",
    "refreshToken",
    "expiresAt",
    // #5326's payload sets this and _insertConnectionRow binds it, but it was
    // missing from this allowlist — so every created row stored NULL however good
```

## diegosouzapw__OmniRoute proposal 5: strings:pricingFieldsSchema@const:NON_TOKEN_FIELDS
overlap: input_cost_per_character input_cost_per_image input_cost_per_pixel input_cost_per_second input_cost_per_video_per_second ocr_cost_per_page output_cost_per_character output_cost_per_image output_cost_per_pixel output_cost_per_second output_cost_per_video_per_second search_unit_cost
producer src/shared/validation/schemas/pricing.ts (pricingFieldsSchema):
```
export const pricingFieldsSchema = z
  .object({
    input: z.number().min(0).optional(),
    output: z.number().min(0).optional(),
    cached: z.number().min(0).optional(),
    reasoning: z.number().min(0).optional(),
    cache_creation: z.number().min(0).optional(),
    // Long-context tier carried by default rows (e.g. OpenAI >272K, xAI >=200K) and echoed
    // back by the dashboard's GET → PATCH round-trip (#12494).
    long_context: z
      .object({
        threshold: z.number().positive(),
```
consumer src/lib/pricingSync.ts (const:NON_TOKEN_FIELDS, scope same-package):
```
    const NON_TOKEN_FIELDS = [
      "input_cost_per_second",
      "output_cost_per_second",
      "input_cost_per_image",
      "output_cost_per_image",
      "input_cost_per_pixel",
      "output_cost_per_pixel",
      "input_cost_per_character",
      "output_cost_per_character",
      "input_cost_per_video_per_second",
```

## diegosouzapw__OmniRoute proposal 6: strings:updateProviderConnectionSchema@const:COLUMNS
overlap: errorCode isActive lastError lastErrorAt lastErrorType priority rateLimitedUntil testStatus
producer src/shared/validation/schemas/provider.ts (updateProviderConnectionSchema):
```
export const updateProviderConnectionSchema = z
  .object({
    name: z.string().max(200).optional(),
    // #6562: `priority` is auto-incremented on connection creation
    // (src/lib/db/providers.ts::createProviderConnection — `MAX(priority)+1` per
    // provider, unbounded) and the edit UI always round-trips the connection's
    // current priority unchanged. Providers whose accounts are commonly rotated
    // in bulk (e.g. Codex OAuth import-bulk, up to 50 entries per call, repeatable)
    // routinely exceed 100 connections, so a `max(100)` UI-only ceiling here
    // rejected re-saving an already-valid, already-persisted priority with
    // "Invalid request" on every edit. Bounded well above any realistic
    // connection count (still rejects garbage input) rather than removed.
```
consumer src/lib/resilience/cooldownManager.ts (const:COLUMNS, scope same-package):
```
const COLUMNS = [
  "id",
  "provider",
  "name",
  "auth_type",
  "priority",
  "is_active",
  "test_status",
  "error_code",
  "last_error",
```

