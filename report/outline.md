# Outline

- Why is TeX bad?
  - Common pitfalls
  - It's slow and error-prone
    - It's old; the design was optimized around memory, not...
      - Incremental compilation
      - Autoformatters / language servers
      - Static analysis
  - Errors are difficult to debug
  - Macro expansion is error-prone
    - *But* incredibly flexible!
  - What do *I* need in a typesetting system that TeX doesn't do?
  - No namespacing or versioning!
  - There's two *parts* of TeX: the typesetting engine and the macro language
  - Weird font system / tons of bespoke formats.
  - What does TeX *not* do, why, and how can we fix it?
  - Further, why haven't we made anything better?
- Designing a better system
  - Cross-referencing of concepts
    - Focusing on mathematics notes in particular, openly acknowledging that
  - Non-PDF output
- Beyond plain text as the representation of code
  - Towards "notebooks" and Scratch-like interfaces


## References

- https://timeflayer.com/d/tex/tldr.html

  Feeb's TeX tl;dr. Highlights a ton of weak points / weirdness.

- https://oberoncore.ru/_media/library/wirth_a_brief_history_of_software_engineering.pdf

- https://www.pc-freak.net/blog/files/NCR_ATM_terminals/www.thecorememory.com/THTHS.pdf

- Parsing TeX is Turing complete: https://tex.stackexchange.com/a/4205

- Xanadu

  https://en.wikipedia.org/wiki/Project_Xanadu
