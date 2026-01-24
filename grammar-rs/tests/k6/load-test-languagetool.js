import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';
import { generateVariedCorpus, getRandomText } from './text-generator.js';

// ============================================================================
// CUSTOM METRICS
// ============================================================================

const errorRate = new Rate('errors');
const matchCount = new Trend('match_count');
const requestsPerTier = new Counter('requests_by_tier');

// Metrics par tier de texte
const shortLatency = new Trend('latency_short');
const mediumLatency = new Trend('latency_medium');
const longLatency = new Trend('latency_long');

// ============================================================================
// TEST CONFIGURATION
// ============================================================================

const BASE_URL = 'https://languagetool-autocorrect.fly.dev';

export const options = {
  scenarios: {
    languagetool: {
      executor: 'constant-vus',
      vus: 50,
      duration: '1m',
    },
  },

  // Thresholds adaptes a LanguageTool (plus lent, Java-based)
  thresholds: {
    // Latence globale (plus permissive pour LanguageTool)
    'http_req_duration': ['p(50)<500', 'p(95)<1500', 'p(99)<3000'],

    // Latence par tier
    'latency_short': ['p(50)<300', 'p(95)<800', 'p(99)<1500'],
    'latency_medium': ['p(50)<500', 'p(95)<1200', 'p(99)<2500'],
    'latency_long': ['p(50)<800', 'p(95)<2000', 'p(99)<4000'],

    // Error rate (plus permissif pour LanguageTool sous charge)
    'errors': ['rate<0.10'], // <10% errors

    // HTTP success rate
    'http_req_failed': ['rate<0.10'],

    // Checks
    'checks': ['rate>0.90'], // >90% de checks passes
  },
};

// ============================================================================
// CORPUS SETUP
// ============================================================================

const CORPUS_SIZE = 500;

// ============================================================================
// MAIN TEST FUNCTION
// ============================================================================

export default function(data) {
  const sample = getRandomText(data.corpus);

  // Determine tier based on text length
  const textLength = sample.text.length;
  let tier;
  if (textLength < 500) {
    tier = 'short';
  } else if (textLength < 2500) {
    tier = 'medium';
  } else {
    tier = 'long';
  }

  // Preparer la requete
  const params = {
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    tags: {
      text_tier: tier,
      text_length: textLength,
    },
    timeout: '30s',
  };

  const payload = `text=${encodeURIComponent(sample.text)}&language=${sample.lang}`;

  // Envoyer la requete
  const response = http.post(`${BASE_URL}/v2/check`, payload, params);

  // Verifications
  const success = check(response, {
    'status is 200': (r) => r.status === 200,
    'response time < 10s': (r) => r.timings.duration < 10000,
    'has matches field': (r) => {
      try {
        const body = JSON.parse(r.body);
        return body.hasOwnProperty('matches');
      } catch {
        return false;
      }
    },
    'has language object': (r) => {
      try {
        const body = JSON.parse(r.body);
        return body.hasOwnProperty('language');
      } catch {
        return false;
      }
    },
  });

  // Metriques custom
  errorRate.add(!success);
  requestsPerTier.add(1, { tier: tier });

  // Latence par tier
  const duration = response.timings.duration;
  if (tier === 'short') {
    shortLatency.add(duration);
  } else if (tier === 'medium') {
    mediumLatency.add(duration);
  } else {
    longLatency.add(duration);
  }

  if (success && response.body) {
    try {
      const body = JSON.parse(response.body);
      const numMatches = body.matches?.length || 0;
      matchCount.add(numMatches, { tier: tier });
    } catch (e) {
      console.error('Failed to parse response body:', e);
    }
  }

  // Throttle: petite pause entre requetes
  const pauseMs = 100 + Math.random() * 200; // 100-300ms
  sleep(pauseMs / 1000);
}

// ============================================================================
// LIFECYCLE HOOKS
// ============================================================================

export function setup() {
  console.log('\n');
  console.log('='.repeat(70));
  console.log('  K6 Load Test: LANGUAGETOOL');
  console.log('='.repeat(70));
  console.log(`URL: ${BASE_URL}`);
  console.log(`Duration: 1 minute`);
  console.log(`Load: 50 VUs constant`);
  console.log(`Corpus: ${CORPUS_SIZE} unique texts`);
  console.log('='.repeat(70));

  // Generate corpus
  console.log('Generating varied corpus...');
  const corpus = generateVariedCorpus(CORPUS_SIZE);
  console.log(`Generated ${corpus.length} unique texts\n`);

  return { corpus };
}

export function teardown(data) {
  console.log('\n');
  console.log('='.repeat(70));
  console.log('  LANGUAGETOOL Test completed!');
  console.log('='.repeat(70));
  console.log('Check the summary above for detailed metrics by tier.\n');
}
