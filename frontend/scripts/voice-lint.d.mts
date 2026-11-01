export interface BannedTerm {
  term: string
  pattern: RegExp
}

export interface ExportEntry {
  id: string
  kind: string
  role: string | null
  template: string
}

export interface ExportCatalogue {
  version?: number
  rules?: {
    banned_phrases?: string[]
    direction_set?: string[]
    vital_words?: string[]
    min_pattern_sample?: number
    match?: string
  }
  entries?: ExportEntry[]
}

export declare const BANNED: BannedTerm[]
export declare const VITAL_WORDS: string[]
export declare const SAMPLE_SIZE: RegExp
export declare const SAMPLE_NOUN: RegExp

export declare function bannedWords(text: string): string[]
export declare function checkCatalogue(
  entries: Record<string, { type: string; text: string }>,
  directionWords: readonly string[],
): string[]
export declare function checkInlineText(file: string, text: string): string[]
export declare function checkExport(
  exported: ExportCatalogue,
  directionWords: readonly string[],
): string[]
