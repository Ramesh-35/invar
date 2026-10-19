## excalidraw__excalidraw: 46 proposal(s), first 20 adjudicated

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
- E5 EditorInterface fields read: canFitSidebar (packages/excalidraw/components/App.tsx:4665), desktopUIMode (packages/excalidraw/components/App.tsx:3727), formFactor (packages/excalidraw/components/App.tsx:2375), isTouchScreen (packages/excalidraw/components/App.tsx:6567), userAgent (packages/excalidraw/components/App.tsx:470)
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

### excalidraw__excalidraw 3: code:arrowEndpointText.ts->App.textTool.ts
- E2 ArrowEndpoint declared at packages/element/src/arrowEndpointText.ts:51
- E4 ArrowEndpoint imported at packages/excalidraw/components/App.textTool.ts:18 from "@excalidraw/element", resolved to packages/element/src/index.ts, re-exported by barrel packages/element/src/index.ts:60 from packages/element/src/arrowEndpointText.ts
- E5 ArrowEndpoint fields read: arrow (packages/excalidraw/components/App.textTool.ts:52), startOrEnd (packages/excalidraw/components/App.textTool.ts:53)
- E5 ArrowEndpoint stoplisted reads: (none)
```
// packages/element/src/arrowEndpointText.ts:51
export type ArrowEndpoint = {
  arrow: NonDeleted<ExcalidrawArrowElement>;
  startOrEnd: "start" | "end";
};

/**
 * Finds a free (unbound) arrow endpoint under the cursor, front-most first.
 * Used by the text tool to offer creating a text label anchored to the tip of
 * an arrow.
 */
export const getUnboundArrowEndpointAtPoint = (
  scenePointer: GlobalPoint,
```
```
// packages/excalidraw/components/App.textTool.ts:52
        type: "arrow",
        elementId: target.endpoint.arrow.id,
        anchor: target.endpoint.startOrEnd,
```
```
// packages/excalidraw/components/App.textTool.ts:53
        elementId: target.endpoint.arrow.id,
        anchor: target.endpoint.startOrEnd,
      };
```
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 4: code:renderElement.ts->staticScene.ts
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

### excalidraw__excalidraw 5: code:types.ts->DebugCanvas.tsx
- E2 FixedPointBinding declared at packages/element/src/types.ts:320
- E4 FixedPointBinding imported at excalidraw-app/components/DebugCanvas.tsx:34 from "@excalidraw/element/types", resolved to packages/element/src/types.ts
- E5 FixedPointBinding fields read: elementId (excalidraw-app/components/DebugCanvas.tsx:146), fixedPoint (excalidraw-app/components/DebugCanvas.tsx:140), mode (excalidraw-app/components/DebugCanvas.tsx:240)
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
```
// excalidraw-app/components/DebugCanvas.tsx:240
          dim,
          element.startBinding?.mode === "orbit" ? "red" : "black",
        );
```
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 6: code:types.ts->restore.ts
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

### excalidraw__excalidraw 7: code:types.ts->history.test.tsx
- E2 FixedPointBinding declared at packages/element/src/types.ts:320
- E4 FixedPointBinding imported at packages/excalidraw/tests/history.test.tsx:34 from "@excalidraw/element/types", resolved to packages/element/src/types.ts
- E5 FixedPointBinding fields read: elementId (packages/excalidraw/tests/history.test.tsx:1046), fixedPoint (packages/excalidraw/tests/history.test.tsx:4602), mode (packages/excalidraw/tests/history.test.tsx:4605)
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
// packages/excalidraw/tests/history.test.tsx:1046
      expect(h.state.selectedLinearElement?.isEditing).toBe(true);
      expect(h.state.selectedLinearElement?.elementId).toBe(h.elements[0].id);
      expect(h.elements).toEqual([
```
```
// packages/excalidraw/tests/history.test.tsx:4602
          (h.elements[2] as ExcalidrawElbowArrowElement).startBinding
            ?.fixedPoint,
        ).toEqual([1, 0.5001]);
```
```
// packages/excalidraw/tests/history.test.tsx:4605
        expect(
          (h.elements[2] as ExcalidrawElbowArrowElement).startBinding?.mode,
        ).toBe("orbit");
```
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 8: code:visualdebug.ts->DebugCanvas.tsx
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

### excalidraw__excalidraw 9: code:restore.ts->App.tsx
- E2 RestoredDataState declared at packages/excalidraw/data/restore.ts:246
- E4 RestoredDataState imported at excalidraw-app/App.tsx:65 from "@excalidraw/excalidraw/data/restore", resolved to packages/excalidraw/data/restore.ts
- E5 RestoredDataState fields read: appState (excalidraw-app/App.tsx:247), elements (excalidraw-app/App.tsx:243)
- E5 RestoredDataState stoplisted reads: (none)
```
// packages/excalidraw/data/restore.ts:246
export type RestoredDataState = {
  elements: OrderedExcalidrawElement[];
  appState: RestoredAppState;
  files: BinaryFiles;
};

const ALLOWED_STROKE_VARIABILITIES = new Set<StrokeVariability>([
  "constant",
  "variable",
]);

const restoreStrokeVariability = (
```
```
// excalidraw-app/App.tsx:247
    }),
    appState: restoreAppState(localDataState?.appState, null),
  };
```
```
// excalidraw-app/App.tsx:243
  } = {
    elements: restoreElements(localDataState?.elements, null, {
      repairBindings: true,
```
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 10: code:types.ts->Collab.tsx
- E2 ImportedDataState declared at packages/excalidraw/data/types.ts:35
- E4 ImportedDataState imported at excalidraw-app/collab/Collab.tsx:38 from "@excalidraw/excalidraw/data/types", resolved to packages/excalidraw/data/types.ts
- E5 ImportedDataState fields read: appState (excalidraw-app/collab/Collab.tsx:665), elements (excalidraw-app/collab/Collab.tsx:441)
- E5 ImportedDataState stoplisted reads: type
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
// excalidraw-app/collab/Collab.tsx:665
                fit: "contain",
              }).appState,
            });
```
```
// excalidraw-app/collab/Collab.tsx:441
  }) => {
    const unfetchedImages = opts.elements
      .filter((element) => {
```
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 11: code:types.ts->index.ts
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

### excalidraw__excalidraw 12: code:types.ts->renderElement.ts
- E2 StaticCanvasRenderConfig declared at packages/excalidraw/scene/types.ts:28
- E4 StaticCanvasRenderConfig imported at packages/element/src/renderElement.ts:43 from "@excalidraw/excalidraw/scene/types", resolved to packages/excalidraw/scene/types.ts
- E5 StaticCanvasRenderConfig fields read: imageCache (packages/element/src/renderElement.ts:100), isExporting (packages/element/src/renderElement.ts:990), theme (packages/element/src/renderElement.ts:207)
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
// packages/element/src/renderElement.ts:207
  canvas: HTMLCanvasElement;
  theme: AppState["theme"];
  scale: number;
```
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 13: code:types.ts->shape.ts
- E2 ElementShapes declared at packages/excalidraw/scene/types.ts:165
- E4 ElementShapes imported at packages/element/src/shape.ts:42 from "@excalidraw/excalidraw/scene/types", resolved to packages/excalidraw/scene/types.ts
- E5 ElementShapes fields read: ellipse (packages/element/src/shape.ts:881), line (packages/element/src/shape.ts:306), rectangle (packages/element/src/shape.ts:815)
- E5 ElementShapes stoplisted reads: (none)
```
// packages/excalidraw/scene/types.ts:165
export type ElementShapes = {
  rectangle: Drawable;
  ellipse: Drawable;
  diamond: Drawable;
  iframe: Drawable;
  embeddable: Drawable;
  freedraw: (Drawable | SVGPathString)[];
  arrow: Drawable[];
  line: Drawable[];
  text: null;
  stickynote: null;
  image: null;
```
```
// packages/element/src/shape.ts:881
    case "ellipse": {
      const shape: ElementShapes[typeof element.type] = generator.ellipse(
        element.width / 2,
```
```
// packages/element/src/shape.ts:306

  return [generator.line(x3, y3, x4, y4, lineOptions)];
};
```
```
// packages/element/src/shape.ts:815
      } else {
        shape = generator.rectangle(
          0,
```
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 14: code:types.ts->CustomFooter.tsx
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

### excalidraw__excalidraw 15: code:types.ts->ExampleApp.tsx
- E2 AppState declared at packages/excalidraw/types.ts:329
- E2 ExcalidrawImperativeAPI declared at packages/excalidraw/types.ts:1364
- E4 AppState imported at examples/with-script-in-browser/components/ExampleApp.tsx:17 from "@excalidraw/excalidraw/types", resolved to packages/excalidraw/types.ts
- E4 ExcalidrawImperativeAPI imported at examples/with-script-in-browser/components/ExampleApp.tsx:17 from "@excalidraw/excalidraw/types", resolved to packages/excalidraw/types.ts
- E5 AppState fields read: activeTool (examples/with-script-in-browser/components/ExampleApp.tsx:405), height (examples/with-script-in-browser/components/ExampleApp.tsx:589), offsetLeft (examples/with-script-in-browser/components/ExampleApp.tsx:581), offsetTop (examples/with-script-in-browser/components/ExampleApp.tsx:580), width (examples/with-script-in-browser/components/ExampleApp.tsx:598)
- E5 AppState stoplisted reads: (none)
- E5 ExcalidrawImperativeAPI fields read: addFiles (examples/with-script-in-browser/components/ExampleApp.tsx:167), getAppState (examples/with-script-in-browser/components/ExampleApp.tsx:391), getFiles (examples/with-script-in-browser/components/ExampleApp.tsx:392), getSceneElements (examples/with-script-in-browser/components/ExampleApp.tsx:390), resetScene (examples/with-script-in-browser/components/ExampleApp.tsx:679), setActiveTool (examples/with-script-in-browser/components/ExampleApp.tsx:467), setViewport (examples/with-script-in-browser/components/ExampleApp.tsx:983), updateLibrary (examples/with-script-in-browser/components/ExampleApp.tsx:309), updateScene (examples/with-script-in-browser/components/ExampleApp.tsx:307)
- E5 ExcalidrawImperativeAPI stoplisted reads: id
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
// examples/with-script-in-browser/components/ExampleApp.tsx:405
  const onPointerDown = (
    activeTool: AppState["activeTool"],
    pointerDownState: ExcalidrawPointerDownState,
```
```
// examples/with-script-in-browser/components/ExampleApp.tsx:589
    }
    if (top + COMMENT_INPUT_HEIGHT > appState.height) {
      top = appState.height - COMMENT_INPUT_HEIGHT - COMMENT_ICON_DIMENSION / 2;
```
```
// examples/with-script-in-browser/components/ExampleApp.tsx:581
    let top = y - COMMENT_ICON_DIMENSION / 2 - appState.offsetTop;
    let left = x - COMMENT_ICON_DIMENSION / 2 - appState.offsetLeft;

```
```
// examples/with-script-in-browser/components/ExampleApp.tsx:580
    );
    let top = y - COMMENT_ICON_DIMENSION / 2 - appState.offsetTop;
    let left = x - COMMENT_ICON_DIMENSION / 2 - appState.offsetLeft;
```
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 16: code:types.ts->App.tsx
- E2 AppState declared at packages/excalidraw/types.ts:329
- E2 ExcalidrawImperativeAPI declared at packages/excalidraw/types.ts:1364
- E2 ExcalidrawProps declared at packages/excalidraw/types.ts:842
- E4 AppState imported at excalidraw-app/App.tsx:71 from "@excalidraw/excalidraw/types", resolved to packages/excalidraw/types.ts
- E4 ExcalidrawImperativeAPI imported at excalidraw-app/App.tsx:71 from "@excalidraw/excalidraw/types", resolved to packages/excalidraw/types.ts
- E4 ExcalidrawProps imported at excalidraw-app/App.tsx:71 from "@excalidraw/excalidraw/types", resolved to packages/excalidraw/types.ts
- E5 AppState fields read: exportBackground (excalidraw-app/App.tsx:784), theme (excalidraw-app/App.tsx:344), viewBackgroundColor (excalidraw-app/App.tsx:785)
- E5 AppState stoplisted reads: name
- E5 ExcalidrawImperativeAPI fields read: addFiles (excalidraw-app/App.tsx:492), getAppState (excalidraw-app/App.tsx:346), getEditorInterface (excalidraw-app/App.tsx:1002), getFiles (excalidraw-app/App.tsx:626), getName (excalidraw-app/App.tsx:968), getSceneElements (excalidraw-app/App.tsx:697), getSceneElementsIncludingDeleted (excalidraw-app/App.tsx:354), history (excalidraw-app/App.tsx:285), refresh (excalidraw-app/App.tsx:1061), setViewport (excalidraw-app/App.tsx:1022), updateLibrary (excalidraw-app/App.tsx:616), updateScene (excalidraw-app/App.tsx:580)
- E5 ExcalidrawImperativeAPI stoplisted reads: id
- E5 ExcalidrawProps fields read: isCollaborating (excalidraw-app/App.tsx:484), onPointerUpdate (excalidraw-app/App.tsx:955), theme (excalidraw-app/App.tsx:344)
- E5 ExcalidrawProps stoplisted reads: name
```
// packages/excalidraw/types.ts:842
export interface ExcalidrawProps {
  className?: string;
  /**
   * Document that owns Excalidraw's mounted DOM.
   *
   * Set only when it differs from the global `document`, such as when code
   * executing in a parent window mounts Excalidraw into an iframe document.
   * The value must remain stable for the editor's lifetime.
   *
   * @default document
   */
  ownerDocument?: Document;
```
```
// excalidraw-app/App.tsx:784
          ...appState,
          viewBackgroundColor: appState.exportBackground
            ? appState.viewBackgroundColor
```
```
// excalidraw-app/App.tsx:344
              ...scene?.appState,
              theme: localDataState?.appState?.theme || scene?.appState?.theme,
            },
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
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 17: code:types.ts->Collab.tsx#2
- E2 Collaborator declared at packages/excalidraw/types.ts:78
- E2 ExcalidrawImperativeAPI declared at packages/excalidraw/types.ts:1364
- E2 UserToFollow declared at packages/excalidraw/types.ts:187
- E4 Collaborator imported at excalidraw-app/collab/Collab.tsx:45 from "@excalidraw/excalidraw/types", resolved to packages/excalidraw/types.ts
- E4 ExcalidrawImperativeAPI imported at excalidraw-app/collab/Collab.tsx:45 from "@excalidraw/excalidraw/types", resolved to packages/excalidraw/types.ts
- E4 UserToFollow imported at excalidraw-app/collab/Collab.tsx:45 from "@excalidraw/excalidraw/types", resolved to packages/excalidraw/types.ts
- E5 Collaborator fields read: button (excalidraw-app/collab/Collab.tsx:935), pointer (excalidraw-app/collab/Collab.tsx:934), socketId (excalidraw-app/collab/Collab.tsx:624), username (excalidraw-app/collab/Collab.tsx:484)
- E5 Collaborator stoplisted reads: id
- E5 ExcalidrawImperativeAPI fields read: addFiles (excalidraw-app/collab/Collab.tsx:801), getAppState (excalidraw-app/collab/Collab.tsx:331), getFiles (excalidraw-app/collab/Collab.tsx:455), getSceneElements (excalidraw-app/collab/Collab.tsx:552), getSceneElementsIncludingDeleted (excalidraw-app/collab/Collab.tsx:301), history (excalidraw-app/collab/Collab.tsx:392), onScrollChange (excalidraw-app/collab/Collab.tsx:227), onUserFollow (excalidraw-app/collab/Collab.tsx:219), resetScene (excalidraw-app/collab/Collab.tsx:550), updateScene (excalidraw-app/collab/Collab.tsx:406)
- E5 ExcalidrawImperativeAPI stoplisted reads: id
- E5 UserToFollow fields read: socketId (excalidraw-app/collab/Collab.tsx:624), username (excalidraw-app/collab/Collab.tsx:484)
- E5 UserToFollow stoplisted reads: (none)
```
// packages/excalidraw/types.ts:187
export type UserToFollow = {
  socketId: SocketId;
  username: string;
};

type _CommonCanvasAppState = {
  zoom: AppState["zoom"];
  scrollX: AppState["scrollX"];
  scrollY: AppState["scrollY"];
  width: AppState["width"];
  height: AppState["height"];
  viewModeEnabled: AppState["viewModeEnabled"];
```
```
// excalidraw-app/collab/Collab.tsx:935
      pointer: SocketUpdateDataSource["MOUSE_LOCATION"]["payload"]["pointer"];
      button: SocketUpdateDataSource["MOUSE_LOCATION"]["payload"]["button"];
      pointersMap: Gesture["pointers"];
```
```
// excalidraw-app/collab/Collab.tsx:934
    (payload: {
      pointer: SocketUpdateDataSource["MOUSE_LOCATION"]["payload"]["pointer"];
      button: SocketUpdateDataSource["MOUSE_LOCATION"]["payload"]["button"];
```
```
// excalidraw-app/collab/Collab.tsx:624

            const socketId: SocketUpdateDataSource["MOUSE_LOCATION"]["payload"]["socketId"] =
              decryptedData.payload.socketId ||
```
```
// excalidraw-app/collab/Collab.tsx:484
  ) => {
    if (!this.state.username) {
      import("@excalidraw/random-username").then(({ getRandomUsername }) => {
```
Judgment (TRUE / NOT TRUE, one sentence):

### excalidraw__excalidraw 18: code:types.ts->AI.tsx
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

### excalidraw__excalidraw 19: code:types.ts->DebugCanvas.tsx#2
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

### excalidraw__excalidraw 20: code:types.ts->FileManager.ts
- E2 BinaryFileData declared at packages/excalidraw/types.ts:121
- E2 ExcalidrawImperativeAPI declared at packages/excalidraw/types.ts:1364
- E4 BinaryFileData imported at excalidraw-app/data/FileManager.ts:13 from "@excalidraw/excalidraw/types", resolved to packages/excalidraw/types.ts
- E4 ExcalidrawImperativeAPI imported at excalidraw-app/data/FileManager.ts:13 from "@excalidraw/excalidraw/types", resolved to packages/excalidraw/types.ts
- E5 BinaryFileData fields read: dataURL (excalidraw-app/data/FileManager.ts:243), mimeType (excalidraw-app/data/FileManager.ts:249)
- E5 BinaryFileData stoplisted reads: id, version
- E5 ExcalidrawImperativeAPI fields read: getFiles (excalidraw-app/data/FileManager.ts:62), updateScene (excalidraw-app/data/FileManager.ts:280)
- E5 ExcalidrawImperativeAPI stoplisted reads: id
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
// excalidraw-app/data/FileManager.ts:243
  for (const [id, fileData] of files) {
    const buffer = new TextEncoder().encode(fileData.dataURL);

```
```
// excalidraw-app/data/FileManager.ts:249
        id,
        mimeType: fileData.mimeType,
        created: Date.now(),
```
```
// excalidraw-app/data/FileManager.ts:62
  }) {
    this._getFiles = getFiles;
    this._saveFiles = saveFiles;
```
```
// excalidraw-app/data/FileManager.ts:280
  }
  params.excalidrawAPI.updateScene({
    elements: params.excalidrawAPI
```
Judgment (TRUE / NOT TRUE, one sentence):

