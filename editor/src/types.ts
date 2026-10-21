export interface Span {
  start: number;
  end: number;
  line: number;
  col: number;
}

export interface UnitLength {
  val: number;
  unit: 'Mm' | 'Cm' | 'Inch' | 'Pt' | 'Px' | 'Em' | 'Percent';
}

export interface PaperConfig {
  size: string;
  orientation: string;
  margin: UnitLength;
}

export interface StyleRule {
  selectors: string[];
  declarations: Record<string, string>;
  span: Span;
}

export interface Stylesheet {
  rules: StyleRule[];
}

export interface TextNode {
  type: 'Text';
  content: string;
  span: Span;
}

export interface ElementNode {
  type: 'Element';
  tag: string;
  attributes: Record<string, string>;
  id?: string;
  classes: string[];
  inline_styles: Record<string, string>;
  children: DocNode[];
  span: Span;
}

export interface SectionNode {
  type: 'Section';
  page_style?: string;
  page_break: boolean;
  attributes: Record<string, string>;
  children: DocNode[];
  span: Span;
}

export interface CommentNode {
  type: 'Comment';
  content: string;
  span: Span;
}

export type DocNode = ElementNode | SectionNode | TextNode | CommentNode;

export interface RunningHeaderFooter {
  left: DocNode[];
  center: DocNode[];
  right: DocNode[];
}

export interface WatermarkDef {
  text: string;
  opacity: number;
  angle_deg: number;
}

export interface PageStyle {
  name: string;
  is_default: boolean;
  running_header?: RunningHeaderFooter;
  running_footer?: RunningHeaderFooter;
  watermark?: WatermarkDef;
}

export interface DocumentProps {
  metadata: Record<string, string>;
  paper: PaperConfig;
  stylesheet: Stylesheet;
  page_styles: Record<string, PageStyle>;
  default_page_style: string;
}

export interface BodyNode {
  attributes: Record<string, string>;
  children: DocNode[];
  span: Span;
}

export interface ParsedDocument {
  props: DocumentProps;
  body: BodyNode;
  span: Span;
}
