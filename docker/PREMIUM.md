# LanguageTool Premium Features

This document explains what premium features would bring to the self-hosted LanguageTool server and the available options.

## Key Finding

**Premium features are NOT available for self-hosted instances.**

According to Daniel Naber (LanguageTool maintainer):
> "The Premium version is only available in the cloud and - for larger customers with 250+ users - as an on-premise version."

The `premium: false` flag in the server logs is expected behavior for self-hosted instances.

---

## What Premium Would Bring (Cloud API Only)

### 1. Additional Grammar Rules

- **+20,000 extra rules** for English, German, French, Spanish, Dutch, Polish, and Portuguese
- Advanced style, punctuation, and grammar detection
- Incorrect name/number detection

### 2. Character Limits

| Version | Limit per Request |
|---------|-------------------|
| Free (cloud) | 20,000 characters |
| Premium (cloud) | 60,000-150,000 characters |
| Self-hosted | Unlimited (configurable) |

### 3. Enhanced Picky Mode

- The `level: "picky"` parameter activates additional rules
- Premium unlocks more rules in picky mode

### 4. AI Paraphrasing

- Free: 3 rephrases/day
- Premium: Unlimited
- **Note: Not available via API**

### 5. User Features

- Custom style guide (custom rules)
- Personal dictionary
- Text statistics

---

## Options to Access Premium Features

### Option A: LanguageTool Cloud API

**Proofreading API** (https://languagetool.org/proofreading-api)

| Daily Calls | Price |
|-------------|-------|
| 100 | ~5/month |
| 250 | ~10/month |
| 500 | ~15/month |
| 1,000 | ~25/month |

- Max 60,000 characters/request
- Servers in Germany, GDPR compliant

**Request parameters:**
```
username: "your-email@example.com"
apiKey: "your-api-key"
level: "picky"  # optional, for more rules
```

### Option B: Premium Personal Subscription

- 4.99/month (annual) or 19.90/month
- Includes a free API key (10,000 requests/day) for personal use
- Contact support to obtain the key

### Option C: On-premise Enterprise

- For organizations with 250+ users
- Contact LanguageTool for a quote

### Option D: Keep Self-hosted (Current Setup)

- Keep the Fly.io server as-is
- Accept free rules limitations
- **Advantages:**
  - No request limits
  - Private data (not sent to LanguageTool.org)
  - Cost-effective (just Fly.io hosting)

---

## Recommendation

For the AutoCorrect extension, **self-hosted is sufficient** because:

1. Basic rules cover most common errors
2. No character/request limits
3. Private data handling
4. Lower cost (only Fly.io fees)

**If premium is needed:** Option B (Premium subscription + free API key) would be the most cost-effective for personal use with 10,000 requests/day.

---

## Upgrading to Cloud API

To switch from self-hosted to LanguageTool cloud API with premium:

1. Get an API key from https://languagetool.org/proofreading-api
2. Update `language-tool-client.ts` to use `https://api.languagetoolplus.com/v2/check`
3. Add authentication headers:
   ```typescript
   const params = new URLSearchParams({
     text: text,
     language: language,
     username: 'your-email@example.com',
     apiKey: 'your-api-key',
     level: 'picky'  // optional
   });
   ```

---

## Sources

- [LanguageTool Premium](https://languagetool.org/premium)
- [Proofreading API](https://languagetool.org/proofreading-api)
- [GitHub Issue #5591 - Premium with self-hosted Docker](https://github.com/languagetool-org/languagetool/issues/5591)
- [Forum - Premium vs Proofreading API](https://forum.languagetool.org/t/premium-vs-proofreading-api-for-http-api/8945)
