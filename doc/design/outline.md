# Outline

## Requirements

What does a notetaking system need to contain?

-   **Support for linear and nonlinear workflows.** Notes are *sometimes* added
    to in a purely linear fashion, but we often need to go back and
    amend/elaborate on/revise previous concepts. The editor needs to support
    this by making jumping to an arbitrary reference or concept (and back!)
    easy.

-   **Support for multiple concurrent organizational schemes.** At the
    minimum, into:

    1.  Sections/chapters/etc., for hierarchical browsing & structuring
        concepts into groups
    2.  Days/lectures/weeks, i.e. chronologically; so that questions like
        "what were we talking about last Thursday" are easy to answer & don't
        come at the expense of a more structured section-based organization

-   **Near-realtime speed.** The entire workflow (including the editor,
    language server, compiler, previewer, etc.) and its operations must be
    incredibly fast --- like, within 30-60 ms, or 1-2 frames. In particular,
    operations like fuzzy-finding a concept, adding a diagram, etc. need to
    *never* slow the user down.

-   **Extensible.** Users must be able to define custom notation, macros,
    etc. in order to:

    1. Keep documents semantically written
    2. Promote understanding concepts rather than particular symbols, styles,
        etc.
    3. Make reorganizing notation and presentation easy

-   **Browsable.** Browsing and searching the notes (either with text search
    or graph queries) should be easy, reliable, and fast.

-   **Have *rich* semantics.** Existing tools have a (bizarrely, in my
    opinion) limited set of semantics, but actual documents need many more
    primitives: concept definitions and references,
    theorems/propositions/proofs, examples/problems, callout sections, etc.

    So in addition to a set of tools for designing *new* organizational
    primitives, the set of always-available tools must be greatly expanded.

## Issues with current tools

What's wrong with what we've got?

-   Broad issues; these span many tools
  -   Poor developer tooling. I need things like jump-to-definition
  -   **Bad editors.** No markup languages are designed to support tooling,
      which makes everything from autocompletion to input validation to quick
      editing difficult.

      In particular, editors will often support some "easy" features (think
      `Ctrl-b` to add `****` to the text and put your cursor in the middle)
      but not the hard ones (for example, no Markdown editor or language
      server I'm aware of will autocomplete link names or help you manage
      them).

-   Specific issues; these are singular tools and why I consider them unfit
    for task

## Potential solutions & their components

## Why it's a lot of work

- Whitespace & language design
- Package managment
- Devtools
- Compiling

## How we can fix it
