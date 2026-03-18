---
name: recipe-ab-test-subject
version: 1.0.0
description: "A/B test two subject lines to optimize open rates."
metadata:
  openclaw:
    category: "recipe"
    domain: "ab-test"
    requires:
      bins: ["nlm"]
      skills: ["nlm-edm-ab-test", "nlm-edm-report"]
---

# Recipe: A/B Test Subject Lines

Run a subject-line A/B test to find which version drives higher open rates,
then review results.

## Prerequisites

- EDM API key configured
- At least one contact list with enough subscribers for a meaningful test
- HTML content file for the campaign body
- Verified sender address

---

## Steps

### Step 1 — Submit the A/B test

Create a campaign with two subject-line variants. The API splits the test
group evenly and sends each variant to half.

```bash
nlm edm ab-test submit \
  --name 'Subject Test' \
  --lists L1 \
  --subject-a 'Version A' \
  --subject-b 'Version B' \
  --from-name F \
  --from-address A \
  --html H
```

The command outputs serial numbers for each variant.

### Step 2 — Wait and check the report

Allow enough time for recipients to open (typically 1-4 hours), then pull
the metrics.

```bash
nlm edm report metrics --sns CAM_SN
```

Look at `open_rate` and `click_rate` for each variant.

### Step 3 — Compare results

Side-by-side comparison of the two variants.

```bash
nlm edm campaign compare --sns SN_A SN_B
```

This outputs a diff-style table showing open rate, click rate, bounce rate,
and unsubscribe rate for each variant.

---

## Tips

- **Sample size matters:** A list smaller than 1,000 may not produce
  statistically significant results. Aim for at least 500 per variant.
- **Test one variable:** Keep the body HTML identical; only change the subject
  line. Testing multiple variables at once makes it impossible to attribute
  differences.
- **Timing:** Send both variants simultaneously so time-of-day does not skew
  results.
- **Follow up:** Once you identify the winner, use that subject line for the
  full campaign send to the remaining list.
