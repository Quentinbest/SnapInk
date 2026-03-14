export interface WindowInfo {
  id: number;
  title: string;
  appName: string;
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface MonitorInfo {
  id: number;
  name: string;
  x: number;
  y: number;
  width: number;
  height: number;
  scaleFactor: number;
  isPrimary: boolean;
}

export interface CaptureRegion {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface HotkeyBinding {
  action: string;
  shortcut: string;
}

export interface OutputSettings {
  savePath: string;
  filenamePattern: string;
  format: string;
  jpegQuality: number;
  retinaClipboard: boolean;
}

export interface CaptureSettings {
  defaultMode: string;
  showCursor: boolean;
  captureDelay: number;
  playSoundOnCapture: boolean;
}

export interface AnnotationSettings {
  defaultColor: string;
  palette: string[];
}

export interface UiSettings {
  theme: string;
  showMenuBarIcon: boolean;
  launchAtLogin: boolean;
}

export interface Settings {
  version: number;
  capture: CaptureSettings;
  afterCapture: string;
  alsoCopyAfterAnnotating: boolean;
  output: OutputSettings;
  hotkeys: HotkeyBinding[];
  annotations: AnnotationSettings;
  ui: UiSettings;
}

export type CaptureMode = 'area' | 'screen' | 'window' | 'scrolling' | 'ocr' | 'repeat';

export type OverlayState = 'idle' | 'dragging' | 'complete';

export type ToolType =
  | 'select'
  | 'rect'
  | 'ellipse'
  | 'line'
  | 'arrow'
  | 'pen'
  | 'blur'
  | 'text'
  | 'step';

export interface Point {
  x: number;
  y: number;
}

export interface AnnotationBase {
  id: string;
  type: ToolType;
  color: string;
  strokeWidth: number;
}

export interface RectAnnotation extends AnnotationBase {
  type: 'rect';
  x: number;
  y: number;
  width: number;
  height: number;
  cornerRadius: number;
  fill: boolean;
}

export interface EllipseAnnotation extends AnnotationBase {
  type: 'ellipse';
  x: number;
  y: number;
  radiusX: number;
  radiusY: number;
}

export interface LineAnnotation extends AnnotationBase {
  type: 'line';
  x1: number;
  y1: number;
  x2: number;
  y2: number;
}

export interface ArrowAnnotation extends AnnotationBase {
  type: 'arrow';
  x1: number;
  y1: number;
  x2: number;
  y2: number;
  filledHead: boolean;
}

export interface TextAnnotation extends AnnotationBase {
  type: 'text';
  x: number;
  y: number;
  text: string;
  fontSize: number;
  bold: boolean;
  background: boolean;
}

export interface PenAnnotation extends AnnotationBase {
  type: 'pen';
  points: number[];
}

export interface BlurAnnotation extends AnnotationBase {
  type: 'blur';
  x: number;
  y: number;
  width: number;
  height: number;
  strength: number;
  mode: 'blur' | 'pixelate';
}

export interface StepAnnotation extends AnnotationBase {
  type: 'step';
  x: number;
  y: number;
  number: number;
}

export type Annotation =
  | RectAnnotation
  | EllipseAnnotation
  | LineAnnotation
  | ArrowAnnotation
  | TextAnnotation
  | PenAnnotation
  | BlurAnnotation
  | StepAnnotation;
