#!/usr/bin/env python3
"""
Download and convert Lefff morphological lexicon for grammar-rs.

Downloads the Lefff dataset from HuggingFace and converts it to TSV format
with LanguageTool-compatible POS tags.

Usage:
    python scripts/download_lefff.py

Output:
    data/morphology/fr_lefff.tsv
"""

import os
import sys

def main():
    try:
        import pandas as pd
    except ImportError:
        print("Installing required packages...")
        os.system(f"{sys.executable} -m pip install pandas pyarrow requests")
        import pandas as pd

    import requests
    from io import BytesIO

    # Direct download of Parquet file
    parquet_url = "https://huggingface.co/datasets/sagot/lefff_morpho/resolve/refs%2Fconvert%2Fparquet/default/test/0000.parquet"
    print("Downloading Lefff dataset from HuggingFace...")

    response = requests.get(parquet_url, stream=True)
    response.raise_for_status()

    print("Parsing Parquet file...")
    df = pd.read_parquet(BytesIO(response.content))
    print(f"Loaded {len(df)} entries")

    # Create output directory
    os.makedirs("data/morphology", exist_ok=True)

    def map_to_lt_pos(row):
        """Map Lefff entry to LanguageTool French POS format"""
        category = row['category']
        msfeatures = row.get('msfeatures', '') or ''
        unimorph = row.get('unimorph', '') or ''

        # Extract gender/number from msfeatures
        gender = 'e'  # epicene
        number = 's'  # singular

        # Parse msfeatures (ms, fs, mp, fp, p, s, etc.)
        if 'ms' in msfeatures or 'm' in msfeatures:
            gender = 'm'
        elif 'fs' in msfeatures or 'f' in msfeatures:
            gender = 'f'

        if 'p' in msfeatures and 's' not in msfeatures:
            number = 'p'
        elif 'sp' in msfeatures:
            number = 'sp'

        # Also check unimorph for more detail
        if unimorph:
            if 'MASC' in unimorph and 'FEM' not in unimorph:
                gender = 'm'
            elif 'FEM' in unimorph and 'MASC' not in unimorph:
                gender = 'f'
            if ';PL' in unimorph:
                number = 'p'
            elif ';SG' in unimorph:
                number = 's'

        # Map categories to LanguageTool POS
        if category == 'det':
            return f"D {gender} {number}"
        elif category == 'adj':
            return f"J {gender} {number}"
        elif category == 'nc':  # common noun
            return f"N {gender} {number}"
        elif category == 'np':  # proper noun
            return f"Z {gender} {number}"
        elif category == 'adv':
            return "R"
        elif category == 'prep':
            return "P"
        elif category in ('coo', 'csu'):  # conjunctions
            return "C"
        elif category == 'pro':
            return f"R {gender} {number}"  # pronoun
        elif category == 'cla':
            return f"R {gender} {number}"  # clitic pronoun
        elif category == 'v':
            # Parse verb features from msfeatures or unimorph
            return parse_verb_pos(msfeatures, unimorph, gender, number)
        elif category == 'auxEtre':
            return parse_verb_pos(msfeatures, unimorph, gender, number)
        elif category == 'auxAvoir':
            return parse_verb_pos(msfeatures, unimorph, gender, number)
        else:
            return category.upper()

    def parse_verb_pos(msfeatures, unimorph, gender, number):
        """Parse verb POS from msfeatures/unimorph"""
        # Default
        mood = "ind"
        tense = "pres"
        person = "3"
        num = "s"

        # Parse from unimorph (more reliable)
        if unimorph:
            if 'INF' in unimorph:
                return "V inf"
            if 'NFIN' in unimorph or 'PTCP' in unimorph:
                if 'PST' in unimorph or 'PASS' in unimorph:
                    return f"V ppa {gender} {number}"
                return "V ppr"

            if 'IND' in unimorph:
                mood = "ind"
            elif 'SBJV' in unimorph:
                mood = "sub"
            elif 'COND' in unimorph:
                mood = "cond"
            elif 'IMP' in unimorph:
                mood = "imp"

            if 'PST' in unimorph:
                tense = "past"
            elif 'FUT' in unimorph:
                tense = "futu"
            elif 'IPFV' in unimorph:
                tense = "impf"
            else:
                tense = "pres"

            if '1;' in unimorph or ';1;' in unimorph:
                person = "1"
            elif '2;' in unimorph or ';2;' in unimorph:
                person = "2"
            else:
                person = "3"

            if ';PL' in unimorph:
                num = "p"
            else:
                num = "s"

            return f"V {mood} {tense} {person} {num}"

        # Fallback: parse from msfeatures (P1s, P3p, S1s, etc.)
        if msfeatures:
            if msfeatures.startswith('W') or msfeatures == 'W':
                return "V inf"
            if msfeatures.startswith('K'):
                return f"V ppa {gender} {number}"
            if msfeatures.startswith('G'):
                return "V ppr"

            if msfeatures.startswith('P'):
                mood = "ind"
                tense = "pres"
            elif msfeatures.startswith('I'):
                mood = "ind"
                tense = "impf"
            elif msfeatures.startswith('J'):
                mood = "ind"
                tense = "past"  # passé simple
            elif msfeatures.startswith('F'):
                mood = "ind"
                tense = "futu"
            elif msfeatures.startswith('C'):
                mood = "cond"
                tense = "pres"
            elif msfeatures.startswith('S'):
                mood = "sub"
                tense = "pres"
            elif msfeatures.startswith('T'):
                mood = "sub"
                tense = "impf"
            elif msfeatures.startswith('Y'):
                mood = "imp"
                tense = "pres"

            # Extract person and number
            if '1' in msfeatures:
                person = "1"
            elif '2' in msfeatures:
                person = "2"
            else:
                person = "3"

            if 'p' in msfeatures.lower():
                num = "p"
            else:
                num = "s"

            return f"V {mood} {tense} {person} {num}"

        return "V"

    # Write TSV file
    output_path = "data/morphology/fr_lefff.tsv"
    print(f"Writing to {output_path}...")

    count = 0
    skipped = 0

    with open(output_path, "w", encoding="utf-8") as f:
        for _, row in df.iterrows():
            form = str(row['form'])
            lemma = str(row['lemma']) if pd.notna(row['lemma']) else ''
            category = str(row['category']) if pd.notna(row['category']) else ''

            # Skip empty or special entries
            if not lemma or not category:
                skipped += 1
                continue

            # Skip punctuation, amalgams, and non-alphabetic forms
            if category in ('poncts', 'amlgm', 'ilimp'):
                skipped += 1
                continue

            # Skip forms that aren't mostly alphabetic (allow apostrophes and hyphens)
            clean_form = form.replace("'", "").replace("-", "").replace("'", "")
            if not clean_form or not clean_form.isalpha():
                skipped += 1
                continue

            # Map to LanguageTool POS
            try:
                pos = map_to_lt_pos(row)
            except Exception as e:
                skipped += 1
                continue

            # Write TSV: form\tlemma\tpos
            f.write(f"{form}\t{lemma}\t{pos}\n")
            count += 1

    print(f"Done! Wrote {count} entries ({skipped} skipped)")

    # Show file size
    size = os.path.getsize(output_path)
    print(f"File size: {size / 1024 / 1024:.2f} MB")

    # Show sample entries
    print("\nSample entries:")
    with open(output_path, "r", encoding="utf-8") as f:
        for i, line in enumerate(f):
            if i >= 15:
                break
            print(f"  {line.strip()}")

    # Verify specific words
    print("\nVerification:")
    with open(output_path, "r", encoding="utf-8") as f:
        lines = f.readlines()

    for word in ['le', 'la', 'les', 'un', 'une', 'grand', 'grande', 'mange', 'pense', 'penses']:
        matches = [l.strip() for l in lines if l.startswith(word + '\t')]
        print(f"  {word}: {matches[:3]}")

if __name__ == "__main__":
    main()
