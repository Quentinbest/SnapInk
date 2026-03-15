import Konva from 'konva';
import type {
  Annotation,
  ArrowAnnotation,
  BlurAnnotation,
  EllipseAnnotation,
  LineAnnotation,
  PenAnnotation,
  RectAnnotation,
  StepAnnotation,
  TextAnnotation,
} from './types';

export class AnnotationEngine {
  stage: Konva.Stage;
  baseLayer: Konva.Layer;
  annotationLayer: Konva.Layer;
  previewLayer: Konva.Layer;
  uiLayer: Konva.Layer;
  transformer: Konva.Transformer;

  /** Draggable endpoint handles shown when a line/arrow is selected. */
  private lineHandleGroup: Konva.Group | null = null;

  /** Called when a line/arrow endpoint is dragged and released. */
  onLineEndpointMoved?: (id: string, x1: number, y1: number, x2: number, y2: number) => void;

  constructor(container: HTMLDivElement, width: number, height: number) {
    this.stage = new Konva.Stage({ container, width, height });

    this.baseLayer = new Konva.Layer();
    this.annotationLayer = new Konva.Layer();
    this.previewLayer = new Konva.Layer();
    this.uiLayer = new Konva.Layer();

    this.stage.add(this.baseLayer);
    this.stage.add(this.annotationLayer);
    this.stage.add(this.previewLayer);
    this.stage.add(this.uiLayer);

    this.transformer = new Konva.Transformer({
      borderStroke: '#0A84FF',
      borderStrokeWidth: 1.5,
      borderDash: [4, 2],
      anchorFill: 'white',
      anchorStroke: '#0A84FF',
      anchorStrokeWidth: 1.5,
      anchorSize: 8,
      anchorCornerRadius: 1,
      rotateEnabled: false,
    });
    this.uiLayer.add(this.transformer);
  }

  setBaseImage(dataUrl: string, onLoad?: () => void) {
    this.baseLayer.destroyChildren();
    const img = new Image();
    img.onload = () => {
      const kImg = new Konva.Image({ image: img, x: 0, y: 0, width: this.stage.width(), height: this.stage.height(), listening: false });
      this.baseLayer.add(kImg);
      this.baseLayer.batchDraw();
      onLoad?.();
    };
    img.src = dataUrl;
  }

  renderAnnotations(annotations: Annotation[], selectedId: string | null) {
    this.annotationLayer.destroyChildren();
    for (const ann of annotations) {
      const node = this.createNode(ann);
      if (node) {
        node.id(ann.id);
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        this.annotationLayer.add(node as any);
      }
    }
    if (selectedId) {
      const node = this.annotationLayer.findOne(`#${selectedId}`);
      const ann = annotations.find((a) => a.id === selectedId);
      if (node && ann) {
        if (ann.type === 'line' || ann.type === 'arrow') {
          // Use draggable endpoint circles instead of the bounding-box Transformer.
          this.transformer.nodes([]);
          this.showLineHandles(ann as LineAnnotation | ArrowAnnotation);
        } else {
          this.hideLineHandles();
          this.transformer.nodes([node]);
          // Disable resize anchors only for step (fixed-size numbered circle).
          if (ann.type === 'step') {
            this.transformer.enabledAnchors([]);
          } else {
            this.transformer.enabledAnchors([
              'top-left', 'top-center', 'top-right',
              'middle-left', 'middle-right',
              'bottom-left', 'bottom-center', 'bottom-right',
            ]);
          }
        }
      } else {
        this.hideLineHandles();
        this.transformer.nodes([]);
      }
    } else {
      this.hideLineHandles();
      this.transformer.nodes([]);
    }
    this.annotationLayer.batchDraw();
    this.uiLayer.batchDraw();
  }

  // ── Endpoint handles for line / arrow ───────────────────────────────

  /**
   * Replace the Transformer with two draggable circles at the line endpoints.
   * During drag: updates the annotation node in real-time.
   * On dragend: calls onLineEndpointMoved to persist to the store.
   */
  showLineHandles(ann: LineAnnotation | ArrowAnnotation) {
    this.hideLineHandles();

    const { id, x1, y1, x2, y2 } = ann;

    const makeHandle = (x: number, y: number) => new Konva.Circle({
      x, y,
      radius: 6,
      fill: 'white',
      stroke: '#0A84FF',
      strokeWidth: 1.5,
      draggable: true,
      hitStrokeWidth: 12,
    });

    const h1 = makeHandle(x1, y1);
    const h2 = makeHandle(x2, y2);

    // Live update the annotation line during drag.
    const updateLine = () => {
      const lineNode = this.annotationLayer.findOne(`#${id}`) as Konva.Line | null;
      if (lineNode) {
        lineNode.points([h1.x(), h1.y(), h2.x(), h2.y()]);
        this.annotationLayer.batchDraw();
      }
    };
    h1.on('dragmove', updateLine);
    h2.on('dragmove', updateLine);

    // Persist final endpoint positions to the store.
    const persist = () => {
      this.onLineEndpointMoved?.(id, h1.x(), h1.y(), h2.x(), h2.y());
    };
    h1.on('dragend', persist);
    h2.on('dragend', persist);

    const group = new Konva.Group();
    group.add(h1);
    group.add(h2);
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    this.uiLayer.add(group as any);
    this.uiLayer.batchDraw();
    this.lineHandleGroup = group;
  }

  hideLineHandles() {
    if (this.lineHandleGroup) {
      this.lineHandleGroup.destroy();
      this.lineHandleGroup = null;
      this.uiLayer.batchDraw();
    }
  }

  // ── Live preview ────────────────────────────────────────────────────

  /** Show a single Konva node on the preview layer (replaces previous). */
  showPreview(node: Konva.Node) {
    this.previewLayer.destroyChildren();
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    this.previewLayer.add(node as any);
    this.previewLayer.batchDraw();
  }

  /** Clear all preview shapes. */
  clearPreview() {
    this.previewLayer.destroyChildren();
    this.previewLayer.batchDraw();
  }

  // ── Node factory (public for preview use) ───────────────────────────

  createNode(ann: Annotation): Konva.Node | null {
    switch (ann.type) {
      case 'rect': return this.createRect(ann);
      case 'ellipse': return this.createEllipse(ann);
      case 'line': return this.createLine(ann);
      case 'arrow': return this.createArrow(ann);
      case 'pen': return this.createPen(ann);
      case 'text': return this.createText(ann);
      case 'step': return this.createStep(ann);
      case 'blur': return this.createBlur(ann);
      default: return null;
    }
  }

  private createRect(ann: RectAnnotation): Konva.Rect {
    return new Konva.Rect({
      x: ann.x,
      y: ann.y,
      width: ann.width,
      height: ann.height,
      stroke: ann.color,
      strokeWidth: ann.strokeWidth,
      fill: ann.fill ? ann.color + '22' : 'transparent',
      cornerRadius: ann.cornerRadius,
      draggable: true,
    });
  }

  private createEllipse(ann: EllipseAnnotation): Konva.Ellipse {
    return new Konva.Ellipse({
      x: ann.x,
      y: ann.y,
      radiusX: ann.radiusX,
      radiusY: ann.radiusY,
      stroke: ann.color,
      strokeWidth: ann.strokeWidth,
      fill: 'transparent',
      draggable: true,
    });
  }

  private createLine(ann: LineAnnotation): Konva.Line {
    return new Konva.Line({
      points: [ann.x1, ann.y1, ann.x2, ann.y2],
      stroke: ann.color,
      strokeWidth: ann.strokeWidth,
      lineCap: 'round',
      draggable: true,
    });
  }

  private createArrow(ann: ArrowAnnotation): Konva.Arrow {
    const style = ann.headStyle ?? (ann.filledHead ? 'filled' : 'open');
    return new Konva.Arrow({
      points: [ann.x1, ann.y1, ann.x2, ann.y2],
      stroke: ann.color,
      strokeWidth: ann.strokeWidth,
      fill: (style === 'filled' || style === 'both') ? ann.color : 'transparent',
      pointerLength: 10,
      pointerWidth: 8,
      pointerAtBeginning: style === 'both',
      pointerAtEnding: style !== 'none',
      lineCap: 'round',
      draggable: true,
    });
  }

  private createPen(ann: PenAnnotation): Konva.Line {
    return new Konva.Line({
      points: ann.points,
      stroke: ann.color,
      strokeWidth: ann.strokeWidth,
      lineCap: 'round',
      lineJoin: 'round',
      tension: 0.5,
      draggable: true,
    });
  }

  private createText(ann: TextAnnotation): Konva.Label {
    const label = new Konva.Label({ x: ann.x, y: ann.y, draggable: true });
    if (ann.background) {
      label.add(new Konva.Tag({
        fill: 'rgba(255,255,255,0.92)',
        stroke: ann.color,
        strokeWidth: 1.5,
        cornerRadius: 5,
        shadowColor: 'rgba(0,0,0,0.15)',
        shadowBlur: 8,
        shadowOffsetY: 2,
      }));
    }
    label.add(new Konva.Text({
      text: ann.text,
      fontSize: ann.fontSize,
      fontStyle: ann.bold ? 'bold' : 'normal',
      fill: ann.color,
      padding: 4,
    }));
    return label;
  }

  private createStep(ann: StepAnnotation): Konva.Group {
    const group = new Konva.Group({ x: ann.x, y: ann.y, draggable: true });
    group.add(new Konva.Circle({
      radius: 12,
      fill: ann.color,
      shadowColor: ann.color + '80',
      shadowBlur: 8,
      shadowOffsetY: 2,
    }));
    group.add(new Konva.Text({
      text: String(ann.number),
      fontSize: 13,
      fontStyle: 'bold',
      fill: 'white',
      align: 'center',
      verticalAlign: 'middle',
      width: 24,
      height: 24,
      x: -12,
      y: -12,
    }));
    return group;
  }

  private createBlur(ann: BlurAnnotation): Konva.Rect {
    return new Konva.Rect({
      x: ann.x,
      y: ann.y,
      width: ann.width,
      height: ann.height,
      fill: 'rgba(180,180,200,0.5)',
      draggable: true,
    });
  }

  // ── Inline text editing ─────────────────────────────────────────────

  /**
   * Open an HTML textarea over the canvas at the given stage-relative
   * position.  Returns a Promise that resolves with the entered text
   * (or empty string if cancelled).
   */
  openTextInput(stageX: number, stageY: number, color: string): Promise<string> {
    return new Promise((resolve) => {
      const container = this.stage.container();
      const containerRect = container.getBoundingClientRect();
      // Convert stage coordinates to screen coordinates.
      const scaleX = containerRect.width / this.stage.width();
      const scaleY = containerRect.height / this.stage.height();

      const textarea = document.createElement('textarea');
      textarea.style.position = 'fixed';
      textarea.style.left = `${containerRect.left + stageX * scaleX}px`;
      textarea.style.top = `${containerRect.top + stageY * scaleY}px`;
      textarea.style.minWidth = '120px';
      textarea.style.minHeight = '28px';
      textarea.style.padding = '4px 6px';
      textarea.style.border = `2px solid ${color}`;
      textarea.style.borderRadius = '5px';
      textarea.style.background = 'rgba(255,255,255,0.95)';
      textarea.style.color = color;
      textarea.style.fontSize = '14px';
      textarea.style.fontFamily = '-apple-system, BlinkMacSystemFont, sans-serif';
      textarea.style.outline = 'none';
      textarea.style.resize = 'both';
      textarea.style.zIndex = '9999';
      textarea.style.boxShadow = '0 2px 12px rgba(0,0,0,0.25)';

      document.body.appendChild(textarea);

      let finished = false;
      const finish = () => {
        if (finished) return;
        finished = true;
        const text = textarea.value.trim();
        textarea.remove();
        resolve(text);
      };

      textarea.addEventListener('keydown', (e) => {
        // Stop propagation so the window-level keydown handler
        // (which handles Escape for deselect, etc.) doesn't interfere.
        e.stopPropagation();
        if (e.key === 'Enter' && !e.shiftKey) {
          e.preventDefault();
          finish();
        }
        if (e.key === 'Escape') {
          textarea.value = '';
          finish();
        }
      });

      // Delay focus and blur registration to the next frame so the
      // browser finishes processing the mousedown that triggered this
      // (otherwise the canvas recaptures focus and blurs the textarea
      // before the user can type).
      requestAnimationFrame(() => {
        textarea.focus();
        textarea.addEventListener('blur', finish, { once: true });
      });
    });
  }

  exportToDataURL(): string {
    this.transformer.nodes([]);
    this.uiLayer.batchDraw();
    const dataUrl = this.stage.toDataURL({ mimeType: 'image/png', pixelRatio: 2 });
    return dataUrl;
  }

  resize(width: number, height: number) {
    this.stage.width(width);
    this.stage.height(height);
  }

  destroy() {
    this.stage.destroy();
  }
}
