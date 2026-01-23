# grammar-rs

Correcteur grammatical ultra-performant en Rust.

## Architecture

Pipeline composable en 4 étapes :

```
Text → Tokenizer → Analyzer → Checker(s) → CheckResult
```

Chaque étape est un trait que vous pouvez implémenter/remplacer.

## Utilisation

```rust
use grammar_rs::prelude::*;

let pipeline = Pipeline::new(
    SimpleTokenizer::new(),
    PassthroughAnalyzer::new(),
)
.with_checker(SpellChecker::with_fst_dictionary(dict))
.with_checker(RuleChecker::new().with_english_rules());

let result = pipeline.check_text("helo  world");

for m in result.matches {
    println!("{}: {} ({:?})", m.rule_id, m.message, m.suggestions);
}
```

## CLI

```bash
cargo run --bin grammar-check -- "Your text here"

# Build FST dictionary
cargo run --bin build-dict -- data/dictionaries/en_US.txt data/dictionaries/en_US.fst
```

## Règles implémentées

### Règles universelles

| ID | Description | Exemple |
|----|-------------|---------|
| `SPELL` | Vérification orthographique (FST 370K mots) | `helo` → `hello` |
| `DOUBLE_SPACE` | Double espace | `hello  world` → `hello world` |
| `REPEATED_WORD` | Mot répété | `the the` → `the` |
| `UPPERCASE_SENTENCE_START` | Majuscule début de phrase | `hello.` → `Hello.` |
| `REPEATED_PUNCTUATION` | Ponctuation répétée | `Hello!!` → `Hello!` |
| `MISSING_SPACE_AFTER_PUNCT` | Espace manquant après ponctuation | `Hello.World` → `Hello. World` |

### Règles anglaises

| ID | Description | Exemple |
|----|-------------|---------|
| `EN_A_AN` | Article a/an basique | `a apple` → `an apple` |
| `EN_A_AN_IMPROVED` | Article a/an avec exceptions (silent h) | `a hour` → `an hour` |
| `SUBJECT_VERB_AGREEMENT` | Accord sujet-verbe | `he go` → `he goes` |
| `EN_ITS_ITS` | Confusion its/it's | `its great` → `it's great` |
| `EN_YOUR_YOURE` | Confusion your/you're | `your welcome` → `you're welcome` |
| `EN_THEIR_THEYRE_THERE` | Confusion their/they're/there | `there going` → `they're going` |
| `COMMA_SPLICE` | Virgule splice | `I went, I saw` → `I went. I saw` |
| `LESS_FEWER` | Less vs fewer (dénombrable) | `less items` → `fewer items` |
| `WHO_WHOM` | Who vs whom | `to who` → `to whom` |
| `GOOD_WELL` | Adjectif vs adverbe | `did good` → `did well` |
| `DOUBLE_NEGATIVE` | Double négation | `don't have no` → `don't have any` |

### Règles de style

| ID | Description | Exemple |
|----|-------------|---------|
| `PASSIVE_VOICE` | Détection voix passive | `was eaten` → active voice suggested |
| `WORDINESS` | Phrases verbeuses | `in order to` → `to` |
| `SENTENCE_FRAGMENT` | Fragment de phrase | `Because it rained.` → needs main clause |
| `TYPOGRAPHIC_QUOTES` | Guillemets typographiques | `"hello"` → `"hello"` or `«hello»` |
| `SENTENCE_LENGTH` | Phrases trop longues (>40 mots) | Suggestion de découper |
| `CLICHE` | Détection de clichés business | `think outside the box` → be creative |
| `REDUNDANCY` | Pléonasmes | `past history` → `history` |
| `CONTRACTION` | Apostrophe manquante | `dont` → `don't` |
| `CONTEXT_WORD` | Mot confus selon contexte | `heroin addict` ✓, `heroin of novel` → `heroine` |
| `DIACRITICS` | Accents manquants | `aperitif` → `apéritif` |

### Règles françaises

| ID | Description | Exemple |
|----|-------------|---------|
| `FR_PUNCT_SPACE` | Espace avant ponctuation | `Bonjour!` → `Bonjour !` |
| `FR_A_ACCENT` | Confusion a/à | `a Paris` → `à Paris` |
| `FR_OU_ACCENT` | Confusion ou/où | `ou est-il` → `où est-il` |
| `FR_CE_SE` | Confusion ce/se | `ce lève` → `se lève` |
| `FR_SUBJECT_VERB` | Accord sujet-verbe | `ils mange` → `ils mangent` |
| `FR_ADJ_NOUN` | Accord adjectif-nom | `une petit maison` → `une petite maison` |
| `FR_CONDITIONNEL_SI` | Pas de conditionnel après si | `si j'aurais` → `si j'avais` |
| `FR_TOUT_ACCORD` | Accord de "tout" | `tout les` → `tous les` |

### Règles de confusion (importées de LanguageTool)

| ID | Description | Paires |
|----|-------------|--------|
| `EN_CONFUSION` | Paires confuses anglaises (homophones) | 1363 mots uniques |
| `FR_CONFUSION` | Paires confuses françaises | 23 mots uniques |

Exemples de paires détectées:
- `the affect` → `the effect`
- `loose weight` → `lose weight`
- `was lead` → `was led`
- `le notre` → `le nôtre`

### Pattern Rules (importées de LanguageTool)

401 règles de patterns multi-mots importées automatiquement :
- **334 règles anglaises** : `could of` → `could have`, `tow the line` → `toe the line`...
- **67 règles françaises** : `mille merci` → `mille mercis`, `au final` → `finalement`...

Utilisent l'algorithme Aho-Corasick pour un matching en O(N) au lieu de O(R × N).

### Style Rules (importées de LanguageTool)

1,398 règles de style importées automatiquement :
- **692 règles de verbosité** : `in order to` → `to`, `a number of` → `many`
- **706 règles de redondance** : `12 noon` → `noon`, `absolutely essential` → `essential`

Utilisent Aho-Corasick pour un matching de phrases en O(N).

### Coherency Rules (importées de LanguageTool)

2,479 paires de cohérence (UK/US spelling) :
- `analyse` vs `analyze` - détecte quand les deux sont utilisés dans le même document
- `organise` vs `organize`
- `cancelled` vs `canceled`

Le CoherencyChecker suit les variantes utilisées et signale les incohérences.

### Diacritics Rules (importées de LanguageTool)

1,219 règles de diacritiques pour mots empruntés :
- `aperitif` → `apéritif`
- `attache` → `attaché`
- `a la carte` → `à la carte`
- `Academie Francaise` → `Académie Française`

Le DiacriticsChecker suggère les accents corrects avec préservation de la casse.

### Contraction Rules (importées de LanguageTool)

179 règles de contractions pour détecter les apostrophes manquantes :
- `dont` → `don't`
- `Im` → `I'm`
- `youre` → `you're`
- `DONT` → `DON'T` (sensible à la casse)

Le ContractionChecker détecte les mots écrits sans apostrophe.

### Determiner Rules (importées de LanguageTool)

4,782 mots pour améliorer la règle a/an :
- **939 mots avec "a"** malgré voyelle initiale : `European`, `university`, `one-time`
- **3,843 mots avec "an"** malgré consonne initiale : `hour`, `honest`, `MBA`, `8th`

```rust
use grammar_rs::checker::{requires_en_a, requires_en_an};

assert!(requires_en_a("European"));  // "a European" (not "an")
assert!(requires_en_an("hour"));     // "an hour" (not "a")
```

### Context-Sensitive Word Rules (importées de LanguageTool)

11 règles pour détecter les mots confus selon le contexte :
- `heroin` vs `heroine` → contexte: "addict/morphine" vs "literature/character"
- `bazaar` vs `bizarre` → contexte: "marketplace" vs "unusual"
- `prescribe` vs `proscribe` → contexte médical vs légal
- `dessert` vs `desert` → contexte: "chocolate/menu" vs "dry/arid"

Le ContextChecker utilise des patterns regex pour analyser le contexte environnant.

### Synonyms (importés de LanguageTool)

167 règles de synonymes pour suggestions de style :
- **25 règles EN** : `synonyms.txt`
- **142 règles FR** : `notamment` → `particulièrement`, `surtout`

```rust
use grammar_rs::checker::get_fr_synonyms;

if let Some(syns) = get_fr_synonyms("notamment") {
    // ["particulièrement", "spécialement", "singulièrement", "surtout", "spécifiquement"]
}
```

### Détection de langue (importée de LanguageTool)

19,619 mots courants pour la détection de langue :
- **9,890 mots anglais** : the, and, is, are, you, we, they...
- **9,729 mots français** : le, la, les, un, une, de, je, tu...

```rust
use grammar_rs::prelude::*;

let detector = LanguageDetector::new();

// Détection automatique
let lang = detector.detect("Bonjour, comment allez-vous ?");
assert_eq!(lang, Language::French);

let lang = detector.detect("Hello, how are you doing?");
assert_eq!(lang, Language::English);

// Avec score de confiance
let result = detector.detect_with_confidence("Le chat est sur le tapis.");
println!("Language: {:?}, Confidence: {:.2}", result.language, result.confidence);
```

Le détecteur utilise :
- Lookup binaire O(log N) sur ~10k mots par langue
- Caractères accentués comme indicateurs forts (é, è, ç...)
- Terminaison précoce pour les textes clairs

## Performance

### Aho-Corasick Pattern Matching

Le `PatternRuleChecker` utilise l'algorithme Aho-Corasick pour un matching multi-pattern en O(N) au lieu de O(R × N).

```rust
// Checker optimisé (recommandé)
let checker = AhoPatternRuleChecker::new(EN_PATTERN_RULES);

// Checker naïf (pour comparaison)
let checker = PatternRuleChecker::new(EN_PATTERN_RULES);
```

**Benchmarks** (334 règles EN, texte typique):

| Test | Naïf | Aho-Corasick | Speedup |
|------|------|--------------|---------|
| Texte court | 887µs | 9.7µs | **91x** |
| Texte avec matches | 282µs | 3.2µs | **88x** |
| Patterns FR | 32µs | 1.4µs | **23x** |

Construction Aho-Corasick : 508µs (coût unique au démarrage).

```bash
# Lancer les benchmarks (~20 secondes)
cargo bench --bench rules
```

## Quality Benchmark

```bash
cargo test quality -- --nocapture
```

### Résultats actuels

```
╔════════════════════════════════════════════╗
║     RÉSUMÉ (grammar-rs v0.1.0)             ║
╚════════════════════════════════════════════╝
Règles implémentées: 32
Cas de test: 97
Precision: 100.0% | Recall: 100.0% | F1: 100.0%

Catégories implémentées: 32/32 (100%)
```

### Couverture par catégorie

```
[✓] Orthographe (SPELL)            - 5/5 (100%)
[✓] Double espace                  - 3/3 (100%)
[✓] Mot répété                     - 3/3 (100%)
[✓] A/An basique                   - 4/4 (100%)
[✓] Ponctuation FR                 - 4/4 (100%)
[✓] Majuscule début phrase         - 3/3 (100%)
[✓] Ponctuation répétée            - 3/3 (100%)
[✓] Espace après ponctuation       - 3/3 (100%)
[✓] Accord sujet-verbe (EN)        - 4/4 (100%)
[✓] Its/It's confusion             - 3/3 (100%)
[✓] Your/You're confusion          - 3/3 (100%)
[✓] A/An amélioré (silent h)       - 3/3 (100%)
[✓] Ce/Se confusion (FR)           - 2/2 (100%)
[✓] Their/They're/There (EN)       - 3/3 (100%)
[✓] Comma splice (EN)              - 3/3 (100%)
[✓] Passive voice detection        - 3/3 (100%)
[✓] Wordiness detection            - 3/3 (100%)
[✓] Sentence fragments             - 3/3 (100%)
[✓] Accord sujet-verbe (FR)        - 3/3 (100%)
[✓] Accord adjectif-nom (FR)       - 3/3 (100%)
[✓] Guillemets typographiques      - 3/3 (100%)
[✓] English confusion pairs        - 3/3 (100%)
[✓] French confusion pairs         - 1/1 (100%)
[✓] Less/Fewer (EN)                - 3/3 (100%)
[✓] Who/Whom (EN)                  - 3/3 (100%)
[✓] Good/Well (EN)                 - 3/3 (100%)
[✓] Double negative (EN)           - 3/3 (100%)
[✓] Conditionnel après si (FR)     - 3/3 (100%)
[✓] Accord de tout (FR)            - 3/3 (100%)
[✓] Longueur de phrase             - 2/2 (100%)
[✓] Clichés détectés               - 3/3 (100%)
[✓] Pléonasmes détectés            - 3/3 (100%)
```

## Validation LanguageTool (610 exemples)

Les pattern rules sont validées contre 610 exemples extraits automatiquement de LanguageTool.

```bash
cargo test --test pattern_validation -- --nocapture
```

### Distribution des exemples

| Langue | Incorrect | Correct | Total |
|--------|-----------|---------|-------|
| Anglais | 361 | 162 | 523 |
| Français | 69 | 18 | 87 |
| **Total** | **430** | **180** | **610** |

### Synchronisation avec LanguageTool

```bash
# Resynchroniser les règles et exemples depuis LanguageTool
cargo run --bin sync-lt -- --path /path/to/languagetool

# Génère automatiquement:
# - src/checker/data/en_patterns.rs (394 règles)
# - src/checker/data/fr_patterns.rs (170 règles)
# - src/checker/data/en_pattern_tests.rs (638 exemples)
# - src/checker/data/fr_pattern_tests.rs (244 exemples)
# - src/checker/data/en_confusion.rs (782 paires)
# - src/checker/data/en_style.rs (1398 règles)
# - src/checker/data/en_coherency.rs (2479 paires)
# - src/checker/data/en_diacritics.rs (1219 règles)
# - src/checker/data/en_contractions.rs (179 règles)
# - src/checker/data/en_determiners.rs (4782 mots)
# - src/checker/data/en_context_words.rs (11 règles)
# - src/checker/data/en_synonyms.rs (25 règles)
# - src/checker/data/fr_synonyms.rs (142 règles)
```

**Sortie complète du sync :**
```
Syncing EN rules...
   grammar.xml: 5544 rules, style.xml: 535 rules -> 394 patterns, 638 examples
   confusion_sets.txt: 782 pairs
   replace.txt: 201 replacements
   wordiness.txt: 692 rules
   redundancies.txt: 706 rules
   coherency.txt: 2479 pairs
   diacritics.txt: 1219 rules
   common_words.txt: 9890 words
   contractions.txt: 179 rules
   det_a.txt + det_an.txt: 4782 words (a: 939, an: 3843)
   wrongWordInContext.txt: 11 rules
   synonyms.txt: 25 rules

Syncing FR rules...
   grammar.xml: 5311 rules, style.xml: 1673 rules -> 170 patterns, 244 examples
   confusion_sets.txt: 61 pairs
   replace.txt: 102 replacements
   common_words.txt: 9729 words
   synonyms.txt: 142 rules

============================================================
Synchronisation complete!
  Patterns: 564 | Confusion: 843 | Replace: 303
  Wordiness: 692 | Redundancy: 706 | Coherency: 2479
  Diacritics: 1219 | Common words: 19619 | Test examples: 882
  Contractions: 179 | Determiners: 4782 (a: 939, an: 3843)
  Context rules: 11 | Synonyms: 167
============================================================
```

## Roadmap : Parité avec LanguageTool

### Phase 1 : Fondations (v0.2.0) ✅
- [x] Dictionnaire FST (370K mots EN)
- [x] Build script pour dictionnaires
- [x] Tokenizer Unicode avec contractions

### Phase 2 : Règles de base (v0.3.0) ✅
- [x] Capitalisation début de phrase
- [x] Ponctuation répétée
- [x] Espace manquant après ponctuation
- [x] Guillemets typographiques

### Phase 3 : Grammaire anglaise (v0.4.0) ✅
- [x] Subject-verb agreement
- [x] Article a/an amélioré (silent h, acronyms)
- [x] Its/it's, your/you're
- [x] Comma splice detection
- [x] Their/they're/there

### Phase 4 : Grammaire française (v0.5.0) ✅
- [x] Accord sujet-verbe FR
- [x] Accord adjectif-nom
- [x] Homophones (a/à, ou/où, ce/se)

### Phase 5 : Style et clarté (v0.6.0) ✅
- [x] Passive voice detection
- [x] Wordiness/redundancy
- [x] Sentence fragment
- [x] Cliché detection
- [x] Redundancy/pleonasm detection
- [x] Sentence length warning

### Phase 6 : Grammaire avancée (v0.7.0) ✅
- [x] Less/fewer (countable nouns)
- [x] Who/whom
- [x] Good/well (adjective vs adverb)
- [x] Double negative detection
- [x] French: conditionnel après si
- [x] French: accord de "tout"
- [x] English confusion pairs (1363 mots)
- [x] French confusion pairs (23 mots)

### Phase 7 : Performance (v1.0.0) ✅
- [x] Aho-Corasick pattern matching (91x speedup)
- [x] Validation avec 610 exemples LanguageTool
- [x] Benchmarks Criterion (<20s)
- [ ] API REST
- [ ] WASM

---

## Roadmap : Parité qualité LanguageTool

### Comparaison actuelle

| Catégorie | LanguageTool | grammar-rs | Gap |
|-----------|--------------|------------|-----|
| Règles grammaire | 6,523 | 564 | 91% |
| Règles style | 1,561 | 1,398 | 10% |
| Paires confusion | 2,000+ | 843 | 58% |
| Contractions | 197 | 179 | 9% |
| Détermineurs a/an | 5,038 | 4,782 | 5% |
| Contexte | 13 | 11 | 15% |
| Synonymes | ~200 | 167 | 17% |
| Filtres anti-FP | 14 classes | 0 | 100% |
| POS tagging | Oui | Non | 100% |
| Modèle de langue | Oui | Non | 100% |

### Phase 8 : Style & Wordiness (v1.1.0) ✅

Import des données de style de LanguageTool :

- [x] **Wordiness** : 692 expressions verbeuses (`wordiness.txt`)
  - `in order to` → `to`
  - `a large number of` → `many`
  - `at this point in time` → `now`
- [x] **Redundancies** : 706 pléonasmes (`redundancies.txt`)
  - `absolutely essential` → `essential`
  - `advance planning` → `planning`
  - `12 noon` → `noon`
- [x] **StyleChecker** avec Aho-Corasick pour matching O(N)
- [x] Sync-lt étendu pour importer ces fichiers

**Résultat** : +1,398 règles de style

**Benchmarks StyleChecker** (1398 règles):
| Test | Temps |
|------|-------|
| Texte sans erreur | 3.4µs |
| Texte verbeux | 5.5µs |
| Document mixte | 7.4µs |
| Construction | 5ms (one-time) |

### Phase 9 : Cohérence & Consistance (v1.2.0) ✅

- [x] **CoherencyChecker** : cohérence orthographique
  - 2,479 paires (UK/US) depuis `coherency.txt`
  - `analyse` vs `analyze` - détecte les incohérences dans un document
  - `organise` vs `organize`
  - `colour` vs `color` (via colourisation/colourization)
- [x] Lookup binaire pour performance O(log N)
- [x] Tracking par document (HashMap word -> pair_id)

**Résultat** : 2,479 paires de cohérence (4,958 mots uniques)

**Benchmarks CoherencyChecker**:
| Test | Temps |
|------|-------|
| Texte sans paires | 17µs |
| Texte UK cohérent | 4µs |
| Texte mixte UK/US | 4µs |
| Construction | 4ns |

### Phase 10 : Diacritiques (v1.3.0) ✅

Suggestion des accents corrects pour mots empruntés :

- [x] **DiacriticsChecker** depuis `diacritics.txt`
  - 1,219 règles pour mots empruntés d'autres langues
  - `aperitif` → `apéritif`
  - `attache` → `attaché`
  - `a la carte` → `à la carte`
- [x] Lookup binaire O(log N)
- [x] Préservation de la casse (Aperitif → Apéritif)

**Résultat** : 1,219 règles diacritiques

### Phase 11 : Confusion contextuelle & Contractions (v1.4.0) ✅

Import automatique de données supplémentaires de LanguageTool :

- [x] **ContractionChecker** depuis `contractions.txt`
  - 179 règles pour détecter apostrophes manquantes
  - `dont` → `don't`, `Im` → `I'm`, `youre` → `you're`
  - Sensible à la casse (DONT → DON'T)
- [x] **Determiner rules** depuis `det_a.txt` et `det_an.txt`
  - 4,782 mots pour améliorer a/an (939 "a", 3,843 "an")
  - `requires_en_a("European")` → true (a European)
  - `requires_en_an("hour")` → true (an hour)
- [x] **ContextChecker** depuis `wrongWordInContext.txt`
  - 11 règles avec patterns regex pour contexte
  - `heroin` vs `heroine` → contexte: drogue vs littérature
  - `bazaar` vs `bizarre` → contexte: marché vs étrange
  - `prescribe` vs `proscribe` → contexte médical vs légal
- [x] **Synonyms** depuis `synonyms.txt`
  - 25 règles EN, 142 règles FR
  - `get_fr_synonyms("notamment")` → synonymes pour style

**Résultat** : +179 contractions, +4,782 determiners, +11 context rules, +167 synonyms

### Phase 12 : Ponctuation avancée (v1.5.0)

- [ ] **UnpairedBracketsRule** : `(`, `)`, `[`, `]`, `{`, `}`
- [ ] **UnpairedQuotesRule** : `"`, `'`, `«`, `»`
- [ ] **DashRule** : en-dash `–` vs em-dash `—`
- [ ] Détection de contractions mal formées

**Objectif** : Couverture ponctuation complète

### Phase 13 : POS Tagging (v1.6.0)

Ajout du Part-of-Speech tagging pour patterns avancés :

- [ ] Intégration d'un POS tagger Rust (rust-bert ou pos-rs)
- [ ] Extension de `PatternRule` pour supporter POS tags
  ```rust
  // Avant: match exact
  pattern: &["could", "of"]

  // Après: match par POS
  pattern: &[Token::Word("could"), Token::Pos("IN")]  // preposition
  ```
- [ ] Import des règles LT avec POS tags (~3,000 règles supplémentaires)
- [ ] Lemmatisation pour variations morphologiques

**Objectif** : Support de 80% des règles LT grammar.xml

### Phase 14 : Filtres anti-faux positifs (v1.7.0)

Réduction des faux positifs avec filtres spécialisés :

- [ ] **DateFilter** : ignorer dates (`January 1st`, `2024-01-15`)
- [ ] **NumberFilter** : ignorer nombres écrits (`twenty-one`)
- [ ] **AbbreviationFilter** : ignorer abréviations connues
- [ ] **ProperNounFilter** : ignorer noms propres (NER basique)
- [ ] **QuotedTextFilter** : ignorer texte entre guillemets
- [ ] **CodeBlockFilter** : ignorer blocs de code

**Objectif** : Réduire FP de 30%

### Phase 15 : Règles L2 / Non-natifs (v1.8.0)

Règles pour apprenants d'anglais :

- [ ] **FalseFriends** pour francophones (`grammar-l2-fr.xml`)
  - `actually` ≠ `actuellement`
  - `eventually` ≠ `éventuellement`
- [ ] **FalseFriends** pour germanophones (`grammar-l2-de.xml`)
- [ ] Mode "ESL" avec règles adaptées
- [ ] Messages d'erreur pédagogiques

**Objectif** : Support apprenants non-natifs

### Phase 16 : Modèle de langue (v2.0.0)

Détection statistique d'erreurs :

- [ ] Intégration n-grams (3-gram, 4-gram)
- [ ] **NgramProbabilityRule** : séquences improbables
  - `I didn't now` → `I didn't know` (basé sur probabilité)
- [ ] Modèle pré-entraîné léger (~50MB)
- [ ] Fallback sur règles si modèle absent

**Objectif** : Détection d'erreurs contextuelles non couvertes par règles

### Phase 17 : API & Intégrations (v2.1.0)

- [ ] **API REST** compatible LanguageTool
  - Endpoint `/v2/check`
  - Format JSON identique
- [ ] **WASM** pour navigateurs
- [ ] **LSP** (Language Server Protocol) pour éditeurs
- [ ] Plugin VS Code

**Objectif** : Drop-in replacement pour LT dans intégrations existantes

---

## Métriques cibles

| Métrique | Actuel | Cible v1.5 | Cible v2.0 |
|----------|--------|------------|------------|
| Règles totales | ~10,500 | 12,000+ | 15,000+ |
| Précision | ~85% | 92% | 95% |
| Recall | ~60% | 80% | 90% |
| F1 Score | ~70% | 86% | 92% |
| Temps check (1KB) | 10µs | 15µs | 20µs |

### Données importées de LanguageTool

| Type | EN | FR | Total |
|------|----|----|-------|
| Patterns | 394 | 170 | 564 |
| Confusion pairs | 782 | 61 | 843 |
| Replace rules | 201 | 102 | 303 |
| Style (wordiness) | 692 | - | 692 |
| Style (redundancy) | 706 | - | 706 |
| Coherency pairs | 2,479 | - | 2,479 |
| Diacritics | 1,219 | - | 1,219 |
| Contractions | 179 | - | 179 |
| Determiners (a/an) | 4,782 | - | 4,782 |
| Context rules | 11 | - | 11 |
| Synonyms | 25 | 142 | 167 |
| Common words | 9,890 | 9,729 | 19,619 |
| Test examples | 638 | 244 | 882 |

## Licence

MIT
