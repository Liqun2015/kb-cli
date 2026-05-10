# Worker Task: Paper Section Extraction Fallback

This task is for a Paper Section OCR/LLM Worker.

## Goal

Recover the `Introduction` and `References` sections from an image-based or poorly extracted paper.

## Input Files

- `raw/papers/example.pdf`
- `processing/text/raw__papers__example.txt` if available

## Required Output

- `processing/sections/raw__papers__example/introduction.txt`
- `processing/sections/raw__papers__example/references.txt`
- `processing/sections/raw__papers__example/section_manifest.md`

## Rules

- Preserve original wording as much as possible.
- Do not summarize the sections.
- Mark uncertainty and OCR errors.
- Do not modify `raw/`.
- Leave the result for Git diff review.
