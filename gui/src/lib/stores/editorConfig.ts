export interface EditorConfig {
  mode: "vim" | "normal" | "emacs";
  font_size: number;
  font_family?: string;
  show_line_numbers: boolean;
  line_wrapping: boolean;
  highlight_active_line: boolean;
  cursor_blink_rate: number;
  tab_size: number;
  use_tabs: boolean;
  close_brackets: boolean;
  bracket_matching: boolean;
  autocomplete: boolean;
  rectangular_selection: boolean;
  fold_gutter: boolean;
  match_highlighting: boolean;
}
