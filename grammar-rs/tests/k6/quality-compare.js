import http from 'k6/http';
import { check } from 'k6';
import { Trend, Counter } from 'k6/metrics';

// ============================================================================
// METRICS
// ============================================================================

const grammarrsMatches = new Trend('grammarrs_matches');
const languagetoolMatches = new Trend('languagetool_matches');
const matchDiff = new Trend('match_difference');
const missedByGrammarrs = new Counter('missed_by_grammarrs');
const extraByGrammarrs = new Counter('extra_by_grammarrs');
const perfectMatch = new Counter('perfect_match');

// ============================================================================
// CONFIG
// ============================================================================

const GRAMMARRS_URL = 'https://grammar-rs-autocorrect.fly.dev';
const LANGUAGETOOL_URL = 'https://languagetool-autocorrect.fly.dev';

export const options = {
  vus: 1,
  iterations: 50, // 50 textes compares
  thresholds: {
    'match_difference': ['avg<5'], // Difference moyenne < 5 erreurs
  },
};

// ============================================================================
// TEST TEXTS - Textes avec erreurs connues
// ============================================================================

const TEST_CASES = [
  // Erreurs simples
  { text: "I have a apple.", lang: "en", expectedErrors: ["a apple"] },
  { text: "He dont like it.", lang: "en", expectedErrors: ["dont"] },
  { text: "Their going to the store.", lang: "en", expectedErrors: ["Their"] },
  { text: "Its a nice day.", lang: "en", expectedErrors: ["Its"] },
  { text: "I would of done it.", lang: "en", expectedErrors: ["would of"] },
  { text: "Your the best.", lang: "en", expectedErrors: ["Your"] },
  { text: "The informations are correct.", lang: "en", expectedErrors: ["informations"] },
  { text: "He play football.", lang: "en", expectedErrors: ["play"] },
  { text: "She don't knows.", lang: "en", expectedErrors: ["knows"] },
  { text: "I seen him yesterday.", lang: "en", expectedErrors: ["seen"] },

  // Confusion pairs
  { text: "I accept your advise.", lang: "en", expectedErrors: ["advise"] },
  { text: "The principle of the school.", lang: "en", expectedErrors: ["principle"] },
  { text: "It's a mute point.", lang: "en", expectedErrors: ["mute"] },
  { text: "For all intensive purposes.", lang: "en", expectedErrors: ["intensive"] },
  { text: "I could care less.", lang: "en", expectedErrors: ["could care"] },

  // Style
  { text: "At this point in time, we should proceed.", lang: "en", expectedErrors: ["At this point in time"] },
  { text: "Due to the fact that it's raining.", lang: "en", expectedErrors: ["Due to the fact that"] },
  { text: "In order to succeed, work hard.", lang: "en", expectedErrors: ["In order to"] },

  // Repeated words
  { text: "I went to the the store.", lang: "en", expectedErrors: ["the the"] },
  { text: "She is is happy.", lang: "en", expectedErrors: ["is is"] },

  // Spelling
  { text: "I recieved your mesage.", lang: "en", expectedErrors: ["recieved", "mesage"] },
  { text: "Definately a good ideea.", lang: "en", expectedErrors: ["Definately", "ideea"] },
  { text: "Accomodate the necesary changes.", lang: "en", expectedErrors: ["Accomodate", "necesary"] },

  // French
  { text: "Je suis alle au magasin.", lang: "fr", expectedErrors: ["alle"] },
  { text: "Il a manger une pomme.", lang: "fr", expectedErrors: ["manger"] },
  { text: "Les informations sont correctes.", lang: "fr", expectedErrors: [] }, // Correct

  // Longer texts
  {
    text: "I would of went to the store, but their was no time. Its a shame because I really wanted to by some groceries. The weather was definately nice though.",
    lang: "en",
    expectedErrors: ["would of", "went", "their", "Its", "by", "definately"]
  },

  // Correct text (should have 0 errors)
  { text: "The quick brown fox jumps over the lazy dog.", lang: "en", expectedErrors: [] },
  { text: "She has been working here for five years.", lang: "en", expectedErrors: [] },
];

// ============================================================================
// HELPER
// ============================================================================

function checkText(url, text, lang) {
  const params = {
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    timeout: '30s',
  };
  const payload = `text=${encodeURIComponent(text)}&language=${lang}`;
  const response = http.post(`${url}/v2/check`, payload, params);

  if (response.status !== 200) {
    return { matches: [], error: true };
  }

  try {
    const body = JSON.parse(response.body);
    return {
      matches: body.matches || [],
      error: false,
      duration: response.timings.duration
    };
  } catch {
    return { matches: [], error: true };
  }
}

function extractErrorContexts(matches) {
  return matches.map(m => ({
    context: m.context?.text?.substring(m.context?.offset, m.context?.offset + m.context?.length) || '',
    rule: m.rule?.id || 'unknown',
    message: m.message || '',
    offset: m.offset,
    length: m.length,
  }));
}

// ============================================================================
// MAIN
// ============================================================================

export default function() {
  const testCase = TEST_CASES[__ITER % TEST_CASES.length];

  // Query both services
  const grResult = checkText(GRAMMARRS_URL, testCase.text, testCase.lang);
  const ltResult = checkText(LANGUAGETOOL_URL, testCase.text, testCase.lang);

  if (grResult.error || ltResult.error) {
    console.log(`ERROR on: "${testCase.text.substring(0, 50)}..."`);
    return;
  }

  const grCount = grResult.matches.length;
  const ltCount = ltResult.matches.length;
  const diff = Math.abs(grCount - ltCount);

  // Record metrics
  grammarrsMatches.add(grCount);
  languagetoolMatches.add(ltCount);
  matchDiff.add(diff);

  if (grCount < ltCount) {
    missedByGrammarrs.add(ltCount - grCount);
  } else if (grCount > ltCount) {
    extraByGrammarrs.add(grCount - ltCount);
  } else {
    perfectMatch.add(1);
  }

  // Log comparison
  const grErrors = extractErrorContexts(grResult.matches);
  const ltErrors = extractErrorContexts(ltResult.matches);

  console.log(`\n${'='.repeat(70)}`);
  console.log(`TEXT: "${testCase.text.substring(0, 60)}${testCase.text.length > 60 ? '...' : ''}"`);
  console.log(`LANG: ${testCase.lang}`);
  console.log(`${'─'.repeat(70)}`);
  console.log(`grammar-rs: ${grCount} matches (${grResult.duration.toFixed(0)}ms)`);
  console.log(`LanguageTool: ${ltCount} matches (${ltResult.duration.toFixed(0)}ms)`);
  console.log(`Difference: ${diff}`);

  if (grCount !== ltCount) {
    console.log(`\ngrammar-rs rules: ${grErrors.map(e => e.rule).join(', ')}`);
    console.log(`LanguageTool rules: ${ltErrors.map(e => e.rule).join(', ')}`);
  }

  // Check against expected errors
  const expectedCount = testCase.expectedErrors.length;
  if (expectedCount > 0) {
    console.log(`\nExpected errors: ${testCase.expectedErrors.join(', ')}`);
    console.log(`grammar-rs found: ${grCount}/${expectedCount}`);
    console.log(`LanguageTool found: ${ltCount}/${expectedCount}`);
  }
}

// ============================================================================
// SUMMARY
// ============================================================================

export function handleSummary(data) {
  const grAvg = data.metrics.grammarrs_matches?.values?.avg || 0;
  const ltAvg = data.metrics.languagetool_matches?.values?.avg || 0;
  const diffAvg = data.metrics.match_difference?.values?.avg || 0;
  const perfect = data.metrics.perfect_match?.values?.count || 0;
  const missed = data.metrics.missed_by_grammarrs?.values?.count || 0;
  const extra = data.metrics.extra_by_grammarrs?.values?.count || 0;

  console.log(`\n${'='.repeat(70)}`);
  console.log('  QUALITY COMPARISON SUMMARY');
  console.log(`${'='.repeat(70)}`);
  console.log(`Average matches per text:`);
  console.log(`  grammar-rs:    ${grAvg.toFixed(2)}`);
  console.log(`  LanguageTool:  ${ltAvg.toFixed(2)}`);
  console.log(`  Difference:    ${diffAvg.toFixed(2)}`);
  console.log(`\nMatch accuracy:`);
  console.log(`  Perfect matches: ${perfect}`);
  console.log(`  Missed by grammar-rs: ${missed}`);
  console.log(`  Extra by grammar-rs: ${extra}`);
  console.log(`${'='.repeat(70)}\n`);

  return {};
}
