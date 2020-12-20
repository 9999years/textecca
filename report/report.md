# Textecca

## Beyond plain text

Using plain text for source code has certain limitations, so we create IDEs or
language servers to add functionality to our text editors. These are tools
which add additional context and parse meaningful information out of source
code; a language server can answer questions like "where is the command under
my cursor defined?" or "where's the documentation for this function?"

These tools become more important when working on larger projects, because they
let users offload [working memory][working-memory] to an automated system — you
don't need to learn all the built-in commands if you can use autocompletion to
see exactly which ones are in scope at any point or hover over them to see
their documentation, along with argument names and counts.

But we can take this concept even further. Language servers paper over plain
text source code by adding a layer of extra contextual information, but what
advantages does plain text bring? Though "integration with preexisting tools"
is a common refrain, downstream consequences of generalized tools are
commonplace: line-based diffs can get noisy, code searchability begins to
depend on formatting choices, and more.

Syntax errors are a common stumbling block for beginners, and they're a result
of plain text source code as well. In plain text, any sequence of characters is
valid, but parsers and compilers only accept a tiny subset of possible inputs.
We can avoid this by allowing our editor to only represent valid programs.
Scratch does this, for instance, by associating each type in the parse tree a
shape: expressions, variables, boolean conditions, and more all have specific
shapes that make it clear how different types of "block" fit together into a
bigger program.

We can leverage this concept to avoid several of TeX's most confusing behaviors
around parsing *while* avoiding the author-side overhead of a more verbose
language.

## DSLs

- I believe TeX's support for [DSLs][dsl] is one of its greatest strengths. The
  same flexibility that makes the macro system so intimidating, confusing, and
  complicated allows package authors to embed nearly any language in TeX,
  limited only by the package author's willingness to write code in TeX.
- What would DSLs look like in non plain text? Would there be style
  instructions? Does defining a DSL turn from a compiler problem to a GUI
  problem?

## Concept-based authoring

[working-memory]: https://en.wikipedia.org/wiki/Working_memory
[dsl]: https://en.wikipedia.org/wiki/Domain-specific_language
