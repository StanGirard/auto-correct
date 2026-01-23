# Roadmap : Parité avec LanguageTool

Ce document détaille le plan pour atteindre une qualité de détection comparable à LanguageTool.

## État actuel vs LanguageTool

| Fonctionnalité | grammar-rs | LanguageTool |
|---------------|------------|--------------|
| Règles actives | 5 | ~3000+ |
| Dictionnaire | ~30 mots (test) | Hunspell complet |
| Langues | EN, FR (partiel) | 30+ langues |
| POS Tagging | Non | Oui |
| N-grams | Non | Oui (optionnel) |
| Performance | ~1M mots/s | ~10K mots/s |

**Objectif** : 80% de la couverture de LanguageTool avec 10x la performance.

---

## Phase 1 : Fondations (v0.2.0)

### 1.1 Dictionnaire complet

**Problème** : Notre dictionnaire de test a ~30 mots.

**Solution** : Intégrer un dictionnaire FST (Finite State Transducer).

```rust
// Ajouter dans Cargo.toml
fst = "0.4"

// Structure proposée
pub struct FstDictionary {
    fst: fst::Set<Vec<u8>>,
}

impl FstDictionary {
    pub fn from_wordlist(path: &str) -> Result<Self, Error>;
    pub fn contains(&self, word: &str) -> bool;
    pub fn suggest(&self, word: &str, max: usize) -> Vec<String>;
}
```

**Sources de données** :
- SCOWL (Spell Checker Oriented Word Lists) : ~150K mots EN
- Lexique.org : ~140K mots FR
- Hunspell dictionaries (conversion)

**Tâches** :
- [ ] Implémenter `FstDictionary`
- [ ] Script de conversion Hunspell → FST
- [ ] Télécharger et intégrer dictionnaires EN/FR
- [ ] Benchmark mémoire et performance

### 1.2 Tokenizer Unicode robuste

**Problème** : Le tokenizer actuel est basique.

**Solution** : Utiliser `unicode-segmentation` pour gérer :
- Contractions : `don't` → `do` + `n't`
- Élisions françaises : `l'homme` → `l'` + `homme`
- Emojis et caractères spéciaux
- URLs, emails, mentions @

```rust
// Ajouter dans Cargo.toml
unicode-segmentation = "1.10"

// API proposée
pub struct UnicodeTokenizer {
    split_contractions: bool,
    preserve_urls: bool,
}
```

**Tâches** :
- [ ] Implémenter `UnicodeTokenizer`
- [ ] Tests sur textes multilingues
- [ ] Benchmark vs tokenizer simple

### 1.3 Suggestions de correction

**Problème** : Les suggestions sont basiques.

**Solution** : Implémenter Levenshtein + fréquence.

```rust
pub fn suggest(word: &str, dictionary: &FstDictionary, max: usize) -> Vec<Suggestion> {
    // 1. Générer candidats par edit distance ≤ 2
    // 2. Filtrer par dictionnaire
    // 3. Trier par fréquence
    // 4. Retourner top N
}
```

**Tâches** :
- [ ] Implémenter distance de Levenshtein optimisée
- [ ] Ajouter fréquences des mots
- [ ] Cacher les candidats communs

---

## Phase 2 : Règles de base (v0.3.0)

### 2.1 Capitalisation

| Règle | Exemple | Correction |
|-------|---------|------------|
| `UPPERCASE_SENTENCE_START` | `hello. world` | `Hello. World` |
| `PROPER_NOUN_CAPS` | `i went to paris` | `I went to Paris` |
| `ALL_CAPS_WARNING` | `THIS IS SHOUTING` | Warning |

**Tâches** :
- [ ] Détecter début de phrase (après `.!?`)
- [ ] Liste de noms propres (pays, villes, prénoms)
- [ ] Règle ALL_CAPS configurable

### 2.2 Ponctuation

| Règle | Exemple | Correction |
|-------|---------|------------|
| `DOUBLE_PUNCTUATION` | `Hello!!` | `Hello!` |
| `MISSING_PERIOD` | `Hello world` (fin) | `Hello world.` |
| `COMMA_BEFORE_AND` | `A, B, and C` | Style check |
| `TYPOGRAPHIC_QUOTES` | `"hello"` | `"hello"` |

**Tâches** :
- [ ] Implémenter règles de ponctuation
- [ ] Support guillemets français « »
- [ ] Espaces insécables (FR)

### 2.3 Espacement

| Règle | Exemple | Correction |
|-------|---------|------------|
| `MULTIPLE_SPACES` | `hello   world` | `hello world` |
| `SPACE_BEFORE_PUNCT` | `Hello .` | `Hello.` |
| `NO_SPACE_AFTER_PUNCT` | `Hello.World` | `Hello. World` |

**Tâches** :
- [ ] Améliorer `DOUBLE_SPACE` → `MULTIPLE_SPACES`
- [ ] Règles espace/ponctuation contextuelles

---

## Phase 3 : Grammaire anglaise (v0.4.0)

### 3.1 Subject-Verb Agreement

C'est la règle la plus importante et la plus complexe.

**Cas courants** :
```
he go → he goes
they goes → they go
the list of items are → the list of items is
```

**Approche** : POS tagging simplifié.

```rust
pub struct SimplePOSTagger {
    // Listes de mots par catégorie
    pronouns_singular: HashSet<&'static str>,  // he, she, it
    pronouns_plural: HashSet<&'static str>,    // they, we
    verbs_base: HashMap<&'static str, &'static str>,  // go → goes
}

impl SimplePOSTagger {
    pub fn check_agreement(&self, subject: &Token, verb: &Token) -> Option<Match>;
}
```

**Tâches** :
- [ ] Implémenter `SimplePOSTagger`
- [ ] Liste des 100 verbes irréguliers
- [ ] Gérer "the X of Y" patterns
- [ ] Tests sur corpus d'erreurs réelles

### 3.2 Article A/An amélioré

**Cas actuels** : Voyelles simples.

**Cas à ajouter** :
```
an hour (h muet)
a university (u prononcé /ju/)
an FBI agent (acronyme commençant par voyelle)
a one-time offer (o prononcé /w/)
```

```rust
pub struct AAnRule {
    silent_h: HashSet<&'static str>,      // hour, honest, heir
    vowel_sound_u: HashSet<&'static str>, // university, union
    consonant_o: HashSet<&'static str>,   // one, once
}
```

**Tâches** :
- [ ] Listes d'exceptions
- [ ] Détection d'acronymes
- [ ] Tests exhaustifs

### 3.3 Possessifs et contractions

| Erreur | Correction | Règle |
|--------|------------|-------|
| `it's color` | `its color` | `ITS_VS_ITS` |
| `your welcome` | `you're welcome` | `YOUR_VS_YOURE` |
| `their going` | `they're going` | `THEIR_VS_THEYRE` |

**Approche** : Contexte lexical.

```rust
// "it's" suivi d'un nom → probablement "its"
// "it's" suivi d'un verbe/adjectif → OK
fn check_its_context(tokens: &[Token], pos: usize) -> Option<Match>;
```

**Tâches** :
- [ ] Règles its/it's, your/you're, their/they're
- [ ] Contexte grammatical simplifié
- [ ] Réduire les faux positifs

### 3.4 Comma splice

**Définition** : Deux phrases indépendantes jointes par une virgule.

```
I went home, I was tired.  → I went home. I was tired.
                           → I went home; I was tired.
                           → I went home because I was tired.
```

**Approche** : Détecter `[phrase complète], [pronom sujet] [verbe]`.

**Tâches** :
- [ ] Détection de phrases complètes
- [ ] Pattern matching sur structure
- [ ] Suggestions multiples

---

## Phase 4 : Grammaire française (v0.5.0)

### 4.1 Accord sujet-verbe

```
Les enfants mange → Les enfants mangent
Le chat et le chien dort → Le chat et le chien dorment
```

**Complexité** : Conjugaison française riche.

**Approche** : Table de conjugaison + patterns.

**Tâches** :
- [ ] Base de conjugaison (Lexique.org)
- [ ] Détection sujet pluriel
- [ ] Gestion "et", "ou", "ni"

### 4.2 Accord adjectif-nom

```
une belle maison → OK
un belle maison → un beau maison / une belle maison
les chats noirs → OK
les chats noir → les chats noirs
```

**Approche** : Dictionnaire genré + règles.

**Tâches** :
- [ ] Genre et nombre dans le dictionnaire
- [ ] Position adjectif (avant/après nom)
- [ ] Adjectifs invariables (marron, orange)

### 4.3 Homophones grammaticaux

| Erreur | Règle | Astuce |
|--------|-------|--------|
| `a` / `à` | Verbe vs préposition | Remplacer par "avait" |
| `ou` / `où` | Conjonction vs lieu | Remplacer par "ou bien" |
| `ce` / `se` | Démonstratif vs réfléchi | "ce" devant nom, "se" devant verbe |
| `ces` / `ses` / `c'est` / `s'est` | Contexte | Analyse grammaticale |

**Tâches** :
- [ ] Règles basées sur le contexte
- [ ] Messages explicatifs avec astuces
- [ ] Tests sur corpus d'erreurs

---

## Phase 5 : Style et clarté (v0.6.0)

### 5.1 Voix passive

```
The report was written by John → John wrote the report
```

**Détection** : `[be conjugué] + [participe passé] + (by ...)?`

### 5.2 Redondances

```
absolutely essential → essential
future plans → plans
free gift → gift
```

**Approche** : Liste de locutions redondantes.

### 5.3 Phrases trop longues

```
Sentences over 40 words are hard to read.
```

**Tâches** :
- [ ] Compteur de mots par phrase
- [ ] Seuil configurable
- [ ] Suggestions de découpage

---

## Phase 6 : Validation et benchmark (v1.0.0)

### 6.1 Corpus de test

Utiliser des corpus standards :
- **BEA-2019** : Grammatical Error Correction
- **NUCLE** : NUS Corpus of Learner English
- **CoNLL-2014** : Shared Task data

### 6.2 Métriques cibles

| Métrique | Cible | LanguageTool |
|----------|-------|--------------|
| Precision | ≥ 85% | ~70-80% |
| Recall | ≥ 60% | ~50-60% |
| F0.5 | ≥ 75% | ~65% |

**Note** : On privilégie la précision (moins de faux positifs) car les faux positifs sont plus frustrants pour l'utilisateur.

### 6.3 Benchmark de performance

```bash
# Corpus de 1M mots
time grammar-rs check corpus.txt
time java -jar languagetool.jar corpus.txt
```

**Cible** : 10x plus rapide que LanguageTool.

---

## Priorités d'implémentation

### Haute priorité (impact maximal)
1. **Dictionnaire FST** - Base de tout
2. **Subject-verb agreement** - Erreur la plus commune
3. **Homophones FR** - a/à, ou/où très fréquents
4. **Capitalisation** - Facile et utile

### Moyenne priorité
5. A/An amélioré
6. Its/it's, your/you're
7. Ponctuation avancée
8. Accord adjectif-nom FR

### Basse priorité (nice to have)
9. Voix passive
10. Style et wordiness
11. Comma splice
12. Phrases trop longues

---

## Ressources

### Dictionnaires
- [SCOWL](http://wordlist.aspell.net/) - English word lists
- [Lexique.org](http://www.lexique.org/) - French lexicon
- [Hunspell dictionaries](https://github.com/wooorm/dictionaries)

### Corpus d'erreurs
- [BEA-2019 Shared Task](https://www.cl.cam.ac.uk/research/nl/bea2019st/)
- [NUCLE Corpus](https://www.comp.nus.edu.sg/~nlp/corpora.html)
- [Lang-8 Corpus](https://sites.google.com/site/naaborulab/resources)

### LanguageTool
- [Rules XML](https://github.com/languagetool-org/languagetool/tree/master/languagetool-language-modules)
- [API Documentation](https://languagetool.org/http-api/)

---

## Prochaines étapes

1. **Immédiat** : Implémenter `FstDictionary` avec SCOWL
2. **Cette semaine** : Règles de capitalisation
3. **Ce mois** : Subject-verb agreement basique
4. **Q1** : Atteindre 50% de couverture LanguageTool
