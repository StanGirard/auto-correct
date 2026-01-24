# Grammar-RS: Features Manquantes

> **État actuel:** ~93% de parité fonctionnelle avec LanguageTool
>
> **Performance:** grammar-rs ~9ms vs LanguageTool ~1.4s (~150x plus rapide)
>
> **Dernière mise à jour:** Disambiguation skip patterns extraits via sync-lt (EN: 24+36, FR: 1+3)

---

## 1. Disambiguation / POS Tagging Avancé - 🔶 Partiellement implémenté

**Description:** Résolution d'ambiguïté grammaticale pour identifier la fonction des mots.

**Exemple:** "I saw the saw" → saw₁ = verbe (VBD), saw₂ = nom (NN)

**État actuel:**
- ✅ Skip patterns extraits (24 EN + 1 FR mots, 36 EN + 3 FR regex)
- ✅ POS single-token rules extraits (24 EN + 28 FR)
- ⏸️ Règles contextuelles multi-tokens: non implémentées
- ⏸️ Modèle statistique HMM/Perceptron: non implémenté

**Stats extraction sync-lt:**
- EN: 547 règles parsées → 24 skip words + 36 regex + 24 POS rules
- FR: 461 règles parsées → 1 skip word + 3 regex + 28 POS rules

**LanguageTool complet:** ~2,000 règles disambiguation.xml + modèle HMM/Perceptron

**Sources LT:**
- `languagetool/org/languagetool/resource/en/disambiguation.xml`

**Priorité:** BASSE (règles contextuelles complexes)

---

## 2. N-gram Language Models - ❌ Non implémenté

**Description:** Modèles statistiques pour détecter erreurs de choix de mots basés sur le contexte.

**Exemple:** "I went to there house" → "their" (basé sur fréquence n-gram)

**État:** 0%

**Problème:** Modèles ~1GB par langue. Pas prioritaire pour solution légère.

**Alternative:** Confusion pairs avec scoring de fréquence (partiellement implémenté).

**Sources LT:**
- `languagetool/org/languagetool/resource/en/ngram-index/`

**Priorité:** BASSE

---

## 3. Pipeline Français - ✅ COMPLÉTÉ

**Description:** Le pipeline FR intègre maintenant les checkers principaux.

| Checker | Données | Pipeline EN | Pipeline FR |
|---------|---------|-------------|-------------|
| PosPatternChecker | 25 règles FR | ✅ | ✅ |
| StyleChecker | 51 règles FR | ✅ | ✅ |
| CompoundWordChecker | 1,345 règles FR | ✅ | ✅ |
| CoherencyChecker | EN only | ✅ | N/A |
| DiacriticsChecker | EN only | ✅ | N/A |
| ContractionChecker | EN only | ✅ | N/A |

**Note:** CoherencyChecker, DiacriticsChecker, ContractionChecker sont spécifiques EN.

**Priorité:** ~~HAUTE~~ TERMINÉ

---

## 4. L2 Learner Confusion Pairs - ✅ FR COMPLÉTÉ

**Description:** Paires de confusion spécifiques aux apprenants L2 selon leur langue maternelle.

**État:** FR intégré, autres langues disponibles mais non intégrées.

| Fichier | Paires | Intégré |
|---------|--------|---------|
| `en_confusion_l2_de.rs` | 75 | ❌ |
| `en_confusion_l2_es.rs` | 26 | ❌ |
| `en_confusion_l2_fr.rs` | 325 | ✅ `L2ConfusionChecker` |
| `en_confusion_l2_nl.rs` | 11 | ❌ |

**API:** `motherTongue=fr` active la détection de faux amis pour francophones.

**Priorité:** ~~MOYENNE~~ FR TERMINÉ

---

## 5. Spelling Infrastructure - ✅ COMPLÉTÉ

**Description:** Spell-checking complet avec suggestions.

**État:** Intégré aux pipelines EN et FR.

| Langue | Dictionnaire | Skip List | État |
|--------|--------------|-----------|------|
| EN | FST 370K mots | 16,590 mots (EN_IGNORE + EN_PROPER_NOUNS + EN_DISAMBIG_SKIP) | ✅ Intégré |
| FR | 34K mots (FR_SPELLING) | 1,507 mots (FR_IGNORE + FR_DISAMBIG_SKIP) | ✅ Intégré |

**Fichiers modifiés:**
- `src/checker/spell.rs` - Ajout support skip_words
- `src/bin/api/state.rs` - Intégration aux pipelines

**Note:** Le dictionnaire FR est limité (34K mots vs 370K EN) mais fonctionnel.

**Priorité:** ~~MOYENNE~~ TERMINÉ

---

## 6. Proper Nouns Skip List - ✅ COMPLÉTÉ

**Description:** Liste de noms propres à ignorer lors du spell-check.

**État:** 5,537 noms propres EN intégrés au SpellChecker (`en_proper_nouns.rs`).

**Intégration:** `SpellChecker.with_skip_words(EN_PROPER_NOUNS.iter().copied())`

**Priorité:** ~~BASSE~~ TERMINÉ

---

## 7. Multiword Expressions - 🔶 Données non intégrées

**Description:** Expressions multi-mots avec traitement spécial.

**État:** Données extraites (`en_multiwords.rs`, `fr_multiwords.rs`), non intégrées.

**Sources LT:**
- `languagetool/org/languagetool/resource/en/multiwords.txt`

**Priorité:** BASSE

---

## 8. Prohibited Words - ✅ COMPLÉTÉ

**Description:** Mots/patterns à signaler systématiquement.

**État:** `ProhibitChecker` intégré au pipeline EN (330 mots).

**Exemples:** "Christoper" → "Christopher", "GDPR-complaint" → "GDPR-compliant"

**Priorité:** ~~BASSE~~ TERMINÉ

---

## 9. Numbers Rules - 🔶 Données non intégrées

**Description:** Règles spécifiques aux nombres (format, cohérence).

**État:** Données extraites (`en_numbers.rs`), non intégrées.

**Priorité:** BASSE

---

## Résumé

| Catégorie | Features | Priorité | État |
|-----------|----------|----------|------|
| ✅ Complété | FR pipeline, ProhibitChecker, L2ConfusionChecker FR, SpellChecker, Proper Nouns, Disambig Skip | - | Intégré |
| 🔶 Partiel | Disambiguation/POS (skip patterns OK, contexte manquant) | BASSE | Skip patterns intégrés |
| ❌ Complexe | N-gram models, Disambiguation contextuelles | BASSE | Nécessite ML/données volumineuses |
| ⏸️ Différé | Multiwords, Numbers | BASSE | Nécessite POS avancé |

**Note:**
- **Disambiguation:** Skip patterns extraits et intégrés, règles contextuelles non implémentées
- **N-gram:** Nécessite modèles statistiques (~1GB par langue)
- **SpellChecker:** ✅ Intégré avec FST 370K mots EN + 34K mots FR + skip patterns disambiguation

---

## Commande de synchronisation

```bash
cd grammar-rs
cargo run --bin sync-lt -- --languagetool-path ../languagetool
```
