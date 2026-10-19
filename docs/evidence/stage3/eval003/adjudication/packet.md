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

## earendil-works__pi: 113 proposal(s), first 20 adjudicated

### earendil-works__pi 1: code:types.ts->index.ts
- E2 AgentToolResult declared at packages/agent/src/types.ts:424
- E4 AgentToolResult imported at packages/coding-agent/examples/extensions/subagent/index.ts:19 from "@earendil-works/pi-agent-core", resolved to packages/agent/src/index.ts, re-exported by barrel packages/agent/src/index.ts:5 from packages/agent/src/types.ts
- E5 AgentToolResult fields read: content (packages/coding-agent/examples/extensions/subagent/index.ts:101), details (packages/coding-agent/examples/extensions/subagent/index.ts:560), usage (packages/coding-agent/examples/extensions/subagent/index.ts:365)
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
// packages/coding-agent/examples/extensions/subagent/index.ts:101
			const filePath = shortenPath(rawPath);
			const content = (args.content || "") as string;
			const lines = content.split("\n").length;
```
```
// packages/coding-agent/examples/extensions/subagent/index.ts:560
								// Combine completed results with current streaming result
								const currentResult = partial.details?.results[0];
								if (currentResult) {
```
```
// packages/coding-agent/examples/extensions/subagent/index.ts:365
					if (msg.role === "assistant") {
						currentResult.usage.turns++;
						const usage = msg.usage;
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 2: code:types.ts->agent-session.ts
- E2 AfterToolCallContext declared at packages/agent/src/types.ts:119
- E2 AfterToolCallResult declared at packages/agent/src/types.ts:92
- E2 AgentContext declared at packages/agent/src/types.ts:502
- E2 AgentState declared at packages/agent/src/types.ts:382
- E2 AgentToolCallOutcome declared at packages/agent/src/types.ts:449
- E2 BeforeToolCallContext declared at packages/agent/src/types.ts:107
- E4 AfterToolCallContext imported at packages/coding-agent/src/core/agent-session.ts:18 from "@earendil-works/pi-agent-core", resolved to packages/agent/src/index.ts, re-exported by barrel packages/agent/src/index.ts:5 from packages/agent/src/types.ts
- E4 AfterToolCallResult imported at packages/coding-agent/src/core/agent-session.ts:18 from "@earendil-works/pi-agent-core", resolved to packages/agent/src/index.ts, re-exported by barrel packages/agent/src/index.ts:5 from packages/agent/src/types.ts
- E4 AgentContext imported at packages/coding-agent/src/core/agent-session.ts:18 from "@earendil-works/pi-agent-core", resolved to packages/agent/src/index.ts, re-exported by barrel packages/agent/src/index.ts:5 from packages/agent/src/types.ts
- E4 AgentState imported at packages/coding-agent/src/core/agent-session.ts:18 from "@earendil-works/pi-agent-core", resolved to packages/agent/src/index.ts, re-exported by barrel packages/agent/src/index.ts:5 from packages/agent/src/types.ts
- E4 AgentToolCallOutcome imported at packages/coding-agent/src/core/agent-session.ts:18 from "@earendil-works/pi-agent-core", resolved to packages/agent/src/index.ts, re-exported by barrel packages/agent/src/index.ts:5 from packages/agent/src/types.ts
- E4 BeforeToolCallContext imported at packages/coding-agent/src/core/agent-session.ts:18 from "@earendil-works/pi-agent-core", resolved to packages/agent/src/index.ts, re-exported by barrel packages/agent/src/index.ts:5 from packages/agent/src/types.ts
- E5 AfterToolCallContext fields read: args (packages/coding-agent/src/core/agent-session.ts:1343), context (packages/coding-agent/src/core/agent-session.ts:796), isError (packages/coding-agent/src/core/agent-session.ts:722)
- E5 AfterToolCallContext stoplisted reads: result
- E5 AfterToolCallResult fields read: content (packages/coding-agent/src/core/agent-session.ts:697), details (packages/coding-agent/src/core/agent-session.ts:698), isError (packages/coding-agent/src/core/agent-session.ts:722), structuredContent (packages/coding-agent/src/core/agent-session.ts:699), usage (packages/coding-agent/src/core/agent-session.ts:701)
- E5 AfterToolCallResult stoplisted reads: (none)
- E5 AgentContext fields read: messages (packages/coding-agent/src/core/agent-session.ts:585), tools (packages/coding-agent/src/core/agent-session.ts:752)
- E5 AgentContext stoplisted reads: (none)
- E5 AgentState fields read: errorMessage (packages/coding-agent/src/core/agent-session.ts:1969), isStreaming (packages/coding-agent/src/core/agent-session.ts:2087), model (packages/coding-agent/src/core/agent-session.ts:617), systemPrompt (packages/coding-agent/src/core/agent-session.ts:3495), thinkingLevel (packages/coding-agent/src/core/agent-session.ts:587)
- E5 AgentState stoplisted reads: (none)
- E5 AgentToolCallOutcome fields read: durationMs (packages/coding-agent/src/core/agent-session.ts:1362), isError (packages/coding-agent/src/core/agent-session.ts:722)
- E5 AgentToolCallOutcome stoplisted reads: result
- E5 BeforeToolCallContext fields read: args (packages/coding-agent/src/core/agent-session.ts:1343), context (packages/coding-agent/src/core/agent-session.ts:796)
- E5 BeforeToolCallContext stoplisted reads: (none)
```
// packages/agent/src/types.ts:107
export interface BeforeToolCallContext {
	/** The assistant message that requested the tool call. */
	assistantMessage: AssistantMessage;
	/** The raw tool call block from `assistantMessage.content`. */
	toolCall: AgentToolCall;
	/** Validated tool arguments for the target tool schema. */
	args: unknown;
	/** Current agent context at the time the tool call is prepared. */
	context: AgentContext;
}

/** Context passed to `afterToolCall`. */
```
```
// packages/coding-agent/src/core/agent-session.ts:1343
				toolName: event.toolName,
				args: event.args,
			};
```
```
// packages/coding-agent/src/core/agent-session.ts:796
				const canonicalContext = {
					...request.context,
					messages: projection.messages,
```
```
// packages/coding-agent/src/core/agent-session.ts:722
			structuredContent: hookResult ? hookResult.structuredContent : result.structuredContent,
			isError: hookResult?.isError ?? isError,
			usage: hookResult?.usage,
```
```
// packages/coding-agent/src/core/agent-session.ts:697
					input: args as Record<string, unknown>,
					content: result.content,
					details: result.details,
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 3: code:types.ts->nested-tool-calls.ts
- E2 AgentToolCallOutcome declared at packages/agent/src/types.ts:449
- E2 AgentToolResult declared at packages/agent/src/types.ts:424
- E4 AgentToolCallOutcome imported at packages/coding-agent/src/core/nested-tool-calls.ts:11 from "@earendil-works/pi-agent-core", resolved to packages/agent/src/index.ts, re-exported by barrel packages/agent/src/index.ts:5 from packages/agent/src/types.ts
- E4 AgentToolResult imported at packages/coding-agent/src/core/nested-tool-calls.ts:11 from "@earendil-works/pi-agent-core", resolved to packages/agent/src/index.ts, re-exported by barrel packages/agent/src/index.ts:5 from packages/agent/src/types.ts
- E5 AgentToolCallOutcome fields read: durationMs (packages/coding-agent/src/core/nested-tool-calls.ts:82), isError (packages/coding-agent/src/core/nested-tool-calls.ts:237)
- E5 AgentToolCallOutcome stoplisted reads: result
- E5 AgentToolResult fields read: content (packages/coding-agent/src/core/nested-tool-calls.ts:155), isError (packages/coding-agent/src/core/nested-tool-calls.ts:237), usage (packages/coding-agent/src/core/nested-tool-calls.ts:88)
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
// packages/coding-agent/src/core/nested-tool-calls.ts:82
		record.status = isError ? "error" : "ok";
		record.durationMs = Math.round(performance.now() - (this.startedAt.get(record) ?? performance.now()));
		this.startedAt.delete(record);
```
```
// packages/coding-agent/src/core/nested-tool-calls.ts:237

		scope.recorder.finish(record, outcome.isError, textOf(outcome.result));
		// Nested results are not persisted, so their usage is only counted through the recorder.
```
```
// packages/coding-agent/src/core/nested-tool-calls.ts:155
function textOf(result: AgentToolResult<unknown>): string {
	return (result.content ?? [])
		.filter((block): block is TextContent => block.type === "text")
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 4: code:types.ts->tool-definition-wrapper.ts
- E2 AgentTool declared at packages/agent/src/types.ts:466
- E4 AgentTool imported at packages/coding-agent/src/core/tools/tool-definition-wrapper.ts:1 from "@earendil-works/pi-agent-core", resolved to packages/agent/src/index.ts, re-exported by barrel packages/agent/src/index.ts:5 from packages/agent/src/types.ts
- E5 AgentTool fields read: execute (packages/coding-agent/src/core/tools/tool-definition-wrapper.ts:22), executionMode (packages/coding-agent/src/core/tools/tool-definition-wrapper.ts:20), outputSchema (packages/coding-agent/src/core/tools/tool-definition-wrapper.ts:17), prepareArguments (packages/coding-agent/src/core/tools/tool-definition-wrapper.ts:19)
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
// packages/coding-agent/src/core/tools/tool-definition-wrapper.ts:22
		execute: (toolCallId, params, signal, onUpdate, ctx?: ExtensionToolContext) =>
			definition.execute(
				toolCallId,
```
```
// packages/coding-agent/src/core/tools/tool-definition-wrapper.ts:20
		prepareArguments: definition.prepareArguments,
		executionMode: definition.executionMode,
		execute: (toolCallId, params, signal, onUpdate, ctx?: ExtensionToolContext) =>
```
```
// packages/coding-agent/src/core/tools/tool-definition-wrapper.ts:17
		parameters: definition.parameters,
		outputSchema: definition.outputSchema,
		constrainedSampling: definition.constrainedSampling,
```
```
// packages/coding-agent/src/core/tools/tool-definition-wrapper.ts:19
		constrainedSampling: definition.constrainedSampling,
		prepareArguments: definition.prepareArguments,
		executionMode: definition.executionMode,
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 5: code:types.ts->execute.ts
- E2 AgentTool declared at packages/agent/src/types.ts:466
- E2 AgentToolCallOutcome declared at packages/agent/src/types.ts:449
- E2 AgentToolResult declared at packages/agent/src/types.ts:424
- E4 AgentTool imported at packages/coding-agent/src/extensions/codemode/execute.ts:6 from "@earendil-works/pi-agent-core", resolved to packages/agent/src/index.ts, re-exported by barrel packages/agent/src/index.ts:5 from packages/agent/src/types.ts
- E4 AgentToolCallOutcome imported at packages/coding-agent/src/extensions/codemode/execute.ts:6 from "@earendil-works/pi-agent-core", resolved to packages/agent/src/index.ts, re-exported by barrel packages/agent/src/index.ts:5 from packages/agent/src/types.ts
- E4 AgentToolResult imported at packages/coding-agent/src/extensions/codemode/execute.ts:6 from "@earendil-works/pi-agent-core", resolved to packages/agent/src/index.ts, re-exported by barrel packages/agent/src/index.ts:5 from packages/agent/src/types.ts
- E5 AgentTool fields read: execute (packages/coding-agent/src/extensions/codemode/execute.ts:530), outputSchema (packages/coding-agent/src/extensions/codemode/execute.ts:430)
- E5 AgentTool stoplisted reads: (none)
- E5 AgentToolCallOutcome fields read: durationMs (packages/coding-agent/src/extensions/codemode/execute.ts:490), isError (packages/coding-agent/src/extensions/codemode/execute.ts:432), toolCall (packages/coding-agent/src/extensions/codemode/execute.ts:489)
- E5 AgentToolCallOutcome stoplisted reads: result
- E5 AgentToolResult fields read: content (packages/coding-agent/src/extensions/codemode/execute.ts:76), isError (packages/coding-agent/src/extensions/codemode/execute.ts:432), structuredContent (packages/coding-agent/src/extensions/codemode/execute.ts:430), usage (packages/coding-agent/src/extensions/codemode/execute.ts:727)
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
// packages/coding-agent/src/extensions/codemode/execute.ts:530
		const store = ctx ? readCodemodeStore(ctx.sessionManager.getBranch()) : {};
		result = await sandbox.execute(code, { signal, store });
	} finally {
```
```
// packages/coding-agent/src/extensions/codemode/execute.ts:430
	const { result } = outcome;
	if (tool.outputSchema && result.structuredContent !== undefined) return result.structuredContent;
	const text = textOf(result);
```
```
// packages/coding-agent/src/extensions/codemode/execute.ts:490
			record.id = outcome.toolCall.id;
			record.durationMs = performance.now() - callStartedAt;
			if (outcome.isError) {
```
```
// packages/coding-agent/src/extensions/codemode/execute.ts:432
	const text = textOf(result);
	if (outcome.isError) throw new Error(text || `Tool "${tool.name}" failed`);
	return text;
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 6: code:types.ts->tools.ts
- E2 AgentToolResult declared at packages/agent/src/types.ts:424
- E4 AgentToolResult imported at packages/coding-agent/src/extensions/mcp/tools.ts:14 from "@earendil-works/pi-agent-core", resolved to packages/agent/src/index.ts, re-exported by barrel packages/agent/src/index.ts:5 from packages/agent/src/types.ts
- E5 AgentToolResult fields read: content (packages/coding-agent/src/extensions/mcp/tools.ts:274), details (packages/coding-agent/src/extensions/mcp/tools.ts:385), isError (packages/coding-agent/src/extensions/mcp/tools.ts:275)
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
// packages/coding-agent/src/extensions/mcp/tools.ts:274
	const converted: (TextContent | ImageContent)[] =
		result.content.length > 0 ? await toModelContent(server, result.content, options) : toLlmContent(result);
	if (result.isError && textOf(converted) === "") {
```
```
// packages/coding-agent/src/extensions/mcp/tools.ts:385
				);
				const fullOutputPath = (result.details as McpToolDetails | undefined)?.fullOutputPath;
				if (fullOutputPath) component.addChild(new Text(theme.fg("muted", `Full output: ${fullOutputPath}`), 0, 0));
```
```
// packages/coding-agent/src/extensions/mcp/tools.ts:275
		result.content.length > 0 ? await toModelContent(server, result.content, options) : toLlmContent(result);
	if (result.isError && textOf(converted) === "") {
		converted.push({ type: "text", text: `MCP tool ${server}/${tool} returned an error` });
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 7: code:types.ts->index.ts#2
- E2 OAuthCredentials declared at packages/ai/src/auth/types.ts:24
- E4 OAuthCredentials imported at packages/coding-agent/examples/extensions/custom-provider-anthropic/index.ts:26 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:24 from packages/ai/src/auth/types.ts
- E5 OAuthCredentials fields read: access (packages/coding-agent/examples/extensions/custom-provider-anthropic/index.ts:612), refresh (packages/coding-agent/examples/extensions/custom-provider-anthropic/index.ts:137)
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
// packages/coding-agent/examples/extensions/custom-provider-anthropic/index.ts:612
			refreshToken: refreshAnthropicToken,
			getApiKey: (cred) => cred.access,
		},
```
```
// packages/coding-agent/examples/extensions/custom-provider-anthropic/index.ts:137
			client_id: CLIENT_ID,
			refresh_token: credentials.refresh,
		}),
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 8: code:types.ts->model-registry.ts
- E2 AuthResult declared at packages/ai/src/auth/types.ts:104
- E4 AuthResult imported at packages/coding-agent/src/core/model-registry.ts:1 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:24 from packages/ai/src/auth/types.ts
- E5 AuthResult fields read: auth (packages/coding-agent/src/core/model-registry.ts:101), env (packages/coding-agent/src/core/model-registry.ts:104)
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
// packages/coding-agent/src/core/model-registry.ts:101
				ok: true,
				apiKey: resolution.auth.apiKey,
				headers: resolution.auth.headers,
```
```
// packages/coding-agent/src/core/model-registry.ts:104
				...(resolution.auth.baseUrl ? { baseUrl: resolution.auth.baseUrl } : {}),
				env: resolution.env,
			};
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 9: code:types.ts->model-runtime.ts
- E2 AuthResult declared at packages/ai/src/auth/types.ts:104
- E4 AuthResult imported at packages/coding-agent/src/core/model-runtime.ts:2 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:24 from packages/ai/src/auth/types.ts
- E5 AuthResult fields read: auth (packages/coding-agent/src/core/model-runtime.ts:404), env (packages/coding-agent/src/core/model-runtime.ts:247), source (packages/coding-agent/src/core/model-runtime.ts:655)
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
// packages/coding-agent/src/core/model-runtime.ts:404
			const storedProviders = new Set(this.snapshot.storedProviders);
			const authByProvider = new Map(this.snapshot.auth);
			if (auth) {
```
```
// packages/coding-agent/src/core/model-runtime.ts:247
			providers,
			process.env.PI_OFFLINE === undefined,
		);
```
```
// packages/coding-agent/src/core/model-runtime.ts:655
		const check = this.snapshot.auth.get(providerId);
		return check ? { configured: true, source: "environment", label: check.source } : { configured: false };
	}
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 10: code:types.ts->provider-composer.ts
- E2 AuthResult declared at packages/ai/src/auth/types.ts:104
- E2 ModelAuth declared at packages/ai/src/auth/types.ts:7
- E4 AuthResult imported at packages/coding-agent/src/core/provider-composer.ts:1 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:24 from packages/ai/src/auth/types.ts
- E4 ModelAuth imported at packages/coding-agent/src/core/provider-composer.ts:1 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:24 from packages/ai/src/auth/types.ts
- E5 AuthResult fields read: auth (packages/coding-agent/src/core/provider-composer.ts:423), env (packages/coding-agent/src/core/provider-composer.ts:411), source (packages/coding-agent/src/core/provider-composer.ts:443)
- E5 AuthResult stoplisted reads: (none)
- E5 ModelAuth fields read: apiKey (packages/coding-agent/src/core/provider-composer.ts:316), baseUrl (packages/coding-agent/src/core/provider-composer.ts:222), headers (packages/coding-agent/src/core/provider-composer.ts:313)
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
// packages/coding-agent/src/core/provider-composer.ts:423
): ApiKeyAuth | undefined {
	const inherited = base?.auth.apiKey;
	const rawKey = configuredApiKey(config, extension);
```
```
// packages/coding-agent/src/core/provider-composer.ts:411
		if (env[name] !== undefined) continue;
		const value = await ctx.env(name);
		if (value !== undefined) env[name] = value;
```
```
// packages/coding-agent/src/core/provider-composer.ts:443
				const resolved = await inherited?.resolve(input);
				return resolved ? { type: "api_key", source: resolved.source } : undefined;
			}
```
```
// packages/coding-agent/src/core/provider-composer.ts:316
		!hasOverrides &&
		!config.apiKey &&
		!config.oauth &&
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 11: code:types.ts->provider.ts
- E2 AuthResult declared at packages/ai/src/auth/types.ts:104
- E4 AuthResult imported at packages/coding-agent/src/extensions/llama/provider.ts:1 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:24 from packages/ai/src/auth/types.ts
- E5 AuthResult fields read: env (packages/coding-agent/src/extensions/llama/provider.ts:27), source (packages/coding-agent/src/extensions/llama/provider.ts:44)
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
// packages/coding-agent/src/extensions/llama/provider.ts:27
function credentialServerUrl(credential: ApiKeyCredential | undefined): string | undefined {
	const value = credential?.env?.LLAMA_BASE_URL;
	return typeof value === "string" && value.trim() ? normalizeLlamaServerUrl(value) : undefined;
```
```
// packages/coding-agent/src/extensions/llama/provider.ts:44
	// Unloaded presets are routable only when llama.cpp router autoload can load them on first use.
	return routerAutoload && model.status.value === "unloaded" && !model.status.failed && model.source === "preset";
}
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 12: code:types.ts->utilities.ts
- E2 OAuthCredentials declared at packages/ai/src/auth/types.ts:24
- E4 OAuthCredentials imported at packages/coding-agent/test/utilities.ts:10 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:24 from packages/ai/src/auth/types.ts
- E5 OAuthCredentials fields read: expires (packages/coding-agent/test/utilities.ts:95), refresh (packages/coding-agent/test/utilities.ts:96)
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
// packages/coding-agent/test/utilities.ts:95
		let credential = entry;
		if (Date.now() >= credential.expires) {
			credential = await oauth.refresh(credential, new AbortController().signal);
```
```
// packages/coding-agent/test/utilities.ts:96
		if (Date.now() >= credential.expires) {
			credential = await oauth.refresh(credential, new AbortController().signal);
			storage[provider] = credential;
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 13: code:models-store.ts->remote-catalog-provider.ts
- E2 ModelsStoreEntry declared at packages/ai/src/models-store.ts:3
- E4 ModelsStoreEntry imported at packages/coding-agent/src/core/remote-catalog-provider.ts:1 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:34 from packages/ai/src/models-store.ts
- E5 ModelsStoreEntry fields read: checkedAt (packages/coding-agent/src/core/remote-catalog-provider.ts:137), etag (packages/coding-agent/src/core/remote-catalog-provider.ts:146), lastModified (packages/coding-agent/src/core/remote-catalog-provider.ts:72), models (packages/coding-agent/src/core/remote-catalog-provider.ts:41)
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
// packages/coding-agent/src/core/remote-catalog-provider.ts:137
				!context.force &&
				stored?.checkedAt !== undefined &&
				stored.lastModified !== undefined &&
```
```
// packages/coding-agent/src/core/remote-catalog-provider.ts:146
			// leave the overlay empty.
			const validator = stored && stored.models.length > 0 ? stored.etag : undefined;
			const response = await requestCatalog(provider.id, catalogBaseUrl, context.signal, validator);
```
```
// packages/coding-agent/src/core/remote-catalog-provider.ts:72
function remoteModels(entry: ModelsStoreEntry | undefined, localGeneratedAt: number | undefined): readonly AnyModel[] {
	return entry && isNewer(entry.lastModified, localGeneratedAt) ? entry.models : [];
}
```
```
// packages/coding-agent/src/core/remote-catalog-provider.ts:41
		? value
		: typeof value === "object" && value !== null && "models" in value && Array.isArray(value.models)
			? value.models
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 14: code:models.ts->model-registry.ts
- E2 Provider declared at packages/ai/src/models.ts:155
- E4 Provider imported at packages/coding-agent/src/core/model-registry.ts:1 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:33 from packages/ai/src/models.ts
- E5 Provider fields read: auth (packages/coding-agent/src/core/model-registry.ts:101), baseUrl (packages/coding-agent/src/core/model-registry.ts:103), headers (packages/coding-agent/src/core/model-registry.ts:97)
- E5 Provider stoplisted reads: name
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
// packages/coding-agent/src/core/model-registry.ts:101
				ok: true,
				apiKey: resolution.auth.apiKey,
				headers: resolution.auth.headers,
```
```
// packages/coding-agent/src/core/model-registry.ts:103
				headers: resolution.auth.headers,
				...(resolution.auth.baseUrl ? { baseUrl: resolution.auth.baseUrl } : {}),
				env: resolution.env,
```
```
// packages/coding-agent/src/core/model-registry.ts:97
				}
				return { ok: true, headers: compatibility.headers };
			}
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 15: code:models.ts->model-runtime.ts
- E2 ModelsRefreshOptions declared at packages/ai/src/models.ts:103
- E2 ModelsRefreshResult declared at packages/ai/src/models.ts:112
- E2 Provider declared at packages/ai/src/models.ts:155
- E4 ModelsRefreshOptions imported at packages/coding-agent/src/core/model-runtime.ts:2 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:33 from packages/ai/src/models.ts
- E4 ModelsRefreshResult imported at packages/coding-agent/src/core/model-runtime.ts:2 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:33 from packages/ai/src/models.ts
- E4 Provider imported at packages/coding-agent/src/core/model-runtime.ts:2 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:33 from packages/ai/src/models.ts
- E5 ModelsRefreshOptions fields read: allowNetwork (packages/coding-agent/src/core/model-runtime.ts:859), providers (packages/coding-agent/src/core/model-runtime.ts:851), signal (packages/coding-agent/src/core/model-runtime.ts:256)
- E5 ModelsRefreshOptions stoplisted reads: (none)
- E5 ModelsRefreshResult fields read: aborted (packages/coding-agent/src/core/model-runtime.ts:381), errors (packages/coding-agent/src/core/model-runtime.ts:612)
- E5 ModelsRefreshResult stoplisted reads: (none)
- E5 Provider fields read: auth (packages/coding-agent/src/core/model-runtime.ts:404), baseUrl (packages/coding-agent/src/core/model-runtime.ts:275), headers (packages/coding-agent/src/core/model-runtime.ts:574)
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
// packages/coding-agent/src/core/model-runtime.ts:381
		return this.runAvailabilityRefresh(seq, errorSeq, effectiveSignal).catch((error) => {
			if (errorSeq === this.availabilityErrorSeq && !effectiveSignal.aborted) {
				this.availabilityError = error instanceof Error ? error.message : String(error);
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 16: code:models.ts->provider-composer.ts
- E2 Provider declared at packages/ai/src/models.ts:155
- E2 RefreshModelsContext declared at packages/ai/src/models.ts:80
- E4 Provider imported at packages/coding-agent/src/core/provider-composer.ts:1 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:33 from packages/ai/src/models.ts
- E4 RefreshModelsContext imported at packages/coding-agent/src/core/provider-composer.ts:1 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:33 from packages/ai/src/models.ts
- E5 Provider fields read: auth (packages/coding-agent/src/core/provider-composer.ts:423), baseUrl (packages/coding-agent/src/core/provider-composer.ts:222), headers (packages/coding-agent/src/core/provider-composer.ts:313)
- E5 Provider stoplisted reads: id, name
- E5 RefreshModelsContext fields read: baseUrl (packages/coding-agent/src/core/provider-composer.ts:222), credential (packages/coding-agent/src/core/provider-composer.ts:439), signal (packages/coding-agent/src/core/provider-composer.ts:365)
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
// packages/coding-agent/src/core/provider-composer.ts:423
): ApiKeyAuth | undefined {
	const inherited = base?.auth.apiKey;
	const rawKey = configuredApiKey(config, extension);
```
```
// packages/coding-agent/src/core/provider-composer.ts:222
	}
	const baseUrl = definition.baseUrl ?? providerConfig.baseUrl ?? defaults?.baseUrl;
	if (!baseUrl) throw new Error(`Provider ${providerId}: "baseUrl" is required when defining custom models.`);
```
```
// packages/coding-agent/src/core/provider-composer.ts:313
		!config.baseUrl &&
		!config.headers &&
		!config.compat &&
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 17: code:models.ts->remote-catalog-provider.ts
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

### earendil-works__pi 18: code:models.ts->provider.ts
- E2 RefreshModelsContext declared at packages/ai/src/models.ts:80
- E4 RefreshModelsContext imported at packages/coding-agent/src/extensions/llama/provider.ts:1 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:33 from packages/ai/src/models.ts
- E5 RefreshModelsContext fields read: allowNetwork (packages/coding-agent/src/extensions/llama/provider.ts:255), credential (packages/coding-agent/src/extensions/llama/provider.ts:255), signal (packages/coding-agent/src/extensions/llama/provider.ts:204), stored (packages/coding-agent/src/extensions/llama/provider.ts:233)
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
// packages/coding-agent/src/extensions/llama/provider.ts:204
					).trim();
					await new LlamaClient(serverUrl, apiKey || undefined).list({ signal: interaction.signal });
					return {
```
```
// packages/coding-agent/src/extensions/llama/provider.ts:233
			const cachedContextWindows = new Map<string, number>();
			if (context.stored) {
				const stored = context.stored.models.filter((model) => model.provider === LLAMA_PROVIDER_ID);
```
Judgment (TRUE / NOT TRUE, one sentence):

### earendil-works__pi 19: code:models.ts->llama-extension.test.ts
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

### earendil-works__pi 20: code:models.ts->remote-catalog-provider.test.ts
- E2 ModelsPublication declared at packages/ai/src/models.ts:73
- E2 RefreshModelsContext declared at packages/ai/src/models.ts:80
- E4 ModelsPublication imported at packages/coding-agent/test/remote-catalog-provider.test.ts:1 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:33 from packages/ai/src/models.ts
- E4 RefreshModelsContext imported at packages/coding-agent/test/remote-catalog-provider.test.ts:1 from "@earendil-works/pi-ai", resolved to packages/ai/src/index.ts, re-exported by barrel packages/ai/src/index.ts:33 from packages/ai/src/models.ts
- E5 ModelsPublication fields read: persist (packages/coding-agent/test/remote-catalog-provider.test.ts:63), update (packages/coding-agent/test/remote-catalog-provider.test.ts:65)
- E5 ModelsPublication stoplisted reads: (none)
- E5 RefreshModelsContext fields read: allowNetwork (packages/coding-agent/test/remote-catalog-provider.test.ts:72), force (packages/coding-agent/test/remote-catalog-provider.test.ts:73), signal (packages/coding-agent/test/remote-catalog-provider.test.ts:20)
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
```
// packages/coding-agent/test/remote-catalog-provider.test.ts:72
		publish,
		allowNetwork: overrides.allowNetwork ?? true,
		force: overrides.force,
```
```
// packages/coding-agent/test/remote-catalog-provider.test.ts:73
		allowNetwork: overrides.allowNetwork ?? true,
		force: overrides.force,
		signal: overrides.signal ?? neverAbortedSignal,
```
Judgment (TRUE / NOT TRUE, one sentence):

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

## puppeteer__puppeteer: 5 proposal(s), first 5 adjudicated

### puppeteer__puppeteer 1: code:ElementHandle.ts->elementhandle.test.ts
- E2 Point declared at packages/puppeteer-core/src/api/ElementHandle.ts:109
- E4 Point imported at test/src/elementhandle.test.ts:9 from "puppeteer-core/internal/api/ElementHandle.js", resolved to packages/puppeteer-core/src/api/ElementHandle.ts
- E5 Point fields read: x (test/src/elementhandle.test.ts:93), y (test/src/elementhandle.test.ts:93)
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
// test/src/elementhandle.test.ts:93
        const rect = e.getBoundingClientRect();
        return {x: rect.x, y: rect.y, width: rect.width, height: rect.height};
      }, element);
```
Judgment (TRUE / NOT TRUE, one sentence):

### puppeteer__puppeteer 2: code:Page.ts->page.test.ts
- E2 Metrics declared at packages/puppeteer-core/src/api/Page.ts:129
- E4 Metrics imported at test/src/page.test.ts:16 from "puppeteer-core/internal/api/Page.js", resolved to packages/puppeteer-core/src/api/Page.ts
- E5 Metrics fields read: Frames (test/src/page.test.ts:133), Timestamp (test/src/page.test.ts:557)
- E5 Metrics stoplisted reads: (none)
```
// packages/puppeteer-core/src/api/Page.ts:129
export interface Metrics {
  /**
   * The timestamp when the metrics sample was taken, in monotonic time
   * (seconds since an arbitrary point in the past).
   */
  Timestamp?: number;
  /**
   * Number of documents in the page.
   */
  Documents?: number;
  /**
   * Number of frames in the page.
```
```
// test/src/page.test.ts:133
      await newPage.goto(server.PREFIX + '/frames/one-frame.html');
      expect(newPage.frames().length).toBe(2);
      await newPage.close();
```
```
// test/src/page.test.ts:557
      await page.evaluate(() => {
        return console.timeStamp('test42');
      });
```
Judgment (TRUE / NOT TRUE, one sentence):

### puppeteer__puppeteer 3: code:Accessibility.ts->accessibility.test.ts
- E2 SerializedAXNode declared at packages/puppeteer-core/src/cdp/Accessibility.ts:18
- E4 SerializedAXNode imported at test/src/accessibility.test.ts:10 from "puppeteer-core/internal/cdp/Accessibility.js", resolved to packages/puppeteer-core/src/cdp/Accessibility.ts
- E5 SerializedAXNode fields read: children (test/src/accessibility.test.ts:217), focused (test/src/accessibility.test.ts:902)
- E5 SerializedAXNode stoplisted reads: name
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
// test/src/accessibility.test.ts:217
    assert(snapshot);
    assert(snapshot.children);
    assert(snapshot.children[0]);
```
```
// test/src/accessibility.test.ts:902
  ): SerializedAXNode | null {
    if (node?.focused) {
      return node;
```
Judgment (TRUE / NOT TRUE, one sentence):

### puppeteer__puppeteer 4: code:WebMCP.ts->webmcp.test.ts
- E2 WebMCPToolCallResult declared at packages/puppeteer-core/src/cdp/WebMCP.ts:207
- E4 WebMCPToolCallResult imported at test/src/cdp/webmcp.test.ts:11 from "puppeteer-core/internal/cdp/WebMCP.js", resolved to packages/puppeteer-core/src/cdp/WebMCP.ts
- E5 WebMCPToolCallResult fields read: call (test/src/cdp/webmcp.test.ts:501), errorText (test/src/cdp/webmcp.test.ts:504), exception (test/src/cdp/webmcp.test.ts:505), output (test/src/cdp/webmcp.test.ts:503)
- E5 WebMCPToolCallResult stoplisted reads: id, status
```
// packages/puppeteer-core/src/cdp/WebMCP.ts:207
export interface WebMCPToolCallResult {
  /**
   * Tool invocation identifier.
   */
  id: string;
  /**
   * The corresponding tool call if available.
   */
  call?: WebMCPToolCall;
  /**
   * Status of the invocation.
   */
```
```
// test/src/cdp/webmcp.test.ts:501
    expect(response.id).toBe(call.id);
    expect(response.call).toBe(call);
    expect(response.status).toBe('Completed');
```
```
// test/src/cdp/webmcp.test.ts:504
    expect(response.output).toBe('hello world');
    expect(response.errorText).toBeUndefined();
    expect(response.exception).toBeUndefined();
```
```
// test/src/cdp/webmcp.test.ts:505
    expect(response.errorText).toBeUndefined();
    expect(response.exception).toBeUndefined();
  });
```
```
// test/src/cdp/webmcp.test.ts:503
    expect(response.status).toBe('Completed');
    expect(response.output).toBe('hello world');
    expect(response.errorText).toBeUndefined();
```
Judgment (TRUE / NOT TRUE, one sentence):

### puppeteer__puppeteer 5: code:LaunchOptions.ts->mocha-utils.ts
- E2 LaunchOptions declared at packages/puppeteer-core/src/node/LaunchOptions.ts:39
- E4 LaunchOptions imported at test/src/mocha-utils.ts:19 from "puppeteer-core/internal/node/LaunchOptions.js", resolved to packages/puppeteer-core/src/node/LaunchOptions.ts
- E5 LaunchOptions fields read: args (test/src/mocha-utils.ts:106), browser (test/src/mocha-utils.ts:197), dumpio (test/src/mocha-utils.ts:94), env (test/src/mocha-utils.ts:65), executablePath (test/src/mocha-utils.ts:117), extraPrefsFirefox (test/src/mocha-utils.ts:104), headless (test/src/mocha-utils.ts:67), timeout (test/src/mocha-utils.ts:175)
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
// test/src/mocha-utils.ts:197
    try {
      if (!state.browser) {
        if (!browserPromise) {
```
```
// test/src/mocha-utils.ts:94
    headless: headless === 'shell' ? ('shell' as const) : isHeadless,
    dumpio: !!process.env['DUMPIO'],
    protocol,
```
```
// test/src/mocha-utils.ts:65
const product =
  process.env['PRODUCT'] || process.env['PUPPETEER_BROWSER'] || 'chrome';

```
Judgment (TRUE / NOT TRUE, one sentence):

