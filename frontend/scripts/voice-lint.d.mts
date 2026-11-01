export interface BannedTerm {
  term: string
  pattern: RegExp
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
