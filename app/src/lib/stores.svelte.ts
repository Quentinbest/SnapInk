import type { Annotation, Settings, ToolType } from './types';

const PALETTE = ['#FF3B30', '#FF9500', '#FFCC00', '#34C759', '#007AFF', '#AF52DE', '#1D1D1F', '#FFFFFF'];

const defaultSettings: Settings = {
  version: 1,
  capture: { defaultMode: 'area', showCursor: false, captureDelay: 0, playSoundOnCapture: false },
  afterCapture: 'open_editor',
  alsoCopyAfterAnnotating: true,
  output: {
    savePath: '',
    filenamePattern: 'SnapInk {YYYY-MM-DD} at {HH.mm.ss}',
    format: 'png',
    jpegQuality: 85,
    retinaClipboard: true,
  },
  hotkeys: [
    { action: 'capture_area', shortcut: 'Ctrl+Shift+4' },
    { action: 'capture_screen', shortcut: 'Ctrl+Shift+3' },
    { action: 'capture_window', shortcut: 'Ctrl+Shift+5' },
    { action: 'capture_scrolling', shortcut: 'Ctrl+Shift+6' },
    { action: 'capture_ocr', shortcut: 'Ctrl+Shift+7' },
    { action: 'repeat_last', shortcut: 'Ctrl+Shift+R' },
  ],
  annotations: { defaultColor: '#FF3B30', palette: PALETTE },
  ui: { theme: 'system', showMenuBarIcon: true, launchAtLogin: false },
};

function createAppStore() {
  let settings = $state<Settings>(defaultSettings);
  let activeTool = $state<ToolType>('select');
  let activeColor = $state('#FF3B30');
  let strokeWidth = $state(2);
  let annotations = $state<Annotation[]>([]);
  let selectedAnnotationId = $state<string | null>(null);
  let captureImageData = $state<string | null>(null);
  let undoStack = $state<Annotation[][]>([]);
  let redoStack = $state<Annotation[][]>([]);
  let stepCounter = $state(1);
  let toastMessage = $state<string | null>(null);
  let toastTimer = $state<ReturnType<typeof setTimeout> | null>(null);

  function setSettings(s: Settings) {
    settings = s;
  }

  function setActiveTool(tool: ToolType) {
    activeTool = tool;
  }

  function setActiveColor(color: string) {
    activeColor = color;
  }

  function setStrokeWidth(w: number) {
    strokeWidth = w;
  }

  function setCaptureImageData(data: string | null) {
    captureImageData = data;
    annotations = [];
    undoStack = [];
    redoStack = [];
    stepCounter = 1;
  }

  function addAnnotation(annotation: Annotation) {
    undoStack = [...undoStack.slice(-99), [...annotations]];
    redoStack = [];
    annotations = [...annotations, annotation];
    if (annotation.type === 'step') {
      stepCounter += 1;
    }
  }

  function updateAnnotation(id: string, patch: Partial<Annotation>) {
    annotations = annotations.map((a) => (a.id === id ? { ...a, ...patch } as Annotation : a));
  }

  function deleteAnnotation(id: string) {
    undoStack = [...undoStack.slice(-99), [...annotations]];
    redoStack = [];
    annotations = annotations.filter((a) => a.id !== id);
    if (selectedAnnotationId === id) {
      selectedAnnotationId = null;
    }
  }

  function selectAnnotation(id: string | null) {
    selectedAnnotationId = id;
  }

  function undo() {
    if (undoStack.length === 0) return;
    redoStack = [...redoStack, [...annotations]];
    annotations = undoStack[undoStack.length - 1];
    undoStack = undoStack.slice(0, -1);
  }

  function redo() {
    if (redoStack.length === 0) return;
    undoStack = [...undoStack, [...annotations]];
    annotations = redoStack[redoStack.length - 1];
    redoStack = redoStack.slice(0, -1);
  }

  function showToast(message: string) {
    if (toastTimer !== null) clearTimeout(toastTimer);
    toastMessage = message;
    toastTimer = setTimeout(() => {
      toastMessage = null;
      toastTimer = null;
    }, 2500);
  }

  function nextStepNumber() {
    return stepCounter;
  }

  return {
    get settings() { return settings; },
    get activeTool() { return activeTool; },
    get activeColor() { return activeColor; },
    get strokeWidth() { return strokeWidth; },
    get annotations() { return annotations; },
    get selectedAnnotationId() { return selectedAnnotationId; },
    get captureImageData() { return captureImageData; },
    get canUndo() { return undoStack.length > 0; },
    get canRedo() { return redoStack.length > 0; },
    get toastMessage() { return toastMessage; },
    get stepCounter() { return stepCounter; },
    setSettings,
    setActiveTool,
    setActiveColor,
    setStrokeWidth,
    setCaptureImageData,
    addAnnotation,
    updateAnnotation,
    deleteAnnotation,
    selectAnnotation,
    undo,
    redo,
    showToast,
    nextStepNumber,
  };
}

export const appStore = createAppStore();
