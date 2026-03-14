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
      const kImg = new Konva.Image({ image: img, x: 0, y: 0, width: this.stage.width(), height: this.stage.height() });
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
      if (node) {
        this.transformer.nodes([node]);
      } else {
        this.transformer.nodes([]);
      }
    } else {
      this.transformer.nodes([]);
    }
    this.annotationLayer.batchDraw();
    this.uiLayer.batchDraw();
  }

  private createNode(ann: Annotation): Konva.Node | null {
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
    return new Konva.Arrow({
      points: [ann.x1, ann.y1, ann.x2, ann.y2],
      stroke: ann.color,
      strokeWidth: ann.strokeWidth,
      fill: ann.filledHead ? ann.color : 'transparent',
      pointerLength: 10,
      pointerWidth: 8,
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
