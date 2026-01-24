export interface LanguageToolMatch {
  offset: number
  length: number
  message: string
  shortMessage: string
  replacements: { value: string }[]
  rule: {
    id: string
    category: {
      id: string
      name: string
    }
  }
  context?: {
    text: string
    offset: number
    length: number
  }
  errorText?: string
}

export interface LanguageToolResponse {
  matches: LanguageToolMatch[]
  language: {
    code: string
    name: string
    detectedLanguage: {
      code: string
      name: string
      confidence: number
    }
  }
}

export interface Settings {
  enabled: boolean
  language: 'auto' | 'fr' | 'en'
  apiUrl: string
  personalDictionary: string[]
  checkLevel: 'default' | 'picky'
  disabledRules: string[]
}

export const DEFAULT_SETTINGS: Settings = {
  enabled: true,
  language: 'auto',
  apiUrl: 'https://grammar-rs-autocorrect.fly.dev',
  personalDictionary: [],
  checkLevel: 'default',
  disabledRules: [],
}

// Common rules that users can toggle
export const COMMON_RULES = [
  {
    id: 'UPPERCASE_SENTENCE_START',
    name: 'Majuscule en debut de phrase',
    languages: ['fr', 'en'],
  },
  {
    id: 'COMMA_PARENTHESIS_WHITESPACE',
    name: 'Espaces et ponctuation',
    languages: ['fr', 'en'],
  },
  {
    id: 'DOUBLES_ESPACES',
    name: 'Doubles espaces',
    languages: ['fr'],
  },
  {
    id: 'FRENCH_WORD_REPEAT_RULE',
    name: 'Mots repetes',
    languages: ['fr'],
  },
  {
    id: 'CONSECUTIVE_SPACES',
    name: 'Doubles espaces',
    languages: ['en'],
  },
] as const

export type MessageType =
  | { type: 'GET_SETTINGS' }
  | { type: 'SET_SETTINGS'; settings: Partial<Settings> }
  | { type: 'SETTINGS_CHANGED'; settings: Settings }
  | { type: 'CHECK_CONNECTION' }
  | { type: 'CONNECTION_STATUS'; connected: boolean }
