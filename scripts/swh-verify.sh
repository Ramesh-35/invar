#!/usr/bin/env bash
set -euo pipefail

TSV_PATH="docs/reference/SWH-ARCHIVAL.tsv"

python3 -c "
import urllib.request, time, sys

tsv_path = '$TSV_PATH'
with open(tsv_path, 'r') as f:
    lines = [l.strip() for l in f.readlines() if l.strip() and not l.startswith('#') and not l.startswith('origin')]

absent_count = 0
fail_count = 0

print(f'Checking {len(lines)} origins against Software Heritage API...')

for line in lines:
    parts = line.split('\t')
    if len(parts) < 2:
        continue
    origin, expected_status = parts[0], parts[1]
    
    if expected_status == 'absent':
        absent_count += 1

    url = f'https://archive.softwareheritage.org/api/1/origin/{origin}/get/'
    req = urllib.request.Request(url, headers={'User-Agent': 'INVAR-Archival-Verifier/1.0'})
    
    success = False
    for attempt in range(4):
        try:
            with urllib.request.urlopen(req) as resp:
                if resp.status == 200:
                    success = True
                    break
        except urllib.error.HTTPError as e:
            if e.code == 429:
                time.sleep(3.0 * (attempt + 1))
                continue
            elif e.code == 404:
                if expected_status == 'archived':
                    print(f'CHECK FAILED (404): {origin}')
                    fail_count += 1
                success = True
                break
            else:
                time.sleep(1.0)
        except Exception:
            time.sleep(1.0)

    if not success and expected_status == 'archived':
        print(f'CHECK FAILED (RATE_LIMITED): {origin}')
        fail_count += 1

    time.sleep(0.3)

print(f'absent: {absent_count}, check failures: {fail_count}')
if fail_count > 0:
    sys.exit(3)
"
